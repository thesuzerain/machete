use std::collections::HashMap;

use crate::models::ids::InternalId;
use crate::models::library::{item::LibraryItem, GameSystem, Rarity};

use crate::models::query::CommaSeparatedVec;
use crate::ServerError;
use serde::{Deserialize, Serialize};

use super::DEFAULT_MAX_LIMIT;

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct ItemFiltering {
    // Custom/complex deserialization types, so we can't use #[flatten]
    // Page needs to be kept separate from flattened structure.
    // https://github.com/serde-rs/serde/issues/1183
    pub ids: Option<CommaSeparatedVec>,
    pub min_level: Option<i8>,
    pub max_level: Option<i8>,
    pub min_price: Option<i32>, // TODO: should this be a Currency struct?
    pub max_price: Option<i32>,
    pub name: Option<String>,
    pub rarity: Option<Rarity>,
    pub game_system: Option<GameSystem>,
    #[serde(default)]
    pub tags: Vec<String>,

    pub limit: Option<u64>,
    pub page: Option<u64>,
}

impl ItemFiltering {
    pub fn from_id(id: u32) -> Self {
        Self {
            ids: Some(CommaSeparatedVec(vec![id])),
            ..Default::default()
        }
    }

    pub fn from_ids(ids: &[u32]) -> Self {
        Self {
            ids: Some(CommaSeparatedVec(ids.to_vec())),
            ..Default::default()
        }
    }
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct ItemSearch {
    pub query: Vec<String>,
    pub min_similarity: Option<f32>, // 0.0 to 1.0
    pub favor_exact_start: Option<bool>,

    // Custom/complex deserialization types, so we can't use #[flatten]
    // Page needs to be kept separate from flattened structure.
    // https://github.com/serde-rs/serde/issues/1183
    pub ids: Option<CommaSeparatedVec>,
    pub min_level: Option<i8>,
    pub max_level: Option<i8>,
    pub min_price: Option<i32>, // TODO: should this be a Currency struct?
    pub max_price: Option<i32>,
    pub name: Option<String>,
    pub rarity: Option<Rarity>,
    pub game_system: Option<GameSystem>,
    #[serde(default)]
    pub tags: Vec<String>,

    pub limit: Option<u64>,
    pub page: Option<u64>,
}

impl From<ItemFiltering> for ItemSearch {
    fn from(filter: ItemFiltering) -> Self {
        Self {
            query: vec!["".to_string()], // Empty search query
            min_similarity: None,
            favor_exact_start: None,
            ids: filter.ids,
            min_level: filter.min_level,
            max_level: filter.max_level,
            min_price: filter.min_price,
            max_price: filter.max_price,
            name: filter.name,
            rarity: filter.rarity,
            game_system: filter.game_system,
            tags: filter.tags,
            limit: filter.limit,
            page: filter.page,
        }
    }
}

pub async fn get_items(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    condition: &ItemFiltering,
) -> crate::Result<Vec<LibraryItem>> {
    get_items_search(
        exec,
        &ItemSearch::from(condition.clone()),
        DEFAULT_MAX_LIMIT,
    )
    .await?
    .into_iter()
    .next()
    .map(|(_, v)| v.into_iter().map(|(_, v)| v).collect())
    .ok_or_else(|| ServerError::NotFound)
}

// TODO: May be prudent to make a separate models system for the database.
pub async fn get_items_search(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    // TODO: Could this use be problematic?
    // A postgres alternative can be found here:
    // https://github.com/launchbadge/sqlx/issues/291
    search: &ItemSearch,
    default_limit: u64,
) -> crate::Result<HashMap<String, Vec<(f32, LibraryItem)>>> {
    // TODO: check on number of queries
    let limit = search.limit.unwrap_or(default_limit);
    let page = search.page.unwrap_or(0);
    let offset = page * limit;

    let min_similarity = search.min_similarity.unwrap_or(0.0);

    let ids = search.ids.clone().map(|t| {
        t.into_inner()
            .into_iter()
            .map(|id| id as i32)
            .collect::<Vec<i32>>()
    });

    // TODO: data type 'as'
    let query = sqlx::query!(
        r#"
        SELECT
            c.*, query as "query!"
        FROM UNNEST($10::text[]) query
        CROSS JOIN LATERAL (
            SELECT 
                -- If we favour exact start, we set similarity to 1.0 if the name starts with the query.
                CASE
                    WHEN $12::bool THEN 
                        CASE
                            WHEN lo.name ILIKE query || '%' THEN 1.01
                            WHEN lo.name ILIKE '%' || query || '%' THEN 1.0
                            ELSE SIMILARITY(lo.name, query)
                        END
                    ELSE SIMILARITY(lo.name, query)
                END AS similarity,
                CASE WHEN $12::bool THEN length(lo.name) ELSE 0 END AS favor_exact_start_length,
                lo.id,
                lo.name,
                lo.game_system,
                lo.url,
                lo.description,
                li.rarity,
                li.level,
                li.price,
                ARRAY_AGG(DISTINCT tag) FILTER (WHERE tag IS NOT NULL) AS tags
            FROM library_objects lo
            INNER JOIN library_items li ON lo.id = li.id
            LEFT JOIN library_objects_tags lot ON lo.id = lot.library_object_id
            LEFT JOIN tags t ON lot.tag_id = t.id

            WHERE
                ($1::text IS NULL OR lo.name ILIKE '%' || $1 || '%')
                AND ($2::int IS NULL OR rarity = $2)
                AND ($3::int IS NULL OR game_system = $3)
                AND ($4::int IS NULL OR level >= $4)
                AND ($5::int IS NULL OR level <= $5)
                AND ($6::int IS NULL OR price >= $6)
                AND ($7::int IS NULL OR price <= $7)
                AND ($8::text IS NULL OR tag ILIKE '%' || $8 || '%')
                AND ($9::int[] IS NULL OR lo.id = ANY($9))
                AND (($12::bool AND lo.name ILIKE '%' || query || '%') OR SIMILARITY(lo.name, query) >= $11)
            GROUP BY lo.id, li.id ORDER BY similarity DESC, favor_exact_start_length, lo.name
            LIMIT $13 OFFSET $14
        ) c
        ORDER BY similarity DESC, favor_exact_start_length, c.name 
    "#,
        search.name,
        search.rarity.as_ref().map(|r| r.as_i64() as i32),
        search.game_system.as_ref().map(|gs| gs.as_i64() as i32),
        search.min_level.map(|l| l as i32),
        search.max_level.map(|l| l as i32),
        search.min_price.map(|p| p as i32),
        search.max_price.map(|p| p as i32),
        search.tags.first(), // TODO: This is incorrect, only returning one tag.
        &ids as _,
        &search.query,
        min_similarity,
        search.favor_exact_start,
        limit as i64,
        offset as i64,
    )
    .fetch_all(exec)
    .await?;

    // create initial hm with empty vecs for each query
    let hm = search
        .query
        .iter()
        .map(|q| (q.clone(), Vec::new()))
        .collect::<HashMap<_, _>>();

    let items = query.into_iter().fold(hm, |map, row| {
        // TODO: conversions still here shouldnt be needed
        // TODO: unwrap_or_default for stuff like rarity / price / level doesn't seem right
        let query = row.query;
        let item = LibraryItem {
            id: InternalId(row.id as u64),
            name: row.name,
            game_system: GameSystem::from_i64(row.game_system as i64),
            rarity: Rarity::from_i64(row.rarity.unwrap_or_default() as i64),
            level: row.level.unwrap_or_default() as i8,
            price: row.price.unwrap_or_default(),
            tags: row.tags.unwrap_or_default(),
            url: row.url,
            description: row.description.unwrap_or_default(),
        };
        let mut map = map;
        map.entry(query)
            .or_default()
            .push((row.similarity.unwrap_or_default(), item));
        map
    });
    Ok(items)
}

pub async fn insert_items(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres> + Copy,
    items: &[LibraryItem],
) -> crate::Result<()> {
    // TODO: Don't *need* two tables for this

    if items.is_empty() {
        return Ok(());
    }

    let ids = sqlx::query!(
        r#"
        INSERT INTO library_objects (name, game_system, url, description)
        SELECT * FROM UNNEST ($1::text[], $2::int[], $3::text[], $4::text[])
        RETURNING id  
    "#,
        &items
            .iter()
            .map(|c| c.name.clone())
            .collect::<Vec<String>>(),
        &items
            .iter()
            .map(|c| c.game_system.as_i64() as i32)
            .collect::<Vec<i32>>(),
        &items
            .iter()
            .map(|c| c.url.clone())
            .collect::<Vec<Option<String>>>() as _,
        &items
            .iter()
            .map(|c| c.description.clone())
            .collect::<Vec<String>>(),
    )
    .fetch_all(exec)
    .await?
    .into_iter()
    .map(|row| Ok(row.id))
    .collect::<Result<Vec<i32>, sqlx::Error>>()?;

    // TODO: as i32 should be unnecessary- fix in models
    sqlx::query!(
        r#"
        INSERT INTO library_items (id, rarity, level, price)
        SELECT * FROM UNNEST ($1::int[], $2::int[], $3::int[], $4::double precision[])
    "#,
        &ids.iter().map(|id| *id as i32).collect::<Vec<i32>>(),
        &items
            .iter()
            .map(|c| c.rarity.as_i64() as i32)
            .collect::<Vec<i32>>(),
        &items.iter().map(|c| c.level as i32).collect::<Vec<i32>>(),
        &items.iter().map(|c| c.price as f64).collect::<Vec<f64>>(),
    )
    .execute(exec)
    .await?;

    Ok(())
}

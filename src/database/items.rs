use std::collections::HashMap;

use crate::models::ids::InternalId;
use crate::models::library::{
    item::{Currency, LibraryItem},
    GameSystem, Rarity,
};

use crate::models::query::CommaSeparatedVec;
use crate::ServerError;
use serde::{Deserialize, Serialize};

use super::DEFAULT_MAX_LIMIT;

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct ItemFilters {
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

impl ItemFilters {
    pub fn from_id(id: u32) -> Self {
        Self {
            ids: Some(CommaSeparatedVec(vec![id])),
            ..Default::default()
        }
    }
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct ItemSearch {
    pub query: Vec<String>,
    pub min_similarity: Option<f32>, // 0.0 to 1.0

    #[serde(flatten)]
    pub filters: ItemFilters,
}

pub async fn get_items(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    condition: &ItemFilters,
) -> crate::Result<Vec<LibraryItem>> {    
    get_items_search(exec, &ItemSearch {
        query: vec!["".to_string()], // Empty search query
        min_similarity: None,
        filters: condition.clone(),
    }, DEFAULT_MAX_LIMIT).await?.into_iter().next().map(|(_, v)| v).ok_or_else(|| ServerError::NotFound)
}

// TODO: May be prudent to make a separate models system for the database.
pub async fn get_items_search(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    // TODO: Could this use be problematic?
    // A postgres alternative can be found here:
    // https://github.com/launchbadge/sqlx/issues/291
    search: &ItemSearch,
    default_limit: u64,
) -> crate::Result<HashMap<String, Vec<LibraryItem>>> {
    let condition = &search.filters;

    // TODO: check on number of queries
    let limit = condition.limit.unwrap_or(default_limit);
    let page = condition.page.unwrap_or(0);
    let offset = page * limit;

    let min_similarity = search.min_similarity.unwrap_or(0.0);

    let ids = condition.ids.clone().map(|t| {
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
                SUM(SIMILARITY(lo.name, query)) AS similarity,
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
                AND SIMILARITY(lo.name, query) >= $11
            
            GROUP BY lo.id, li.id ORDER BY lo.name
            LIMIT $12 OFFSET $13
        ) c
        ORDER BY similarity DESC
    "#,
        condition.name,
        condition.rarity.as_ref().map(|r| r.as_i64() as i32),
        condition.game_system.as_ref().map(|gs| gs.as_i64() as i32),
        condition.min_level.map(|l| l as i32),
        condition.max_level.map(|l| l as i32),
        condition.min_price.map(|p| p as i32),
        condition.max_price.map(|p| p as i32),
        condition.tags.first(), // TODO: This is incorrect, only returning one tag.
        &ids as _,
        &search.query,
        min_similarity,
        limit as i64,
        offset as i64,
    )
    .fetch_all(exec)
    .await?;

    let items = query
        .into_iter()
        .fold(HashMap::new(), |map, row| {
            // TODO: conversions still here shouldnt be needed
            // TODO: unwrap_or_default for stuff like rarity / price / level doesn't seem right
            let query = row.query;
            let item = LibraryItem {
                id: InternalId(row.id as u64),
                name: row.name,
                game_system: GameSystem::from_i64(row.game_system as i64),
                rarity: Rarity::from_i64(row.rarity.unwrap_or_default() as i64),
                level: row.level.unwrap_or_default() as i8,
                price: Currency::from_base_unit(row.price.unwrap_or_default() as u32),
                tags: row.tags.unwrap_or_default(),
                url: row.url,
                description: row.description.unwrap_or_default(),
            };
            let mut map = map;
            map.entry(query).or_insert_with(Vec::new).push(item);
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
        SELECT * FROM UNNEST ($1::int[], $2::int[], $3::int[], $4::int[])
    "#,
        &ids.iter().map(|id| *id as i32).collect::<Vec<i32>>(),
        &items
            .iter()
            .map(|c| c.rarity.as_i64() as i32)
            .collect::<Vec<i32>>(),
        &items.iter().map(|c| c.level as i32).collect::<Vec<i32>>(),
        &items
            .iter()
            .map(|c| c.price.as_base_unit() as i32)
            .collect::<Vec<i32>>(),
    )
    .execute(exec)
    .await?;

    Ok(())
}

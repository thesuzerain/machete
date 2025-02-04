use std::collections::HashMap;

use crate::models::ids::InternalId;
use crate::models::library::{
    hazard::{HazardType, LibraryHazard},
    GameSystem, Rarity,
};

use crate::models::query::CommaSeparatedVec;
use crate::ServerError;
use serde::{Deserialize, Serialize};

use super::DEFAULT_MAX_LIMIT;

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct HazardFilters {
    pub ids: Option<CommaSeparatedVec>,
    pub min_level: Option<i8>,
    pub max_level: Option<i8>,
    pub hazard_type: Option<HazardType>,
    pub name: Option<String>,
    pub rarity: Option<Rarity>,
    pub game_system: Option<GameSystem>,
    #[serde(default)]
    pub tags: Vec<String>,
}

impl HazardFilters {
    pub fn from_id(id: u32) -> Self {
        HazardFilters {
            ids: Some(CommaSeparatedVec(vec![id])),
            ..Default::default()
        }
    }

    pub fn from_ids(ids: &[u32]) -> Self {
        HazardFilters {
            ids: Some(CommaSeparatedVec(ids.to_vec())),
            ..Default::default()
        }
    }
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct HazardFiltering {
    // Page needs to be kept separate from flattened structure.
    // https://github.com/serde-rs/serde/issues/1183
    pub limit: Option<u64>,
    pub page: Option<u64>,
    #[serde(flatten)]
    pub filters: HazardFilters,
}

impl From<HazardFilters> for HazardFiltering {
    fn from(filters: HazardFilters) -> Self {
        HazardFiltering {
            filters,
            ..Default::default()
        }
    }
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct HazardSearch {
    pub query: Vec<String>,
    pub min_similarity: Option<f32>, // 0.0 to 1.0
    pub favor_exact_start: Option<bool>,

    // Page needs to be kept separate from flattened structure.
    // https://github.com/serde-rs/serde/issues/1183
    pub page: Option<u64>,
    pub limit: Option<u64>,

    #[serde(flatten)]
    pub filters: HazardFilters,
}

// TODO: May be prudent to make a separate models system for the database.
pub async fn get_hazards(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    condition: &HazardFiltering,
) -> crate::Result<Vec<LibraryHazard>> {
    get_hazards_search(
        exec,
        &HazardSearch {
            query: vec!["".to_string()], // Empty search query
            min_similarity: None,
            filters: condition.filters.clone(),
            favor_exact_start: None,
            page: condition.page,
            limit: condition.limit,
        },
        DEFAULT_MAX_LIMIT,
    )
    .await?
    .into_iter()
    .next()
    .map(|(_, v)| v.into_iter().map(|(_, v)| v).collect())
    .ok_or_else(|| ServerError::NotFound)
}

// TODO: May be prudent to make a separate models system for the database.
pub async fn get_hazards_search(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    // TODO: Could this use be problematic?
    // A postgres alternative can be found here:
    // https://github.com/launchbadge/sqlx/issues/291
    search: &HazardSearch,
    default_limit: u64,
) -> crate::Result<HashMap<String, Vec<(f32, LibraryHazard)>>> {
    let condition = &search.filters;

    // TODO: check on number of queries
    let limit = search.limit.unwrap_or(default_limit);
    let page = search.page.unwrap_or(0);
    let offset = page * limit;
    let min_similarity = search.min_similarity.unwrap_or(0.0);

    let ids = condition.ids.clone().map(|t| {
        t.into_inner()
            .into_iter()
            .map(|id| id as i32)
            .collect::<Vec<i32>>()
    });

    let query = sqlx::query!(
        r#"
        SELECT
            c.*, query as "query!"
        FROM UNNEST($8::text[]) query
        CROSS JOIN LATERAL (
            SELECT 
                -- If we favour exact start, we set similarity to 1.0 if the name starts with the query.
                CASE
                    WHEN $10::bool THEN 
                        CASE
                            WHEN lo.name ILIKE query || '%' THEN 1.01
                            WHEN lo.name ILIKE '%' || query || '%' THEN 1.0
                            ELSE SIMILARITY(lo.name, query)
                        END
                    ELSE SIMILARITY(lo.name, query)
                END AS similarity,
                CASE WHEN $10::bool THEN length(lo.name) ELSE 0 END AS favor_exact_start_length,
                lo.id,
                lo.name,
                lo.game_system,
                lo.url,
                lo.description,
                rarity,
                level,
                ARRAY_AGG(DISTINCT tag) FILTER (WHERE tag IS NOT NULL) AS tags
            FROM library_objects lo
            INNER JOIN library_hazards lc ON lo.id = lc.id
            LEFT JOIN library_objects_tags lot ON lo.id = lot.library_object_id
            LEFT JOIN tags t ON lot.tag_id = t.id

            WHERE 1=1
                AND ($1::text IS NULL OR lo.name ILIKE '%' || $1 || '%')
                AND ($2::int IS NULL OR rarity = $2)
                AND ($3::int IS NULL OR game_system = $3)
                AND ($4::int IS NULL OR level >= $4)
                AND ($5::int IS NULL OR level <= $5)
                AND ($6::text IS NULL OR tag ILIKE '%' || $6 || '%')
                AND ($7::int[] IS NULL OR lo.id = ANY($7))
                AND (($10::bool AND lo.name ILIKE '%' || query || '%') OR SIMILARITY(lo.name, query) >= $9)
            GROUP BY lo.id, lc.id ORDER BY similarity DESC, favor_exact_start_length, lo.name
            LIMIT $11 OFFSET $12
        ) c
        ORDER BY similarity DESC, favor_exact_start_length, c.name 
    "#,
        condition.name,
        condition.rarity.as_ref().map(|r| r.as_i64() as i32),
        condition.game_system.as_ref().map(|gs| gs.as_i64() as i32),
        condition.min_level.map(|r| r as i32),
        condition.max_level.map(|r| r as i32),
        condition.tags.first(), // TODO: Incorrect, only returning one tag.
        &ids as _,
        &search.query,
        min_similarity,
        search.favor_exact_start,
        limit as i64,
        offset as i64,
    );

    // create initial hm with empty vecs for each query
    let hm = search
        .query
        .iter()
        .map(|q| (q.clone(), Vec::new()))
        .collect::<HashMap<_, _>>();
    let hazards = query
        .fetch_all(exec)
        .await?
        .into_iter()
        .fold(hm, |mut map, row| {
            let query = row.query;
            let hazard = LibraryHazard {
                id: InternalId(row.id as u64),
                name: row.name,
                game_system: GameSystem::from_i64(row.game_system as i64),
                rarity: Rarity::from_i64(row.rarity.unwrap_or_default() as i64),
                level: row.level.unwrap_or_default() as i8,
                tags: row.tags.unwrap_or_default(),
                url: row.url,
                description: row.description.unwrap_or_default(),
            };
            map.entry(query)
                .or_default()
                .push((row.similarity.unwrap_or_default(), hazard));
            map
        });
    Ok(hazards)
}

pub async fn insert_hazards(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres> + Copy,
    hazards: &[LibraryHazard],
) -> crate::Result<()> {
    // TODO: Do we *need* two tables for this?

    if hazards.is_empty() {
        return Ok(());
    }
    let ids = sqlx::query!(
        r#"
        INSERT INTO library_objects (name, game_system, url, description)
        SELECT * FROM UNNEST ($1::text[], $2::int[], $3::text[], $4::text[])
        RETURNING id  
    "#,
        &hazards
            .iter()
            .map(|c| c.name.clone())
            .collect::<Vec<String>>(),
        &hazards
            .iter()
            .map(|c| c.game_system.as_i64() as i32)
            .collect::<Vec<i32>>(),
        &hazards
            .iter()
            .map(|c| c.url.clone())
            .collect::<Vec<Option<String>>>() as _,
        &hazards
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
        INSERT INTO library_hazards (id, rarity, level)
        SELECT * FROM UNNEST ($1::int[], $2::int[], $3::int[])
    "#,
        &ids.iter().map(|id| *id as i32).collect::<Vec<i32>>(),
        &hazards
            .iter()
            .map(|c| c.rarity.as_i64() as i32)
            .collect::<Vec<i32>>(),
        &hazards.iter().map(|c| c.level as i32).collect::<Vec<i32>>(),
    )
    .execute(exec)
    .await?;

    Ok(())
}

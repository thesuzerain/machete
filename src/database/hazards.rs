use std::collections::HashMap;

use crate::models::ids::InternalId;
use crate::models::library::{
    hazard::{HazardType, LibraryHazard},
    GameSystem, Rarity,
};

use crate::models::query::CommaSeparatedVec;
use crate::ServerError;
use serde::{Deserialize, Serialize};

use super::{check_library_requested_ids, LegacyStatus, DEFAULT_MAX_LIMIT};

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct HazardFiltering {
    // Custom/complex deserialization types, so we can't use #[flatten]
    // Page needs to be kept separate from flattened structure.
    // https://github.com/serde-rs/serde/issues/1183
    pub ids: Option<CommaSeparatedVec>,
    pub min_level: Option<i8>,
    pub max_level: Option<i8>,
    pub hazard_type: Option<HazardType>,
    pub name: Option<String>,
    pub rarity: Option<Rarity>,
    pub complex: Option<bool>,
    pub haunt: Option<bool>,
    pub game_system: Option<GameSystem>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub legacy: LegacyStatus,

    pub limit: Option<u64>,
    pub page: Option<u64>,
}

impl HazardFiltering {
    pub fn from_id(id: u32) -> Self {
        HazardFiltering {
            ids: Some(CommaSeparatedVec(vec![id])),
            ..Default::default()
        }
    }

    pub fn from_ids(ids: &[u32]) -> Self {
        HazardFiltering {
            ids: Some(CommaSeparatedVec(ids.to_vec())),
            ..Default::default()
        }
    }
}
#[derive(Default, Serialize, Deserialize, Debug)]
pub struct HazardSearch {
    pub query: Vec<String>,
    pub min_similarity: Option<f32>, // 0.0 to 1.0
    pub favor_exact_start: Option<bool>,

    // Custom/complex deserialization types, so we can't use #[flatten]
    // Page needs to be kept separate from flattened structure.
    // https://github.com/serde-rs/serde/issues/1183
    pub ids: Option<CommaSeparatedVec>,
    pub min_level: Option<i8>,
    pub max_level: Option<i8>,
    pub hazard_type: Option<HazardType>,
    pub name: Option<String>,
    pub rarity: Option<Rarity>,
    pub complex: Option<bool>,
    pub haunt: Option<bool>,
    pub game_system: Option<GameSystem>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub legacy: LegacyStatus,

    pub limit: Option<u64>,
    pub page: Option<u64>,
}

impl From<HazardFiltering> for HazardSearch {
    fn from(filter: HazardFiltering) -> Self {
        Self {
            query: vec!["".to_string()],
            name: filter.name,
            min_similarity: None,
            favor_exact_start: None,
            ids: filter.ids,
            min_level: filter.min_level,
            max_level: filter.max_level,
            hazard_type: filter.hazard_type,
            rarity: filter.rarity,
            complex: filter.complex,
            haunt: filter.haunt,
            game_system: filter.game_system,
            tags: filter.tags,
            legacy: filter.legacy,
            limit: filter.limit,
            page: filter.page,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct InsertLibraryHazard {
    pub requested_id: Option<InternalId>,
    pub name: String,
    pub game_system: GameSystem,
    pub rarity: Rarity,
    pub level: i8,
    pub tags: Vec<String>,
    pub legacy: bool,
    pub remastering_alt_id: Option<InternalId>,

    pub url: Option<String>,
    pub description: String,

    pub complex: bool,
    pub haunt: bool,
}

// TODO: May be prudent to make a separate models system for the database.
pub async fn get_hazards(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    condition: &HazardFiltering,
) -> crate::Result<Vec<LibraryHazard>> {
    get_hazards_search(
        exec,
        &HazardSearch::from(condition.clone()),
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
                complex,
                haunt,
                level,
                ARRAY_AGG(DISTINCT tag) FILTER (WHERE tag IS NOT NULL) AS tags,
                legacy,
                remastering_alt_id
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
                AND NOT (NOT $11::bool AND lo.legacy = FALSE)
                AND NOT (NOT $12::bool AND lo.legacy = TRUE)
                AND NOT ($13::bool AND lo.remastering_alt_id IS NOT NULL AND lo.legacy = TRUE)
                AND NOT ($14::bool AND lo.remastering_alt_id IS NOT NULL AND lo.legacy = FALSE)
                AND ($15::bool IS NULL OR lc.haunt = $15)
                AND ($16::bool IS NULL OR lc.complex = $16)
            GROUP BY lo.id, lc.id ORDER BY similarity DESC, favor_exact_start_length, lo.name
            LIMIT $17 OFFSET $18
        ) c
        ORDER BY similarity DESC, favor_exact_start_length, c.name 
    "#,
        search.name,
        search.rarity.as_ref().map(|r| r.as_i64() as i32),
        search.game_system.as_ref().map(|gs| gs.as_i64() as i32),
        search.min_level.map(|r| r as i32),
        search.max_level.map(|r| r as i32),
        search.tags.first(), // TODO: Incorrect, only returning one tag.
        &ids as _,
        &search.query,
        min_similarity,
        search.favor_exact_start,
        search.legacy.include_remaster(),
        search.legacy.include_legacy(),
        search.legacy.favor_remaster(),
        search.legacy.favor_legacy(),
        search.haunt,
        search.complex,
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
                id: InternalId(row.id as u32),
                name: row.name,
                game_system: GameSystem::from_i64(row.game_system as i64),
                rarity: Rarity::from_i64(row.rarity.unwrap_or_default() as i64),
                complex: row.complex,
                haunt: row.haunt,
                level: row.level.unwrap_or_default() as i8,
                tags: row.tags.unwrap_or_default(),
                url: row.url,
                description: row.description.unwrap_or_default(),
                legacy: row.legacy,
            };
            map.entry(query)
                .or_default()
                .push((row.similarity.unwrap_or_default(), hazard));
            map
        });
    Ok(hazards)
}

pub async fn insert_hazards(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    hazards: &[InsertLibraryHazard],
) -> crate::Result<()> {
    // TODO: Do we *need* two tables for this?

    if hazards.is_empty() {
        return Ok(());
    }

    // First, check to make sure all requested ids are not already in use
    let requested_ids = hazards
        .iter()
        .filter_map(|i| i.requested_id)
        .map(|id| id.0 as i32)
        .collect::<Vec<i32>>();
    check_library_requested_ids(&mut **tx, &requested_ids).await?;

    let ids = sqlx::query!(
        r#"
        INSERT INTO library_objects (id, name, game_system, url, description, legacy, remastering_alt_id)
        SELECT * FROM UNNEST ($1::int[], $2::text[], $3::int[], $4::text[], $5::text[], $6::bool[], $7::int[])
        RETURNING id  
    "#,
        &hazards
            .iter()
            .map(|c| c.requested_id.map(|id| id.0 as i32))
            .collect::<Vec<Option<i32>>>() as _,
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
        &hazards
            .iter()
            .map(|c| c.legacy)
            .collect::<Vec<bool>>(),
        &hazards
            .iter()
            .map(|c| c.remastering_alt_id.map(|id| id.0 as i32))
            .collect::<Vec<Option<i32>>>() as _,
    )
    .fetch_all(&mut **tx)
    .await?
    .into_iter()
    .map(|row| Ok(row.id))
    .collect::<Result<Vec<i32>, sqlx::Error>>()?;

    // TODO: as i32 should be unnecessary- fix in models
    sqlx::query!(
        r#"
        INSERT INTO library_hazards (id, rarity, level, haunt, complex)
        SELECT * FROM UNNEST ($1::int[], $2::int[], $3::int[], $4::bool[], $5::bool[])
    "#,
        &ids.iter().map(|id| *id as i32).collect::<Vec<i32>>(),
        &hazards
            .iter()
            .map(|c| c.rarity.as_i64() as i32)
            .collect::<Vec<i32>>(),
        &hazards.iter().map(|c| c.level as i32).collect::<Vec<i32>>(),
        &hazards.iter().map(|c| c.haunt).collect::<Vec<bool>>(),
        &hazards.iter().map(|c| c.complex).collect::<Vec<bool>>(),
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}

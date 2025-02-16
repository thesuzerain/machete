use std::collections::HashMap;

use crate::models::library::{classes::LibraryClass, GameSystem, Rarity};

use crate::models::ids::InternalId;
use crate::ServerError;
use serde::{Deserialize, Serialize};

use super::{check_library_requested_ids, LegacyStatus, DEFAULT_MAX_LIMIT};

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct ClassFilters {
    pub name: Option<String>,
    pub legacy: LegacyStatus,

    pub limit: Option<u64>,
    pub page: Option<u64>,
}

// Full search functionality for class is maybe overkill, but if we extend to homebrew, etc, it may be useful.
#[derive(Default, Serialize, Deserialize, Debug)]
pub struct ClassSearch {
    pub query: Vec<String>,
    pub min_similarity: Option<f32>, // 0.0 to 1.0
    pub favor_exact_start: Option<bool>,

    // Custom/complex deserialization types, so we can't use #[flatten]
    // Page needs to be kept separate from flattened structure.
    // https://github.com/serde-rs/serde/issues/1183
    pub name: Option<String>,
    pub legacy: LegacyStatus,

    pub limit: Option<u64>,
    pub page: Option<u64>,
}

impl From<ClassFilters> for ClassSearch {
    fn from(filters: ClassFilters) -> Self {
        Self {
            query: vec!["".to_string()],
            name: filters.name,
            limit: filters.limit,
            page: filters.page,
            ..Default::default()
        }
    }
}

// TODO: 'Filterable' is kind of a mess
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct InsertLibraryClass {
    pub requested_id: Option<InternalId>,
    pub name: String,
    pub game_system: GameSystem,
    pub hp: u32,
    pub traditions: Vec<String>,
    pub rarity: Rarity,
    pub url: Option<String>,
    pub description: String,
    pub tags: Vec<String>,

    pub legacy: bool,
    pub remastering_alt_id: Option<InternalId>,
}

// TODO: May be prudent to make a separate models system for the database.
pub async fn get_classes(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    condition: &ClassFilters,
) -> crate::Result<Vec<LibraryClass>> {
    get_classes_search(
        exec,
        &ClassSearch::from(condition.clone()),
        DEFAULT_MAX_LIMIT,
    )
    .await?
    .into_iter()
    .next()
    .map(|(_, v)| v.into_iter().map(|(_, v)| v).collect())
    .ok_or_else(|| ServerError::NotFound)
}
// TODO: May be prudent to make a separate models system for the database.
pub async fn get_classes_search(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    // TODO: Could this use be problematic?
    // A postgres alternative can be found here:
    // https://github.com/launchbadge/sqlx/issues/291
    search: &ClassSearch,
    default_limit: u64,
) -> crate::Result<HashMap<String, Vec<(f32, LibraryClass)>>> {
    let limit = search.limit.unwrap_or(default_limit);
    let page = search.page.unwrap_or(0);
    let offset = page * limit;

    let query = sqlx::query!(
        r#"
        SELECT
            c.*, query as "query!"
        FROM UNNEST($2::text[]) query
        CROSS JOIN LATERAL (
            SELECT 
                -- If we favour exact start, we set similarity to 1.0 if the name starts with the query.
                CASE
                    WHEN $4::bool THEN 
                        CASE
                            WHEN lo.name ILIKE query || '%' THEN 1.01
                            WHEN lo.name ILIKE '%' || query || '%' THEN 1.0
                            ELSE SIMILARITY(lo.name, query)
                        END
                    ELSE SIMILARITY(lo.name, query)
                END AS similarity,
                CASE WHEN $4::bool THEN length(lo.name) ELSE 0 END AS favor_exact_start_length,
                lo.id,
                lo.name,
                lo.game_system,
                lo.url,
                lo.description,
                rarity,
                hp,
                traditions,
                ARRAY_AGG(DISTINCT tag) FILTER (WHERE tag IS NOT NULL) AS tags,
                legacy,
                remastering_alt_id
            FROM library_objects lo
            INNER JOIN library_classes lc ON lo.id = lc.id
            LEFT JOIN library_objects_tags lot ON lo.id = lot.library_object_id
            LEFT JOIN tags t ON lot.tag_id = t.id
            WHERE 1=1
                AND ($1::text IS NULL OR lo.name ILIKE '%' || $1 || '%')
                AND (($4::bool AND lo.name ILIKE '%' || query || '%') OR SIMILARITY(lo.name, query) >= $3)
                AND NOT (NOT $5::bool AND lo.legacy = FALSE)
                AND NOT (NOT $6::bool AND lo.legacy = TRUE)
                AND NOT ($7::bool AND lo.remastering_alt_id IS NOT NULL AND lo.legacy = FALSE)
                AND NOT ($8::bool AND lo.remastering_alt_id IS NOT NULL AND lo.legacy = TRUE)

            GROUP BY lo.id, lc.id ORDER BY similarity DESC, favor_exact_start_length, lo.name
            LIMIT $9 OFFSET $10
        ) c
        ORDER BY similarity DESC, favor_exact_start_length, c.name 
    "#,
        search.name,
        &search.query,
        search.min_similarity.unwrap_or(0.0),
        search.favor_exact_start.unwrap_or(false),
        search.legacy.include_remaster(),
        search.legacy.include_legacy(),
        search.legacy.favor_remaster(),
        search.legacy.favor_legacy(),
        limit as i64,
        offset as i64,
    );

    // create initial hm with empty vecs for each query
    let hm = search
        .query
        .iter()
        .map(|q| (q.clone(), Vec::new()))
        .collect::<HashMap<_, _>>();

    let classes = query.fetch_all(exec).await?.into_iter().fold(
        hm,
        |mut acc: HashMap<String, Vec<(f32, LibraryClass)>>, row| {
            let query = row.query;
            let class = LibraryClass {
                id: InternalId(row.id as u32),
                name: row.name,
                game_system: GameSystem::from_i64(row.game_system as i64),
                rarity: Rarity::from_i64(row.rarity as i64),
                tags: row.tags.unwrap_or_default(),
                hp: row.hp as u32,
                url: row.url,
                description: row.description.unwrap_or_default(),
                traditions: row.traditions,
                legacy: row.legacy,
                remastering_alt_id: row.remastering_alt_id.map(|id| InternalId(id as u32)),
            };

            acc.entry(query)
                .or_default()
                .push((row.similarity.unwrap_or_default(), class));
            acc
        },
    );
    Ok(classes)
}

pub async fn insert_classes(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres> + Copy,
    classes: &[InsertLibraryClass],
) -> crate::Result<()> {
    // TODO: Do we *need* two tables for this?

    if classes.is_empty() {
        return Ok(());
    }
    // First, check to make sure all requested ids are not already in use
    let requested_ids = classes
        .iter()
        .filter_map(|i| i.requested_id)
        .map(|id| id.0 as i32)
        .collect::<Vec<i32>>();
    check_library_requested_ids(exec, &requested_ids).await?;

    let ids = sqlx::query!(
        r#"
        INSERT INTO library_objects (id, name, game_system, url, description, legacy, remastering_alt_id)
        SELECT * FROM UNNEST ($1::int[], $2::text[], $3::int[], $4::text[], $5::text[], $6::bool[], $7::int[])
        RETURNING id  
    "#,
        &classes
            .iter()
            .map(|c| c.requested_id.map(|id| id.0 as i32))
            .collect::<Vec<Option<i32>>>() as _,
        &classes
            .iter()
            .map(|c| c.name.clone())
            .collect::<Vec<String>>(),
        &classes
            .iter()
            .map(|c| c.game_system.as_i64() as i32)
            .collect::<Vec<i32>>(),
        &classes
            .iter()
            .map(|c| c.url.clone())
            .collect::<Vec<Option<String>>>() as _,
        &classes
            .iter()
            .map(|c| c.description.clone())
            .collect::<Vec<String>>(),
        &classes
            .iter()
            .map(|c| c.legacy)
            .collect::<Vec<bool>>(),
        &classes
            .iter()
            .map(|c| c.remastering_alt_id.map(|id| id.0 as i32))
            .collect::<Vec<Option<i32>>>() as _,
    )
    .fetch_all(exec)
    .await?
    .into_iter()
    .map(|row| Ok(row.id))
    .collect::<Result<Vec<i32>, sqlx::Error>>()?;

    // TODO: Unfortunately, sqlx does not support multidimensional arrays
    // No built in way to UNNEST like in other insertion functions in postgres- unnest entirely flattens.
    // https://wiki.postgresql.org/wiki/Unnest_multidimensional_array
    // Unfortunately, sqlx does not support insertion of multidimensional arrays anyhow.
    // TODO: as i32 should be unnecessary- fix in models
    for (id, class) in classes.iter().enumerate() {
        sqlx::query!(
            r#"
            INSERT INTO library_classes (id, rarity, hp, traditions)
            VALUES ($1, $2, $3, $4)
        "#,
            ids[id] as i32,
            class.rarity.as_i64() as i32,
            class.hp as i32,
            &class.traditions,
        )
        .execute(exec)
        .await?;
    }

    Ok(())
}

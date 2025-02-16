use super::check_library_requested_ids;
use super::LegacyStatus;
use super::DEFAULT_MAX_LIMIT;
use crate::models::ids::InternalId;
use crate::models::library::{
    creature::{Alignment, LibraryCreature, Size},
    GameSystem, Rarity,
};
use crate::models::query::CommaSeparatedVec;
use crate::ServerError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct CreatureFiltering {
    // Custom/complex deserialization types, so we can't use #[flatten]
    // Page needs to be kept separate from flattened structure.
    // https://github.com/serde-rs/serde/issues/1183
    pub ids: Option<CommaSeparatedVec>,
    pub min_level: Option<i8>,
    pub max_level: Option<i8>,
    pub name: Option<String>,
    pub rarity: Option<Rarity>,
    pub alignment: Option<Alignment>,
    pub size: Option<Size>,
    pub game_system: Option<GameSystem>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub legacy: LegacyStatus,

    pub limit: Option<u64>,
    pub page: Option<u64>,
}

impl CreatureFiltering {
    pub fn from_id(id: u32) -> Self {
        CreatureFiltering {
            ids: Some(CommaSeparatedVec(vec![id])),
            ..Default::default()
        }
    }

    pub fn from_ids(ids: &[u32]) -> Self {
        CreatureFiltering {
            ids: Some(CommaSeparatedVec(ids.to_vec())),
            ..Default::default()
        }
    }
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct CreatureSearch {
    pub query: Vec<String>,
    pub min_similarity: Option<f32>, // 0.0 to 1.0
    pub favor_exact_start: Option<bool>,

    // Custom/complex deserialization types, so we can't use #[flatten]
    // Page needs to be kept separate from flattened structure.
    // https://github.com/serde-rs/serde/issues/1183
    pub ids: Option<CommaSeparatedVec>,
    pub min_level: Option<i8>,
    pub max_level: Option<i8>,
    pub name: Option<String>,
    pub rarity: Option<Rarity>,
    pub alignment: Option<Alignment>,
    pub size: Option<Size>,
    pub game_system: Option<GameSystem>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub legacy: LegacyStatus,

    pub page: Option<u64>,
    pub limit: Option<u64>,
}

impl From<CreatureFiltering> for CreatureSearch {
    fn from(filter: CreatureFiltering) -> Self {
        Self {
            query: vec!["".to_string()],
            name: filter.name,
            min_similarity: None,
            favor_exact_start: None,
            ids: filter.ids,
            min_level: filter.min_level,
            max_level: filter.max_level,
            rarity: filter.rarity,
            alignment: filter.alignment,
            size: filter.size,
            game_system: filter.game_system,
            tags: filter.tags,
            legacy: filter.legacy,
            limit: filter.limit,
            page: filter.page,
        }
    }
}

// TODO: 'Filterable' is kind of a mess
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct InsertLibraryCreature {
    pub requested_id: Option<InternalId>,
    pub name: String,
    pub game_system: GameSystem,
    pub rarity: Rarity,
    pub level: i8,
    pub tags: Vec<String>,
    pub alignment: Alignment,
    pub size: Size,
    pub legacy: bool,
    pub remastering_alt_id: Option<InternalId>,
    pub traits: Vec<String>,

    pub url: Option<String>,
    pub description: String,
}

// TODO: May be prudent to make a separate models system for the database.
pub async fn get_creatures(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    condition: &CreatureFiltering,
) -> crate::Result<Vec<LibraryCreature>> {
    get_creatures_search(
        exec,
        &CreatureSearch::from(condition.clone()),
        DEFAULT_MAX_LIMIT,
    )
    .await?
    .into_iter()
    .next()
    .map(|(_, v)| v.into_iter().map(|(_, v)| v).collect())
    .ok_or_else(|| ServerError::NotFound)
}

// TODO: May be prudent to make a separate models system for the database.
pub async fn get_creatures_search(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    // TODO: Could this use be problematic?
    // A postgres alternative can be found here:
    // https://github.com/launchbadge/sqlx/issues/291
    search: &CreatureSearch,
    default_limit: u64,
) -> crate::Result<HashMap<String, Vec<(f32, LibraryCreature)>>> {
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
                rarity,
                level,
                alignment,
                size,
                ARRAY_AGG(DISTINCT tag) FILTER (WHERE tag IS NOT NULL) AS tags,
                lo.legacy,
                lo.remastering_alt_id,
                traits
            FROM library_objects lo
            INNER JOIN library_creatures lc ON lo.id = lc.id
            LEFT JOIN library_objects_tags lot ON lo.id = lot.library_object_id
            LEFT JOIN tags t ON lot.tag_id = t.id

            WHERE 
                ($1::text IS NULL OR lo.name ILIKE '%' || $1 || '%')
                AND ($2::int IS NULL OR rarity = $2)
                AND ($3::int IS NULL OR game_system = $3)
                AND ($4::int IS NULL OR level >= $4)
                AND ($5::int IS NULL OR level <= $5)
                AND ($6::int IS NULL OR alignment = $6)
                AND ($7::int IS NULL OR size = $7)
                AND ($8::text IS NULL OR tag ILIKE '%' || $8 || '%')
                AND ($9::int[] IS NULL OR lo.id = ANY($9))
                AND (($12::bool AND lo.name ILIKE '%' || query || '%') OR SIMILARITY(lo.name, query) >= $11)
                AND NOT (NOT $13::bool AND lo.legacy = FALSE)
                AND NOT (NOT $14::bool AND lo.legacy = TRUE)
                AND NOT ($15::bool AND lo.remastering_alt_id IS NOT NULL AND lo.legacy = FALSE)
                AND NOT ($16::bool AND lo.remastering_alt_id IS NOT NULL AND lo.legacy = TRUE)

            GROUP BY lo.id, lc.id ORDER BY similarity DESC, favor_exact_start_length, lo.name
            LIMIT $17 OFFSET $18
        ) c
        ORDER BY similarity DESC, favor_exact_start_length, c.name 
    "#,
        search.name,
        search.rarity.as_ref().map(|r| r.as_i64() as i32),
        search.game_system.as_ref().map(|gs| gs.as_i64() as i32),
        search.min_level.map(|l| l as i32),
        search.max_level.map(|l| l as i32),
        search.alignment.as_ref().map(|a| a.as_i64() as i32),
        search.size.as_ref().map(|s| s.as_i64() as i32),
        search.tags.first(), // TODO: This is entirely incorrect, only returning one tag.
        &ids as _,
        &search.query,
        min_similarity,
        search.favor_exact_start.unwrap_or_default(),
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

    let creatures = query
        .fetch_all(exec)
        .await?
        .into_iter()
        .fold(hm, |mut map, row| {
            let query = row.query;
            let creature = LibraryCreature {
                id: InternalId(row.id as u32),
                name: row.name,
                game_system: GameSystem::from_i64(row.game_system as i64),
                rarity: Rarity::from_i64(row.rarity.unwrap_or_default() as i64),
                level: row.level.unwrap_or_default() as i8,
                alignment: Alignment::from_i64(row.alignment.unwrap_or_default() as i64),
                size: Size::from_i64(row.size.unwrap_or_default() as i64),
                tags: row.tags.unwrap_or_default(),
                url: row.url,
                description: row.description.unwrap_or_default(),
                legacy: row.legacy,
                remastering_alt_id: row.remastering_alt_id.map(|id| InternalId(id as u32)),
                traits: row.traits,
            };

            map.entry(query)
                .or_default()
                .push((row.similarity.unwrap_or_default(), creature));
            map
        });

    Ok(creatures)
}

pub async fn insert_creatures(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres> + Copy,
    creatures: &[InsertLibraryCreature],
) -> crate::Result<()> {
    // TODO: we don't need two tables for this.

    if creatures.is_empty() {
        return Ok(());
    }
    // First, check to make sure all requested ids are not already in use
    let requested_ids = creatures
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
        &creatures
            .iter()
            .map(|c| c.requested_id.map(|id| id.0 as i32))
            .collect::<Vec<Option<i32>>>() as _,
        &creatures
            .iter()
            .map(|c| c.name.clone())
            .collect::<Vec<String>>(),
        &creatures
            .iter()
            .map(|c| c.game_system.as_i64() as i32)
            .collect::<Vec<i32>>(),
        &creatures
            .iter()
            .map(|c| c.url.clone())
            .collect::<Vec<Option<String>>>() as _,
        &creatures
            .iter()
            .map(|c| c.description.clone())
            .collect::<Vec<String>>(),
        &creatures
            .iter()
            .map(|c| c.legacy)
            .collect::<Vec<bool>>(),
        &creatures
            .iter()
            .map(|c| c.remastering_alt_id.map(|id| id.0 as i32))
            .collect::<Vec<Option<i32>>>() as _,
    )
    .fetch_all(exec)
    .await?
    .into_iter()
    .map(|row| Ok(row.id))
    .collect::<Result<Vec<i32>, sqlx::Error>>()?;

    // TODO: annoying no unnest thing
    for (creature, id) in creatures.iter().zip(ids.iter()) {
        sqlx::query!(
            r#"
            INSERT INTO library_creatures (id, rarity, level, alignment, size, traits)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            id,
            creature.rarity.as_i64() as i32,
            creature.level as i32,
            creature.alignment.as_i64() as i32,
            creature.size.as_i64() as i32,
            &creature.traits,
        )
        .execute(exec)
        .await?;
    }

    Ok(())
}

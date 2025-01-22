use std::collections::HashMap;
use crate::models::ids::InternalId;
use crate::models::library::{
    creature::{Alignment, LibraryCreature, Size},
    GameSystem, Rarity,
};
use crate::ServerError;
use serde::{Deserialize, Serialize};
use crate::models::query::CommaSeparatedVec;
use super::DEFAULT_MAX_LIMIT;

// TODO: Consider (for this and others) moving the limit/page to both a separate struct AND 'search'. 
// They both use both fields the same way internally, but they mean slightly different things in the returned data, and we 
// sometimes need to apply limits midway through the pipeline.
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct CreatureFilters {
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

    pub limit: Option<u64>,
    pub page: Option<u64>,
}

impl CreatureFilters {
    pub fn from_id(id: u32) -> Self {
        CreatureFilters {
            ids: Some(CommaSeparatedVec(vec![id])),
            ..Default::default()
        }
    }

    pub fn from_ids(ids: &[u32]) -> Self {
        CreatureFilters {
            ids: Some(CommaSeparatedVec(ids.to_vec())),
            ..Default::default()
        }
    }
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct CreatureSearch {
    pub query: Vec<String>,
    pub min_similarity: Option<f32>, // 0.0 to 1.0

    #[serde(flatten)]
    pub filters: CreatureFilters,
}

// TODO: May be prudent to make a separate models system for the database.
pub async fn get_creatures(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    condition: &CreatureFilters,
) -> crate::Result<Vec<LibraryCreature>> {    
    get_creatures_search(exec, &CreatureSearch {
        query: vec!["".to_string()], // Empty search query
        min_similarity: None,
        filters: condition.clone(),
    }, DEFAULT_MAX_LIMIT).await?.into_iter().next()
    .map(|(_, v)| 
    v.into_iter().map(|(_, v)| v).collect()
).ok_or_else(|| ServerError::NotFound)
}

// TODO: May be prudent to make a separate models system for the database.
pub async fn get_creatures_search(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    // TODO: Could this use be problematic?
    // A postgres alternative can be found here:
    // https://github.com/launchbadge/sqlx/issues/291
    search: &CreatureSearch,
    default_limit: u64
) -> crate::Result<HashMap<String, Vec<(f32,LibraryCreature)>>> {
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

    let query = sqlx::query!(
        r#"
        SELECT
            c.*, query as "query!"
        FROM UNNEST($10::text[]) query
        CROSS JOIN LATERAL (
            SELECT 
                SIMILARITY(lo.name, query) AS similarity,
                lo.id,
                lo.name,
                lo.game_system,
                lo.url,
                lo.description,
                rarity,
                level,
                alignment,
                size,
                ARRAY_AGG(DISTINCT tag) FILTER (WHERE tag IS NOT NULL) AS tags
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
                AND SIMILARITY(lo.name, query) >= $11
            GROUP BY lo.id, lc.id ORDER BY similarity DESC
            LIMIT $12 OFFSET $13
        ) c
        ORDER BY similarity DESC
    "#,
        condition.name,
        condition.rarity.as_ref().map(|r| r.as_i64() as i32),
        condition.game_system.as_ref().map(|gs| gs.as_i64() as i32),
        condition.min_level.map(|l| l as i32),
        condition.max_level.map(|l| l as i32),
        condition.alignment.as_ref().map(|a| a.as_i64() as i32),
        condition.size.as_ref().map(|s| s.as_i64() as i32),
        condition.tags.first(), // TODO: This is entirely incorrect, only returning one tag.
        &ids as _,
        &search.query,
        min_similarity,
        limit as i64,
        offset as i64,
    ); 

    // create initial hm with empty vecs for each query
    let hm = search.query.iter().map(|q| (q.clone(), Vec::new())).collect::<HashMap<_,_>>();

    let creatures = query
        .fetch_all(exec)
        .await?
        .into_iter()
        .fold(hm,|mut map, row| {
            let query = row.query;
            let creature = LibraryCreature {
                id: InternalId(row.id as u64),
                name: row.name,
                game_system: GameSystem::from_i64(row.game_system as i64),
                rarity: Rarity::from_i64(row.rarity.unwrap_or_default() as i64),
                level: row.level.unwrap_or_default() as i8,
                alignment: Alignment::from_i64(row.alignment.unwrap_or_default() as i64),
                size: Size::from_i64(row.size.unwrap_or_default() as i64),
                tags: row.tags.unwrap_or_default(),
                url: row.url,
                description: row.description.unwrap_or_default(),
            };

            map.entry(query).or_insert_with(Vec::new).push((row.similarity.unwrap_or_default(),creature));
            map
        });

    Ok(creatures)
}

pub async fn insert_creatures(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres> + Copy,
    creatures: &[LibraryCreature],
) -> crate::Result<()> {
    // TODO: we don't need two tables for this.

    if creatures.is_empty() {
        return Ok(());
    }
    let ids = sqlx::query!(
        r#"
        INSERT INTO library_objects (name, game_system, url, description)
        SELECT * FROM UNNEST ($1::text[], $2::int[], $3::text[], $4::text[])
        RETURNING id  
    "#,
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
    )
    .fetch_all(exec)
    .await?
    .into_iter()
    .map(|row| Ok(row.id))
    .collect::<Result<Vec<i32>, sqlx::Error>>()?;

    // TODO: 'as i32' should be unnecessary- fix in models
    sqlx::query!(
        r#"
        INSERT INTO library_creatures (id, rarity, level, alignment, size)
        SELECT * FROM UNNEST ($1::int[], $2::int[], $3::int[], $4::int[], $5::int[])
        "#,
        &ids.iter().map(|id| *id as i32).collect::<Vec<i32>>(),
        &creatures
            .iter()
            .map(|c| c.rarity.as_i64() as i32)
            .collect::<Vec<i32>>(),
        &creatures
            .iter()
            .map(|c| c.level as i32)
            .collect::<Vec<i32>>(),
        &creatures
            .iter()
            .map(|c| c.alignment.as_i64() as i32)
            .collect::<Vec<i32>>(),
        &creatures
            .iter()
            .map(|c| c.size.as_i64() as i32)
            .collect::<Vec<i32>>(),
    )
    .execute(exec)
    .await?;

    Ok(())
}


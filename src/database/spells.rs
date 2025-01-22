use std::collections::HashMap;

use crate::models::ids::InternalId;
use crate::models::library::{spell::LibrarySpell, GameSystem, Rarity};
use crate::ServerError;

use serde::{Deserialize, Serialize};

use crate::models::query::CommaSeparatedVec;

use super::DEFAULT_MAX_LIMIT;

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct SpellFilters {
    pub ids: Option<CommaSeparatedVec>,
    pub min_rank: Option<u8>,
    pub max_rank: Option<u8>,
    pub name: Option<String>,
    pub rarity: Option<Rarity>,
    pub game_system: Option<GameSystem>,
    #[serde(default)]
    pub tags: Vec<String>,

    pub limit: Option<u64>,
    pub page: Option<u64>,
}

impl SpellFilters {
    pub fn from_id(id: u32) -> Self {
        Self {
            ids: Some(CommaSeparatedVec(vec![id])),
            ..Default::default()
        }
    }
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct SpellSearch {
    pub query: Vec<String>,
    pub min_similarity: Option<f32>, // 0.0 to 1.0

    #[serde(flatten)]
    pub filters: SpellFilters,
}

// TODO: May be prudent to make a separate models system for the database.
pub async fn get_spells(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    condition: &SpellFilters,
) -> crate::Result<Vec<LibrarySpell>> {    
    get_spells_search(exec, &SpellSearch {
        query: vec!["".to_string()], // Empty search query
        min_similarity: None,
        filters: condition.clone(),
    }, DEFAULT_MAX_LIMIT).await?.into_iter().next()
    .map(|(_, v)| 
    v.into_iter().map(|(_, v)| v).collect()
).ok_or_else(|| ServerError::NotFound)
}


// TODO: May be prudent to make a separate models system for the database.
pub async fn get_spells_search(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    // TODO: Could this use be problematic?
    // A postgres alternative can be found here:
    // https://github.com/launchbadge/sqlx/issues/291
    search: &SpellSearch,
    default_limit: u64,
) -> crate::Result<HashMap<String, Vec<(f32,LibrarySpell)>>> {
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
        FROM UNNEST($8::text[]) query
        CROSS JOIN LATERAL (
            SELECT 
                SIMILARITY(lo.name, query) AS similarity,
                lo.id,
                lo.name,
                lo.game_system,
                lo.url,
                lo.description,
                rarity,
                rank,
                traditions,
                ARRAY_AGG(DISTINCT tag) FILTER (WHERE tag IS NOT NULL) AS tags
            FROM library_objects lo
            INNER JOIN library_spells lc ON lo.id = lc.id
            LEFT JOIN library_objects_tags lot ON lo.id = lot.library_object_id
            LEFT JOIN tags t ON lot.tag_id = t.id

            WHERE 1=1
                AND ($1::text IS NULL OR lo.name ILIKE '%' || $1 || '%')
                AND ($2::int IS NULL OR rarity = $2)
                AND ($3::int IS NULL OR game_system = $3)
                AND ($4::int IS NULL OR rank >= $4)
                AND ($5::int IS NULL OR rank <= $5)
                AND ($6::text IS NULL OR tag ILIKE '%' || $6 || '%')
                AND ($7::int[] IS NULL OR lo.id = ANY($7))
                AND ($9::float IS NULL OR SIMILARITY(lo.name, query) >= $9)

            GROUP BY lo.id, lc.id ORDER BY lo.name
            LIMIT $10 OFFSET $11
        ) c
        ORDER BY similarity DESC
    "#,
        condition.name,
        condition.rarity.as_ref().map(|r| r.as_i64() as i32),
        condition.game_system.as_ref().map(|gs| gs.as_i64() as i32),
        condition.min_rank.map(|r| r as i32),
        condition.max_rank.map(|r| r as i32),
        condition.tags.first(), // TODO: Incorrect, only returning one tag.
        &ids as _,
        &search.query,
        min_similarity as f64,
        limit as i64,
        offset as i64,
    );

    // create initial hm with empty vecs for each query
    let hm = search.query.iter().map(|q| (q.clone(), Vec::new())).collect::<HashMap<_,_>>();
    let spells = query
        .fetch_all(exec)
        .await?
        .into_iter()
        .fold(hm, |mut map, row| {
            let query = row.query;
            let spell = LibrarySpell {
                id: InternalId(row.id as u64),
                name: row.name,
                game_system: GameSystem::from_i64(row.game_system as i64),
                rarity: Rarity::from_i64(row.rarity.unwrap_or_default() as i64),
                rank: row.rank.unwrap_or_default() as u8,
                tags: row.tags.unwrap_or_default(),
                url: row.url,
                description: row.description.unwrap_or_default(),
                traditions: row.traditions,
            };
            map.entry(query).or_insert_with(Vec::new).push((row.similarity.unwrap_or_default(), spell));
            map
        });
    Ok(spells)
}

pub async fn insert_spells(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres> + Copy,
    spells: &[LibrarySpell],
) -> crate::Result<()> {
    // TODO: Do we *need* two tables for this?

    if spells.is_empty() {
        return Ok(());
    }
    let ids = sqlx::query!(
        r#"
        INSERT INTO library_objects (name, game_system, url, description)
        SELECT * FROM UNNEST ($1::text[], $2::int[], $3::text[], $4::text[])
        RETURNING id  
    "#,
        &spells
            .iter()
            .map(|c| c.name.clone())
            .collect::<Vec<String>>(),
        &spells
            .iter()
            .map(|c| c.game_system.as_i64() as i32)
            .collect::<Vec<i32>>(),
        &spells
            .iter()
            .map(|c| c.url.clone())
            .collect::<Vec<Option<String>>>() as _,
        &spells
            .iter()
            .map(|c| c.description.clone())
            .collect::<Vec<String>>(),
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
    for (id, spell) in spells.iter().enumerate() {
        sqlx::query!(
            r#"
            INSERT INTO library_spells (id, rarity, rank, traditions)
            VALUES ($1, $2, $3, $4)
        "#,
            ids[id] as i32,
            spell.rarity.as_i64() as i32,
            spell.rank as i32,
            &spell.traditions,
        )
        .execute(exec)
        .await?;
    }

    Ok(())
}

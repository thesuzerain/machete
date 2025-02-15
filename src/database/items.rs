use std::collections::HashMap;

use crate::models::characters::Stat;
use crate::models::ids::InternalId;
use crate::models::library::item::{Rune, RuneItemType, SkillPotency};
use crate::models::library::{item::LibraryItem, GameSystem, Rarity};

use crate::models::query::CommaSeparatedVec;
use crate::ServerError;
use serde::{Deserialize, Serialize};

use super::{LegacyStatus, DEFAULT_MAX_LIMIT};

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
    pub legacy: LegacyStatus,

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

// TODO: Add tests for this to make sure we dont add oneto search and not filters
//eg json deserialization and serialization

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
    pub legacy: LegacyStatus,

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
            legacy: filter.legacy,
            tags: filter.tags,
            limit: filter.limit,
            page: filter.page,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct InsertLibraryItem {
    pub name: String,
    pub game_system: GameSystem,
    pub rarity: Rarity,
    pub level: i8,
    pub tags: Vec<String>,
    pub price: Option<f64>,

    pub url: Option<String>,
    pub description: String,
    pub item_categories : Vec<String>,
    pub traits: Vec<String>,
    pub consumable: bool,
    pub magical: bool,
    pub legacy: bool,
    pub item_type: RuneItemType,
    pub skill_boosts: Vec<SkillPotency>,
    pub runes : Vec<Rune>,
    pub apex_stat: Option<Stat>,

    // Extra insertion-specific fields
    pub runic_context: Option<InsertRune>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct InsertRune {
    pub runic_stat_boost_category_bonus_id : Option<i32>,// If a rune, what stat boost category does it belong to?
    pub potency : i8,
    pub base_rune: Option<String>, // If a rune, what other name should it be
    pub applied_to: RuneItemType, // What is this rune applied to?
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
                ARRAY_AGG(DISTINCT tag) FILTER (WHERE tag IS NOT NULL) AS tags,
                li.item_categories,
                li.traits,
                li.consumable,
                li.magical,
                li.item_type,
                li.apex_stat,
                li.legacy,
        
                JSON_AGG(JSON_BUILD_OBJECT('name', r.name, 'potency', r.potency)) FILTER (WHERE r.potency IS NOT NULL) AS runes,
                JSON_AGG(JSON_BUILD_OBJECT('skill', sb.skill, 'bonus', sb.bonus)) FILTER (WHERE sb.bonus IS NOT NULL) AS skill_boosts
            FROM library_objects lo
            INNER JOIN library_items li ON lo.id = li.id
            LEFT JOIN library_objects_tags lot ON lo.id = lot.library_object_id
            LEFT JOIN tags t ON lot.tag_id = t.id
            LEFT JOIN library_items_runes lir ON lo.id = lir.item_id
            LEFT JOIN runes r ON lir.rune_id = r.id
            LEFT JOIN library_items_skill_boosts sb ON lo.id = sb.item_id

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
                AND NOT (NOT $13::bool AND li.legacy = FALSE)
                AND NOT (NOT $14::bool AND li.legacy = TRUE)

                -- TODO: Do some tests on this, this might be a bad pattern
                -- Or at least- maybe requires a WHERE = name + index?
                AND (NOT $15::bool OR NOT li.legacy OR
                    lo.name NOT IN (
                        SELECT name
                        FROM library_objects inner_lo
                        INNER JOIN library_items inner_li ON inner_lo.id = inner_li.id
                        WHERE inner_li.legacy = TRUE
                    )
                )
            GROUP BY lo.id, li.id ORDER BY similarity DESC, favor_exact_start_length, lo.name
            LIMIT $16 OFFSET $17
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
        search.legacy.include_remaster(),
        search.legacy.include_legacy(),
        search.legacy.favor_remaster(),
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

        let runes : Option<Vec<serde_json::Value>> = serde_json::from_value(row.runes.unwrap_or_default()).unwrap_or_default();
        let runes = runes.unwrap_or_default().into_iter().map(|r| {
            let name = r["name"].as_str().unwrap_or_default().to_string();
            let potency = r["potency"].as_i64().unwrap_or_default() as i8;
            let rune_item_type = r["item_type"].as_str().unwrap_or_default();
            Rune::from_parts(&name, potency, RuneItemType::from_str(rune_item_type))
        }).collect();

        let item = LibraryItem {
            id: InternalId(row.id as u64),
            name: row.name,
            game_system: GameSystem::from_i64(row.game_system as i64),
            rarity: Rarity::from_i64(row.rarity.unwrap_or_default() as i64),
            level: row.level.unwrap_or_default() as i8,
            price: row.price,
            tags: row.tags.unwrap_or_default(),
            url: row.url,
            description: row.description.unwrap_or_default(),
            item_categories: row.item_categories,
            traits: row.traits,
            consumable: row.consumable,
            legacy: row.legacy,
            magical: row.magical,
            item_type: RuneItemType::from_str(&row.item_type.unwrap_or_default()),
            skill_boosts: serde_json::from_value(row.skill_boosts.unwrap_or_default()).unwrap_or_default(),
            apex_stat: Stat::from_str(&row.apex_stat.unwrap_or_default()),
            runes,

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
    items: &[InsertLibraryItem],
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

    // TODO: Unnest problem with arrays again
    for (item, id) in items.iter().zip(ids.iter()) {
        let apex_stat = item.apex_stat.as_ref().map(|s| s.to_string());
                sqlx::query!(
            r#"
            INSERT INTO library_items (id, rarity, level, price, item_categories, traits, consumable, magical, item_type, apex_stat, legacy)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        "#,
            id,
            item.rarity.as_i64() as i32,
            item.level as i32,
            item.price,
            &item.item_categories,
            &item.traits,
            item.consumable,
            item.magical,
            &item.item_type.to_string(),
            apex_stat.as_ref(),
            item.legacy,
        )
        .execute(exec)
        .await?;
        
        // Insert runes, if it is a rune
        if let Some(runic_context) = &item.runic_context {
            let rune_category_id = runic_context.runic_stat_boost_category_bonus_id.as_ref().map(|id| *id);
            let potency = runic_context.potency;
            // Use basename if it exists, for consistency between legacy and non-legacy items
            let rune_base_name = runic_context.base_rune.as_ref().map(|s| s.as_str()).unwrap_or(&item.name);
            let applied_to = runic_context.applied_to;

            sqlx::query!(
                r#"
                INSERT INTO runes (item_id, name, fundamental, stat_boost_category_id, legacy, potency, applied_to_item_type)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
                *id as i32,
                rune_base_name,
                matches!(item.item_type, RuneItemType::FundamentalRune),
                rune_category_id.map(|i| i as i16),
                item.legacy,
                potency as i16,
                applied_to.to_string(),
            )
            .execute(exec)
            .await?;
        } else {
            // Insert any runes that this may have!
            sqlx::query!(
                r#"
                INSERT INTO library_items_runes (item_id, rune_id)
                SELECT
                    $1::int as item_id,
                    r.id as rune_id
                FROM UNNEST($2::text[], $3::int[]) insertion(name, potency)
                CROSS JOIN LATERAL (
                    SELECT r.id
                    FROM runes r, library_items li
                    WHERE lower(r.name) = lower(insertion.name) AND li.id = $1 AND li.legacy = r.legacy
                    AND r.potency = insertion.potency
                ) r
            "#,
                *id as i32,
                &item.runes.iter().map(|r| r.to_parts().0).collect::<Vec<String>>(),
                &item.runes.iter().map(|r| r.to_parts().1 as i32).collect::<Vec<i32>>(),
            
            )
            .fetch_all(exec)
            .await?;
        }

        // Insert stat boosts
        let skills = item.skill_boosts.iter().map(|sb| sb.skill.clone().map(|s| s.to_string())).collect::<Vec<Option<String>>>();
        sqlx::query!(
            r#"
            INSERT INTO library_items_skill_boosts (item_id, skill, bonus)
            SELECT $1, skill, bonus FROM UNNEST ($2::text[], $3::int[]) skill_boosts(skill, bonus)
        "#,
            *id as i32,
            skills.as_slice() as _,
            &item.skill_boosts.iter().map(|sb| sb.bonus as i32).collect::<Vec<i32>>(),
        )
        .execute(exec)
        .await?;
        
    }

    Ok(())

}

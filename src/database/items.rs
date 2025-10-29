use std::collections::HashMap;

use crate::models::characters::Stat;
use crate::models::ids::InternalId;
use crate::models::library::item::{Rune, RuneItemType, SkillPotency};
use crate::models::library::{item::LibraryItem, GameSystem, Rarity};

use crate::models::query::CommaSeparatedVec;
use crate::ServerError;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use sqlx::PgConnection;

use super::sorts::{Sortable, SortableColumn};
use super::{check_library_requested_ids, tags, LegacyStatus, DEFAULT_MAX_LIMIT};

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
    pub traits_all: Option<Vec<String>>,
    pub traits_any: Option<Vec<String>>,
    #[serde(default)]
    pub legacy: LegacyStatus,
    pub relic_gift: Option<bool>,
    pub consumable: Option<bool>,
    pub magical: Option<bool>,
    pub cursed: Option<bool>,

    pub limit: Option<u64>,
    pub page: Option<u64>,
    pub sort_by: Option<String>,
    pub order_by: Option<String>,
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
    pub traits_all: Option<Vec<String>>,
    pub traits_any: Option<Vec<String>>,
    #[serde(default)]
    pub legacy: LegacyStatus,

    pub relic_gift: Option<bool>,
    pub consumable: Option<bool>,
    pub magical: Option<bool>,
    pub cursed: Option<bool>,

    pub limit: Option<u64>,
    pub page: Option<u64>,
    pub sort_by: Option<String>,
    pub order_by: Option<String>,
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
            traits_all: filter.traits_all,
            traits_any: filter.traits_any,
            limit: filter.limit,
            page: filter.page,
            relic_gift: filter.relic_gift,
            consumable: filter.consumable,
            magical: filter.magical,
            cursed: filter.cursed,
            sort_by: filter.sort_by,
            order_by: filter.order_by,
        }
    }
}

impl Sortable for LibraryItem {
    fn get_allowed_fields() -> &'static [&'static str] {
        &["name", "level", "price", "rarity"]
    }

    fn default_sort() -> Option<&'static str> {
        Some("name")
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct InsertLibraryItem {
    pub requested_id: Option<InternalId>,
    pub name: String,
    pub game_system: GameSystem,
    pub rarity: Rarity,
    pub level: i8,
    pub price: Option<f64>,

    pub url: Option<String>,
    pub description: String,
    pub item_categories: Vec<String>,
    pub tags: Vec<InternalId>,
    pub consumable: bool,
    pub magical: bool,
    pub legacy: bool,
    pub cursed: bool,
    pub relic_gift_stage: Option<i8>,
    pub item_type: RuneItemType,
    pub skill_boosts: Vec<SkillPotency>,
    pub runes: Vec<Rune>,
    pub apex_stat: Option<Stat>,
    pub remastering_alt_id: Option<InternalId>,
    pub base_item_id: Option<InternalId>,

    // Extra insertion-specific fields
    pub runic_context: Option<InsertRune>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct InsertRune {
    pub runic_stat_boost_category_bonus_id: Option<i32>, // If a rune, what stat boost category does it belong to?
    pub potency: i8,
    pub base_rune: Option<String>, // If a rune, what other name should it be
    pub applied_to: RuneItemType,  // What is this rune applied to?
}

pub async fn get_items(
    conn: &mut PgConnection,
    condition: &ItemFiltering,
) -> crate::Result<Vec<LibraryItem>> {
    get_items_search(
        conn,
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
    conn: &mut PgConnection,
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
    let sort = SortableColumn::<LibraryItem>::try_parse(
        search.sort_by.as_deref(),
        search.order_by.as_deref(),
    )?;

    let min_similarity = search.min_similarity.unwrap_or(0.0);

    let ids = search.ids.clone().map(|t| {
        t.into_inner()
            .into_iter()
            .map(|id| id as i32)
            .collect::<Vec<i32>>()
    });

    let matching_tags =
        tags::get_tag_matches(&mut *conn, &search.traits_all, &search.traits_any).await?;
    // TODO: data type 'as'
    // TODO: Consider meilisearch/elasticsearch for this
    let query = sqlx::query!(
        r#"
        SELECT
            c.*, query as "query!"
        FROM UNNEST($11::text[]) query
        CROSS JOIN LATERAL (
            SELECT 
                -- If we favour exact start, we set similarity to 1.0 if the name starts with the query.
                CASE
                    WHEN $13::bool THEN 
                        CASE
                            WHEN lo.name ILIKE query || '%' THEN 1.01
                            WHEN lo.name ILIKE '%' || query || '%' THEN 1.0
                            ELSE SIMILARITY(lo.name, query)
                        END
                    ELSE SIMILARITY(lo.name, query)
                END AS similarity,
                CASE WHEN $13::bool THEN length(lo.name) ELSE 0 END AS favor_exact_start_length,
                lo.id,
                lo.name,
                lo.game_system,
                lo.url,
                lo.description,
                li.rarity,
                li.level,
                li.price,
                li.item_categories,
                any_value(tags.tags) AS tags,
                any_value(tags.traits) AS traits,
                li.consumable,
                li.magical,
                li.cursed,
                li.relic_gift_stage,
                li.item_type,
                li.apex_stat,
                lo.legacy,
                lo.remastering_alt_id,
                JSON_AGG(JSON_BUILD_OBJECT('name', r.name, 'potency', r.potency)) FILTER (WHERE r.potency IS NOT NULL) AS runes,
                JSON_AGG(JSON_BUILD_OBJECT('skill', sb.skill, 'bonus', sb.bonus)) FILTER (WHERE sb.bonus IS NOT NULL) AS skill_boosts
            FROM library_objects lo
            INNER JOIN library_items li ON lo.id = li.id
            LEFT JOIN (
                SELECT
                    library_object_id AS lo_id,
                    ARRAY_AGG(t.tag) FILTER (WHERE t.trait) AS traits,
                    ARRAY_AGG(t.tag) FILTER (WHERE NOT t.trait) AS tags
                FROM library_objects_tags lot
                INNER JOIN library_tags t ON lot.tag_id = t.id
                GROUP BY lot.library_object_id
            ) AS tags ON lo.id = tags.lo_id
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
                AND ($8::text[] IS NULL OR tags.traits::text[] && $8::text[])
                AND ($9::text[] IS NULL OR tags.traits::text[] @> $9::text[])
                AND ($10::int[] IS NULL OR lo.id = ANY($10))
                AND (($13::bool AND lo.name ILIKE '%' || query || '%') OR SIMILARITY(lo.name, query) >= $12)
                AND NOT (NOT $14::bool AND lo.legacy = FALSE)
                AND NOT (NOT $15::bool AND lo.legacy = TRUE)
                AND NOT ($16::bool AND lo.remastering_alt_id IS NOT NULL AND lo.legacy = TRUE)
                AND NOT ($17::bool AND lo.remastering_alt_id IS NOT NULL AND lo.legacy = FALSE)
                AND ($18::bool IS NULL OR ($18::bool AND li.relic_gift_stage IS NOT NULL) OR ($18::bool = FALSE AND li.relic_gift_stage IS NULL))
                AND ($19::bool IS NULL OR ($19::bool AND li.consumable = TRUE) OR ($19::bool = FALSE AND li.consumable = FALSE))
                AND ($20::bool IS NULL OR ($20::bool AND li.magical = TRUE) OR ($20::bool = FALSE AND li.magical = FALSE))
                AND ($21::bool IS NULL OR ($21::bool AND li.cursed = TRUE) OR ($21::bool = FALSE AND li.cursed = FALSE))
            GROUP BY lo.id, li.id 
            ORDER BY similarity DESC, favor_exact_start_length,
                CASE WHEN $22::text = 'name' AND $23::int = 1 THEN lo.name::text END ASC,
                CASE WHEN $22::text = 'name' AND $23::int = -1 THEN lo.name::text END DESC,
                CASE WHEN $22::text = 'level' THEN level::integer * $23::int END ASC,
                CASE WHEN $22::text = 'price' THEN price::integer * $23::int END ASC,
                CASE WHEN $22::text = 'rarity' THEN li.rarity::integer * $23::int END ASC
            LIMIT $24 OFFSET $25
        ) c
        ORDER BY similarity DESC, favor_exact_start_length 
    "#,
        search.name,
        search.rarity.as_ref().map(|r| r.as_i64() as i32),
        search.game_system.as_ref().map(|gs| gs.as_i64() as i32),
        search.min_level.map(|l| l as i32),
        search.max_level.map(|l| l as i32),
        search.min_price.map(|p| p as i32),
        search.max_price.map(|p| p as i32),
        matching_tags.any_traits.as_deref() as _,
        matching_tags.all_traits.as_deref() as _,
        &ids as _,
        &search.query,
        min_similarity,
        search.favor_exact_start,
        search.legacy.include_remaster(),
        search.legacy.include_legacy(),
        search.legacy.favor_remaster(),
        search.legacy.favor_legacy(),
        search.relic_gift,
        search.consumable,
        search.magical,
        search.cursed,
        sort.get_column(),
        sort.get_sort_direction_i32(),
        limit as i64,
        offset as i64,
    )
    .fetch_all(&mut *conn)
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

        let runes: Option<Vec<serde_json::Value>> =
            serde_json::from_value(row.runes.unwrap_or_default()).unwrap_or_default();
        let runes = runes
            .unwrap_or_default()
            .into_iter()
            .map(|r| {
                let name = r["name"].as_str().unwrap_or_default().to_string();
                let potency = r["potency"].as_i64().unwrap_or_default() as i8;
                let rune_item_type = r["item_type"].as_str().unwrap_or_default();
                Rune::from_parts(&name, potency, RuneItemType::from_str(rune_item_type))
            })
            .collect();

        let item = LibraryItem {
            id: InternalId(row.id as u32),
            name: row.name,
            game_system: GameSystem::from_i64(row.game_system as i64),
            rarity: Rarity::from_i64(row.rarity.unwrap_or_default() as i64),
            level: row.level.unwrap_or_default() as i8,
            price: row.price,
            tags: row.tags.unwrap_or_default(),
            url: row.url,
            description: row.description.unwrap_or_default(),
            item_categories: row.item_categories,
            traits: row.traits.unwrap_or_default(),
            consumable: row.consumable,
            legacy: row.legacy,
            magical: row.magical,
            cursed: row.cursed,
            relic_gift_stage: row.relic_gift_stage.map(|i| i as i8),
            item_type: RuneItemType::from_str(&row.item_type.unwrap_or_default()),
            skill_boosts: serde_json::from_value(row.skill_boosts.unwrap_or_default())
                .unwrap_or_default(),
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
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    items: &[InsertLibraryItem],
) -> crate::Result<()> {
    // TODO: Don't *need* two tables for this
    if items.is_empty() {
        return Ok(());
    }

    // First, check to make sure all requested ids are not already in use
    let requested_ids = items
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
        &items
            .iter()
            .map(|c| c.requested_id.as_ref().map(|id| id.0 as i32))
            .collect::<Vec<Option<i32>>>() as _,
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
        &items
            .iter()
            .map(|c| c.legacy)
            .collect::<Vec<bool>>(),
        &items
            .iter()
            .map(|c| c.remastering_alt_id.as_ref().map(|id| id.0 as i32))
            .collect::<Vec<Option<i32>>>() as _,
    )
    .fetch_all(&mut **tx)
    .await?
    .into_iter()
    .map(|row| Ok(row.id))
    .collect::<Result<Vec<i32>, sqlx::Error>>()?;

    // TODO: Unnest problem with arrays again
    for (item, id) in items.iter().zip(ids.iter()) {
        let apex_stat = item.apex_stat.as_ref().map(|s| s.to_string());
        sqlx::query!(
            r#"
            INSERT INTO library_items (id, rarity, level, price, item_categories, consumable, magical, cursed, relic_gift_stage, item_type, apex_stat)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        "#,
            id,
            item.rarity.as_i64() as i32,
            item.level as i32,
            item.price,
            &item.item_categories,
            item.consumable,
            item.magical,
            item.cursed,
            item.relic_gift_stage.map(|i| i as i32),
            &item.item_type.to_string(),
            apex_stat.as_ref(),
        )
        .execute(&mut **tx)
        .await?;

        // Insert runes, if it is a rune
        if let Some(runic_context) = &item.runic_context {
            let rune_category_id = runic_context
                .runic_stat_boost_category_bonus_id
                .as_ref()
                .map(|id| *id);
            let potency = runic_context.potency;
            // Use basename if it exists, for consistency between legacy and non-legacy items
            let rune_base_name = runic_context.base_rune.as_deref().unwrap_or(&item.name);
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
            .execute(&mut **tx)
            .await?;
        } else {
            // Insert any runes that this may have!
            // We do so by matching names + potency. We also match equivalent legacy status if possible.
            sqlx::query!(
                r#"
                INSERT INTO library_items_runes (item_id, rune_id)
                SELECT
                    $1::int as item_id,
                    r.id as rune_id
                FROM UNNEST($2::text[], $3::int[]) insertion(name, potency)
                CROSS JOIN LATERAL (
                    SELECT DISTINCT ON (r.name) r.id
                    FROM runes r, library_objects lo
                    WHERE lower(r.name) = lower(insertion.name) AND lo.id = $1
                    AND r.potency = insertion.potency
                    ORDER BY r.name, r.legacy != lo.legacy  -- Prefer matching legacy status
                ) r
            "#,
                *id as i32,
                &item
                    .runes
                    .iter()
                    .map(|r| r.to_parts().0)
                    .collect::<Vec<String>>(),
                &item
                    .runes
                    .iter()
                    .map(|r| r.to_parts().1 as i32)
                    .collect::<Vec<i32>>(),
            )
            .fetch_all(&mut **tx)
            .await?;
        }

        // Insert stat boosts
        let skills = item
            .skill_boosts
            .iter()
            .map(|sb| sb.skill.clone().to_string())
            .collect::<Vec<String>>();
        sqlx::query!(
            r#"
            INSERT INTO library_items_skill_boosts (item_id, skill, bonus)
            SELECT $1, skill, bonus FROM UNNEST ($2::text[], $3::int[]) skill_boosts(skill, bonus)
        "#,
            *id as i32,
            skills.as_slice() as _,
            &item
                .skill_boosts
                .iter()
                .map(|sb| sb.bonus as i32)
                .collect::<Vec<i32>>(),
        )
        .execute(&mut **tx)
        .await?;

        // Insert tags
        sqlx::query!(
            r#"
            INSERT INTO library_objects_tags (library_object_id, tag_id)
            SELECT $1, tag_id FROM UNNEST ($2::int[]) tag_ids(tag_id)
            "#,
            *id as i32,
            &item
                .tags
                .iter()
                .map(|id| id.0 as i32)
                .sorted()
                .collect::<Vec<i32>>(),
        )
        .execute(&mut **tx)
        .await?;
    }

    Ok(())
}

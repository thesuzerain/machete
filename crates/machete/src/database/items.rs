use std::collections::HashMap;

use crate::models::library::{
    item::{Currency, ItemFilters, LibraryItem},
    GameSystem, Rarity,
};
use machete_core::filters::Filter;

use super::QueryableStruct;

// TODO: May be prudent to make a separate models system for the database.
pub async fn get_items(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    // TODO: Could this use be problematic?
    // A postgres alternative can be found here:
    // https://github.com/launchbadge/sqlx/issues/291
    condition: &ItemFilters,
) -> crate::Result<Vec<LibraryItem>> {
    // TODO: This doesn't use sqlx::query! because it needs to be dynamic. Is there a better way to do this?
    // TODO: data type 'as'
    let query = sqlx::query!(
        r#"
        SELECT 
            lo.id,
            lo.name,
            lo.game_system,
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
        
        GROUP BY lo.id, li.id ORDER BY lo.name
    "#,
        condition.name,
        condition.rarity.as_ref().map(|r| r.as_i64() as i32),
        condition.game_system.as_ref().map(|gs| gs.as_i64() as i32),
        condition.min_level.map(|l| l as i32),
        condition.max_level.map(|l| l as i32),
        condition.min_price.map(|p| p as i32),
        condition.max_price.map(|p| p as i32),
        condition.tags.first(), // TODO: bad
    )
    .fetch_all(exec)
    .await?;

    let items = query
        .into_iter()
        .map(|row| {
            // TODO: conversions still here shouldnt be needed
            // TODO: unwrao_or_default for stuff like rarity / price / level is a bit weird
            Ok(LibraryItem {
                name: row.name,
                game_system: GameSystem::from_i64(row.game_system as i64),
                rarity: Rarity::from_i64(row.rarity.unwrap_or_default() as i64),
                level: row.level.unwrap_or_default() as i8,
                price: Currency::from_base_unit(row.price.unwrap_or_default() as u32),
                tags: row.tags.unwrap_or_default(),
            })
        })
        .collect::<Result<Vec<LibraryItem>, sqlx::Error>>()?;
    Ok(items)
}

pub async fn insert_items(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres> + Copy,
    items: Vec<LibraryItem>,
    tag_hashmap: HashMap<String, i32>,
) -> crate::Result<()> {
    // TODO: This doesn't use sqlx::query! because it needs to be dynamic. Is there a better way to do this?
    // Maybe postgres + unnest as in labrinth?
    // TODO: Do we *need* two tables for this?
    let ids = sqlx::query!(
        r#"
        INSERT INTO library_objects (name, game_system)
        SELECT * FROM UNNEST ($1::text[], $2::int[])  
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

    // Next, insert tags

    for (id, item) in ids.iter().zip(items.iter()) {
        // separate builders to not hit limit
        // todo: no :()
        sqlx::query!(
            r#"
            INSERT INTO library_objects_tags (library_object_id, tag_id)
            SELECT * FROM UNNEST ($1::int[], $2::int[])
            "#,
            &vec![*id as i32; item.tags.len()],
            &item
                .tags
                .iter()
                .map(|tag| tag_hashmap.get(tag).unwrap().clone() as i32)
                .collect::<Vec<i32>>(),
        )
        .execute(exec)
        .await?;
    }

    Ok(())
}

impl QueryableStruct for LibraryItem {
    // TODO: bundle with macro?
    async fn query_get<'a>(
        exec: impl sqlx::Executor<'a, Database = sqlx::Postgres>,
        filters: &Vec<Filter<LibraryItem>>,
    ) -> crate::Result<Vec<LibraryItem>> {
        let mut item_filters = ItemFilters::default();
        for filter in filters {
            // TODO: clone
            let filter = filter.clone();
            if let Ok(filter) = ItemFilters::try_from(filter) {
                item_filters = item_filters.merge(filter);
            }
        }

        get_items(exec, &item_filters).await
    }
}

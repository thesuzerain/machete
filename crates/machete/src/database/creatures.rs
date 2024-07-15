use std::collections::HashMap;

use super::QueryableStruct;
use crate::models::library::{
    creature::{Alignment, CreatureFilters, LibraryCreature, Size},
    GameSystem, Rarity,
};
use machete_core::filters::Filter;

// TODO: May be prudent to make a separate models system for the database.
pub async fn get_creatures(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    // TODO: Could this use be problematic?
    // A postgres alternative can be found here:
    // https://github.com/launchbadge/sqlx/issues/291
    condition: &CreatureFilters,
) -> crate::Result<Vec<LibraryCreature>> {
    // TODO: This doesn't use sqlx::query! because it needs to be dynamic. Is there a better way to do this?
    let query = sqlx::query!(
        r#"
        SELECT 
            lo.id,
            lo.name,
            lo.game_system,
            rarity,
            level,
            alignment,
            size,
            ARRAY_AGG(tag) AS tags
        FROM library_objects lo
        INNER JOIN library_creatures lc ON lo.id = lc.id
        LEFT JOIN library_objects_tags lot ON lo.id = lot.library_object_id
        LEFT JOIN tags t ON lot.tag_id = t.id

        WHERE 
            ($1::text IS NULL OR lo.name LIKE '%' || $1 || '%')
            AND ($2::int IS NULL OR rarity = $2)
            AND ($3::int IS NULL OR game_system = $3)
            AND ($4::int IS NULL OR level >= $4)
            AND ($5::int IS NULL OR level <= $5)
            AND ($6::int IS NULL OR alignment = $6)
            AND ($7::int IS NULL OR size = $7)
            AND ($8::text IS NULL OR tag LIKE '%' || $8 || '%')
        
        GROUP BY lo.id, lc.id ORDER BY lo.name
    "#,
        condition.name,
        condition.rarity.as_ref().map(|r| r.as_i64() as i32),
        condition.game_system.as_ref().map(|gs| gs.as_i64() as i32),
        condition.min_level.map(|l| l as i32),
        condition.max_level.map(|l| l as i32),
        condition.alignment.as_ref().map(|a| a.as_i64() as i32),
        condition.size.as_ref().map(|s| s.as_i64() as i32),
        condition.tags.first(), // TODO: BAD
    );

    let creatures = query
        .fetch_all(exec)
        .await?
        .into_iter()
        .map(|row| {
            Ok(LibraryCreature {
                name: row.name,
                game_system: GameSystem::from_i64(row.game_system as i64),
                rarity: Rarity::from_i64(row.rarity.unwrap_or_default() as i64),
                level: row.level.unwrap_or_default() as i8,
                alignment: Alignment::from_i64(row.alignment.unwrap_or_default() as i64),
                size: Size::from_i64(row.size.unwrap_or_default() as i64),
                tags: row.tags.unwrap_or_default(),
            })
        })
        .collect::<Result<Vec<LibraryCreature>, sqlx::Error>>()?;
    Ok(creatures)
}

pub async fn insert_creatures(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres> + Copy,
    creatures: Vec<LibraryCreature>,
    // not sure if i like this patern, but if we are keeping it, document it
    tag_hashmap: HashMap<String, i32>,
) -> crate::Result<()> {
    // TODO: This doesn't use sqlx::query! because it needs to be dynamic. Is there a better way to do this?
    // Maybe postgres + unnest as in labrinth?
    // TODO: Do we *need* two tables for this?

    // TODO: i32
    let ids = sqlx::query!(
        r#"
        INSERT INTO library_objects (name, game_system)
        SELECT * FROM UNNEST ($1::text[], $2::int[])  
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
    )
    .fetch_all(exec)
    .await?
    .into_iter()
    .map(|row| Ok(row.id))
    .collect::<Result<Vec<i32>, sqlx::Error>>()?;

    // TODO: as i32 should be unnecessary- fix in models
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

    // Next, insert tags
    for (id, creature) in ids.iter().zip(creatures.iter()) {
        // separate builders to not hit limit
        // todo: no :()

        sqlx::query!(
            r#"
            INSERT INTO library_objects_tags (library_object_id, tag_id)
            SELECT * FROM UNNEST ($1::int[], $2::int[])
            "#,
            &vec![*id as i32; creature.tags.len()],
            &creature
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

impl QueryableStruct for LibraryCreature {
    async fn query_get(
        pool: sqlx::Pool<sqlx::Postgres>,
        filters: &Vec<Filter<LibraryCreature>>,
    ) -> crate::Result<Vec<LibraryCreature>> {
        let mut creature_filters = CreatureFilters::default();
        for filter in filters {
            let filter = (*filter).clone();
            // TODO: include with macro...? or at least better functions?
            // todo: remove clone
            if let Ok(cf) = CreatureFilters::try_from(filter) {
                creature_filters = creature_filters.merge(cf);
            }
        }

        get_creatures(&pool, &creature_filters).await
    }
}

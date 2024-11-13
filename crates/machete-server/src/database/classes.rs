use machete::models::library::{
    classes::{ClassFilters, LibraryClass},
    GameSystem, Rarity,
};
use machete_core::ids::InternalId;

use super::DEFAULT_MAX_LIMIT;

// TODO: May be prudent to make a separate models system for the database.
pub async fn get_classes(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    // TODO: Could this use be problematic?
    // A postgres alternative can be found here:
    // https://github.com/launchbadge/sqlx/issues/291
    condition: &ClassFilters,
) -> crate::Result<Vec<LibraryClass>> {
    let limit = condition.limit.unwrap_or(DEFAULT_MAX_LIMIT);
    let page = condition.page.unwrap_or(0);
    let offset = page * limit;

    let query = sqlx::query!(
        r#"
        SELECT 
            lo.id,
            lo.name,
            lo.game_system,
            lo.url,
            lo.description,
            rarity,
            hp,
            traditions,
            ARRAY_AGG(DISTINCT tag) FILTER (WHERE tag IS NOT NULL) AS tags
        FROM library_objects lo
        INNER JOIN library_classes lc ON lo.id = lc.id
        LEFT JOIN library_objects_tags lot ON lo.id = lot.library_object_id
        LEFT JOIN tags t ON lot.tag_id = t.id

        WHERE 1=1
            AND ($1::text IS NULL OR lo.name ILIKE '%' || $1 || '%')
        GROUP BY lo.id, lc.id ORDER BY lo.name
        LIMIT $2 OFFSET $3
    "#,
        condition.name,
        limit as i64,
        offset as i64,
    );

    let classes = query
        .fetch_all(exec)
        .await?
        .into_iter()
        .map(|row| {
            Ok(LibraryClass {
                id: InternalId(row.id as u64),
                name: row.name,
                game_system: GameSystem::from_i64(row.game_system as i64),
                rarity: Rarity::from_i64(row.rarity as i64),
                tags: row.tags.unwrap_or_default(),
                hp: row.hp as u32,
                url: row.url,
                description: row.description.unwrap_or_default(),
                traditions: row.traditions,
            })
        })
        .collect::<Result<Vec<LibraryClass>, sqlx::Error>>()?;
    Ok(classes)
}

pub async fn insert_classes(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres> + Copy,
    classes: &Vec<LibraryClass>,
) -> crate::Result<()> {
    // TODO: Do we *need* two tables for this?

    if classes.is_empty() {
        return Ok(());
    }
    let ids = sqlx::query!(
        r#"
        INSERT INTO library_objects (name, game_system, url, description)
        SELECT * FROM UNNEST ($1::text[], $2::int[], $3::text[], $4::text[])
        RETURNING id  
    "#,
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

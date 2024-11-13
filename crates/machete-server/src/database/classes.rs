use machete::models::library::{
    classes::{ClassFilters, LibraryClass},
    GameSystem, Rarity,
};
use machete_core::ids::InternalId;

// TODO: May be prudent to make a separate models system for the database.
pub async fn get_classes(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    // TODO: Could this use be problematic?
    // A postgres alternative can be found here:
    // https://github.com/launchbadge/sqlx/issues/291
    condition: &ClassFilters,
) -> crate::Result<Vec<LibraryClass>> {
    let query = sqlx::query!(
        r#"
        SELECT 
            lo.id,
            lo.name,
            lo.game_system,
            rarity,
            ARRAY_AGG(DISTINCT tag) FILTER (WHERE tag IS NOT NULL) AS tags
        FROM library_objects lo
        INNER JOIN library_classes lc ON lo.id = lc.id
        LEFT JOIN library_objects_tags lot ON lo.id = lot.library_object_id
        LEFT JOIN tags t ON lot.tag_id = t.id

        WHERE 1=1
            AND ($1::text IS NULL OR lo.name ILIKE '%' || $1 || '%')
        GROUP BY lo.id, lc.id ORDER BY lo.name
    "#,
        condition.name,
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
                rarity: Rarity::from_i64(row.rarity.unwrap_or_default() as i64),
                tags: row.tags.unwrap_or_default(),
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
        INSERT INTO library_objects (name, game_system)
        SELECT * FROM UNNEST ($1::text[], $2::int[])  
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
    )
    .fetch_all(exec)
    .await?
    .into_iter()
    .map(|row| Ok(row.id))
    .collect::<Result<Vec<i32>, sqlx::Error>>()?;

    // TODO: as i32 should be unnecessary- fix in models
    sqlx::query!(
        r#"
        INSERT INTO library_classes (id, rarity)
        SELECT * FROM UNNEST ($1::int[], $2::int[])
    "#,
        &ids.iter().map(|id| *id as i32).collect::<Vec<i32>>(),
        &classes
            .iter()
            .map(|c| c.rarity.as_i64() as i32)
            .collect::<Vec<i32>>(),
    )
    .execute(exec)
    .await?;

    Ok(())
}

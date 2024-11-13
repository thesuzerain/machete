use machete::models::library::{
    spell::{LibrarySpell, SpellFilters},
    GameSystem, Rarity,
};
use machete_core::ids::InternalId;

// TODO: May be prudent to make a separate models system for the database.
pub async fn get_spells(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    // TODO: Could this use be problematic?
    // A postgres alternative can be found here:
    // https://github.com/launchbadge/sqlx/issues/291
    condition: &SpellFilters,
) -> crate::Result<Vec<LibrarySpell>> {
    let query = sqlx::query!(
        r#"
        SELECT 
            lo.id,
            lo.name,
            lo.game_system,
            rarity,
            rank,
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

        GROUP BY lo.id, lc.id ORDER BY lo.name
    "#,
        condition.name,
        condition.rarity.as_ref().map(|r| r.as_i64() as i32),
        condition.game_system.as_ref().map(|gs| gs.as_i64() as i32),
        condition.min_rank.map(|r| r as i32),
        condition.max_rank.map(|r| r as i32),
        condition.tags.first(), // TODO: Incorrect, only returning one tag.
    );

    let spells = query
        .fetch_all(exec)
        .await?
        .into_iter()
        .map(|row| {
            Ok(LibrarySpell {
                id: InternalId(row.id as u64),
                name: row.name,
                game_system: GameSystem::from_i64(row.game_system as i64),
                rarity: Rarity::from_i64(row.rarity.unwrap_or_default() as i64),
                rank: row.rank.unwrap_or_default() as u8,
                tags: row.tags.unwrap_or_default(),
            })
        })
        .collect::<Result<Vec<LibrarySpell>, sqlx::Error>>()?;
    Ok(spells)
}

pub async fn insert_spells(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres> + Copy,
    spells: &Vec<LibrarySpell>,
) -> crate::Result<()> {
    // TODO: Do we *need* two tables for this?

    if spells.is_empty() {
        return Ok(());
    }
    let ids = sqlx::query!(
        r#"
        INSERT INTO library_objects (name, game_system)
        SELECT * FROM UNNEST ($1::text[], $2::int[])  
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
    )
    .fetch_all(exec)
    .await?
    .into_iter()
    .map(|row| Ok(row.id))
    .collect::<Result<Vec<i32>, sqlx::Error>>()?;

    // TODO: as i32 should be unnecessary- fix in models
    sqlx::query!(
        r#"
        INSERT INTO library_spells (id, rarity, rank)
        SELECT * FROM UNNEST ($1::int[], $2::int[], $3::int[])
    "#,
        &ids.iter().map(|id| *id as i32).collect::<Vec<i32>>(),
        &spells
            .iter()
            .map(|c| c.rarity.as_i64() as i32)
            .collect::<Vec<i32>>(),
        &spells.iter().map(|c| c.rank as i32).collect::<Vec<i32>>(),
    )
    .execute(exec)
    .await?;

    Ok(())
}

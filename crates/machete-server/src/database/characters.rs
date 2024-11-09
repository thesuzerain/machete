use machete::models::characters::Character;
use machete_core::ids::InternalId;
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct CharacterFilters {
    pub name: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct InsertCharacter {
    pub name: String,
    pub player: Option<String>,
}

// TODO: May be prudent to make a separate models system for the database.
pub async fn get_characters(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    owner: InternalId,
    campaign_id: InternalId,
    // TODO: Could this use be problematic?
    // A postgres alternative can be found here:
    // https://github.com/launchbadge/sqlx/issues/291
    condition: &CharacterFilters,
) -> crate::Result<Vec<Character>> {
    // TODO: Campaign needs to be checked for ownership
    let query = sqlx::query!(
        r#"
        SELECT 
            ch.id,
            ch.name,
            ch.player
        FROM characters ch
        LEFT JOIN campaigns ca ON ch.campaign = ca.id
        WHERE 
            ($1::text IS NULL OR ch.name ILIKE '%' || $1 || '%')
            AND ($2::int IS NULL OR ca.id = $2)
    "#,
        condition.name,
        campaign_id.0 as i32,
    );

    let characters = query
        .fetch_all(exec)
        .await?
        .into_iter()
        .map(|row| {
            Ok(Character {
                id: InternalId(row.id as u64),
                name: row.name,
                player: row.player,
            })
        })
        .collect::<Result<Vec<Character>, sqlx::Error>>()?;
    Ok(characters)
}

pub async fn insert_characters(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres> + Copy,
    owner: InternalId,
    campaign_id: InternalId,
    characters: &Vec<InsertCharacter>,
) -> crate::Result<()> {
    // TODO: Campaign needs to be checked for ownership

    let campaign_id = campaign_id.0 as i32;
    let (names, players): (Vec<String>, Vec<Option<String>>) = characters
        .iter()
        .map(|e| {
            // TODO: remove clones
            (e.name.clone(), e.player.clone())
        })
        .unzip();

    sqlx::query!(
        r#"
        INSERT INTO characters (name, player, campaign)
        SELECT * FROM UNNEST ($1::varchar[], $2::varchar[], array[$3::int])
        "#,
        &names,
        &players as _,
        campaign_id
    )
    .execute(exec)
    .await?;

    Ok(())
}

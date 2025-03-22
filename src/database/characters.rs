use crate::models::characters::Character;
use crate::models::ids::InternalId;

use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct CharacterFilters {
    pub name: Option<String>,
}

#[derive(serde::Deserialize, Debug)]
pub struct InsertCharacter {
    pub name: String,
    pub player: Option<String>,
    pub class: InternalId,
}

#[derive(serde::Deserialize)]
pub struct ModifyCharacter {
    pub name: Option<String>,
    pub player: Option<String>,
    pub class: Option<InternalId>,
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
            ch.player,
            ch.class
        FROM characters ch
        LEFT JOIN campaigns ca ON ch.campaign = ca.id
        WHERE 
            ($1::text IS NULL OR ch.name ILIKE '%' || $1 || '%')
            AND ($2::int IS NULL OR ca.id = $2)
            AND ca.owner = $3
    "#,
        condition.name,
        campaign_id.0 as i32,
        owner.0 as i32,
    );

    let characters = query
        .fetch_all(exec)
        .await?
        .into_iter()
        .map(|row| {
            Ok(Character {
                id: InternalId(row.id as u32),
                name: row.name,
                player: row.player,
                class: InternalId(row.class as u32),
            })
        })
        .collect::<Result<Vec<Character>, sqlx::Error>>()?;
    Ok(characters)
}

pub async fn get_chracter_id(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    character_id: InternalId,
    owner: InternalId,
) -> crate::Result<Option<Character>> {
    let query = sqlx::query!(
        r#"
        SELECT 
            ch.id,
            ch.name,
            ch.player,
            ch.class
        FROM characters ch
        LEFT JOIN campaigns ca ON ch.campaign = ca.id
        WHERE 
            ch.id = $1
            AND ca.owner = $2
    "#,
        character_id.0 as i32,
        owner.0 as i32,
    );

    let character = query.fetch_optional(exec).await?.map(|row| Character {
        id: InternalId(row.id as u32),
        name: row.name,
        player: row.player,
        class: InternalId(row.class as u32),
    });
    Ok(character)
}

pub async fn edit_character(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres> + Copy,
    character_id: InternalId,
    character: &ModifyCharacter,
) -> crate::Result<()> {
    let query = sqlx::query!(
        r#"
        UPDATE characters
        SET name = COALESCE($1, name),
            player = COALESCE($2, player),
            class = COALESCE($3, class)
        WHERE id = $4
        "#,
        character.name,
        character.player,
        character.class.as_ref().map(|c| c.0 as i32),
        character_id.0 as i32,
    );

    query.execute(exec).await?;

    Ok(())
}

pub async fn insert_characters(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    campaign_id: InternalId,
    characters: &[InsertCharacter],
) -> crate::Result<Vec<InternalId>> {
    // TODO: Campaign needs to be checked for ownership
    if characters.is_empty() {
        return Ok(vec![]);
    }

    let campaign_id = std::iter::once(campaign_id.0 as i32)
        .cycle()
        .take(characters.len())
        .collect::<Vec<i32>>();
    let (names, players): (Vec<String>, Vec<Option<String>>) = characters
        .iter()
        .map(|e| {
            // TODO: remove clones
            (e.name.clone(), e.player.clone())
        })
        .unzip();

    let ids = sqlx::query!(
        r#"
        INSERT INTO characters (name, player, campaign, class)
        SELECT * FROM UNNEST ($1::varchar[], $2::varchar[], $3::int[], $4::int[])
        RETURNING id
        "#,
        &names,
        &players as _,
        &campaign_id,
        &characters
            .iter()
            .map(|c| c.class.0 as i32)
            .collect::<Vec<i32>>(),
    )
    .fetch_all(&mut **tx)
    .await?
    .into_iter()
    .map(|row| InternalId(row.id as u32))
    .collect();

    Ok(ids)
}

pub async fn delete_character(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres> + Copy,
    character_id: InternalId,
) -> crate::Result<()> {
    // TODO:  Ensure FE has suitable checks for this (campaign ownership, but also, confirmation modal)

    sqlx::query!(
        r#"
        DELETE FROM events
        WHERE character = $1
        "#,
        character_id.0 as i32,
    )
    .execute(exec)
    .await?;

    sqlx::query!(
        r#"
        DELETE FROM campaign_session_characters
        WHERE character_id = $1
        "#,
        character_id.0 as i32,
    )
    .execute(exec)
    .await?;

    sqlx::query!(
        r#"
        DELETE FROM campaign_session_character_items
        WHERE character_id = $1
        "#,
        character_id.0 as i32,
    )
    .execute(exec)
    .await?;

    sqlx::query!(
        r#"
        DELETE FROM characters
        WHERE id = $1
        "#,
        character_id.0 as i32,
    )
    .execute(exec)
    .await?;

    Ok(())
}

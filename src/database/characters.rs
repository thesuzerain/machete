use crate::models::characters::Character;
use crate::models::ids::InternalId;

use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct CharacterFilters {
    pub name: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct InsertCharacter {
    pub name: String,
    pub player: Option<String>,
    pub class: InternalId,
    pub level: u8,
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
            ch.level,
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
                id: InternalId(row.id as u64),
                name: row.name,
                level: row.level as u8,
                player: row.player,
                class: InternalId(row.class as u64),
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
            ch.level,
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
        id: InternalId(row.id as u64),
        name: row.name,
        level: row.level as u8,
        player: row.player,
        class: InternalId(row.class as u64),
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
        character.class.as_ref().map(|c| c.0 as i64),
        character_id.0 as i32,
    );

    query.execute(exec).await?;

    Ok(())
}

pub async fn insert_characters(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres> + Copy,
    campaign_id: InternalId,
    characters: &Vec<InsertCharacter>,
) -> crate::Result<()> {
    // TODO: Campaign needs to be checked for ownership
    if characters.is_empty() {
        return Ok(());
    }

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
        INSERT INTO characters (name, player, campaign, class, level)
        SELECT * FROM UNNEST ($1::varchar[], $2::varchar[], array[$3::int], $4::int[], $5::int[])
        "#,
        &names,
        &players as _,
        campaign_id,
        &characters
            .iter()
            .map(|c| c.class.0 as i32)
            .collect::<Vec<i32>>(),
        &characters
            .iter()
            .map(|c| c.level as i32)
            .collect::<Vec<i32>>(),
    )
    .execute(exec)
    .await?;

    Ok(())
}

pub async fn delete_character(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres> + Copy,
    character_id: InternalId,
) -> crate::Result<()> {
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

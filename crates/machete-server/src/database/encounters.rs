use itertools::Itertools;
use machete::models::{encounter::{CompletionStatus, Encounter}, events::{Event, EventType}, library::item::Currency};
use machete_core::ids::InternalId;
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct EncounterFilters {
    pub name : Option<String>,
    pub status : Option<CompletionStatus>,
}

#[derive(serde::Deserialize, Debug)]
pub struct InsertEncounter {
    pub name: String,
    pub description: String,

    pub enemies: Vec<InternalId>,
    pub hazards: Vec<InternalId>,

    pub treasure_items: Vec<InternalId>,
    pub treasure_currency: u64,
}

// TODO: May be prudent to make a separate models system for the database.
pub async fn get_encounters(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    owner: InternalId,
    campaign_id: InternalId,
    // TODO: Could this use be problematic?
    // A postgres alternative can be found here:
    // https://github.com/launchbadge/sqlx/issues/291
    condition: &EncounterFilters,
) -> crate::Result<Vec<Encounter>> {
    // TODO: Campaign needs to be checked for ownership

    let query = sqlx::query!(
        r#"
        SELECT 
            en.id,
            en.name,
            en.description,
            en.enemies,
            en.hazards,
            en.treasure_items,
            en.treasure_currency,
            en.status
        FROM encounters en
        WHERE 
            ($1::text IS NULL OR en.name LIKE '%' || $1 || '%')
            AND ($2::integer IS NULL OR en.status = $2)
    "#,
        condition.name,
        condition.status.as_ref().map(|s| s.as_i32()),
    );

    let events = query
        .fetch_all(exec)
        .await?
        .into_iter()
        .map(|row| {
            Ok(Encounter {
                id: InternalId(row.id as u64),
                name: row.name,
                description: row.description,
                status: CompletionStatus::from_i32(row.status),
                enemies: row.enemies.iter().map(|id| InternalId(*id as u64)).collect(),
                hazards: row.hazards.iter().map(|id| InternalId(*id as u64)).collect(),
                treasure_items: row.treasure_items.iter().map(|id| InternalId(*id as u64)).collect(),
                treasure_currency: Currency::from_base_unit(row.treasure_currency.unwrap_or(0) as u32),
            })
        })
        .collect::<Result<Vec<Encounter>, crate::ServerError>>()?;
    Ok(events)
}

pub async fn insert_encounters(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres> + Copy,
    owner: InternalId,
    encounters: &Vec<InsertEncounter>,
) -> crate::Result<Vec<InternalId>> {
    if encounters.is_empty() {
        return Ok(Vec::new());
    }

    let mut ids = Vec::new();

    // TODO: Unfortunately, sqlx does not support multidimensional arrays
    // No built in way to UNNEST like in other insertion functions in postgres- unnest entirely flattens.
    // https://wiki.postgresql.org/wiki/Unnest_multidimensional_array
    // Unfortunately, sqlx does not support insertion of multidimensional arrays anyhow.
    for encounter in encounters {
        let encounter_id = sqlx::query!(
            r#"
            INSERT INTO encounters (name, description, enemies, hazards, treasure_items, treasure_currency, status)
            VALUES ($1, $2, $3, $4, $5, $6, 0)
            RETURNING id
            "#,
            &encounter.name,
            &encounter.description,
            &encounter.enemies.iter().map(|id| id.0 as i64).collect::<Vec<i64>>(),
            &encounter.hazards.iter().map(|id| id.0 as i64).collect::<Vec<i64>>(),
            &encounter.treasure_items.iter().map(|id| id.0 as i64).collect::<Vec<i64>>(),
            encounter.treasure_currency as i64,
        )
        .fetch_one(exec)
        .await?
        .id;

        ids.push(InternalId(encounter_id as u64));
    }

    Ok(ids)
}

pub async fn edit_encounter(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres> + Copy,
    owner: InternalId,
    encounter_id: InternalId,
    new_encounter: &InsertEncounter,
) -> crate::Result<()> {

    sqlx::query!(
        r#"
        UPDATE encounters
        SET name = $1, description = $2, enemies = $3, hazards = $4, treasure_items = $5, treasure_currency = $6
        WHERE id = $7
        "#,
        &new_encounter.name,
        &new_encounter.description,
        &new_encounter.enemies.iter().map(|id| id.0 as i64).collect::<Vec<i64>>(),
        &new_encounter.hazards.iter().map(|id| id.0 as i64).collect::<Vec<i64>>(),
        &new_encounter.treasure_items.iter().map(|id| id.0 as i64).collect::<Vec<i64>>(),
        new_encounter.treasure_currency as i64,
        encounter_id.0 as i64,
    )
    .execute(exec)
    .await?;

    Ok(())
}

pub async fn delete_encounters(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres> + Copy,
    owner: InternalId,
    encounter_id: &Vec<InternalId>,
) -> crate::Result<()> {
    if encounter_id.is_empty() {
        return Ok(());
    }

    sqlx::query!(
        r#"
        DELETE FROM encounters
        WHERE id = ANY($1::int[])
        "#,
        &encounter_id.iter().map(|id| id.0 as i32).collect::<Vec<i32>>(),
    )
    .execute(exec)
    .await?;

    Ok(())
}
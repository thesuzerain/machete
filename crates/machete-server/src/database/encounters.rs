use itertools::Itertools;
use machete::models::{
    encounter::{CompletionStatus, Encounter},
    events::{Event, EventType},
    library::item::Currency,
};
use machete_core::ids::InternalId;
use serde::{Deserialize, Serialize};

use crate::models::currency::CurrencyOrGold;

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct EncounterFilters {
    pub name: Option<String>,
    pub status: Option<CompletionStatus>,
}

#[derive(serde::Deserialize, Debug)]
pub struct InsertEncounter {
    pub name: String,
    pub description: String,

    pub enemies: Vec<InternalId>,
    pub hazards: Vec<InternalId>,

    pub treasure_items: Vec<InternalId>,
    pub treasure_currency: CurrencyOrGold,
}

#[derive(serde::Deserialize, Debug)]
pub struct ModifyEncounter {
    pub name: Option<String>,
    pub description: Option<String>,

    pub enemies: Option<Vec<InternalId>>,
    pub hazards: Option<Vec<InternalId>>,

    pub treasure_items: Option<Vec<InternalId>>,
    pub treasure_currency: Option<CurrencyOrGold>,

    pub status: Option<CompletionStatus>,
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
                status: CompletionStatus::from_i32(row.status as i32),
                enemies: row
                    .enemies
                    .iter()
                    .map(|id| InternalId(*id as u64))
                    .collect(),
                hazards: row
                    .hazards
                    .iter()
                    .map(|id| InternalId(*id as u64))
                    .collect(),
                treasure_items: row
                    .treasure_items
                    .iter()
                    .map(|id| InternalId(*id as u64))
                    .collect(),
                treasure_currency: Currency::from_base_unit(
                    row.treasure_currency.unwrap_or(0) as u32
                ),
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
            encounter.treasure_currency.as_currency().as_base_unit() as i64,
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
    new_encounter: &ModifyEncounter,
) -> crate::Result<()> {
    let enemies = if let Some(enemies) = &new_encounter.enemies {
        Some(enemies.iter().map(|id| id.0 as i64).collect::<Vec<i64>>())
    } else {
        None
    };

    let hazards = if let Some(hazards) = &new_encounter.hazards {
        Some(hazards.iter().map(|id| id.0 as i64).collect::<Vec<i64>>())
    } else {
        None
    };

    let treasure_items = if let Some(treasure_items) = &new_encounter.treasure_items {
        Some(
            treasure_items
                .iter()
                .map(|id| id.0 as i64)
                .collect::<Vec<i64>>(),
        )
    } else {
        None
    };

    sqlx::query!(
        r#"
        UPDATE encounters
        SET name = COALESCE($1, name),
        description = COALESCE($2, description),
        enemies = COALESCE($3, enemies),
        hazards = COALESCE($4, hazards),
        treasure_items = COALESCE($5, treasure_items),
        treasure_currency = COALESCE($6, treasure_currency),
        status = COALESCE($7, status)
        WHERE id = $8
        "#,
        new_encounter.name.as_deref(),
        new_encounter.description.as_deref(),
        enemies.as_deref(),
        hazards.as_deref(),
        treasure_items.as_deref(),
        new_encounter
            .treasure_currency
            .as_ref()
            .map(|c| c.as_currency().as_base_unit() as i32),
        new_encounter
            .status
            .as_ref()
            .map(|s| s.as_i32() as i16)
            .unwrap_or_default(),
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
        &encounter_id
            .iter()
            .map(|id| id.0 as i32)
            .collect::<Vec<i32>>(),
    )
    .execute(exec)
    .await?;

    Ok(())
}

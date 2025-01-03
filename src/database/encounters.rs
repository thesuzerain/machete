use crate::models::currency::CurrencyOrGold;
use crate::models::ids::InternalId;
use crate::models::{
    encounter::{CompletionStatus, Encounter},
    library::item::Currency,
};
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct EncounterFilters {
    pub name: Option<String>,
    pub status: Option<CompletionStatus>,
}

#[derive(serde::Deserialize, Debug, Default)]
pub struct InsertEncounter {
    pub name: String,
    pub description: String,

    pub enemies: Vec<InternalId>,
    pub hazards: Vec<InternalId>,

    pub party_level: u8,
    pub party_size: u8,

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

    pub party_level: Option<u8>,
    pub party_size: Option<u8>,

    pub status: Option<CompletionStatus>,
}

// TODO: May be prudent to make a separate models system for the database.
pub async fn get_encounters(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    owner: InternalId,
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
            en.party_size,
            en.party_level,
            en.status,
            en.owner
        FROM encounters en
        WHERE 
            ($1::text IS NULL OR en.name LIKE '%' || $1 || '%')
            AND ($2::integer IS NULL OR en.status = $2)
            AND en.owner = $3
    "#,
        condition.name,
        condition.status.as_ref().map(|s| s.as_i32()),
        owner.0 as i64,
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
                party_level: row.party_level as u32,
                party_size: row.party_size as u32,
                owner: InternalId(row.owner as u64),
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

pub async fn get_owned_encounter_ids(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    encounter_id: &[InternalId],
    owner: InternalId,
) -> crate::Result<Vec<InternalId>> {
    let ids = sqlx::query!(
        r#"
        SELECT 
            en.id
        FROM encounters en
        WHERE 
            en.id = ANY($1::int[])
            AND en.owner = $2
    "#,
        &encounter_id
            .iter()
            .map(|id| id.0 as i32)
            .collect::<Vec<i32>>(),
        owner.0 as i32,
    )
    .fetch_all(exec)
    .await?
    .into_iter()
    .map(|row| InternalId(row.id as u64))
    .collect();

    Ok(ids)
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
            INSERT INTO encounters (name, description, enemies, hazards, treasure_items, treasure_currency, status, party_size, party_level, owner)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id
            "#,
            &encounter.name,
            &encounter.description,
            &encounter.enemies.iter().map(|id| id.0 as i64).collect::<Vec<i64>>(),
            &encounter.hazards.iter().map(|id| id.0 as i64).collect::<Vec<i64>>(),
            &encounter.treasure_items.iter().map(|id| id.0 as i64).collect::<Vec<i64>>(),
            encounter.treasure_currency.as_currency().as_base_unit() as i64,
            CompletionStatus::default().as_i32() as i64,
            encounter.party_size as i64,
            encounter.party_level as i64,
            owner.0 as i64,
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
    encounter_id: InternalId,
    new_encounter: &ModifyEncounter,
) -> crate::Result<()> {
    let enemies = new_encounter
        .enemies
        .as_ref()
        .map(|enemies| enemies.iter().map(|id| id.0 as i64).collect::<Vec<i64>>());

    let hazards = new_encounter
        .hazards
        .as_ref()
        .map(|hazards| hazards.iter().map(|id| id.0 as i64).collect::<Vec<i64>>());

    let treasure_items = new_encounter.treasure_items.as_ref().map(|treasure_items| {
        treasure_items
            .iter()
            .map(|id| id.0 as i64)
            .collect::<Vec<i64>>()
    });

    sqlx::query!(
        r#"
        UPDATE encounters
        SET name = COALESCE($1, name),
        description = COALESCE($2, description),
        enemies = COALESCE($3, enemies),
        hazards = COALESCE($4, hazards),
        treasure_items = COALESCE($5, treasure_items),
        treasure_currency = COALESCE($6, treasure_currency),
        status = COALESCE($7, status),
        party_size = COALESCE($8, party_size),
        party_level = COALESCE($9, party_level)
        WHERE id = $10
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
        new_encounter.party_size.map(|s| s as i32),
        new_encounter.party_level.map(|l| l as i32),
        encounter_id.0 as i64,
    )
    .execute(exec)
    .await?;

    Ok(())
}

pub async fn delete_encounters(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres> + Copy,
    encounter_id: &[InternalId],
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

pub async fn insert_user_encounter_draft(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres> + Copy,
    owner: InternalId,
    encounter: &InsertEncounter,
) -> crate::Result<InternalId> {
    // First, clear any existing drafts
    // TODO: transaction
    clear_user_encounter_draft(exec, owner).await?;

    let encounter_id = sqlx::query!(
        r#"
        INSERT INTO encounters (name, description, enemies, hazards, treasure_items, treasure_currency, status, party_size, party_level, owner)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING id
        "#,
        &encounter.name,
        &encounter.description,
        &encounter.enemies.iter().map(|id| id.0 as i64).collect::<Vec<i64>>(),
        &encounter.hazards.iter().map(|id| id.0 as i64).collect::<Vec<i64>>(),
        &encounter.treasure_items.iter().map(|id| id.0 as i64).collect::<Vec<i64>>(),
        encounter.treasure_currency.as_currency().as_base_unit() as i64,
        CompletionStatus::Draft.as_i32() as i64,
        encounter.party_size as i64,
        encounter.party_level as i64,
        owner.0 as i64,
    )
    .fetch_one(exec)
    .await?
    .id;

    Ok(InternalId(encounter_id as u64))
}

pub async fn clear_user_encounter_draft(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres> + Copy,
    owner: InternalId,
) -> crate::Result<()> {
    sqlx::query!(
        r#"
        DELETE FROM encounters
        WHERE status = $1
        AND owner = $2
        "#,
        CompletionStatus::Draft.as_i32() as i64,
        owner.0 as i64,
    )
    .execute(exec)
    .await?;

    Ok(())
}

pub async fn get_encounter_draft(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres> + Copy,
    owner: InternalId,
) -> crate::Result<Option<Encounter>> {
    let encounter = sqlx::query!(
        r#"
        SELECT 
            en.id,
            en.name,
            en.description,
            en.enemies,
            en.hazards,
            en.treasure_items,
            en.treasure_currency,
            en.status,
            en.party_size,
            en.party_level,
            en.owner
        FROM encounters en
        WHERE en.status = $1
        AND en.owner = $2
    "#,
        CompletionStatus::Draft.as_i32() as i16,
        owner.0 as i64,
    )
    .fetch_optional(exec)
    .await?;

    if let Some(row) = encounter {
        Ok(Some(Encounter {
            id: InternalId(row.id as u64),
            name: row.name,
            description: row.description,
            status: CompletionStatus::from_i32(row.status as i32),
            owner: InternalId(row.owner as u64),
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
            party_level: row.party_level as u32,
            party_size: row.party_size as u32,
            treasure_currency: Currency::from_base_unit(row.treasure_currency.unwrap_or(0) as u32),
        }))
    } else {
        // If no draft exists- create one
        insert_user_encounter_draft(exec, owner, &InsertEncounter::default())
            .await
            .map(|id| {
                Some(Encounter {
                    id,
                    name: "".to_string(),
                    description: Some("".to_string()),
                    owner,
                    status: CompletionStatus::Draft,
                    enemies: vec![],
                    hazards: vec![],
                    treasure_items: vec![],
                    treasure_currency: Currency::default(),
                    party_level: 0,
                    party_size: 0,
                })
            })
    }
}

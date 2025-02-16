use std::collections::HashMap;

use crate::models;
use crate::models::encounter::EncounterEnemy;
use crate::models::encounter::{CompletionStatus, Encounter};
use crate::models::ids::InternalId;
use crate::models::query::CommaSeparatedVec;
use serde::{Deserialize, Serialize};

use super::creatures::CreatureFiltering;
use super::hazards::HazardFiltering;
use super::items::ItemFiltering;

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct EncounterFilters {
    pub ids: Option<CommaSeparatedVec>,
    pub name: Option<String>,
    pub status: Option<CompletionStatus>,
}

impl EncounterFilters {
    pub fn from_ids(ids: &[InternalId]) -> Self {
        Self {
            ids: Some(CommaSeparatedVec(ids.iter().map(|id| id.0).collect())),
            ..Default::default()
        }
    }
}

#[derive(serde::Deserialize, Debug, Default)]
pub struct InsertEncounter {
    pub name: String,
    pub description: String,

    pub session_id: Option<InternalId>,

    pub enemies: Vec<InsertEncounterEnemy>,
    pub hazards: Vec<InternalId>,

    pub party_level: u8,
    pub party_size: u8,

    pub treasure_items: Vec<InternalId>,
    pub treasure_currency: f32,
    pub extra_experience: i32,

    #[serde(default)]
    pub status: CompletionStatus,

    // These are derived values. If provided, they will be considered as an override.
    pub total_experience: Option<i32>,
    pub total_treasure_value: Option<f32>,
}

#[derive(serde::Deserialize, Debug)]
#[serde(untagged)]
pub enum InsertEncounterEnemy {
    Id(InternalId),
    IdAndLevelAdjustment {
        id: InternalId,
        level_adjustment: i8,
    },
}

impl InsertEncounterEnemy {
    pub fn to_id(&self) -> InternalId {
        match self {
            InsertEncounterEnemy::Id(id) => *id,
            InsertEncounterEnemy::IdAndLevelAdjustment { id, .. } => *id,
        }
    }

    pub fn to_level_adjustment(&self) -> Option<i8> {
        match self {
            InsertEncounterEnemy::Id(_) => None,
            InsertEncounterEnemy::IdAndLevelAdjustment {
                level_adjustment, ..
            } => Some(*level_adjustment),
        }
    }
}

#[derive(serde::Deserialize, Debug)]
pub struct ModifyEncounter {
    pub name: Option<String>,
    pub description: Option<String>,

    pub enemies: Option<Vec<InsertEncounterEnemy>>,
    pub hazards: Option<Vec<InternalId>>,

    pub treasure_items: Option<Vec<InternalId>>,
    pub treasure_currency: Option<f32>,
    pub extra_experience: Option<i32>,

    pub party_level: Option<u8>,
    pub party_size: Option<u8>,

    pub status: Option<CompletionStatus>,

    // These are derived values. If provided, they will be considered as an override.
    pub total_experience: Option<i32>,
    pub total_treasure_value: Option<f32>,
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
    let ids = condition.ids.clone().map(|t| {
        t.into_inner()
            .into_iter()
            .map(|id| id as i32)
            .collect::<Vec<i32>>()
    });

    let query = sqlx::query!(
        r#"
        SELECT 
            en.id,
            en.name,
            en.description,
            en.session_id,
            en.enemies,
            en.enemy_level_adjustments,
            en.hazards,
            en.treasure_items,
            en.treasure_currency,
            en.party_size,
            en.party_level,
            en.status,
            en.extra_experience as "extra_experience!",
            en.total_experience,
            en.total_treasure_value,
            en.owner
        FROM encounters en
        WHERE 
            ($1::text IS NULL OR en.name LIKE '%' || $1 || '%')
            AND ($2::integer IS NULL OR en.status = $2)
            AND ($3::int[] IS NULL OR en.id = ANY($3::int[]))
            AND en.owner = $4
    "#,
        condition.name,
        condition.status.as_ref().map(|s| s.as_i32()),
        &ids as _,
        owner.0 as i64,
    );

    let events = query
        .fetch_all(exec)
        .await?
        .into_iter()
        .map(|row| {
            Ok(Encounter {
                id: InternalId(row.id as u32),
                name: row.name,
                description: row.description,
                session_id: row.session_id.map(|id| InternalId(id as u32)),
                status: CompletionStatus::from_i32(row.status as i32),
                party_level: row.party_level as u32,
                party_size: row.party_size as u32,
                owner: InternalId(row.owner as u32),
                enemies: row
                    .enemies
                    .iter()
                    .zip(row.enemy_level_adjustments.iter())
                    .map(|(id, adj)| EncounterEnemy {
                        id: InternalId(*id as u32),
                        level_adjustment: *adj,
                    })
                    .collect(),
                hazards: row
                    .hazards
                    .iter()
                    .map(|id| InternalId(*id as u32))
                    .collect(),
                treasure_items: row
                    .treasure_items
                    .iter()
                    .map(|id| InternalId(*id as u32))
                    .collect(),
                treasure_currency: row.treasure_currency.unwrap_or(0.0) as f32,
                extra_experience: row.extra_experience,
                total_experience: row.total_experience,
                total_treasure_value: row.total_treasure_value as i32,
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
    .map(|row| InternalId(row.id as u32))
    .collect();

    Ok(ids)
}

pub async fn insert_encounters(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
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
        // TODO: Mofiy this so that it only does these db call if needed
        // TODO: Also do this in 'edit_encounter'
        let enemy_ids = encounter
            .enemies
            .iter()
            .map(|e| e.to_id())
            .collect::<Vec<InternalId>>();
        let enemy_levels = get_levels_enemies(
            &mut **tx,
            &enemy_ids,
            &encounter.enemies.iter().map(|_| 0).collect::<Vec<i8>>(),
        )
        .await?;
        let hazard_levels = get_levels_hazards(&mut **tx, &encounter.hazards).await?;
        let treasure_values = get_values_items(&mut **tx, &encounter.treasure_items).await?;

        let derived_total_experience = models::encounter::calculate_total_adjusted_experience(
            &enemy_levels,
            &hazard_levels,
            encounter.party_level,
            encounter.party_size,
        );

        let derived_total_treasure_value =
            treasure_values.iter().sum::<f32>() + encounter.treasure_currency;

        let total_experience = encounter
            .total_experience
            .unwrap_or(derived_total_experience as i32);
        let total_treasure_value = encounter
            .total_treasure_value
            .unwrap_or(derived_total_treasure_value as f32);

        let encounter_id = sqlx::query!(
            r#"
            INSERT INTO encounters (name, description, enemies, enemy_level_adjustments, hazards, treasure_items, treasure_currency, status, party_size, party_level, extra_experience, total_experience, total_treasure_value, owner)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            RETURNING id
            "#,
            &encounter.name,
            &encounter.description,
            &encounter.enemies.iter().map(|e| e.to_id().0 as i32).collect::<Vec<i32>>(),
            &encounter.enemies.iter().map(|e| e.to_level_adjustment().unwrap_or(0) as i16).collect::<Vec<i16>>(),
            &encounter.hazards.iter().map(|id| id.0 as i32).collect::<Vec<i32>>(),
            &encounter.treasure_items.iter().map(|id| id.0 as i32).collect::<Vec<i32>>(),
            encounter.treasure_currency as f64,
            encounter.status.as_i32() as i64,
            encounter.party_size as i64,
            encounter.party_level as i64,
            encounter.extra_experience as i64,
            total_experience as i64,
            total_treasure_value as i64,
            owner.0 as i64,
        )
        .fetch_one(&mut **tx)
        .await?
        .id;

        if let Some(session_id) = encounter.session_id {
            super::sessions::link_encounter_to_session(
                &mut *tx,
                InternalId(encounter_id as u32),
                session_id,
            )
            .await?;
        }

        ids.push(InternalId(encounter_id as u32));
    }

    Ok(ids)
}

pub async fn edit_encounter(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    encounter_id: InternalId,
    new_encounter: &ModifyEncounter,
) -> crate::Result<()> {
    let enemies = new_encounter.enemies.as_ref().map(|enemies| {
        enemies
            .iter()
            .map(|e| e.to_id().0 as i32)
            .collect::<Vec<i32>>()
    });
    let enemy_level_adjustments = new_encounter.enemies.as_ref().map(|enemies| {
        enemies
            .iter()
            .map(|e| e.to_level_adjustment().unwrap_or(0) as i16)
            .collect::<Vec<i16>>()
    });

    let hazards = new_encounter
        .hazards
        .as_ref()
        .map(|hazards| hazards.iter().map(|id| id.0 as i32).collect::<Vec<i32>>());

    let treasure_items = new_encounter.treasure_items.as_ref().map(|treasure_items| {
        treasure_items
            .iter()
            .map(|id| id.0 as i32)
            .collect::<Vec<i32>>()
    });

    // First, unlink the encounter from the session
    // TODO: We can refactor this editing to not need to explicitly unlinking/relinking (by being more explicit)
    let unlinked_session_id =
        super::sessions::unlink_encounter_from_session(&mut *tx, encounter_id).await?;

    sqlx::query!(
        r#"
        UPDATE encounters
        SET name = COALESCE($1, name),
        description = COALESCE($2, description),
        enemies = COALESCE($3, enemies),
        enemy_level_adjustments = COALESCE($4, enemy_level_adjustments),
        hazards = COALESCE($5, hazards),
        treasure_items = COALESCE($6, treasure_items),
        treasure_currency = COALESCE($7, treasure_currency),
        status = COALESCE($8, status),
        party_size = COALESCE($9, party_size),
        party_level = COALESCE($10, party_level),
        extra_experience = COALESCE($11, extra_experience),
        
        total_experience = COALESCE($12, total_experience),
        total_treasure_value = COALESCE($13, total_treasure_value)
        WHERE id = $14
        "#,
        new_encounter.name.as_deref(),
        new_encounter.description.as_deref(),
        enemies.as_deref(),
        enemy_level_adjustments.as_deref(),
        hazards.as_deref(),
        treasure_items.as_deref(),
        new_encounter.treasure_currency.as_ref().map(|c| *c as f64),
        new_encounter.status.as_ref().map(|s| s.as_i32() as i16),
        new_encounter.party_size.map(|s| s as i32),
        new_encounter.party_level.map(|l| l as i32),
        new_encounter.extra_experience.map(|e| e as i32),
        new_encounter.total_experience.map(|e| e as i32),
        new_encounter.total_treasure_value.map(|e| e as f64),
        encounter_id.0 as i64,
    )
    .fetch_optional(&mut **tx)
    .await?;

    // Unlink and re-link the encounter to the session if needed
    // TODO: We can refactor this editing to not need to explicitly unlinking/relinking (by being more explicit)
    if let Some(session_id) = unlinked_session_id {
        super::sessions::link_encounter_to_session(&mut *tx, encounter_id, session_id).await?;
    }

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
        INSERT INTO encounters (name, description, enemies, enemy_level_adjustments, hazards, treasure_items, treasure_currency, status, party_size, party_level, extra_experience, owner)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        RETURNING id
        "#,
        &encounter.name,
        &encounter.description,
        &encounter.enemies.iter().map(|e| e.to_id().0 as i32).collect::<Vec<i32>>(),
        &encounter.enemies.iter().map(|e| e.to_level_adjustment().unwrap_or(0) as i16).collect::<Vec<i16>>(),
        &encounter.hazards.iter().map(|id| id.0 as i32).collect::<Vec<i32>>(),
        &encounter.treasure_items.iter().map(|id| id.0 as i32).collect::<Vec<i32>>(),
        encounter.treasure_currency as f64,
        CompletionStatus::Draft.as_i32() as i64,
        encounter.party_size as i64,
        encounter.party_level as i64,
        encounter.extra_experience as i64,
        owner.0 as i64,
    )
    .fetch_one(exec)
    .await?
    .id;

    Ok(InternalId(encounter_id as u32))
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
            en.enemy_level_adjustments,
            en.hazards,
            en.treasure_items,
            en.treasure_currency,
            en.status,
            en.party_size,
            en.party_level,
            en.extra_experience as "extra_experience!",
            en.total_experience,
            en.total_treasure_value,
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
            id: InternalId(row.id as u32),
            name: row.name,
            description: row.description,
            status: CompletionStatus::from_i32(row.status as i32),
            owner: InternalId(row.owner as u32),
            session_id: None,
            enemies: row
                .enemies
                .iter()
                .zip(row.enemy_level_adjustments.iter())
                .map(|(id, adj)| EncounterEnemy {
                    id: InternalId(*id as u32),
                    level_adjustment: *adj,
                })
                .collect(),
            hazards: row
                .hazards
                .iter()
                .map(|id| InternalId(*id as u32))
                .collect(),
            treasure_items: row
                .treasure_items
                .iter()
                .map(|id| InternalId(*id as u32))
                .collect(),
            party_level: row.party_level as u32,
            party_size: row.party_size as u32,
            extra_experience: row.extra_experience,
            total_experience: row.total_experience,
            total_treasure_value: row.total_treasure_value as i32,
            treasure_currency: row.treasure_currency.unwrap_or(0.0) as f32,
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
                    session_id: None,
                    owner,
                    status: CompletionStatus::Draft,
                    enemies: vec![],
                    hazards: vec![],
                    treasure_items: vec![],
                    treasure_currency: 0.0,
                    extra_experience: 0,
                    total_experience: 0,
                    total_treasure_value: 0,
                    party_level: 1,
                    party_size: 4,
                })
            })
    }
}

// Helper function accessing creatures databases to get levels of enemies given their ids and adjustments
// Used for default experience calculation
async fn get_levels_enemies(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    enemies: &[InternalId],
    enemy_level_adjustments: &[i8],
) -> crate::Result<Vec<i8>> {
    let ids = enemies.iter().map(|id| id.0).collect::<Vec<u32>>();
    let creatures = super::creatures::get_creatures(exec, &CreatureFiltering::from_ids(&ids))
        .await?
        .into_iter()
        .map(|c| (c.id, c.level))
        .collect::<HashMap<_, _>>();

    let mut levels = vec![];
    for (enemy, adjustment) in enemies.iter().zip(enemy_level_adjustments.iter()) {
        levels.push(
            creatures
                .get(enemy)
                .map(|l| l + adjustment)
                .unwrap_or_default(),
        );
    }

    Ok(levels)
}

// Helper function accessing hazards databases to get levels of hazards given their ids
// Used for default experience calculation
async fn get_levels_hazards(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    hazards: &[InternalId],
) -> crate::Result<Vec<i8>> {
    let ids = hazards.iter().map(|id| id.0).collect::<Vec<u32>>();
    let hazards_fetched = super::hazards::get_hazards(exec, &HazardFiltering::from_ids(&ids))
        .await?
        .into_iter()
        .map(|h| (h.id, h.level))
        .collect::<HashMap<_, _>>();

    let mut levels = vec![];
    for hazard in hazards {
        levels.push(hazards_fetched.get(hazard).copied().unwrap_or_default());
    }

    Ok(levels)
}

// Helper function accessing items databases to get values of items given their ids
// Used for default treasure value calculation
async fn get_values_items(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    items: &[InternalId],
) -> crate::Result<Vec<f32>> {
    // TODO: This needs to handle 'priceless' items better- currently just estimates as 0
    let ids = items.iter().map(|id| id.0).collect::<Vec<u32>>();
    let items_fetched = super::items::get_items(exec, &ItemFiltering::from_ids(&ids))
        .await?
        .into_iter()
        .map(|i| (i.id, i.price.unwrap_or(0.0) as f32))
        .collect::<HashMap<_, _>>();

    let mut values = vec![];
    for item in items {
        values.push(items_fetched.get(item).copied().unwrap_or_default());
    }

    Ok(values)
}

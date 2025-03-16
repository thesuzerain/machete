use std::collections::HashMap;

use crate::models;
use crate::models::encounter::{CompletionStatus, Encounter};
use crate::models::encounter::{
    EncounterEnemy, EncounterSubsystemCheck, EncounterSubsystemType, EncounterType,
};
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
    pub description: Option<String>,

    pub session_id: Option<InternalId>,

    pub party_level: u8,
    pub party_size: u8,

    #[serde(flatten)]
    pub encounter_type: EncounterType,

    pub treasure_items: Vec<InternalId>,
    pub treasure_currency: f32,
    pub extra_experience: i32,

    #[serde(default)]
    pub status: CompletionStatus,

    // These are derived values. If provided, they will be considered as an override.
    pub total_experience: Option<i32>,
    pub total_items_value: Option<f32>,
}

#[derive(serde::Deserialize, Debug)]
pub struct ModifyEncounter {
    pub name: Option<String>,
    pub description: Option<String>,

    pub enemies: Option<Vec<EncounterEnemy>>,
    pub hazards: Option<Vec<InternalId>>,

    pub subsystem_rolls: Option<Vec<EncounterSubsystemCheck>>,
    pub subsystem_type: Option<EncounterSubsystemType>,

    pub encounter_type_id: Option<u8>,

    pub treasure_items: Option<Vec<InternalId>>,
    pub treasure_currency: Option<f32>,
    pub extra_experience: Option<i32>,

    pub initialization_encounter: Option<bool>,

    pub party_level: Option<u8>,
    pub party_size: Option<u8>,

    pub status: Option<CompletionStatus>,

    // These are derived values. If provided, they will be considered as an override.
    pub total_experience: Option<i32>,
    pub total_items_value: Option<f32>,
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
            ARRAY_AGG(ee.enemy) FILTER (WHERE ee.enemy IS NOT NULL) as enemies,
            ARRAY_AGG(ee.level_adjustment) FILTER (WHERE ee.level_adjustment IS NOT NULL) as enemy_level_adjustments,
            ARRAY_AGG(eh.hazard) FILTER (WHERE eh.hazard IS NOT NULL) as hazards,
            ARRAY_AGG(eti.item) FILTER (WHERE eti.item IS NOT NULL) as treasure_items,
            en.treasure_currency,
            en.party_size,
            en.party_level,
            en.status,
            en.extra_experience as "extra_experience!",
            en.total_experience,
            en.total_items_value,
            en.encounter_type_id,
            en.subsystem_type_id,
            JSONB_AGG(jsonb_build_object('name', esc.name, 'vp', esc.vp, 'roll_options', esc.roll_options)) as subsystem_rolls,
            en.owner
        FROM encounters en
        LEFT JOIN encounter_enemies ee ON en.id = ee.encounter
        LEFT JOIN encounter_hazards eh ON en.id = eh.encounter
        LEFT JOIN encounter_treasure_items eti ON en.id = eti.encounter
        LEFT JOIN LATERAL (
            SELECT JSONB_AGG(jsonb_build_object('skill', escr.roll, 'dc', escr.dc)) as roll_options, esc.name, esc.vp, esc.order_index
            FROM encounter_skill_checks esc
            LEFT JOIN encounter_skill_check_rolls escr ON esc.id = escr.encounter_skill_check_id
            WHERE esc.encounter_id = en.id
            GROUP BY esc.id
        ) esc ON TRUE
        WHERE 
            ($1::text IS NULL OR en.name LIKE '%' || $1 || '%')
            AND ($2::integer IS NULL OR en.status = $2)
            AND ($3::int[] IS NULL OR en.id = ANY($3::int[]))
            AND en.owner = $4
        GROUP BY en.id
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
            let enemies = row
                .enemies
                .unwrap_or_default()
                .iter()
                .zip(row.enemy_level_adjustments.unwrap_or_default().iter())
                .map(|(id, adj)| EncounterEnemy {
                    id: InternalId(*id as u32),
                    level_adjustment: *adj,
                })
                .collect();

            let hazards = row
                .hazards
                .unwrap_or_default()
                .iter()
                .map(|id| InternalId(*id as u32))
                .collect();

            let subsystem_rolls: Vec<EncounterSubsystemCheck> =
                serde_json::from_value(row.subsystem_rolls.unwrap_or_default()).unwrap_or_default();
            let encounter_subsystem_type = row
                .subsystem_type_id
                .map(|id| EncounterSubsystemType::from_i32(id as i32));
            let encounter_type = EncounterType::from_id_and_parts(
                row.encounter_type_id,
                enemies,
                hazards,
                encounter_subsystem_type,
                subsystem_rolls,
            );

            Ok(Encounter {
                id: InternalId(row.id as u32),
                name: row.name,
                description: row.description,
                session_id: row.session_id.map(|id| InternalId(id as u32)),
                status: CompletionStatus::from_i32(row.status as i32),
                party_level: row.party_level as u32,
                party_size: row.party_size as u32,
                owner: InternalId(row.owner as u32),
                encounter_type,
                treasure_items: row
                    .treasure_items
                    .unwrap_or_default()
                    .iter()
                    .map(|id| InternalId(*id as u32))
                    .collect(),
                treasure_currency: row.treasure_currency.unwrap_or(0.0) as f32,
                extra_experience: row.extra_experience,
                total_experience: row.total_experience,
                total_items_value: row.total_items_value as i32,
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
        let encounter_type_id = encounter.encounter_type.get_id();
        let enemies = encounter.encounter_type.get_enemies();
        let hazards = encounter.encounter_type.get_hazards();
        let subsystem_checks = encounter.encounter_type.get_subsystem_checks();
        let encounter_subsystem_type = encounter.encounter_type.get_subsystem_type();

        let enemy_ids = enemies.iter().map(|e| e.id).collect::<Vec<InternalId>>();
        let enemy_levels = get_levels_enemies(
            &mut **tx,
            &enemy_ids,
            &enemies.iter().map(|_| 0).collect::<Vec<i16>>(),
        )
        .await?;
        let hazard_levels = get_levels_hazards(&mut **tx, &hazards).await?;
        let treasure_values = get_values_items(&mut **tx, &encounter.treasure_items).await?;

        // TODO: Mofiy this so that it only does these db call if needed
        // (This may be redundant- it's related to the total_experience value)
        // TODO: Also do this in 'edit_encounter'
        let derived_total_experience = models::encounter::calculate_total_adjusted_experience(
            &enemy_levels,
            &hazard_levels,
            encounter.party_level,
            encounter.party_size,
        );

        let derived_total_treasure_value = treasure_values.iter().sum::<f32>();

        let total_experience = encounter
            .total_experience
            .unwrap_or(derived_total_experience as i32);
        let total_items_value = encounter
            .total_items_value
            .unwrap_or(derived_total_treasure_value as f32);

        let encounter_id = sqlx::query!(
            r#"
            INSERT INTO encounters (name, description, encounter_type_id, subsystem_type_id, treasure_currency, status, party_size, party_level, extra_experience, total_experience, total_items_value, owner)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING id
            "#,
            &encounter.name,
            encounter.description.as_deref(),
            encounter_type_id as i32,
            encounter_subsystem_type.as_i32() as i32,
            encounter.treasure_currency as f64,
            CompletionStatus::Prepared.as_i32() as i16,
            encounter.party_size as i64,
            encounter.party_level as i64,
            encounter.extra_experience as i64,
            total_experience as i64,
            total_items_value as f64,
            owner.0 as i64,
        )
        .fetch_one(&mut **tx)
        .await?
        .id;

        sqlx::query!(
            r#"
            INSERT INTO encounter_enemies (encounter, enemy, level_adjustment)
            SELECT $1, enemy, level_adjustment
            FROM UNNEST($2::int[], $3::smallint[]) AS t(enemy, level_adjustment)
            "#,
            encounter_id as i32,
            &enemy_ids.iter().map(|id| id.0 as i32).collect::<Vec<i32>>(),
            &enemy_levels,
        )
        .execute(&mut **tx)
        .await?;

        // Insert new encounter_hazards
        sqlx::query!(
            r#"
            INSERT INTO encounter_hazards (encounter, hazard)
            SELECT $1, hazard
            FROM UNNEST($2::int[]) AS t(hazard)
            "#,
            encounter_id as i32,
            &hazards.iter().map(|id| id.0 as i32).collect::<Vec<i32>>(),
        )
        .execute(&mut **tx)
        .await?;

        // Insert new encounter_treasure_items
        sqlx::query!(
            r#"
            INSERT INTO encounter_treasure_items (encounter, item)
            SELECT $1, item
            FROM UNNEST($2::int[]) AS t(item)
            "#,
            encounter_id as i32,
            &encounter
                .treasure_items
                .iter()
                .map(|id| id.0 as i32)
                .collect::<Vec<i32>>(),
        )
        .execute(&mut **tx)
        .await?;

        // Insert new encounter_skill_checks
        let mut order_index = 0;
        for check in subsystem_checks {
            let check_id = sqlx::query!(
                r#"
                INSERT INTO encounter_skill_checks (encounter_id, name, vp, order_index)
                VALUES ($1, $2, $3, $4)
                RETURNING id
                "#,
                encounter_id as i32,
                check.name,
                check.vp as i16,
                order_index,
            )
            .fetch_one(&mut **tx)
            .await?
            .id;
            order_index += 1;

            let (rolls, dcs): (Vec<String>, Vec<i16>) = check
                .roll_options
                .iter()
                .map(|r| (r.skill.to_string(), r.dc as i16))
                .unzip();

            sqlx::query!(
                r#"
                INSERT INTO encounter_skill_check_rolls (encounter_skill_check_id, roll, dc)
                SELECT $1, roll, dc
                FROM UNNEST($2::varchar(16)[], $3::smallint[]) AS t(roll, dc)
                "#,
                check_id as i32,
                &rolls,
                &dcs,
            )
            .execute(&mut **tx)
            .await?;
        }

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
    let enemies = new_encounter
        .enemies
        .as_ref()
        .map(|enemies| enemies.iter().map(|e| e.id.0 as i32).collect::<Vec<i32>>());
    let enemy_level_adjustments = new_encounter.enemies.as_ref().map(|enemies| {
        enemies
            .iter()
            .map(|e| e.level_adjustment)
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
        treasure_currency = COALESCE($3, treasure_currency),
        status = COALESCE($4, status),
        party_size = COALESCE($5, party_size),
        party_level = COALESCE($6, party_level),
        extra_experience = COALESCE($7, extra_experience),
        encounter_type_id = COALESCE($8, encounter_type_id),
        subsystem_type_id = COALESCE($9, subsystem_type_id),
        
        total_experience = COALESCE($10, total_experience),
        total_items_value = COALESCE($11, total_items_value)
        WHERE id = $12
        "#,
        new_encounter.name.as_deref(),
        new_encounter.description.as_deref(),
        new_encounter.treasure_currency.as_ref().map(|c| *c as f64),
        new_encounter.status.as_ref().map(|s| s.as_i32() as i16),
        new_encounter.party_size.map(|s| s as i32),
        new_encounter.party_level.map(|l| l as i32),
        new_encounter.extra_experience.map(|e| e as i32),
        new_encounter.encounter_type_id.map(|e| e as i32),
        new_encounter
            .subsystem_type
            .as_ref()
            .map(|e| e.as_i32() as i32),
        new_encounter.total_experience.map(|e| e as i32),
        new_encounter.total_items_value.map(|e| e as f64),
        encounter_id.0 as i64,
    )
    .fetch_optional(&mut **tx)
    .await?;

    if let Some(enemies) = enemies {
        // Drop all existing encounter_enemies
        sqlx::query!(
            r#"
            DELETE FROM encounter_enemies
            WHERE encounter = $1
            "#,
            encounter_id.0 as i32,
        )
        .execute(&mut **tx)
        .await?;

        // Insert new encounter_enemies
        for (enemy, adjustment) in enemies
            .iter()
            .zip(enemy_level_adjustments.unwrap_or_default())
        {
            sqlx::query!(
                r#"
                INSERT INTO encounter_enemies (encounter, enemy, level_adjustment)
                VALUES ($1, $2, $3)
                "#,
                encounter_id.0 as i32,
                *enemy,
                adjustment as i16,
            )
            .execute(&mut **tx)
            .await?;
        }
    }

    if let Some(hazards) = hazards {
        // Drop all existing encounter_hazards
        sqlx::query!(
            r#"
            DELETE FROM encounter_hazards
            WHERE encounter = $1
            "#,
            encounter_id.0 as i32,
        )
        .execute(&mut **tx)
        .await?;

        // Insert new encounter_hazards
        for hazard in hazards {
            sqlx::query!(
                r#"
                INSERT INTO encounter_hazards (encounter, hazard)
                VALUES ($1, $2)
                "#,
                encounter_id.0 as i32,
                hazard,
            )
            .execute(&mut **tx)
            .await?;
        }
    }

    if let Some(treasure_items) = treasure_items {
        // Drop all existing encounter_treasure_items
        sqlx::query!(
            r#"
            DELETE FROM encounter_treasure_items
            WHERE encounter = $1
            "#,
            encounter_id.0 as i32,
        )
        .execute(&mut **tx)
        .await?;

        // Insert new encounter_treasure_items
        for item in treasure_items {
            sqlx::query!(
                r#"
                INSERT INTO encounter_treasure_items (encounter, item)
                VALUES ($1, $2)
                "#,
                encounter_id.0 as i32,
                item,
            )
            .execute(&mut **tx)
            .await?;
        }
    }

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

    // TODO: Check this again- may be easier to use cascade delete
    sqlx::query!(
        r#"
        DELETE FROM encounter_treasure_items
        WHERE encounter = ANY($1::int[])
        "#,
        &encounter_id
            .iter()
            .map(|id| id.0 as i32)
            .collect::<Vec<i32>>(),
    )
    .execute(exec)
    .await?;

    sqlx::query!(
        r#"
        DELETE FROM encounter_enemies
        WHERE encounter = ANY($1::int[])
        "#,
        &encounter_id
            .iter()
            .map(|id| id.0 as i32)
            .collect::<Vec<i32>>(),
    )
    .execute(exec)
    .await?;

    sqlx::query!(
        r#"
        DELETE FROM encounter_hazards
        WHERE encounter = ANY($1::int[])
        "#,
        &encounter_id
            .iter()
            .map(|id| id.0 as i32)
            .collect::<Vec<i32>>(),
    )
    .execute(exec)
    .await?;

    sqlx::query!(
        r#"
        DELETE FROM encounter_skill_check_rolls
        WHERE encounter_skill_check_id IN (
            SELECT id
            FROM encounter_skill_checks
            WHERE encounter_id = ANY($1::int[])
        )
        "#,
        &encounter_id
            .iter()
            .map(|id| id.0 as i32)
            .collect::<Vec<i32>>(),
    )
    .execute(exec)
    .await?;

    sqlx::query!(
        r#"
        DELETE FROM encounter_skill_checks
        WHERE encounter_id = ANY($1::int[])
        "#,
        &encounter_id
            .iter()
            .map(|id| id.0 as i32)
            .collect::<Vec<i32>>(),
    )
    .execute(exec)
    .await?;

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
        INSERT INTO encounters (name, description, encounter_type_id, treasure_currency, status, party_size, party_level, extra_experience, subsystem_type_id, owner)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING id
        "#,
        &encounter.name,
        encounter.description.as_deref(),
        encounter.encounter_type.get_id() as i32,
        encounter.treasure_currency as f64,
        CompletionStatus::Draft.as_i32() as i64,
        encounter.party_size as i64,
        encounter.party_level as i64,
        encounter.extra_experience as i64,
        encounter.encounter_type.get_subsystem_type().as_i32(),
        owner.0 as i64,
    )
    .fetch_one(exec)
    .await?
    .id;

    // Add enemies
    let enemies: Vec<InternalId> = encounter
        .encounter_type
        .get_enemies()
        .iter()
        .map(|e| e.id)
        .collect();

    let enemy_adjustments = encounter
        .encounter_type
        .get_enemies()
        .iter()
        .map(|e| e.level_adjustment)
        .collect::<Vec<i16>>();

    sqlx::query!(
        r#"
        INSERT INTO encounter_enemies (encounter, enemy, level_adjustment)
        SELECT $1, enemy, level_adjustment
        FROM UNNEST($2::int[], $3::smallint[]) AS t(enemy, level_adjustment)
        "#,
        encounter_id as i32,
        &enemies.iter().map(|id| id.0 as i32).collect::<Vec<i32>>(),
        &enemy_adjustments,
    )
    .execute(exec)
    .await?;

    // Add hazards
    sqlx::query!(
        r#"
        INSERT INTO encounter_hazards (encounter, hazard)
        SELECT $1, hazard
        FROM UNNEST($2::int[]) AS t(hazard)
        "#,
        encounter_id as i32,
        &encounter
            .encounter_type
            .get_hazards()
            .iter()
            .map(|id| id.0 as i32)
            .collect::<Vec<i32>>(),
    )
    .execute(exec)
    .await?;

    // Add treasure items
    sqlx::query!(
        r#"
        INSERT INTO encounter_treasure_items (encounter, item)
        SELECT $1, item
        FROM UNNEST($2::int[]) AS t(item)
        "#,
        encounter_id as i32,
        &encounter
            .treasure_items
            .iter()
            .map(|id| id.0 as i32)
            .collect::<Vec<i32>>(),
    )
    .execute(exec)
    .await?;

    // Add subsystem rolls
    let subsystem_checks = encounter.encounter_type.get_subsystem_checks();
    let mut order_index = 0;
    for check in subsystem_checks {
        let check_id = sqlx::query!(
            r#"
            INSERT INTO encounter_skill_checks (encounter_id, name, vp, order_index)
            VALUES ($1, $2, $3, $4)
            RETURNING id
            "#,
            encounter_id as i32,
            check.name,
            check.vp as i16,
            order_index,
        )
        .fetch_one(exec)
        .await?
        .id;
        order_index += 1;

        let (rolls, dcs): (Vec<String>, Vec<i16>) = check
            .roll_options
            .iter()
            .map(|r| (r.skill.to_string(), r.dc as i16))
            .unzip();

        sqlx::query!(
            r#"
            INSERT INTO encounter_skill_check_rolls (encounter_skill_check_id, roll, dc)
            SELECT $1, roll, dc
            FROM UNNEST($2::varchar(16)[], $3::smallint[]) AS t(roll, dc)
            "#,
            check_id as i32,
            &rolls,
            &dcs,
        )
        .execute(exec)
        .await?;
    }
    Ok(InternalId(encounter_id as u32))
}

pub async fn clear_user_encounter_draft(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres> + Copy,
    owner: InternalId,
) -> crate::Result<()> {
    let Some(draft_id) = sqlx::query!(
        r#"
        SELECT id
        FROM encounters
        WHERE status = $1
        AND owner = $2
        "#,
        CompletionStatus::Draft.as_i32() as i64,
        owner.0 as i64,
    )
    .fetch_optional(exec)
    .await?
    .map(|r| r.id) else {
        return Ok(());
    };

    delete_encounters(exec, &[InternalId(draft_id as u32)]).await?;

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
            ARRAY_AGG(ee.enemy) FILTER (WHERE ee.enemy IS NOT NULL) as enemies,
            ARRAY_AGG(ee.level_adjustment) FILTER (WHERE ee.level_adjustment IS NOT NULL) as enemy_level_adjustments,
            ARRAY_AGG(eh.hazard) FILTER (WHERE eh.hazard IS NOT NULL) as hazards,
            ARRAY_AGG(eti.item) FILTER (WHERE eti.item IS NOT NULL) as treasure_items,
            en.treasure_currency,
            en.status,
            en.party_size,
            en.party_level,
            en.extra_experience as "extra_experience!",
            en.total_experience,
            en.total_items_value,
            en.encounter_type_id,
            en.subsystem_type_id,
            JSONB_AGG(jsonb_build_object('name', esc.name, 'vp', esc.vp, 'roll_options', esc.roll_options)) as subsystem_rolls,
            en.owner
        FROM encounters en
        LEFT JOIN encounter_enemies ee ON en.id = ee.encounter
        LEFT JOIN encounter_hazards eh ON en.id = eh.encounter
        LEFT JOIN encounter_treasure_items eti ON en.id = eti.encounter
        LEFT JOIN LATERAL (
            SELECT JSONB_AGG(jsonb_build_object('skill', escr.roll, 'dc', escr.dc)) as roll_options, esc.name, esc.vp, esc.order_index
            FROM encounter_skill_checks esc
            LEFT JOIN encounter_skill_check_rolls escr ON esc.id = escr.encounter_skill_check_id
            WHERE esc.encounter_id = en.id
            GROUP BY esc.id
        ) esc ON TRUE
        WHERE en.status = $1
        AND en.owner = $2
        GROUP BY en.id
    "#,
        CompletionStatus::Draft.as_i32() as i16,
        owner.0 as i64,
    )
    .fetch_optional(exec)
    .await?;

    let res = if let Some(row) = encounter {
        let enemies = row
            .enemies
            .unwrap_or_default()
            .iter()
            .zip(row.enemy_level_adjustments.unwrap_or_default().iter())
            .map(|(id, adj)| EncounterEnemy {
                id: InternalId(*id as u32),
                level_adjustment: *adj,
            })
            .collect();

        let hazards = row
            .hazards
            .unwrap_or_default()
            .iter()
            .map(|id| InternalId(*id as u32))
            .collect();

        let subsystem_rolls: Vec<EncounterSubsystemCheck> =
            serde_json::from_value(row.subsystem_rolls.unwrap_or_default()).unwrap_or_default();
        let encounter_subsystem_type = row
            .subsystem_type_id
            .map(|id| EncounterSubsystemType::from_i32(id as i32));
        let encounter_type = EncounterType::from_id_and_parts(
            row.encounter_type_id,
            enemies,
            hazards,
            encounter_subsystem_type,
            subsystem_rolls,
        );

        Ok(Some(Encounter {
            id: InternalId(row.id as u32),
            name: row.name,
            description: row.description,
            status: CompletionStatus::from_i32(row.status as i32),
            owner: InternalId(row.owner as u32),
            session_id: None,
            encounter_type,
            treasure_items: row
                .treasure_items
                .unwrap_or_default()
                .iter()
                .map(|id| InternalId(*id as u32))
                .collect(),
            party_level: row.party_level as u32,
            party_size: row.party_size as u32,
            extra_experience: row.extra_experience,
            total_experience: row.total_experience,
            total_items_value: row.total_items_value as i32,
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
                    encounter_type: EncounterType::Combat {
                        enemies: vec![],
                        hazards: vec![],
                    },
                    status: CompletionStatus::Draft,
                    treasure_items: vec![],
                    treasure_currency: 0.0,
                    extra_experience: 0,
                    total_experience: 0,
                    total_items_value: 0,
                    party_level: 1,
                    party_size: 4,
                })
            })
    };
    res
}

// Helper function accessing creatures databases to get levels of enemies given their ids and adjustments
// Used for default experience calculation
async fn get_levels_enemies(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    enemies: &[InternalId],
    enemy_level_adjustments: &[i16],
) -> crate::Result<Vec<i16>> {
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
                .map(|l| *l as i16 + adjustment)
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
) -> crate::Result<Vec<i16>> {
    let ids = hazards.iter().map(|id| id.0).collect::<Vec<u32>>();
    let hazards_fetched = super::hazards::get_hazards(exec, &HazardFiltering::from_ids(&ids))
        .await?
        .into_iter()
        .map(|h| (h.id, h.level))
        .collect::<HashMap<_, _>>();

    let mut levels = vec![];
    for hazard in hazards {
        levels.push(hazards_fetched.get(hazard).copied().unwrap_or_default() as i16);
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

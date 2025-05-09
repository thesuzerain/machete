use std::collections::HashMap;

use crate::models;
use crate::models::encounter::Encounter;
use crate::models::encounter::{
    EncounterEnemy, EncounterSubsystemCheck, EncounterSubsystemType, EncounterType,
};
use crate::models::ids::InternalId;
use crate::models::query::CommaSeparatedVec;
use serde::{Deserialize, Serialize};
use sqlx::PgConnection;

use super::creatures::CreatureFiltering;
use super::hazards::HazardFiltering;
use super::items::ItemFiltering;

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct EncounterFilters {
    pub ids: Option<CommaSeparatedVec>,
    pub campaign_id: Option<InternalId>,
    pub name: Option<String>,
    pub encounter_type: Option<String>,
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
}

#[derive(serde::Deserialize, Debug)]
pub struct ModifyEncounter {
    pub name: Option<String>,
    pub description: Option<String>,

    pub enemies: Option<Vec<EncounterEnemy>>,
    pub hazards: Option<Vec<InternalId>>,

    pub subsystem_checks: Option<Vec<EncounterSubsystemCheck>>,
    pub subsystem_type: Option<EncounterSubsystemType>,

    pub encounter_type_id: Option<u8>,

    pub treasure_items: Option<Vec<InternalId>>,
    pub treasure_currency: Option<f32>,
    pub extra_experience: Option<i32>,

    pub initialization_encounter: Option<bool>,

    pub party_level: Option<u8>,
    pub party_size: Option<u8>,
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
            any_value(cs.campaign_id) as campaign_id,
            ee.enemies,
            ee.level_adjustments as enemy_level_adjustments,
            eh.hazards,
            eti.items as treasure_items,
            en.treasure_currency,
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
        LEFT JOIN campaign_sessions cs ON en.session_id = cs.id
        LEFT JOIN LATERAL (
            SELECT 
                ARRAY_AGG(enemy) FILTER (WHERE ee.enemy IS NOT NULL) as enemies, 
                ARRAY_AGG(level_adjustment) FILTER (WHERE ee.enemy IS NOT NULL) as level_adjustments 
            FROM encounter_enemies ee WHERE en.id = ee.encounter
        ) ee ON TRUE
        LEFT JOIN LATERAL (
            SELECT ARRAY_AGG(hazard) FILTER (WHERE eh.hazard IS NOT NULL) as hazards
            FROM encounter_hazards eh WHERE en.id = eh.encounter
        ) eh ON TRUE
        LEFT JOIN LATERAL (
            SELECT ARRAY_AGG(item) FILTER (WHERE eti.item IS NOT NULL) as items
            FROM encounter_treasure_items eti WHERE en.id = eti.encounter
        ) eti ON TRUE
        LEFT JOIN LATERAL (
            SELECT JSONB_AGG(jsonb_build_object('skill', escr.roll, 'dc', escr.dc)) as roll_options, esc.name, esc.vp, esc.order_index
            FROM encounter_skill_checks esc
            LEFT JOIN encounter_skill_check_rolls escr ON esc.id = escr.encounter_skill_check_id
            WHERE esc.encounter_id = en.id
            GROUP BY esc.id
        ) esc ON TRUE
        WHERE 
            ($1::text IS NULL OR en.name LIKE '%' || $1 || '%')
            AND ($2::int[] IS NULL OR en.id = ANY($2::int[]))
            AND ($3::integer IS NULL OR en.encounter_type_id = $4)
            AND en.owner = $4
            AND ($5::int IS NULL OR cs.campaign_id = $5)
        GROUP BY en.id, ee.enemies, ee.level_adjustments, eh.hazards, eti.items
    "#,
        condition.name,
        &ids as _,
        condition
            .encounter_type
            .as_deref()
            .map(|x| EncounterType::id_from_string(x)),
        owner.0 as i64,
        condition.campaign_id.map(|id| id.0 as i32),
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
                .map(EncounterSubsystemType::from_i32);
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
                campaign_id: row.campaign_id.map(|id| InternalId(id as u32)),
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
        let enemy_level_adjustments = enemies
            .iter()
            .map(|e| e.level_adjustment)
            .collect::<Vec<i16>>();
        let enemy_levels =
            get_levels_enemies(&mut **tx, &enemy_ids, &enemy_level_adjustments).await?;
        let hazard_level_complexities =
            get_levels_complexities_hazards(&mut **tx, &hazards).await?;
        let treasure_values = get_values_items(&mut **tx, &encounter.treasure_items).await?;

        // TODO: Mofiy this so that it only does these db call if needed
        // (This may be redundant- it's related to the total_experience value)
        // TODO: Also do this in 'edit_encounter'

        let derived_total_experience = models::encounter::calculate_total_adjusted_experience(
            &enemy_levels,
            &hazard_level_complexities,
            encounter.party_level,
            encounter.party_size,
        ) + encounter.extra_experience;
        let derived_total_treasure_value =
            treasure_values.iter().sum::<f32>() + encounter.treasure_currency;

        let encounter_id = sqlx::query!(
            r#"
            INSERT INTO encounters (name, description, encounter_type_id, subsystem_type_id, treasure_currency, party_size, party_level, extra_experience, total_experience, total_items_value, owner)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING id
            "#,
            &encounter.name,
            encounter.description.as_deref(),
            encounter_type_id as i32,
            encounter_subsystem_type.as_i32() as i32,
            encounter.treasure_currency as f64,
            encounter.party_size as i64,
            encounter.party_level as i64,
            encounter.extra_experience as i64,
            derived_total_experience as i64,
            derived_total_treasure_value as f64,
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
            &enemy_level_adjustments,
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
    owner: InternalId,
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
        party_size = COALESCE($4, party_size),
        party_level = COALESCE($5, party_level),
        extra_experience = COALESCE($6, extra_experience),
        encounter_type_id = COALESCE($7, encounter_type_id),
        subsystem_type_id = COALESCE($8, subsystem_type_id)
        WHERE id = $9
        "#,
        new_encounter.name.as_deref(),
        new_encounter.description.as_deref(),
        new_encounter.treasure_currency.as_ref().map(|c| *c as f64),
        new_encounter.party_size.map(|s| s as i32),
        new_encounter.party_level.map(|l| l as i32),
        new_encounter.extra_experience.map(|e| e as i32),
        new_encounter.encounter_type_id.map(|e| e as i32),
        new_encounter
            .subsystem_type
            .as_ref()
            .map(|e| e.as_i32() as i32),
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

    // Update subsystem stuff if needed
    if let Some(subsystem_rolls) = new_encounter.subsystem_checks.as_ref() {
        sqlx::query!(
            r#"
            DELETE FROM encounter_skill_check_rolls
            WHERE encounter_skill_check_id IN (
                SELECT id
                FROM encounter_skill_checks
                WHERE encounter_id = $1
            )
            "#,
            encounter_id.0 as i32,
        )
        .execute(&mut **tx)
        .await?;

        // Drop all existing encounter_skill_checks
        sqlx::query!(
            r#"
            DELETE FROM encounter_skill_checks
            WHERE encounter_id = $1
            "#,
            encounter_id.0 as i32,
        )
        .execute(&mut **tx)
        .await?;

        // Insert new encounter_skill_checks
        let mut order_index = 0;
        for check in subsystem_rolls {
            let check_id = sqlx::query!(
                r#"
                INSERT INTO encounter_skill_checks (encounter_id, name, vp, order_index)
                VALUES ($1, $2, $3, $4)
                RETURNING id
                "#,
                encounter_id.0 as i32,
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
    }

    // Calculate the new total_experience and total_items_value. Fetch the encounter.
    // TODO: This is a bit of a hack.
    // TODO: This maybe also modularizable
    let encounter = get_encounters(
        &mut **tx,
        owner,
        &EncounterFilters::from_ids(&[encounter_id]),
    )
    .await?
    .into_iter()
    .next()
    .ok_or(crate::ServerError::NotFound)?;
    let enemy_ids = encounter
        .encounter_type
        .get_enemies()
        .iter()
        .map(|e| e.id)
        .collect::<Vec<InternalId>>();
    let enemy_level_adjustments = encounter
        .encounter_type
        .get_enemies()
        .iter()
        .map(|e| e.level_adjustment)
        .collect::<Vec<i16>>();
    let enemy_levels = get_levels_enemies(&mut **tx, &enemy_ids, &enemy_level_adjustments).await?;
    let hazard_level_complexities =
        get_levels_complexities_hazards(&mut **tx, &encounter.encounter_type.get_hazards()).await?;
    let treasure_values = get_values_items(&mut **tx, &encounter.treasure_items).await?;

    let derived_total_experience = models::encounter::calculate_total_adjusted_experience(
        &enemy_levels,
        &hazard_level_complexities,
        encounter.party_level as u8,
        encounter.party_size as u8,
    ) + encounter.extra_experience as i32;
    let derived_total_treasure_value =
        treasure_values.iter().sum::<f32>() + encounter.treasure_currency;

    sqlx::query!(
        r#"
        UPDATE encounters
        SET total_experience = $1,
            total_items_value = $2
        WHERE id = $3
        "#,
        derived_total_experience as i64,
        derived_total_treasure_value as f64,
        encounter_id.0 as i64,
    )
    .execute(&mut **tx)
    .await?;

    // Unlink and re-link the encounter to the session if needed
    // TODO: We can refactor this editing to not need to explicitly unlinking/relinking (by being more explicit)
    if let Some(session_id) = unlinked_session_id {
        super::sessions::link_encounter_to_session(&mut *tx, encounter_id, session_id).await?;
    }

    Ok(())
}

pub async fn delete_encounters(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    encounter_id: &[InternalId],
) -> crate::Result<()> {
    if encounter_id.is_empty() {
        return Ok(());
    }

    // First, for all encounters, unlink them from their sessions
    for id in encounter_id {
        super::sessions::unlink_encounter_from_session(&mut *tx, *id).await?;
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
    .execute(&mut **tx)
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
    .execute(&mut **tx)
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
    .execute(&mut **tx)
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
    .execute(&mut **tx)
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
    .execute(&mut **tx)
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
    .execute(&mut **tx)
    .await?;

    Ok(())
}

// Helper function accessing creatures databases to get levels of enemies given their ids and adjustments
// Used for default experience calculation
async fn get_levels_enemies(
    conn: &mut PgConnection,
    enemies: &[InternalId],
    enemy_level_adjustments: &[i16],
) -> crate::Result<Vec<i16>> {
    let ids = enemies.iter().map(|id| id.0).collect::<Vec<u32>>();
    let creatures = super::creatures::get_creatures(conn, &CreatureFiltering::from_ids(&ids))
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
async fn get_levels_complexities_hazards(
    conn: &mut PgConnection,
    hazards: &[InternalId],
) -> crate::Result<Vec<(i16, bool)>> {
    let ids = hazards.iter().map(|id| id.0).collect::<Vec<u32>>();
    let hazards_fetched = super::hazards::get_hazards(conn, &HazardFiltering::from_ids(&ids))
        .await?
        .into_iter()
        .map(|h| (h.id, (h.level, h.complex)))
        .collect::<HashMap<_, _>>();

    let mut levels = vec![];
    for hazard in hazards {
        let (level, complex) = hazards_fetched.get(hazard).copied().unwrap_or_default();
        levels.push((level as i16, complex));
    }

    Ok(levels)
}

// Helper function accessing items databases to get values of items given their ids
// Used for default treasure value calculation
async fn get_values_items(
    conn: &mut PgConnection,
    items: &[InternalId],
) -> crate::Result<Vec<f32>> {
    // TODO: This needs to handle 'priceless' items better- currently just estimates as 0
    let ids = items.iter().map(|id| id.0).collect::<Vec<u32>>();
    let items_fetched = super::items::get_items(conn, &ItemFiltering::from_ids(&ids))
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

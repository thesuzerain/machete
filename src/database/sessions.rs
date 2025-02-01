use std::collections::{HashMap, HashSet};

use crate::models::campaign::{CampaignSession, CampaignSessionCharacterRewards};
use crate::models::ids::InternalId;
use chrono::{DateTime, Utc};
use itertools::Itertools;
use serde::Deserialize;

#[derive(serde::Deserialize)]
pub struct InsertSession {
    pub session_order: u32,
    pub name: Option<String>,
    pub description: Option<String>,
    pub play_date: Option<DateTime<Utc>>,
}

#[derive(serde::Deserialize, Debug)]
pub struct ModifySession {
    pub session_order: Option<u32>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub play_date: Option<DateTime<Utc>>,
}

#[derive(serde::Deserialize)]
pub struct LinkEncounterSession {
    pub encounter_id: InternalId,
}

#[derive(serde::Deserialize, Debug)]
pub struct UpdateEncounterSessions {
    // encounter_id -> character_id -> rewards
    pub compiled_gold_rewards: HashMap<InternalId, HashMap<InternalId, f64>>,
    pub compiled_item_rewards: HashMap<InternalId, HashMap<InternalId, Vec<InternalId>>>,

    pub unassigned_gold_rewards: HashMap<InternalId, f64>,
    pub unassigned_item_rewards: HashMap<InternalId, Vec<InternalId>>,
}

// TODO: May be prudent to make a separate models system for the database.
pub async fn get_sessions(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres> + Copy,
    owner: InternalId,
    campaign_id: InternalId,
) -> crate::Result<Vec<CampaignSession>> {

    // TODO: Campaign needs to be checked for ownership
    let query = sqlx::query!(
        r#"
        SELECT 
            s.id,
            s.session_order,
            s.name,
            s.description,
            s.play_date,
            ARRAY_AGG(e.id) filter (where e.id is not null) as encounter_ids,
            unassigned_gold_rewards,
            unassigned_item_rewards,
            COALESCE(JSONB_AGG(
                JSONB_BUILD_OBJECT(
                    'session_id', csc.session_id,
                    'character_id', csc.character_id,
                    'gold_rewards', csc.gold_rewards,
                    'item_rewards', csc.item_rewards
                )
            ), '[]') as "character_rewards!"
        FROM campaign_sessions s
        LEFT JOIN campaigns ca ON s.campaign_id = ca.id
        LEFT JOIN encounters e ON s.id = e.session_id
        LEFT JOIN campaign_session_characters csc ON s.id = csc.session_id
        WHERE 
            ca.id = $1
            AND ca.owner = $2
        GROUP BY s.id
        ORDER BY s.session_order ASC
    "#,
        campaign_id.0 as i32,
        owner.0 as i32,
    );

    let sessions = query
        .fetch_all(exec)
        .await?
        .into_iter()
        .map(|row| {
            #[derive(Deserialize, Debug)]
            struct RowCharacterRewards {
                session_id: i32,
                character_id: i32,
                gold_rewards: f64,
                item_rewards: Vec<i32>,
            }

            let character_rewards: Vec<RowCharacterRewards> = serde_json::from_value(row.character_rewards).unwrap();
            let compiled_rewards: HashMap<InternalId, CampaignSessionCharacterRewards> = character_rewards
                .into_iter()
                .fold(HashMap::new(), |mut acc, row| {
                    let character_id = InternalId(row.character_id as u64);
                    acc.insert(
                        character_id,
                        CampaignSessionCharacterRewards {
                            items: row.item_rewards.iter().map(|e| InternalId(*e as u64)).collect(),
                            gold: row.gold_rewards,
                        },
                    );
                    acc
                });
            let encounter_ids = row.encounter_ids.unwrap_or_default().iter().map(|e| InternalId(*e as u64)).collect::<Vec<InternalId>>();


            Ok(CampaignSession {
                id: InternalId(row.id as u64),
                name: row.name,
                description: row.description,
                session_order: row.session_order as u32,
                play_date: row.play_date,
                encounter_ids,
                compiled_rewards,
                unassigned_gold_rewards: row.unassigned_gold_rewards,
                unassigned_item_rewards: row.unassigned_item_rewards.iter().map(|e| InternalId(*e as u64)).collect(),
            })
        })
        .collect::<Result<Vec<CampaignSession>, sqlx::Error>>()?;

    Ok(sessions)
}

pub async fn update_sessions(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres> + Copy,
    sessions: &HashMap<InternalId, ModifySession>,
) -> crate::Result<()> {
    // TODO: Create non-iterative version of this (or rather just move iteration onto postgres side)
    for (session_id, session) in sessions.iter() {
        let query = sqlx::query!(
            r#"
            UPDATE campaign_sessions
            SET
                session_order = COALESCE($2, session_order),
                name = COALESCE($3, name),
                description = COALESCE($4, description),
                play_date = COALESCE($5, play_date)
            WHERE id = $1
            "#,
            session_id.0 as i32,
            session.session_order.map(|e| e as i32),
            session.name.clone(),
            session.description.clone(),
            session.play_date
        );
        query.execute(exec).await?;
    }
    Ok(())
}

pub async fn insert_sessions(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres> + Copy,
    campaign_id: InternalId,
    sessions: &[InsertSession],
) -> crate::Result<()> {
    // TODO: Campaign needs to be checked for ownership
    if sessions.is_empty() {
        return Ok(());
    }

    let campaign_id = std::iter::once(campaign_id.0 as i32)
        .cycle()
        .take(sessions.len())
        .collect::<Vec<i32>>();
    let (session_orders, names, descriptions, play_dates): (
        Vec<i32>,
        Vec<Option<String>>,
        Vec<Option<String>>,
        Vec<DateTime<Utc>>,
    ) = sessions
        .iter()
        .map(|e| {
            // TODO: remove clones
            let date_or_now = e.play_date.unwrap_or_else(Utc::now);
            (
                e.session_order as i32,
                e.name.clone(),
                e.description.clone(),
                date_or_now,
            )
        })
        .multiunzip();

    sqlx::query!(
        r#"
        INSERT INTO campaign_sessions (session_order, name, description, play_date, campaign_id)
        SELECT * FROM UNNEST ($1::int[], $2::varchar[], $3::varchar[], $4::timestamptz[], $5::int[])
        "#,
        &session_orders as _,
        &names.as_ref() as &[Option<String>],
        &descriptions.as_ref() as &[Option<String>],
        &play_dates as _,
        &campaign_id as _,
    )
    .execute(exec)
    .await?;

    Ok(())
}

pub async fn delete_session(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres> + Copy,
    session_id: InternalId,
) -> crate::Result<()> {
    // TODO:  Ensure FE has suitable checks for this (campaign ownership, but also, confirmation modal)
    sqlx::query!(
        r#"
        DELETE FROM campaign_sessions
        WHERE id = $1
        "#,
        session_id.0 as i32,
    )
    .execute(exec)
    .await?;

    Ok(())
}

pub async fn get_owned_session_ids(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres> + Copy,
    session_ids: &[InternalId],
    owner: InternalId,
) -> crate::Result<Vec<InternalId>> {
    let query = sqlx::query!(
        r#"
        SELECT 
            s.id AS "id!"
        FROM campaign_sessions s
        LEFT JOIN campaigns ca ON s.campaign_id = ca.id
        WHERE 
            s.id = ANY($1::int[])
            AND ca.owner = $2
        "#,
        &session_ids
            .iter()
            .map(|id| id.0 as i32)
            .collect::<Vec<i32>>(),
        owner.0 as i32,
    )
    .fetch_all(exec)
    .await?
    .iter()
    .map(|row| InternalId(row.id as u64))
    .collect();

    Ok(query)
}

pub async fn link_encounter_to_session(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres> + Copy,
    encounter_id: InternalId,
    session_id: InternalId,
) -> crate::Result<()> {
    // First, unlink the encounter from any existing session
    unlink_encounter_from_session(exec, encounter_id, session_id).await?;

    // Second, link the encounter to the new session
    sqlx::query!(
        r#"
        UPDATE encounters
        SET session_id = $1
        WHERE id = $2
        "#,
        session_id.0 as i32,
        encounter_id.0 as i64,
    )
    .execute(exec)
    .await?;

    sqlx::query!(
        r#"
        INSERT INTO campaign_session_encounters (session_id, encounter_id, unassigned_gold_rewards, unassigned_item_rewards)
        SELECT  $1, $2, treasure_currency, treasure_items
        FROM encounters
        WHERE id = $2
        "#,
        session_id.0 as i32,
        encounter_id.0 as i64,
    )
    .execute(exec)
    .await?;

    sqlx::query!(
        r#"
        INSERT INTO campaign_session_encounter_character_assignments (session_id, encounter_id, character_id, gold_rewards, item_rewards)
        SELECT $1, $2, ch.id, 0, '{}'
        FROM characters ch
        INNER JOIN campaigns cp ON ch.campaign = cp.id
        INNER JOIN campaign_sessions cs ON cp.id = cs.campaign_id
        WHERE cs.id = $1
    "#,
        session_id.0 as i32,
        encounter_id.0 as i64,
    )
    .execute(exec)
    .await?;

    Ok(())
}

pub async fn unlink_encounter_from_session(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres> + Copy,
    encounter_id: InternalId,
    session_id: InternalId,
) -> crate::Result<()> {
    sqlx::query!(
        r#"
        UPDATE encounters
        SET session_id = NULL
        WHERE id = $1
        RETURNING treasure_currency
        "#,
        encounter_id.0 as i64,
    )
    .fetch_one(exec)
    .await?;

    // Delete associated assignment records
    sqlx::query!(
        r#"
        DELETE FROM campaign_session_encounters
        WHERE session_id = $1
        AND encounter_id = $2
        "#,
        session_id.0 as i32,
        encounter_id.0 as i64,
    )
    .execute(exec)
    .await?;

    sqlx::query!(
        r#"
        DELETE FROM campaign_session_encounter_character_assignments
        WHERE session_id = $1
        AND encounter_id = $2
        "#,
        session_id.0 as i32,
        encounter_id.0 as i64,
    )
    .execute(exec)
    .await?;

    Ok(())
}

// TODO: This function needs to be revisited soon. A lot of updates (even when we are only updating a small part), and it uses unnest poorly. In addition, this route may get hit A LOT.
pub async fn edit_encounter_session_character_assignments(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    session_id: InternalId,
    updates: &UpdateEncounterSessions,
) -> crate::Result<()> {
    let tx_id: (i64,) = sqlx::query_as("SELECT txid_current()")
        .fetch_one(&mut **tx)
        .await?;
    let tx_id = tx_id.0;

    // Pre-select as fast insertion/deletion can cause inconsistency here
    // TODO: Slow down the calls to this route on the front end to prevent this from happening
    sqlx::query!(
        "SELECT 1 as one FROM campaign_sessions WHERE id = $1 FOR UPDATE",
        session_id.0 as i32
    )
    .fetch_all(&mut **tx)
    .await?;

    // Delet all existing character assignments for the session
    sqlx::query!(
        r#"
            DELETE FROM campaign_session_encounter_character_assignments WHERE session_id = $1
        "#,
        session_id.0 as i32,
    )
    .execute(&mut **tx)
    .await?;

    // TODO: This looping is really unacceptable- need a better solution w.r.t one-level UNNEST
    let encounter_ids = updates
        .compiled_gold_rewards
        .keys()
        .chain(updates.compiled_item_rewards.keys())
        .copied()
        .collect::<HashSet<InternalId>>();
    for encounter_id in encounter_ids {
        let enc_err = |e| {
            format!("Encounter {} not found in {}. All encounters must be provided for all fields in a bulk update. Error: {}", encounter_id, session_id, e)
        };

        let gold_rewards = updates
            .compiled_gold_rewards
            .get(&encounter_id)
            .ok_or_else(|| crate::ServerError::BadRequest(enc_err("compiled_gold_rewards")))?;
        let item_rewards = updates
            .compiled_item_rewards
            .get(&encounter_id)
            .ok_or_else(|| crate::ServerError::BadRequest(enc_err("compiled_item_rewards")))?;
        let unassigned_gold_rewards = updates
            .unassigned_gold_rewards
            .get(&encounter_id)
            .ok_or_else(|| crate::ServerError::BadRequest(enc_err("unassigned_gold_rewards")))?;
        let unassigned_item_rewards = updates
            .unassigned_item_rewards
            .get(&encounter_id)
            .ok_or_else(|| crate::ServerError::BadRequest(enc_err("unassigned_item_rewards")))?;

        let character_ids = gold_rewards
            .keys()
            .chain(item_rewards.keys())
            .copied()
            .collect::<HashSet<InternalId>>();

        for character_id in character_ids {
            let gold_reward = gold_rewards.get(&character_id).copied().unwrap_or_default();
            let item_reward = item_rewards
                .get(&character_id)
                .cloned()
                .unwrap_or_default()
                .iter()
                .map(|e| e.0 as i32)
                .collect::<Vec<i32>>();

            sqlx::query!(
                r#"
                INSERT INTO campaign_session_encounter_character_assignments (session_id, encounter_id, character_id, gold_rewards, item_rewards)
                VALUES ($1, $2, $3, $4, $5)
                "#,
                session_id.0 as i32,
                encounter_id.0 as i64,
                character_id.0 as i64,
                gold_reward,
                &item_reward,
            )
            .execute(&mut **tx)
            .await?;
        }

        sqlx::query!(
            r#"
            UPDATE campaign_session_encounters
            SET
                unassigned_gold_rewards = $1,
                unassigned_item_rewards = $2
            WHERE session_id = $3
            AND encounter_id = $4
            "#,
            unassigned_gold_rewards,
            &unassigned_item_rewards
                .iter()
                .map(|e| e.0 as i32)
                .collect::<Vec<i32>>(),
            session_id.0 as i32,
            encounter_id.0 as i64,
        )
        .execute(&mut **tx)
        .await?;
    }

    Ok(())
}

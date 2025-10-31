use std::collections::HashMap;

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

    // Present characters
    pub characters: Option<Vec<InternalId>>,
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

/// Update structure for session-character assignments
/// 'Unassigned' fields are calculated by the route.
// TODO: You recently moved to move unassigned to no longer be explicitly assigned, at the cost of an extra db call here. Reevaluate this decision.
#[derive(serde::Deserialize, Debug)]
pub struct UpdateCharacterSessions {
    //  character_id -> rewards
    pub compiled_rewards: HashMap<InternalId, CampaignSessionCharacterRewards>,
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
            unassigned_items.unassigned_items,
            csc.character_rewards,
            SUM(e.total_items_value + e.treasure_currency) as total_combined_treasure_value,
            SUM(e.total_experience) as total_experience,
            SUM(SUM(e.total_experience)::int) OVER (ORDER BY s.session_order, s.id ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW) as accumulated_total_experience
        FROM campaign_sessions s
        LEFT JOIN campaigns ca ON s.campaign_id = ca.id
        LEFT JOIN encounters e ON s.id = e.session_id
        LEFT JOIN LATERAL (
            SELECT session_id, ARRAY_AGG(library_item_id) filter (where library_item_id is not null) as unassigned_items
            FROM item_instances ii
            WHERE ii.character_id IS NULL
            GROUP BY ii.session_id
        ) unassigned_items ON unassigned_items.session_id = s.id
        LEFT JOIN LATERAL (
            SELECT
                csc.session_id,
                JSONB_AGG(
                JSONB_BUILD_OBJECT(
                    'session_id', csc.session_id,
                    'character_id', csc.character_id,
                    'gold_rewards', csc.gold_rewards,
                    'item_rewards', csc.item_rewards,
                    'present', csc.present
                )
            ) filter (where csc.session_id is not null) as character_rewards
            FROM (
                SELECT
                    csc.session_id, csc.character_id, csc.gold_rewards, csc.present,
                    JSONB_AGG(
                        JSONB_BUILD_OBJECT(
                            'id', ci.id,
                            'library_item_id', ci.library_item_id
                        )                    
                    ) FILTER (WHERE ci.id IS NOT NULL) as item_rewards                     
                FROM campaign_session_characters csc
                FULL OUTER JOIN item_instances ci ON ci.character_id = csc.character_id AND ci.session_id = csc.session_id
                GROUP BY csc.session_id, csc.character_id
             ) csc
            GROUP BY csc.session_id
        ) csc ON s.id = csc.session_id

        WHERE 
            ca.id = $1
            AND ca.owner = $2
        GROUP BY s.id, character_rewards, unassigned_items.unassigned_items
        ORDER BY s.session_order, s.id ASC
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
                character_id: i32,
                present: bool,
                gold_rewards: f64,
                item_rewards: Option<Vec<RowCharacterItems>>,
                session_id: i32,
            }

            #[derive(Deserialize, Debug)]
            struct RowCharacterItems {
                id: i32,
                library_item_id: i32,
            }

            let character_rewards: Vec<RowCharacterRewards> = row
                .character_rewards
                .and_then(|x| serde_json::from_value(x).unwrap())
                .unwrap_or_default();

            let compiled_rewards: HashMap<InternalId, CampaignSessionCharacterRewards> =
                character_rewards
                    .into_iter()
                    .fold(HashMap::new(), |mut acc, row| {
                        let character_id = InternalId(row.character_id as u32);
                        acc.insert(
                            character_id,
                            CampaignSessionCharacterRewards {
                                items: row
                                    .item_rewards
                                    .unwrap_or_default()
                                    .iter()
                                    .map(|e| InternalId(e.library_item_id as u32))
                                    .collect(),
                                present: row.present,
                                gold: row.gold_rewards,
                            },
                        );
                        acc
                    });

            let encounter_ids = row
                .encounter_ids
                .unwrap_or_default()
                .iter()
                .map(|e| InternalId(*e as u32))
                .collect::<Vec<InternalId>>();

            let accumulated_total_experience = row.accumulated_total_experience.unwrap_or(0) as f64;
            let level_at_end = 1 + (accumulated_total_experience / 1000.0).floor() as u8;
            let experience_at_end = (accumulated_total_experience % 1000.0) as u64;

            Ok(CampaignSession {
                id: InternalId(row.id as u32),
                name: row.name,
                description: row.description,
                session_order: row.session_order as u32,
                play_date: row.play_date,
                encounter_ids,
                compiled_rewards,
                unassigned_gold_rewards: row.unassigned_gold_rewards,
                unassigned_item_rewards: row
                    .unassigned_items
                    .unwrap_or_default()
                    .iter()
                    .map(|e| InternalId::from_i32(*e))
                    .collect(),
                total_experience: row.total_experience.map(|e| e as u64).unwrap_or_default(),
                experience_at_end,
                level_at_end,
                total_combined_treasure_value: row
                    .total_combined_treasure_value
                    .unwrap_or_default(),
            })
        })
        .collect::<Result<Vec<CampaignSession>, sqlx::Error>>()?;

    Ok(sessions)
}

pub async fn update_sessions(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
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
        query.execute(&mut **tx).await?;
    }
    Ok(())
}

pub async fn insert_sessions(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    campaign_id: InternalId,
    sessions: &[InsertSession],
) -> crate::Result<Vec<InternalId>> {
    // TODO: Campaign needs to be checked for ownership
    if sessions.is_empty() {
        return Ok(vec![]);
    }

    let campaign_id = std::iter::once(campaign_id.0 as i32)
        .cycle()
        .take(sessions.len())
        .collect::<Vec<i32>>();

    #[allow(clippy::type_complexity)]
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

    let ids = sqlx::query!(
        r#"
        INSERT INTO campaign_sessions (session_order, name, description, play_date, campaign_id)
        SELECT * FROM UNNEST ($1::int[], $2::varchar[], $3::varchar[], $4::timestamptz[], $5::int[])
        RETURNING id
        "#,
        &session_orders as _,
        &names.as_ref() as &[Option<String>],
        &descriptions.as_ref() as &[Option<String>],
        &play_dates as _,
        &campaign_id as _,
    )
    .fetch_all(&mut **tx)
    .await?
    .into_iter()
    .map(|row| InternalId(row.id as u32))
    .collect::<Vec<InternalId>>();

    // Insert empty campaign_session_characters for each session
    // TODO: Assumes same order, which I think is fine, but should be checked
    for (i, session) in sessions.iter().enumerate() {
        let Some(ref characters) = session.characters else {
            continue;
        };
        let characters = characters.iter().map(|e| e.0 as i32).collect::<Vec<i32>>();
        sqlx::query!(
            r#"
            INSERT INTO campaign_session_characters (session_id, character_id, gold_rewards, present)
            SELECT $1, character_id, 0, true
            FROM UNNEST($2::int[]) as c(character_id)
        "#,
            ids[i].0 as i32,
            &characters,

        )
        .execute(&mut **tx)
        .await?;
    }

    Ok(ids)
}

pub async fn delete_session(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    session_id: InternalId,
) -> crate::Result<()> {
    sqlx::query!(
        r#"
        UPDATE encounters
        SET session_id = NULL
        WHERE session_id = $1
        "#,
        session_id.0 as i32,
    )
    .execute(&mut **tx)
    .await?;

    // First delete campaign_session_characters, item_instances
    sqlx::query!(
        r#"
        UPDATE item_instances
        SET session_id = NULL, character_id = NULL
        WHERE session_id = $1
        "#,
        session_id.0 as i32,
    )
    .execute(&mut **tx)
    .await?;

    sqlx::query!(
        r#"
        DELETE FROM campaign_session_characters
        WHERE session_id = $1
        "#,
        session_id.0 as i32,
    )
    .execute(&mut **tx)
    .await?;

    sqlx::query!(
        r#"
        DELETE FROM campaign_sessions
        WHERE id = $1
        "#,
        session_id.0 as i32,
    )
    .execute(&mut **tx)
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
    .map(|row| InternalId(row.id as u32))
    .collect();

    Ok(query)
}

pub async fn link_encounter_to_session(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    encounter_id: InternalId,
    session_id: InternalId,
) -> crate::Result<()> {
    // Link the encounter to the new session
    sqlx::query!(
        r#"
        UPDATE encounters
        SET session_id = $1
        WHERE id = $2
        "#,
        session_id.0 as i32,
        encounter_id.0 as i64,
    )
    .execute(&mut **tx)
    .await?;

    // Add gold of linked encounter to unassigned rewards
    sqlx::query!(
        r#"
            UPDATE campaign_sessions
            SET 
                unassigned_gold_rewards = unassigned_gold_rewards + e.treasure_currency
            FROM (SELECT * FROM encounters WHERE id = $2) e
            WHERE campaign_sessions.id = $1
        "#,
        session_id.0 as i32,
        encounter_id.0 as i64,
    )
    .execute(&mut **tx)
    .await?;

    // Add items of linked encounter to unassigned rewards
    sqlx::query!(
        r#"
        UPDATE item_instances
        SET session_id = $1, character_id = NULL, 
            campaign_id = (SELECT campaign_id FROM campaign_sessions WHERE id = $1)
        WHERE encounter_id = $2 AND session_id IS NULL
        "#,
        session_id.0 as i32,
        encounter_id.0 as i64,
    )
    .execute(&mut **tx)
    .await?;

    // Update campaign experience to be sum of all encounters
    // TODO: Trigger in db? Just to ensure it recalculates correctly
    sqlx::query!(
        r#"
        UPDATE campaigns
            SET total_experience = campaigns.total_experience + e.total_experience
        FROM (SELECT * FROM encounters WHERE id = $1) e
        WHERE campaigns.id = (SELECT campaign_id FROM campaign_sessions WHERE id = $2)
        "#,
        encounter_id.0 as i64,
        session_id.0 as i32,
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}

/// Unlinks an encounter from a session, returning the session id if there was one to unlink.
pub async fn unlink_encounter_from_session(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    encounter_id: InternalId,
) -> crate::Result<Option<InternalId>> {
    // First, check if the encounter is linked to a session, and return some metadata. If not, return early.
    let res = sqlx::query!(
        r#"
        SELECT session_id, treasure_currency
        FROM encounters e
        WHERE id = $1
        "#,
        encounter_id.0 as i64,
    )
    .fetch_one(&mut **tx)
    .await?;

    let Some(session_id) = res.session_id else {
        return Ok(None);
    };

    // Update encounter to unlink it from the session
    sqlx::query!(
        r#"
        UPDATE encounters SET session_id = NULL WHERE id = $1
        "#,
        encounter_id.0 as i64,
    )
    .execute(&mut **tx)
    .await?;

    // remove experience from campaign
    // TODO: replace with triggers
    sqlx::query!(
        r#"
        UPDATE campaigns
        SET total_experience = campaigns.total_experience - e.total_experience
        FROM (SELECT * FROM encounters WHERE id = $1) e
        WHERE campaigns.id = (SELECT campaign_id FROM campaign_sessions WHERE id = $2)
        "#,
        encounter_id.0 as i64,
        session_id as i32,
    )
    .execute(&mut **tx)
    .await?;

    let mut gold: f64 = res.treasure_currency.unwrap_or_default();
    let gold_copy = gold;

    // Fetch unassigned gold and items
    let res = sqlx::query!(
        r#"
        SELECT unassigned_gold_rewards
        FROM campaign_sessions
        WHERE id = $1
        "#,
        session_id as i32,
    )
    .fetch_one(&mut **tx)
    .await?;
    let mut unassigned_gold_rewards: f64 = res.unassigned_gold_rewards;

    // Remove as much gold and items as the encounter contributed as possible from unassigned rewards
    remove_contributions_from_character(&mut gold, &mut unassigned_gold_rewards);

    sqlx::query!(
        r#"
        UPDATE campaign_sessions
        SET unassigned_gold_rewards = $1
        WHERE id = $2
        "#,
        unassigned_gold_rewards,
        session_id as i32,
    )
    .execute(&mut **tx)
    .await?;

    // Exit early if all gold and items were removed
    if gold == 0.0 {
        return Ok(Some(InternalId(session_id as u32)));
    }

    // Fetch total gold and items per character
    let character_golds = sqlx::query!(
        r#"
        SELECT csc.character_id, csc.gold_rewards
        FROM campaign_session_characters csc
        WHERE csc.session_id = $1
        GROUP BY csc.character_id, csc.session_id
        "#,
        session_id as i32,
    )
    .fetch_all(&mut **tx)
    .await?
    .into_iter()
    .fold(HashMap::new(), |mut acc, row| {
        acc.insert(row.character_id as u64, row.gold_rewards);
        acc
    });

    // Remove as much gold as the encounter contributed as possible from character rewards
    for (character_id, mut character_gold) in character_golds {
        remove_contributions_from_character(&mut gold, &mut character_gold);
        sqlx::query!(
            r#"
            UPDATE campaign_session_characters SET gold_rewards = $1
            WHERE session_id = $2 AND character_id = $3
            "#,
            character_gold,
            session_id as i32,
            character_id as i64,
        )
        .execute(&mut **tx)
        .await?;
    }

    // Remove all items- by setting session_id to NULL for all items for which the session and encounter match
    sqlx::query!(
        r#"
        UPDATE item_instances
        SET session_id = NULL, character_id = NULL
        WHERE session_id = $1 AND encounter_id = $2
        "#,
        session_id as i32,
        encounter_id.0 as i64,
    )
    .execute(&mut **tx)
    .await?;

    // Failure to remove all gold from rewards is an internal error
    // It should be invariant that sum of gold from encounter == sum of gold from rewards
    // Should not get into either of these blocks unless there is a bug in the code
    if gold > 0.0 {
        return Err(crate::ServerError::InternalError(format!("Could not unlink successfully: inconsistent number of gold. {} gold in characters/unassigned, {} gold in encounter.", gold_copy - gold, gold_copy)));
    }

    Ok(Some(InternalId(session_id as u32)))
}

// TODO: This function could be made more efficient- or perhaps moved entirely into Postgres
fn remove_contributions_from_character(remove_gold: &mut f64, character_gold: &mut f64) {
    // Remove as much gold as possible
    if remove_gold <= character_gold {
        *character_gold -= *remove_gold;
        *remove_gold = 0.0;
    } else {
        *remove_gold -= *character_gold;
        *character_gold = 0.0;
    }
}

// TODO: This function needs to be revisited soon. A lot of updates (even when we are only updating a small part), and it uses unnest poorly. In addition, this route may get hit A LOT.
// TODO: Update this route with new item_instances logic
// TODO: This is no longer working because the ids no longer refer to the library but to the item
pub async fn edit_encounter_session_character_assignments(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    session_id: InternalId,
    updates: &UpdateCharacterSessions,
) -> crate::Result<()> {
    // Pre-select as fast insertion/deletion can cause inconsistency here. We lock on the session to prevent this.
    // TODO: Slow down the calls to this route on the front end to prevent this from happening
    sqlx::query!(
        "SELECT 1 as one FROM campaign_sessions WHERE id = $1 FOR UPDATE",
        session_id.0 as i32
    )
    .fetch_all(&mut **tx)
    .await?;

    // Delete all existing character assignments + character item assignments for the session
    sqlx::query!(
        r#"
            UPDATE item_instances
            SET character_id = NULL
            WHERE session_id = $1
        "#,
        session_id.0 as i32,
    )
    .execute(&mut **tx)
    .await?;

    sqlx::query!(
        r#"
            DELETE FROM campaign_session_characters WHERE session_id = $1
        "#,
        session_id.0 as i32,
    )
    .execute(&mut **tx)
    .await?;

    for (character_id, update) in &updates.compiled_rewards {
        sqlx::query!(
            r#"
            INSERT INTO campaign_session_characters (session_id, character_id, gold_rewards, present)
            VALUES ($1, $2, $3, $4)
            "#,
            session_id.0 as i32,
            character_id.0 as i64,
            update.gold,
            update.present,
        )
        .execute(&mut **tx)
        .await?;

        // TODO: Could be rewritten to use a single query with two-column unnest (For all users in one query)
        sqlx::query!(
            r#"
            UPDATE item_instances
            SET character_id = $1
            FROM UNNEST($3::int[]) as item_id
            WHERE library_item_id = item_id AND session_id = $2
            -- TODO: Reinstate this
            -- WHERE item_instances.id = item_id
            "#,
            character_id.0 as i64,
            session_id.0 as i32,
            &update
                .items
                .iter()
                .map(|e| e.0 as i32)
                .collect::<Vec<i32>>(),
        )
        .execute(&mut **tx)
        .await?;
    }

    // Sets the unassigned gold and item rewards for the session to be
    // the difference between the total gold and item rewards from the encounters
    // and the total gold and item rewards assigned to characters
    sqlx::query!(
        r#"
        UPDATE campaign_sessions
        SET
            unassigned_gold_rewards = COALESCE(teg.total_encounter_gold,0) - COALESCE(tcg.total_characters_gold,0)
        FROM (
            SELECT SUM(e.treasure_currency) as total_encounter_gold
              FROM encounters e
              WHERE session_id = $1
        ) teg,
        (
            SELECT SUM(csc.gold_rewards) as total_characters_gold
            FROM campaign_session_characters csc
            WHERE session_id = $1
        ) tcg
        WHERE campaign_sessions.id = $1
        "#,
        session_id.0 as i32,
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}

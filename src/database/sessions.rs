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
            unassigned_item_rewards,
            JSONB_AGG(
                JSONB_BUILD_OBJECT(
                    'session_id', csc.session_id,
                    'character_id', csc.character_id,
                    'gold_rewards', csc.gold_rewards,
                    'item_rewards', csc.item_rewards
                )
            ) filter (where csc.session_id is not null) as character_rewards,
            SUM(e.total_items_value + e.treasure_currency) as total_combined_treasure_value,
            SUM(e.total_experience) as total_experience
        FROM campaign_sessions s
        LEFT JOIN campaigns ca ON s.campaign_id = ca.id
        LEFT JOIN encounters e ON s.id = e.session_id

        LEFT JOIN LATERAL (
            SELECT 
                csc.session_id,
                csc.character_id,
                csc.gold_rewards,
                ARRAY_AGG(item_id) FILTER (WHERE item_id IS NOT NULL) as item_rewards
            FROM campaign_session_characters csc
            LEFT JOIN campaign_session_character_items csci ON csci.character_id = csc.character_id AND csci.session_id = csc.session_id
            GROUP BY csc.session_id, csc.character_id
        ) csc ON s.id = csc.session_id

        WHERE 
            ca.id = $1
            AND ca.owner = $2
        GROUP BY s.id
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
                gold_rewards: f64,
                item_rewards: Option<Vec<i32>>,
                session_id: i32,
            }

            println!("Row ch: {:?}", &row.character_rewards);

            let character_rewards: Vec<RowCharacterRewards> = row
                .character_rewards
                .and_then(|x| serde_json::from_value(x).unwrap())
                .unwrap_or_default();

            println!("Character rewards: {:?}", &character_rewards);
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
                                    .map(|e| InternalId(*e as u32))
                                    .collect(),
                                gold: row.gold_rewards,
                            },
                        );
                        acc
                    });
            println!("Compiled rewards: {:?}", &compiled_rewards);
            let encounter_ids = row
                .encounter_ids
                .unwrap_or_default()
                .iter()
                .map(|e| InternalId(*e as u32))
                .collect::<Vec<InternalId>>();

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
                    .unassigned_item_rewards
                    .iter()
                    .map(|e| InternalId(*e as u32))
                    .collect(),
                total_experience: row.total_experience.map(|e| e as u64).unwrap_or_default(),
                total_combined_treasure_value: row
                    .total_combined_treasure_value
                    .unwrap_or_default(),
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

    println!(
        "Inserting campaign sessions with data: {:?}, {:?}, {:?}, {:?}, {:?}",
        &session_orders, &names, &descriptions, &play_dates, &campaign_id
    );

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

    Ok(ids)
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

    // Add items and gold of linked encounter to unassigned rewards
    sqlx::query!(
        r#"
            UPDATE campaign_sessions
            SET 
                unassigned_gold_rewards = unassigned_gold_rewards + e.treasure_currency,
                unassigned_item_rewards = unassigned_item_rewards || ARRAY(
                    SELECT item
                    FROM encounter_treasure_items
                    WHERE encounter = $2
                )
            FROM (SELECT * FROM encounters WHERE id = $2) e
            WHERE campaign_sessions.id = $1
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

    // Add  empty characters to the session
    // TODO: Eventually, we may not want to add every character to the session, but for now, this is fine. Some default set, etc.
    sqlx::query!(
        r#"
        INSERT INTO campaign_session_characters (session_id, character_id, gold_rewards)
        SELECT $1, ch.id, 0
        FROM characters ch
        INNER JOIN campaigns cp ON ch.campaign = cp.id
        INNER JOIN campaign_sessions cs ON cp.id = cs.campaign_id
        WHERE cs.id = $1
        ON CONFLICT DO NOTHING
    "#,
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
        SELECT session_id, treasure_currency, ARRAY_AGG(item) filter (where item is not null) as treasure_items
        FROM encounters e
        LEFT JOIN encounter_treasure_items eti ON e.id = eti.encounter
        WHERE id = $1
        GROUP BY e.id
        "#,
        encounter_id.0 as i64,
    )
    .fetch_one(&mut **tx)
    .await?;

    let Some(session_id) = res.session_id else {
        return Ok(None);
    };

    // Update encounter, getting the gold and items it contributed
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
    let mut items: Vec<i32> = res.treasure_items.unwrap_or_default();

    let gold_copy = gold;
    let items_copy = items.clone();

    // Fetch unassigned gold and items
    let res = sqlx::query!(
        r#"
        SELECT unassigned_gold_rewards, unassigned_item_rewards
        FROM campaign_sessions
        WHERE id = $1
        "#,
        session_id as i32,
    )
    .fetch_one(&mut **tx)
    .await?;
    let mut unassigned_gold_rewards: f64 = res.unassigned_gold_rewards;
    let mut unassigned_item_rewards: Vec<i32> = res.unassigned_item_rewards;

    // Remove as much gold and items as the encounter contributed as possible from unassigned rewards
    remove_contributions_from_character(
        &mut gold,
        &mut items,
        &mut unassigned_gold_rewards,
        &mut unassigned_item_rewards,
    );

    sqlx::query!(
        r#"
        UPDATE campaign_sessions
        SET unassigned_gold_rewards = $1, unassigned_item_rewards = $2
        WHERE id = $3
        "#,
        unassigned_gold_rewards,
        &unassigned_item_rewards,
        session_id as i32,
    )
    .execute(&mut **tx)
    .await?;

    // Exit early if all gold and items were removed
    if gold == 0.0 && items.is_empty() {
        return Ok(Some(InternalId(session_id as u32)));
    }

    // Fetch total gold and items per character
    let character_items = sqlx::query!(
        r#"
        SELECT csc.character_id, csc.gold_rewards, ARRAY_AGG(item_id) filter (where item_id is not null) as item_rewards
        FROM campaign_session_characters csc
        LEFT JOIN campaign_session_character_items csci ON csc.character_id = csci.character_id AND csc.session_id = csci.session_id
        WHERE csc.session_id = $1
        GROUP BY csc.character_id, csc.session_id
        "#,
        session_id as i32,
    )
    .fetch_all(&mut **tx)
    .await?
    .into_iter()
    .fold(HashMap::new(), |mut acc, row| {
        acc.insert(
            row.character_id as u64,
            (row.gold_rewards, row.item_rewards.unwrap_or_default()),
        );
        acc
    });
    let character_items_clone = character_items.clone();

    // Remove as much gold and items as the encounter contributed as possible from character rewards
    for (character_id, (mut character_gold, mut character_items)) in character_items {
        remove_contributions_from_character(
            &mut gold,
            &mut items,
            &mut character_gold,
            &mut character_items,
        );
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

        sqlx::query!(
            r#"
            DELETE FROM campaign_session_character_items
            WHERE session_id = $1 AND character_id = $2
            "#,
            session_id as i32,
            character_id as i64,
        )
        .execute(&mut **tx)
        .await?;

        sqlx::query!(
            r#"
            INSERT INTO campaign_session_character_items (session_id, character_id, item_id)
            SELECT $1, $2, item_id
            FROM UNNEST($3::int[]) as item_id
            "#,
            session_id as i32,
            character_id as i64,
            &character_items
        )
        .execute(&mut **tx)
        .await?;
    }

    // Failure to remove all gold and items from rewards is an internal error
    // It should be invariant that sum of gold and items from encounter == sum of gold and items from rewards
    // Should not get into either of these blocks unless there is a bug in the code
    if gold > 0.0 {
        return Err(crate::ServerError::InternalError(format!("Could not unlink successfully: inconsistent number of gold. {} gold in characters/unassigned, {} gold in encounter.", gold_copy - gold, gold_copy)));
    }

    if !items.is_empty() {
        let character_items = character_items_clone
            .values()
            .flat_map(|(_, items)| items)
            .copied()
            .collect::<Vec<i32>>();
        return Err(crate::ServerError::InternalError(format!("Could not unlink successfully: inconsistent number of items. {:?} items in characters/unassigned, {:?} items in encounter.", character_items, items_copy)));
    }

    Ok(Some(InternalId(session_id as u32)))
}

// TODO: This function could be made more efficient- or perhaps moved entirely into Postgres
fn remove_contributions_from_character(
    remove_gold: &mut f64,
    remove_items: &mut Vec<i32>,
    character_gold: &mut f64,
    character_items: &mut Vec<i32>,
) {
    // Remove as much gold as possible
    if remove_gold <= character_gold {
        *character_gold -= *remove_gold;
        *remove_gold = 0.0;
    } else {
        *remove_gold -= *character_gold;
        *character_gold = 0.0;
    }

    // Remove as many items as possible
    // Backwards iteration to allow for removal of items (clone to allow for mutability)
    for (remove_rx, remove_item) in remove_items.clone().iter().enumerate().rev() {
        if let Some(pos) = character_items.iter().position(|e| e == remove_item) {
            character_items.remove(pos);
            remove_items.remove(remove_rx);
        }
    }
}

// TODO: This function needs to be revisited soon. A lot of updates (even when we are only updating a small part), and it uses unnest poorly. In addition, this route may get hit A LOT.
pub async fn edit_encounter_session_character_assignments(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    session_id: InternalId,
    updates: &UpdateCharacterSessions,
) -> crate::Result<()> {
    // Pre-select as fast insertion/deletion can cause inconsistency here. We lock on the session to prevent this.
    // TODO: Slow down the calls to this route on the front end to prevent this from happening
    log::info!(
        "\n\nMaking the following updates {}: {:?}",
        session_id.0,
        updates.compiled_rewards
    );
    sqlx::query!(
        "SELECT 1 as one FROM campaign_sessions WHERE id = $1 FOR UPDATE",
        session_id.0 as i32
    )
    .fetch_all(&mut **tx)
    .await?;

    // Delete all existing character assignments + character item assignments for the session
    sqlx::query!(
        r#"
            DELETE FROM campaign_session_character_items WHERE session_id = $1
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
            INSERT INTO campaign_session_characters (session_id, character_id, gold_rewards)
            VALUES ($1, $2, $3)
            "#,
            session_id.0 as i32,
            character_id.0 as i64,
            update.gold,
        )
        .execute(&mut **tx)
        .await?;

        sqlx::query!(
            r#"
            INSERT INTO campaign_session_character_items (session_id, character_id, item_id)
            SELECT $1, $2, item_id
            FROM UNNEST($3::int[]) as item_id
            "#,
            session_id.0 as i32,
            character_id.0 as i64,
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

    println!("Session ID: {}", session_id.0 as i32);
    sqlx::query!(
        r#"
        UPDATE campaign_sessions
        SET
            unassigned_gold_rewards = COALESCE(teg.total_encounter_gold,0) - COALESCE(tcg.total_characters_gold,0),
            unassigned_item_rewards = COALESCE(items_agg.items, '{}')
        FROM (
            SELECT SUM(e.treasure_currency) as total_encounter_gold
              FROM encounters e
              WHERE session_id = $1
        ) teg,
        (
            SELECT SUM(csc.gold_rewards) as total_characters_gold
            FROM campaign_session_characters csc
            WHERE session_id = $1
        ) tcg,
       (
            SELECT ARRAY_AGG(item_reward) as items 
            FROM (
                SELECT eti.item as item_reward
                FROM encounter_treasure_items eti
                INNER JOIN encounters e ON eti.encounter = e.id
                WHERE session_id = $1

                EXCEPT
                    
                SELECT item_id as item_reward
                FROM campaign_session_character_items
                WHERE session_id = $1
            ) all_remaining_items
        ) items_agg
        WHERE campaign_sessions.id = $1
        "#,
        session_id.0 as i32,
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}

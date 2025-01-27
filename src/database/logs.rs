use crate::models::events::EventGroup;
use crate::models::ids::InternalId;
use crate::models::log::Log;
use serde::{Deserialize, Serialize};

use super::events::{self, InsertEvent};

#[derive(serde::Deserialize, Debug)]
pub struct InsertLog {
    pub name: String,
    pub timestamp: Option<chrono::DateTime<chrono::Utc>>,
    pub session_id: Option<InternalId>,
    pub description: Option<String>,
    pub events: Vec<InsertEvent>,
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct LogFilters {
    pub character: Option<i32>,
}

pub async fn get_logs(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres> + Copy,
    owner: InternalId,
    campaign_id: InternalId,
    condition: &LogFilters,
) -> crate::Result<Vec<Log>> {
    let query = sqlx::query!(
        r#"
        SELECT 
            eg.id,
            eg.name,
            eg.timestamp,
            eg.description,
            array_agg(DISTINCT ev.id) filter (where ev.id is not null) AS events
        FROM event_groups eg
        LEFT JOIN events ev ON eg.id = ev.event_group
        WHERE eg.campaign = $1 AND ($2::int IS NULL OR ev.character = $2)
        AND EXISTS (
            SELECT 1 FROM campaigns ca
            WHERE ca.id = eg.campaign AND ca.owner = $3
        )
        GROUP BY eg.id
        "#,
        campaign_id.0 as i32,
        condition.character,
        owner.0 as i32,
    );

    let logs = query.fetch_all(exec).await?;

    let all_event_ids = logs
        .iter()
        .flat_map(|log| log.events.clone())
        .flatten()
        .map(|id| InternalId(id as u64))
        .collect::<Vec<_>>();
    let all_events = events::get_events_ids(exec, &all_event_ids).await?;

    let logs = logs
        .into_iter()
        .map(|row| {
            let event_group = EventGroup {
                id: InternalId(row.id as u64),
                name: row.name,
                timestamp: row.timestamp.and_utc(),
                description: row.description,
                events: row
                    .events
                    .unwrap_or_default()
                    .iter()
                    .map(|id| InternalId(*id as u64))
                    .collect(),
            };
            Log::from_log_events(event_group, &all_events)
        })
        .collect();

    Ok(logs)
}

pub async fn get_owned_logs_ids(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres> + Copy,
    log_ids: &[InternalId],
    owner: InternalId,
) -> crate::Result<Vec<InternalId>> {
    let query = sqlx::query!(
        r#"
        SELECT 
            eg.id AS "id!"
        FROM event_groups eg
        WHERE 
            eg.id = ANY($1::int[])
            AND EXISTS (
                SELECT 1 FROM campaigns ca
                WHERE ca.id = eg.campaign AND ca.owner = $2
            )
        "#,
        &log_ids.iter().map(|id| id.0 as i32).collect::<Vec<i32>>(),
        owner.0 as i32,
    )
    .fetch_all(exec)
    .await?
    .iter()
    .map(|row| InternalId(row.id as u64))
    .collect();

    Ok(query)
}

/// Campaign ownership needs to already be checked
pub async fn insert_log(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres> + Copy,
    campaign_id: InternalId,
    log: &InsertLog,
) -> crate::Result<InternalId> {
    let log_id = sqlx::query!(
        r#"
        INSERT INTO event_groups (name, timestamp, campaign, description)
        SELECT $1::text, $2::timestamptz, $3::int, $4::text
        RETURNING id
        "#,
        &log.name,
        log.timestamp.unwrap_or_else(|| chrono::Utc::now()),
        campaign_id.0 as i32,
        log.description.as_deref(),
    )
    .fetch_one(exec)
    .await?
    .id;

    // Add events associated with the log
    events::insert_events(
        exec,
        campaign_id,
        Some(InternalId(log_id as u64)),
        &log.events,
    )
    .await?;

    Ok(InternalId(log_id as u64))
}

// InsertLog is used, but list of events is ignored
pub async fn edit_log(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres> + Copy,
    owner: InternalId,
    log_id: InternalId,
    new_log: &InsertLog,
) -> crate::Result<()> {
    sqlx::query!(
        r#"
        UPDATE event_groups
        SET name = $1::text, timestamp = $2::timestamptz, description = $3::text
        WHERE id = $4
        AND EXISTS (
            SELECT 1 FROM campaigns ca
            WHERE ca.id = event_groups.campaign AND ca.owner = $5
        )
        "#,
        &new_log.name,
        new_log.timestamp,
        new_log.description.as_deref(),
        log_id.0 as i32,
        owner.0 as i32,
    )
    .execute(exec)
    .await?;

    Ok(())
}

pub async fn delete_log(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres> + Copy,
    owner: InternalId,
    log_id: InternalId,
) -> crate::Result<()> {
    // TODO: Should be a transaction, with checks,etc

    // Get all events associated with the log
    let event_ids = sqlx::query!(
        r#"
        SELECT id
        FROM events
        WHERE event_group = $1
        "#,
        log_id.0 as i32,
    )
    .fetch_all(exec)
    .await?
    .iter()
    .map(|row| InternalId(row.id as u64))
    .collect::<Vec<_>>();

    // Delete all events associated with the log
    events::delete_events(exec, owner, &event_ids).await?;

    sqlx::query!(
        r#"
        DELETE FROM event_groups
        -- TODO: owner check through 'campaign'
        WHERE id = $1
        AND EXISTS (
            SELECT 1 FROM campaigns ca
            WHERE ca.id = event_groups.campaign AND ca.owner = $2
        )
        "#,
        log_id.0 as i32,
        owner.0 as i32,
    )
    .execute(exec)
    .await?;

    Ok(())
}

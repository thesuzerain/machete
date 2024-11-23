use crate::models::events::{Event, EventType};
use crate::models::ids::InternalId;
use itertools::Itertools;

use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct EventFilters {
    pub character: Option<i32>,
    pub log: Option<i32>,
    pub event_type: Option<String>,
}

#[derive(serde::Deserialize, Debug)]
pub struct InsertEvent {
    pub character: Option<InternalId>,
    #[serde(flatten)]
    pub event_type: EventType,
}

#[derive(serde::Deserialize, Debug)]
pub struct EditEvent {
    pub character: Option<InternalId>,
    #[serde(flatten)]
    pub event_type: Option<EventType>,
}

// TODO: May be prudent to make a separate models system for the database.
pub async fn get_events(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    owner: InternalId,
    campaign_id: InternalId,
    // TODO: Could this use be problematic?
    // A postgres alternative can be found here:
    // https://github.com/launchbadge/sqlx/issues/291
    condition: &EventFilters,
) -> crate::Result<Vec<Event>> {
    // TODO: Campaign needs to be checked for ownership

    let query = sqlx::query!(
        r#"
        SELECT 
            ev.id,
            ch.id AS "character?",
            ev.timestamp,
            ev.event_group AS "log?",
            ev.event_data
        FROM events ev
        LEFT JOIN characters ch ON ev.character = ch.id
        LEFT JOIN campaigns ca ON ev.campaign = ca.id
        LEFT JOIN event_groups eg ON ev.event_group = eg.id
        WHERE 
            ($1::int IS NULL OR ev.character = $1)
            AND ca.id = $2
            AND ($3::text IS NULL OR ev.event_data->>'type' = $3)
        ORDER BY ev.timestamp
    "#,
        condition.character,
        campaign_id.0 as i32,
        condition.event_type,
    );

    let events = query
        .fetch_all(exec)
        .await?
        .into_iter()
        .map(|row| {
            Ok(Event {
                id: InternalId(row.id as u64),
                log: row.log.map(|l| InternalId(l as u64)),
                character: row.character.map(|c| InternalId(c as u64)),
                timestamp: row.timestamp.and_utc(),
                event_type: serde_json::from_value(row.event_data)?,
            })
        })
        .collect::<Result<Vec<Event>, crate::ServerError>>()?;
    Ok(events)
}

pub async fn get_events_ids(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    event_ids: Vec<InternalId>,
) -> crate::Result<Vec<Event>> {
    // TODO: Campaign needs to be checked for ownership

    let query = sqlx::query!(
        r#"
        SELECT 
            ev.id AS "id!",
            ch.id AS "character?",
            ev.timestamp AS "timestamp!",
            ev.event_group AS "log?",
            ev.event_data AS "event_data!"
        FROM events ev
        LEFT JOIN characters ch ON ev.character = ch.id
        LEFT JOIN campaigns ca ON ev.campaign = ca.id
        LEFT JOIN event_groups eg ON ev.event_group = eg.id
        WHERE 
            ev.id = ANY($1::int[])
        ORDER BY ev.timestamp
    "#,
        &event_ids.iter().map(|id| id.0 as i32).collect::<Vec<i32>>(),
    );

    let events = query
        .fetch_all(exec)
        .await?
        .into_iter()
        .map(|row| {
            Ok(Event {
                id: InternalId(row.id as u64),
                log: row.log.map(|l| InternalId(l as u64)),
                character: row.character.map(|c| InternalId(c as u64)),
                timestamp: row.timestamp.and_utc(),
                event_type: serde_json::from_value(row.event_data)?,
            })
        })
        .collect::<Result<Vec<Event>, crate::ServerError>>()?;
    Ok(events)
}

pub async fn insert_events(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres> + Copy,
    owner: InternalId,
    campaign_id: InternalId,
    log_id: Option<InternalId>,
    events: &Vec<InsertEvent>,
) -> crate::Result<()> {
    if events.is_empty() {
        return Ok(());
    }

    // TODO: Campaign needs to be checked for ownership
    let (characters, campaigns, event_types, event_groups): (
        Vec<Option<i32>>,
        Vec<i32>,
        Vec<serde_json::Value>,
        Vec<Option<i32>>,
    ) = events
        .iter()
        .filter_map(|e| {
            Some((
                e.character.map(|c| c.0 as i32),
                campaign_id.0 as i32,
                serde_json::to_value(&e.event_type).ok()?,
                log_id.map(|l| l.0 as i32),
            ))
        })
        .multiunzip();

    sqlx::query!(
        r#"
        INSERT INTO events (character, campaign, event_data, event_group)
        SELECT * FROM UNNEST ($1::int[], $2::int[], $3::jsonb[], $4::int[])
        "#,
        characters as _,
        &campaigns,
        &event_types,
        event_groups as _,
    )
    .execute(exec)
    .await?;

    Ok(())
}

pub async fn edit_event(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres> + Copy,
    owner: InternalId,
    event_id: InternalId,
    new_event: &EditEvent,
) -> crate::Result<()> {
    let event_type = new_event
        .event_type
        .clone()
        .map(|et| serde_json::to_value(&et))
        .transpose()?;

    sqlx::query!(
        r#"
        UPDATE events
        SET event_data = COALESCE($1, event_data),
            character = COALESCE($2, character)
        WHERE id = $3
        AND EXISTS (
            SELECT 1 FROM campaigns ca
            WHERE ca.id = events.campaign AND ca.owner = $4
        )
        "#,
        event_type,
        // TODO: Ensure no bad conversions with these
        new_event.character.map(|c| c.0 as i64),
        event_id.0 as i32,
        owner.0 as i32,
    )
    .execute(exec)
    .await?;

    Ok(())
}

pub async fn delete_events(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres> + Copy,
    owner: InternalId,
    event_id: &Vec<InternalId>,
) -> crate::Result<()> {
    if event_id.is_empty() {
        return Ok(());
    }

    sqlx::query!(
        r#"
        DELETE FROM events
        -- TODO: owner check through 'campaign'
        WHERE id = ANY($1::int[])
        AND EXISTS (
            SELECT 1 FROM campaigns ca
            WHERE ca.id = events.campaign AND ca.owner = $2
        )
        "#,
        &event_id.iter().map(|id| id.0 as i32).collect::<Vec<i32>>(),
        owner.0 as i32,
    )
    .execute(exec)
    .await?;

    Ok(())
}

use machete::models::events::{Event, EventType};
use machete_core::ids::InternalId;
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct EventFilters {
    pub character: Option<i32>,
    pub event_type: Option<String>,
}

#[derive(serde::Deserialize, Debug)]
pub struct InsertEvent {
    pub character: Option<InternalId>,
    #[serde(flatten)]
    pub event_type: EventType,
}

// TODO: May be prudent to make a separate models system for the database.
pub async fn get_campaigns(
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
            ev.event_data
        FROM events ev
        LEFT JOIN characters ch ON ev.character = ch.id
        LEFT JOIN campaigns ca ON ev.campaign = ca.id
        WHERE 
            ($1::int IS NULL OR ev.character = $1)
            AND ca.id = $2
            AND ($3::text IS NULL OR ev.event_data->>'type' = $3)
    "#,
        condition.character,
        campaign_id.0 as i32,
        condition.event_type,
    );

    let campaigns = query
        .fetch_all(exec)
        .await?
        .into_iter()
        .map(|row| {
            Ok(Event {
                id: InternalId(row.id as u64),
                character: row.character.map(|c| InternalId(c as u64)),
                timestamp: row.timestamp.unwrap_or_default().and_utc(),
                event_type: serde_json::from_value(row.event_data)?,
            })
        })
        .collect::<Result<Vec<Event>, crate::ServerError>>()?;
    Ok(campaigns)
}

pub async fn insert_events(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres> + Copy,
    owner: InternalId,
    campaign_id: InternalId,
    events: &Vec<InsertEvent>,
) -> crate::Result<()> {
    if events.is_empty() {
        return Ok(());
    }

    // TODO: Campaign needs to be checked for ownership
    let (characters, event_types): (Vec<Option<i32>>, Vec<serde_json::Value>) = events
        .iter()
        .filter_map(|e| {
            Some((
                e.character.map(|c| c.0 as i32),
                serde_json::to_value(&e.event_type).ok()?,
            ))
        })
        .unzip();

    sqlx::query!(
        r#"
        INSERT INTO events (character, campaign, event_data)
        SELECT * FROM UNNEST ($1::int[], array[$2::int], $3::jsonb[])
        "#,
        characters as _,
        campaign_id.0 as i32,
        &event_types,
    )
    .execute(exec)
    .await?;

    Ok(())
}

pub async fn edit_event(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres> + Copy,
    owner: InternalId,
    event_id: InternalId,
    new_event: &EventType,
) -> crate::Result<()> {
    let event_type = serde_json::to_value(&new_event).unwrap();

    sqlx::query!(
        r#"
        UPDATE events
        SET event_data = $1::jsonb
        WHERE id = $2
        AND EXISTS (
            SELECT 1 FROM campaigns ca
            WHERE ca.id = events.campaign AND ca.owner = $3
        )
        "#,
        &event_type,
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
        -- owner check through 'campaign'
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
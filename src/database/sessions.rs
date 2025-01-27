use std::{collections::HashMap, iter};

use crate::models::campaign::CampaignSession;
use crate::models::ids::InternalId;
use chrono::{DateTime, Utc};
use itertools::Itertools;

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

// TODO: May be prudent to make a separate models system for the database.
pub async fn get_sessions(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
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
            ARRAY_AGG(e.id) filter (where e.id is not null) as encounter_ids
        FROM campaign_sessions s
        LEFT JOIN campaigns ca ON s.campaign_id = ca.id
        LEFT JOIN encounters e ON s.id = e.session_id
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
            Ok(CampaignSession {
                id: InternalId(row.id as u64),
                name: row.name,
                description: row.description,
                session_order: row.session_order as u32,
                play_date: row.play_date.into(),
                encounter_ids: row
                    .encounter_ids
                    .unwrap_or_default()
                    .iter()
                    .map(|e| InternalId(*e as u64))
                    .collect(),
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
            session.play_date,
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
            let date_or_now = e.play_date.unwrap_or_else(|| Utc::now());
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

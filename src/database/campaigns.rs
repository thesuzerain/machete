use crate::models::ids::InternalId;
use crate::models::{campaign::CampaignPartial, encounter::CompletionStatus};
use crate::ServerError;

use super::encounters::{self, InsertEncounter};

#[derive(serde::Deserialize, Debug)]
pub struct InsertCampaign {
    pub name: String,
    pub description: Option<String>,
    pub initialization: Option<InsertCampaignInitialization>,
}

#[derive(serde::Deserialize, Debug)]
pub struct InsertCampaignInitialization {
    pub experience: Option<u64>,
    pub gold: f32,
    pub items: Vec<InternalId>,
    pub characters: Vec<InsertCampaignInitializationCharacter>,
}

#[derive(serde::Deserialize, Debug)]
pub struct InsertCampaignInitializationCharacter {
    pub name: String,
    pub class: InternalId,
    pub gold: u64,
    pub items: Vec<InternalId>,
}

// TODO: May be prudent to make a separate models system for the database.
pub async fn get_campaigns_owner(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    owner: InternalId,
) -> crate::Result<Vec<CampaignPartial>> {
    let query = sqlx::query!(
        r#"
        SELECT 
            ca.id,
            ca.name,
            description,
            total_experience,
            level
        FROM campaigns ca
        WHERE 
            ca.owner = $1
    "#,
        owner.0 as i32,
    );

    let campaigns = query
        .fetch_all(exec)
        .await?
        .into_iter()
        .map(|row| {
            Ok(CampaignPartial {
                id: InternalId(row.id as u32),
                name: row.name,
                description: row.description,
                total_experience: row.total_experience as u64,
                level: row.level as u8,
            })
        })
        .collect::<Result<Vec<CampaignPartial>, sqlx::Error>>()?;
    Ok(campaigns)
}

pub async fn get_owned_campaign_id(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    campaign_id: InternalId,
    owner: InternalId,
) -> crate::Result<Option<InternalId>> {
    let query = sqlx::query!(
        r#"
        SELECT 
            ca.id AS "id!"
        FROM campaigns ca
        WHERE 
            ca.id = $1
            AND ca.owner = $2
    "#,
        campaign_id.0 as i32,
        owner.0 as i32,
    )
    .fetch_optional(exec)
    .await?
    .map(|row| InternalId(row.id as u32));

    Ok(query)
}

pub async fn insert_campaign(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    insert: &InsertCampaign,
    include_initial_sessions: bool,
    owner: InternalId,
) -> crate::Result<InternalId> {
    log::info!("insert: {:?}", insert);
    let total_experience = insert
        .initialization
        .as_ref()
        .map(|init| init.experience)
        .unwrap_or(Some(0))
        .unwrap_or(0);
    let id = sqlx::query!(
        r#"
        INSERT INTO campaigns (name, owner, description, total_experience)
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#,
        &insert.name,
        owner.0 as i32,
        insert.description.as_ref(),
        total_experience as i64,
    )
    .fetch_all(&mut **tx)
    .await?
    .into_iter()
    .next()
    .ok_or(ServerError::InternalError(
        "Failed to insert campaign".to_string(),
    ))?
    .id;

    // New campaign- also, create a single session zero for it.
    if include_initial_sessions {
        let cs = sqlx::query!(
            r#"
            INSERT INTO campaign_sessions (session_order, name, play_date, campaign_id)
            VALUES (10000, 'Untitled session', now(), $1)
            RETURNING id
            "#,
            id as i32,
        )
        .fetch_all(&mut **tx)
        .await?;

        // To the first session, we are going to add an encounter with the initialization data.
        // TODO: When we have multiple kinds of encounters, it makes sense for this to be a unique one or a rewards only one
        if let Some(initailization) = &insert.initialization {
            let session_id = cs[0].id;
            let experience = initailization.experience.map(|i| i as i32).unwrap_or(0);
            let gold = initailization.gold;
            let treasure_items = initailization.items.clone();
            let insert_encounter = InsertEncounter {
                name: "Initialization encounter".to_string(),
                session_id: Some(InternalId(session_id as u32)),
                description: Some("An initialization encounter, including all the characters and items at this point in the campaign.".to_string()),
                extra_experience: experience,
                treasure_currency: gold,
                treasure_items,

                // TODO: Come back to this
                initialization_encounter: Some(true),
                enemies: Vec::new(),
                hazards: Vec::new(),
                party_level: 1,
                party_size: 1,
                status: CompletionStatus::Archived,

                // We can safely leave these None here, as they are calculated from the characters and items.
                total_experience: None,
                total_items_value: None,
            };
            encounters::insert_encounters(&mut *tx, owner, &vec![insert_encounter]).await?;
        }
    }
    Ok(InternalId(id as u32))
}

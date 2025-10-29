use crate::models::campaign::CampaignPartial;
use crate::models::encounter::EncounterType;
use crate::models::ids::InternalId;
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

#[derive(serde::Deserialize, Debug)]
pub struct ModifyCampaign {
    pub name: Option<String>,
    pub description: Option<String>,
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
                encounter_type: EncounterType::RewardInitialization,
                party_level: 1,
                party_size: 1,
            };
            encounters::insert_encounters(&mut *tx, owner, &vec![insert_encounter]).await?;
        }
    }
    Ok(InternalId(id as u32))
}

pub async fn delete_campaign(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    campaign_id: InternalId,
) -> crate::Result<()> {
    // First, unlink all encounters
    sqlx::query!(
        r#"
        UPDATE encounters
        SET session_id = NULL
        WHERE session_id IN (
            SELECT id
            FROM campaign_sessions
            WHERE campaign_id = $1
        )
        "#,
        campaign_id.0 as i32,
    )
    .execute(&mut **tx)
    .await?;

    // Then, delete all campaign_sessions, campaign_session_characters, campaign_session_character_items
    sqlx::query!(
        r#"
        UPDATE campaign_items
        SET session_id = NULL
        WHERE session_id IN (
            SELECT id
            FROM campaign_sessions
            WHERE campaign_id = $1
        )
        "#,
        campaign_id.0 as i32,
    )
    .execute(&mut **tx)
    .await?;

    sqlx::query!(
        r#"
        DELETE FROM campaign_session_characters
        WHERE session_id IN (
            SELECT id
            FROM campaign_sessions
            WHERE campaign_id = $1
        )
        "#,
        campaign_id.0 as i32,
    )
    .execute(&mut **tx)
    .await?;

    sqlx::query!(
        r#"
        DELETE FROM campaign_sessions
        WHERE campaign_id = $1
        "#,
        campaign_id.0 as i32,
    )
    .execute(&mut **tx)
    .await?;

    // Delete characters
    sqlx::query!(
        r#"
        DELETE FROM characters
        WHERE id IN (
            SELECT id
            FROM characters
            WHERE campaign = $1
        )
        "#,
        campaign_id.0 as i32,
    )
    .execute(&mut **tx)
    .await?;

    // Finally, delete the campaign
    sqlx::query!(
        r#"
        DELETE FROM campaigns
        WHERE id = $1
        "#,
        campaign_id.0 as i32,
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}

pub async fn edit_campaign(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    campaign_id: InternalId,
    modify: &ModifyCampaign,
) -> crate::Result<()> {
    let query = sqlx::query!(
        r#"
        UPDATE campaigns
        SET name = COALESCE($1, name),
            description = COALESCE($2, description)
        WHERE id = $3
        "#,
        modify.name.as_ref(),
        modify.description.as_ref(),
        campaign_id.0 as i32,
    );

    query.execute(&mut **tx).await?;

    Ok(())
}

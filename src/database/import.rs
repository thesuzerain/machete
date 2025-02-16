use crate::{
    database::{
        characters::{CharacterFilters, InsertCharacter},
        encounters::{EncounterFilters, InsertEncounter, InsertEncounterEnemy},
        sessions::InsertSession,
    },
    models::{
        campaign::CampaignSessionCharacterRewards, encounter::CompletionStatus, ids::InternalId,
    },
    ServerError,
};
use chrono::{DateTime, Utc};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;

use super::{campaigns::InsertCampaign, sessions::UpdateCharacterSessions};

/*

{
    "name": "Campaign 1",
    "characters": [
        {
            "name": "Alden",
            "player": "John",
            "level": 1,
            "class": 1
        } ...
    ],
    "sessions": [
        {
            "name": "Session 1",
            "description": "The party is ambushed by goblins",
            "date": "2021-01-01",
            "compiled_rewards": {
                "Alden": {
                    "gold": 0.5,
                    "items": [1, 2]
                }
            },

        }
    ],
    "encounters": [
        {
            "name": "Goblin Ambush",
            "description": null,
            "session_ix": 0,

            "party_level": 1,
            "party_size": 4,

            "enemies": [
                {
                    "id": 1,
                    "level_adjustment": 0
                }
            ],
            "hazards": [1, 2],
            "treasure_items": [1, 2],
            "treasure_currency": 0.5,
            "extra_experience": 50,
        }
    ]
}



*/

#[derive(Serialize, Debug, Deserialize)]
pub struct ImportCampaign {
    pub name: String,
    pub description: Option<String>,
    pub characters: Vec<ImportCharacter>,
    pub sessions: Vec<ImportSession>,
    pub encounters: Vec<ImportEncounter>,
}

#[derive(Serialize, Debug, Deserialize)]
pub struct ImportCharacter {
    pub name: String,
    pub player: Option<String>,
    pub level: u32,
    pub class: InternalId,
}

#[derive(Serialize, Debug, Deserialize)]
pub struct ImportSession {
    pub name: Option<String>,
    pub description: Option<String>,
    pub date: Option<DateTime<Utc>>,
    pub compiled_rewards: HashMap<String, ImportSessionCharacterRewards>,
}

#[derive(Serialize, Debug, Deserialize)]
pub struct ImportSessionCharacterRewards {
    pub gold: f32,
    pub items: Vec<InternalId>,
}

#[derive(Serialize, Debug, Deserialize)]
pub struct ImportEncounter {
    pub name: String,
    pub description: Option<String>,
    pub session_ix: usize,

    pub party_level: u32,
    pub party_size: u32,

    pub enemies: Vec<ImportEncounterEnemy>,
    pub hazards: Vec<InternalId>,

    pub treasure_items: Vec<InternalId>,
    pub treasure_currency: f32,
    pub extra_experience: i32,
}

#[derive(Serialize, Debug, Deserialize)]
pub struct ImportEncounterEnemy {
    pub id: InternalId,
    pub level_adjustment: i16,
}

// TODO: Probably can make this function more efficient- unnests, etc
pub async fn import(
    campaign: ImportCampaign,
    pool: &PgPool,
    owner: u32,
) -> Result<(), ServerError> {
    let mut conn = pool.begin().await?;

    // Throw error if two characters have the same name
    let mut character_names = Vec::new();
    for character in &campaign.characters {
        if character_names.contains(&character.name) {
            return Err(ServerError::BadRequest(
                "Duplicate character names".to_string(),
            ));
        }
        character_names.push(character.name.clone());
    }

    // Insert campaign
    let campaign_id = sqlx::query!(
        r#"
        INSERT INTO campaigns (name, description, owner)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
        campaign.name,
        campaign.description,
        owner as i32
    )
    .fetch_one(&mut *conn)
    .await?
    .id;

    let mut character_name_to_id = HashMap::new();
    for character in &campaign.characters {
        let character = sqlx::query!(
            r#"
            INSERT INTO characters (campaign, name, player, level, class)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, name
            "#,
            campaign_id,
            character.name,
            character.player,
            character.level as i32,
            character.class.0 as i32
        )
        .fetch_one(&mut *conn)
        .await?;

        character_name_to_id.insert(character.name, character.id);
    }

    let mut session_ids_in_order = Vec::new();
    for (i, session) in campaign.sessions.iter().enumerate() {
        let session_id = sqlx::query!(
            r#"
            INSERT INTO campaign_sessions (campaign_id, name, description, play_date, session_order)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id
            "#,
            campaign_id,
            session.name,
            session.description,
            session.date,
            (i * 1000) as i32
        )
        .fetch_one(&mut *conn)
        .await?
        .id;
        session_ids_in_order.push(session_id);

        for (character_name, rewards) in &session.compiled_rewards {
            let character_id =
                character_name_to_id
                    .get(character_name)
                    .ok_or(ServerError::BadRequest(format!(
                        "Character {} not found in 'characters'",
                        character_name
                    )))?;

            sqlx::query!(
                r#"
                INSERT INTO campaign_session_characters (session_id, character_id, gold_rewards, item_rewards)
                VALUES ($1, $2, $3, $4)
                "#,
                session_id,
                character_id,
                rewards.gold as f64,
                &rewards.items.iter().map(|id| id.0 as i32).collect::<Vec<i32>>()
            )
            .execute(&mut *conn)
            .await?;
        }
    }

    for encounter in &campaign.encounters {
        let session_id =
            session_ids_in_order
                .get(encounter.session_ix)
                .ok_or(ServerError::BadRequest(format!(
                    "Session index {} not found in 'sessions'",
                    encounter.session_ix
                )))?;
        let encounter_id = sqlx::query!(
            r#"
            INSERT INTO encounters (status, name, description, session_id, enemies, hazards, party_size, party_level, extra_experience, total_experience, total_treasure_value, owner)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING id
            "#,
            CompletionStatus::Prepared.as_i32() as i16,
            encounter.name,
            encounter.description,
            session_id,
            &encounter.enemies.iter().map(|enemy| enemy.id.0 as i32).collect::<Vec<i32>>(),
            &encounter.hazards.iter().map(|id| id.0 as i32).collect::<Vec<i32>>(),
            encounter.party_size as i32,
            encounter.party_level as i32,
            encounter.extra_experience,
            0, // TODO: Calculate total experience
            0.0, // TODO: Calculate total treasure value
            owner as i32

        )
        .fetch_one(&mut *conn)
        .await?
        .id;
    }

    conn.commit().await?;

    Ok(())
}

pub async fn import_with_functions(
    campaign: ImportCampaign,
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    owner: InternalId,
) -> Result<InternalId, ServerError> {
    // Throw error if two characters have the same name
    let mut character_names = Vec::new();
    for character in &campaign.characters {
        if character_names.contains(&character.name) {
            return Err(ServerError::BadRequest(
                "Duplicate character names".to_string(),
            ));
        }
        character_names.push(character.name.clone());
    }

    // Insert campaign
    let campaign_id = super::campaigns::insert_campaign(
        &mut *tx,
        &InsertCampaign {
            name: campaign.name.clone(),
            description: campaign.description.clone(),
        },
        false,
        owner,
    )
    .await?;

    // Insert characters
    let ids = super::characters::insert_characters(
        &mut *tx,
        campaign_id,
        &campaign
            .characters
            .iter()
            .map(|c| InsertCharacter {
                name: c.name.clone(),
                player: c.player.clone(),
                level: c.level as u8,
                class: c.class,
            })
            .collect_vec(),
    )
    .await?;
    let character_name_to_id = campaign
        .characters
        .iter()
        .map(|c| c.name.clone())
        .zip(ids)
        .collect::<HashMap<String, InternalId>>();

    // Insert sessions
    let insert_sessions = campaign
        .sessions
        .iter()
        .enumerate()
        .map(|(ix, s)| InsertSession {
            name: s.name.clone(),
            description: s.description.clone(),
            session_order: (ix * 1000) as u32,
            play_date: s.date,
        })
        .collect_vec();
    let session_ids_in_order =
        super::sessions::insert_sessions(&mut *tx, campaign_id, &insert_sessions).await?;

    // Insert encounters
    let insert_encounters = campaign
        .encounters
        .iter()
        .map(|e| InsertEncounter {
            name: e.name.clone(),
            description: e.description.clone(),
            status: CompletionStatus::Prepared,

            session_id: session_ids_in_order.get(e.session_ix).cloned(),
            party_level: e.party_level as u8,
            party_size: e.party_size as u8,
            enemies: e
                .enemies
                .iter()
                .map(|enemy| InsertEncounterEnemy::IdAndLevelAdjustment {
                    id: enemy.id,
                    level_adjustment: enemy.level_adjustment,
                })
                .collect(),
            hazards: e.hazards.clone(),
            treasure_items: e.treasure_items.clone(),
            treasure_currency: e.treasure_currency,
            extra_experience: e.extra_experience,

            total_experience: None,     // TODO: Calculate total experience
            total_treasure_value: None, // TODO: Calculate total treasure value
        })
        .collect_vec();
    super::encounters::insert_encounters(&mut *tx, owner, &insert_encounters).await?;

    // Assign character contributions
    for (session, session_id) in campaign.sessions.iter().zip(session_ids_in_order) {
        let compiled_rewards = session
            .compiled_rewards
            .iter()
            .map(|(character_name, rewards)| {
                let character_id =
                    character_name_to_id
                        .get(character_name)
                        .ok_or(ServerError::BadRequest(format!(
                            "Character {} not found in 'characters'",
                            character_name
                        )))?;
                Ok((
                    *character_id,
                    CampaignSessionCharacterRewards {
                        gold: rewards.gold as f64,
                        items: rewards.items.clone(),
                    },
                ))
            })
            .collect::<Result<_, ServerError>>()?;
        let updates = UpdateCharacterSessions { compiled_rewards };
        super::sessions::edit_encounter_session_character_assignments(
            &mut *tx, session_id, &updates,
        )
        .await?;
    }

    Ok(campaign_id)
}

pub async fn export(
    campaign_id: InternalId,
    pool: &PgPool,
    owner: InternalId,
) -> Result<ImportCampaign, ServerError> {
    let campaigns = super::campaigns::get_campaigns_owner(pool, owner).await?;
    let campaign = campaigns
        .iter()
        .find(|c| c.id == campaign_id)
        .ok_or(ServerError::BadRequest("Campaign not found".to_string()))?;

    let characters =
        super::characters::get_characters(pool, owner, campaign_id, &CharacterFilters::default())
            .await?;
    let sessions = super::sessions::get_sessions(pool, owner, campaign_id).await?;
    let encounters =
        super::encounters::get_encounters(pool, owner, &EncounterFilters::default()).await?;

    let encounters = encounters
        .into_iter()
        .filter_map(|e| {
            // Only include encounters that are part of a session
            let session_ix = sessions.iter().position(|s| Some(s.id) == e.session_id)?;
            Some(ImportEncounter {
                name: e.name,
                description: e.description,
                session_ix,
                party_level: e.party_level,
                party_size: e.party_size,
                enemies: e
                    .enemies
                    .iter()
                    .map(|enemy| ImportEncounterEnemy {
                        id: enemy.id,
                        level_adjustment: enemy.level_adjustment,
                    })
                    .collect(),
                hazards: e.hazards,
                treasure_items: e.treasure_items,
                treasure_currency: e.treasure_currency,
                extra_experience: e.extra_experience,
            })
        })
        .collect_vec();

    let sessions = sessions
        .into_iter()
        .map(|s| ImportSession {
            name: Some(s.name),
            description: s.description,
            date: Some(s.play_date),
            compiled_rewards: s
                .compiled_rewards
                .into_iter()
                .map(|(character_id, rewards)| {
                    let character = characters.iter().find(|c| c.id == character_id).unwrap();
                    (
                        character.name.clone(),
                        ImportSessionCharacterRewards {
                            gold: rewards.gold as f32,
                            items: rewards.items.to_vec(),
                        },
                    )
                })
                .collect(),
        })
        .collect_vec();

    let characters = characters
        .into_iter()
        .map(|c| ImportCharacter {
            name: c.name,
            player: c.player,
            level: c.level as u32,
            class: c.class,
        })
        .collect();

    let campaign = ImportCampaign {
        name: campaign.name.clone(),
        description: campaign.description.clone(),
        characters,
        sessions,
        encounters,
    };

    Ok(campaign)
}

use crate::{
    ServerError, database::{
        characters::{CharacterFilters, InsertCharacter},
        encounters::{EncounterFilters, InsertEncounter},
        sessions::InsertSession,
    }, models::{
        campaign::CampaignSessionCharacterRewards, encounter::EncounterType, ids::InternalId,
    }, v2::{database::item_instances::InsertItemInstance, models::item_instances::ItemInstance}
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
    pub id_hash: u32,
    pub name: String,
    pub level: u8,
    pub description: Option<String>,
    pub characters: Vec<ImportCharacter>,
    pub sessions: Vec<ImportSession>,
    pub encounters: Vec<ImportEncounter>,
    pub items: Vec<ImportItemInstance>
}

#[derive(Serialize, Debug, Deserialize)]
pub struct ImportCharacter {
    pub id_hash: u32,
    pub name: String,
    pub player: Option<String>,
    pub class: InternalId,
}

#[derive(Serialize, Debug, Deserialize)]
pub struct ImportItemInstance { 
    pub id_hash: u32,
     // This is a library item, and not a local referenced id, so we explicitly use InternalId
    pub library_item_id: InternalId,
    pub parent_item_id: Option<u32>,
    pub campaign_id: Option<u32>,
    pub encounter_id: Option<u32>,
    pub character_id: Option<u32>,
    pub session_id: Option<u32>,
    pub is_reward: bool,
    pub quantity: u16,
    pub nickname: Option<String>,
    pub notes: Option<String>,
}

#[derive(Serialize, Debug, Deserialize)]
pub struct ImportSession {
    pub id_hash: u32,
    pub name: Option<String>,
    pub description: Option<String>,
    pub date: Option<DateTime<Utc>>,
    pub compiled_rewards: HashMap<u32, ImportSessionCharacterRewards>,
}

#[derive(Serialize, Debug, Deserialize)]
pub struct ImportSessionCharacterRewards {
    pub gold: f32,
    // Default to true (if the reward struct is provided) for backwards compatibility
    pub present: Option<bool>,
}

#[derive(Serialize, Debug, Deserialize)]
pub struct ImportEncounter {
    pub id_hash: u32,

    pub name: String,
    pub description: Option<String>,
    pub session_ix: usize,

    pub party_level: u32,
    pub party_size: u32,

    #[serde(flatten)]
    pub encounter_type: EncounterType,

    pub treasure_items: Vec<InternalId>,
    pub treasure_currency: f32,
    pub extra_experience: i32,
}

#[derive(Serialize, Debug, Deserialize)]
pub struct ImportEncounterEnemy {
    pub id: InternalId,
    pub level_adjustment: i16,
}

pub async fn import_with_functions(
    campaign: ImportCampaign,
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    owner: InternalId,
) -> Result<InternalId, ServerError> {

    // The 'id of the import json' to 'internal id' mapping, added to as they get generated.
    let mut id_hash_to_internal_id = HashMap::new();

    // Insert campaign
    let campaign_id = super::campaigns::insert_campaign(
        &mut *tx,
        &InsertCampaign {
            name: campaign.name.clone(),
            description: campaign.description.clone(),
            initialization: None,
        },
        false,
        owner,
    )
    .await?;
    id_hash_to_internal_id.insert(
        campaign.id_hash,
        campaign_id,
    );

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
                class: c.class,
            })
            .collect_vec(),
    )
    .await?;
    id_hash_to_internal_id.extend(
        campaign
            .characters
            .iter()
            .zip(ids.iter())
            .map(|(c, id)| (c.id_hash, *id)),
    );

    // Insert sessions
    let insert_sessions = campaign
        .sessions
        .iter()
        .enumerate()
        .map(|(ix, s)| {
            Ok(InsertSession {
                name: s.name.clone(),
                description: s.description.clone(),
                session_order: (ix * 1000) as u32,
                play_date: s.date,
                characters: Some(
                    s.compiled_rewards
                        .keys()
                        .map(|character_name| {
                            id_hash_to_internal_id
                                .get(character_name)
                                .ok_or(ServerError::BadRequest(format!(
                                    "Character {} not found in 'characters'",
                                    character_name
                                )))
                                .copied()
                        })
                        .collect::<Result<Vec<_>, _>>()?,
                ),
            })
        })
        .collect::<Result<Vec<_>, ServerError>>()?;
    let session_ids_in_order =
        super::sessions::insert_sessions(&mut *tx, campaign_id, &insert_sessions).await?;
    id_hash_to_internal_id.extend(
        campaign
            .sessions
            .iter()
            .zip(session_ids_in_order.iter())
            .map(|(s, id)| (s.id_hash, *id)),
    );

    // Insert encounters
    let insert_encounters = campaign
        .encounters
        .iter()
        .map(|e| InsertEncounter {
            name: e.name.clone(),
            description: e.description.clone(),

            session_id: session_ids_in_order.get(e.session_ix).cloned(),
            party_level: e.party_level as u8,
            party_size: e.party_size as u8,
            encounter_type: e.encounter_type.clone(),
            treasure_items: e.treasure_items.clone(),
            treasure_currency: e.treasure_currency,
            extra_experience: e.extra_experience,
        })
        .collect_vec();
    let encounter_ids = super::encounters::insert_encounters(&mut *tx, owner, &insert_encounters).await?;
    id_hash_to_internal_id.extend(
        campaign
            .encounters
            .iter()
            .zip(encounter_ids.iter())
            .map(|(e, id)| (e.id_hash, *id)),
    );
    println!("id_hash_to_internal_id: {:?}", id_hash_to_internal_id);

    // Assign character contributions
    for (session, session_id) in campaign.sessions.iter().zip(session_ids_in_order) {
        let compiled_rewards = session
            .compiled_rewards
            .iter()
            .map(|(character_id_hash, rewards)| {
                let character_id =
                    id_hash_to_internal_id
                        .get(character_id_hash)
                        .ok_or(ServerError::BadRequest(format!(
                            "Character {} not found in 'characters'",
                            character_id_hash
                        )))?;
                Ok((
                    *character_id,
                    CampaignSessionCharacterRewards {
                        gold: rewards.gold as f64,
                        present: rewards.present.unwrap_or(true),
                        items: vec![], // TODO: This was removed to match new v2
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

    // Insert item instances
    let item_instances = campaign.items.into_iter().map(|item| {
        Ok(InsertItemInstance {
            library_item_id: item.library_item_id,
            parent_item_id: item.parent_item_id.map(|id| {
                id_hash_to_internal_id
                    .get(&id)
                    .copied()
                    .ok_or(ServerError::BadRequest(format!(
                        "Parent item {} not found in 'item_instances'",
                        id
                    )))
            }).transpose()?,
            campaign_id: item.campaign_id.map(|id| id_hash_to_internal_id.get(&id).copied()
                .ok_or(ServerError::BadRequest(format!(
                    "Campaign item {} not found in 'campaigns'",
                    id
                )))).transpose()?,
            encounter_id: item.encounter_id.map(|id| id_hash_to_internal_id.get(&id).copied()
                .ok_or(ServerError::BadRequest(format!(
                    "Encounter item {} not found in 'encounters'",
                    id
                )))).transpose()?,
            character_id: item.character_id.map(|id| id_hash_to_internal_id.get(&id).copied()
                .ok_or(ServerError::BadRequest(format!(
                    "Character item {} not found in 'characters'",
                    id
                )))).transpose()?,
            session_id: item.session_id.map(|id| id_hash_to_internal_id.get(&id).copied()
                .ok_or(ServerError::BadRequest(format!(
                    "Session item {} not found in 'sessions'",
                    id
                )))).transpose()?,
            is_reward: item.is_reward,
            quantity: item.quantity,
            nickname: item.nickname,
            notes: item.notes,
        })
    }).collect::<Result<Vec<_>, ServerError>>()?;
    crate::v2::database::item_instances::insert_item_instances(&mut *tx, item_instances)
        .await?;


    // Recalculate encounter summary derived data.
    // TODO: Antipattern, but not sure what the best way about it is. Postgres function, MV?
    super::encounters::recalculate_encounter_summary(&mut *tx, owner, &encounter_ids)
        .await?;

    println!("Done importing campaign: {}", campaign.name);

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
                id_hash: internal_id_to_hash(e.id),
                name: e.name,
                description: e.description,
                session_ix,
                party_level: e.party_level,
                party_size: e.party_size,
                encounter_type: e.encounter_type,
                treasure_items: e.treasure_items,
                treasure_currency: e.treasure_currency,
                extra_experience: e.extra_experience,
            })
        })
        .collect_vec();

    let sessions = sessions
        .into_iter()
        .map(|s| ImportSession {
            id_hash: internal_id_to_hash(s.id),
            name: Some(s.name),
            description: s.description,
            date: Some(s.play_date),
            compiled_rewards: s
                .compiled_rewards
                .into_iter()
                .map(|(character_id, rewards)| {
                    let character = characters.iter().find(|c| c.id == character_id).unwrap();
                    (
                        internal_id_to_hash(character.id),
                        ImportSessionCharacterRewards {
                            gold: rewards.gold as f32,
                            present: Some(rewards.present),
                            // items: rewards.items.to_vec(), TODO: This was removed to match new v2
                        },
                    )
                })
                .collect(),
        })
        .collect_vec();

    let characters = characters
        .into_iter()
        .map(|c| ImportCharacter {
            id_hash: internal_id_to_hash(c.id),
            name: c.name,
            player: c.player,
            class: c.class,
        })
        .collect();

        // TODO: This is v2 in a mishmash of v1/v2.
    let items = crate::v2::database::item_instances::get_item_instances(
        pool).await?.into_iter().map(|x| ImportItemInstance {
            id_hash: i32_to_hash(x.id),
            library_item_id: InternalId::from_i32(x.library_item_id),
            // TODO: Parent ids are intentionally disabled, as they would require the items to be made to get the ids.
            // It's somewhat self-referential. Minor refactor to fix..
            parent_item_id: None,
            campaign_id: x.campaign_id.map(i32_to_hash),
            encounter_id: x.encounter_id.map(i32_to_hash),
            character_id: x.character_id.map(i32_to_hash),
            session_id: x.session_id.map(i32_to_hash),
            is_reward: x.is_reward,
            quantity: x.quantity,
            nickname: x.nickname,
            notes: x.notes,
        })
        .collect_vec();

    let campaign = ImportCampaign {
        id_hash: internal_id_to_hash(campaign.id),
        name: campaign.name.clone(),
        level: campaign.level,
        description: campaign.description.clone(),
        characters,
        sessions,
        encounters,
        items
    };

    Ok(campaign)
}

// TODO: This is unnecessarily complicated
fn internal_id_to_hash(id: InternalId) -> u32 {
    // Convert InternalId to a hash
    let bytes = id.0.to_le_bytes();
    u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
}

fn i32_to_hash(id: i32) -> u32 {
    let internal_id = InternalId::from_i32(id);
    internal_id_to_hash(internal_id)
}
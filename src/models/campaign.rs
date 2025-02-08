use std::collections::HashMap;

use super::{characters::Character, events::EventLog, ids::InternalId};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// TODO: It may be prudent here to remove Clone, to prevent accidental duplication of what may be large data structures.
/// An entire campaign and its components.
/// Currently serving as an application level model for campaign context.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Campaign {
    pub id: InternalId,
    pub name: String,
    pub party: Vec<Character>,
    pub log: EventLog,
    pub sessions: Vec<CampaignSession>,
}

impl Default for Campaign {
    fn default() -> Self {
        Self {
            id: InternalId::new(),
            name: "New Campaign".to_string(),
            party: vec![],
            log: EventLog::default(),
            sessions: vec![],
        }
    }
}

// TODO: Campaign vs CampaignPartial - remove campaign? only use campaign?
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CampaignPartial {
    pub id: InternalId,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CampaignSession {
    pub id: InternalId,
    pub session_order: u32,
    pub name: String,
    pub description: Option<String>,
    pub play_date: DateTime<Utc>,
    pub encounter_ids: Vec<InternalId>,

    // Aggregation of encounter rewards, for easy reference.
    pub total_experience: u64,
    pub total_treasure_value: f64,

    // These are reward assignments from the encounters linked to this session.
    // Their encounter information is not considered here- it's fungible.
    // This is primarily for tracking who gets what rewards.
    pub compiled_rewards: HashMap<InternalId, CampaignSessionCharacterRewards>,
    pub unassigned_gold_rewards: f64,
    pub unassigned_item_rewards: Vec<InternalId>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CampaignSessionCharacterRewards {
    pub gold: f64,
    pub items: Vec<InternalId>,
}

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

    // encounter_id -> character_id -> rewards
    pub compiled_gold_rewards: HashMap<InternalId, HashMap<InternalId, i32>>,
    pub compiled_item_rewards: HashMap<InternalId, HashMap<InternalId, Vec<InternalId>>>,
    pub unassigned_gold_rewards: HashMap<InternalId, i32>,
    pub unassigned_item_rewards: HashMap<InternalId, Vec<InternalId>>,
}

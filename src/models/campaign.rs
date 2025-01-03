use super::{characters::Character, events::EventLog, ids::InternalId};

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
}

impl Default for Campaign {
    fn default() -> Self {
        Self {
            id: InternalId::new(),
            name: "New Campaign".to_string(),
            party: vec![],
            log: EventLog::default(),
        }
    }
}

// TODO: Campaign vs CampaignPartial - remove campaign? only use campaign?
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CampaignPartial {
    pub id: InternalId,
    pub name: String,
}

// Add this struct for campaign metadata
#[derive(Debug, Serialize)]
pub struct CampaignMetadata {
    pub total_sessions: i32,
    pub average_level: f32,
    pub total_experience: i32,
    pub last_session: Option<String>,
}

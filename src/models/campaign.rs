use super::events::EventLog;
use serde::{Deserialize, Serialize};

// TODO: It may be prudent here to remove Clone, to prevent accidental duplication of what may be large data structures.
/// An entire campaign and its components.
/// Currently serving as an application level model for campaign context.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Campaign {
    pub name: String,
    pub party: Vec<Character>,
    pub log: EventLog,
}

impl Default for Campaign {
    fn default() -> Self {
        Self {
            name: "New Campaign".to_string(),
            party: vec![],
            log: EventLog::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Character {
    pub name: String,
    pub player: String,
}

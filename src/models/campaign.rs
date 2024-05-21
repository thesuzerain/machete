use serde::{Deserialize, Serialize};

/// An entire campaign and its components.
/// Currently serving as an application level model
/// for campaign context.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Campaign {
    pub name: String,
    pub party: Vec<Character>,
}

impl Default for Campaign {
    fn default() -> Self {
        Self {
            name: "New Campaign".to_string(),
            party: vec![],
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Character {
    pub name: String,
    pub player: String,
}

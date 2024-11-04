use machete_core::ids::InternalId;
use serde::{Deserialize, Serialize};


#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Character {
    pub id : InternalId,
    pub name: String,
    pub player: Option<String>,
}
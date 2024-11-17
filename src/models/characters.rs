
use serde::{Deserialize, Serialize};

use super::ids::InternalId;


#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Character {
    pub id : InternalId,
    pub name: String,
    pub level: u8,
    pub player: Option<String>,
    pub class: InternalId,
}
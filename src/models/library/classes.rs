use super::{GameSystem, Rarity};
use crate::models::ids::InternalId;
use serde::{Deserialize, Serialize};

// TODO: 'Filterable' is kind of a mess
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct LibraryClass {
    pub id: InternalId,
    pub name: String,
    pub game_system: GameSystem,
    pub hp: u32,
    pub traditions: Vec<String>,
    pub rarity: Rarity,
    pub url: Option<String>,
    pub description: String,
    pub tags: Vec<String>,
}

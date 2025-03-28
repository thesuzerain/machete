use super::{GameSystem, Rarity};
use crate::models::ids::InternalId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct LibrarySpell {
    pub id: InternalId,
    pub name: String,
    pub game_system: GameSystem,
    pub rarity: Rarity,
    pub rank: u8,
    pub tags: Vec<String>,
    pub legacy: bool,
    pub traits: Vec<String>,

    pub traditions: Vec<String>,

    pub url: Option<String>,
    pub description: String,
}

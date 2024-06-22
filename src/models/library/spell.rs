use super::Rarity;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct LibrarySpell {
    pub name: String,
    pub game_system: String,
    pub rarity: Rarity,
    pub rank: u8,
    pub tags: Vec<String>,
}


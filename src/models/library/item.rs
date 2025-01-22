use super::{GameSystem, Rarity};
use crate::models::ids::InternalId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct LibraryItem {
    pub id: InternalId,
    pub name: String,
    pub game_system: GameSystem,
    pub rarity: Rarity,
    pub level: i8,
    pub tags: Vec<String>,
    pub price: f64,

    pub url: Option<String>,
    pub description: String,
}
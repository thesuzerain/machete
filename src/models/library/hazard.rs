
use serde::{Deserialize, Serialize};
use super::{GameSystem, Rarity};
use crate::models::ids::InternalId;

// TODO: 'Filterable' is kind of a mess
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct LibraryHazard {
    pub id : InternalId,
    pub name: String,
    pub game_system: GameSystem,
    pub rarity: Rarity,
    pub level: i8,
    pub tags: Vec<String>,

    pub url: Option<String>,
    pub description: String,

}

#[derive(Default, Debug, Serialize, Deserialize)]
pub enum HazardType {
    #[default]
    Trap,
    Haunt,
    Environmental,
}

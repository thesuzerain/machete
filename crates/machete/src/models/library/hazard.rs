use machete_core::ids::InternalId;
use serde::{Deserialize, Serialize};
use machete_macros::Filterable;
use super::{GameSystem, Rarity};

// TODO: 'Filterable' is kind of a mess
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Filterable)]
pub struct LibraryHazard {
    pub id : InternalId,
    #[filter(default, string)]
    pub name: String,
    #[filter(iter(GameSystem))]
    pub game_system: GameSystem,
    #[filter(iter(Rarity))]
    pub rarity: Rarity,
    #[filter(number)]
    pub level: i8,
    #[filter(string)]
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

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct HazardFilters {
    pub min_level: Option<i8>,
    pub max_level: Option<i8>,
    pub hazard_type: Option<HazardType>,
    pub name: Option<String>,
    pub rarity: Option<Rarity>,
    pub game_system: Option<GameSystem>,
    #[serde(default)]
    pub tags: Vec<String>,

    pub limit : Option<u64>,
    pub page : Option<u64>,
}
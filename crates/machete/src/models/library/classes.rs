use machete_core::ids::InternalId;
use serde::{Deserialize, Serialize};
use machete_macros::Filterable;
use super::{GameSystem, Rarity};

// TODO: 'Filterable' is kind of a mess
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Filterable)]
pub struct LibraryClass {
    pub id : InternalId,
    #[filter(default, string)]
    pub name: String,
    #[filter(iter(GameSystem))]
    pub game_system: GameSystem,
    #[filter(iter(Rarity))]
    pub hp : u32,
    pub traditions : Vec<String>,
    pub rarity: Rarity,
    pub url: Option<String>,
    pub description: String,
    #[filter(string)]
    pub tags: Vec<String>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct ClassFilters {
    pub name : Option<String>,

    pub limit : Option<u64>,
    pub page : Option<u64>,
}

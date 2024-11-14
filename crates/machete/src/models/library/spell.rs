use std::str::FromStr;

use super::{GameSystem, Rarity};
use machete_core::{filters::{Filter, FilterType}, ids::InternalId};
use machete_macros::Filterable;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Filterable)]
pub struct LibrarySpell {
    pub id: InternalId,
    #[filter(default, string)]
    pub name: String,
    #[filter(iter(GameSystem))]
    pub game_system: GameSystem,
    #[filter(iter(Rarity))]
    pub rarity: Rarity,
    #[filter(number)]
    pub rank: u8,
    #[filter(string)]
    pub tags: Vec<String>,

    pub traditions : Vec<String>,

    pub url: Option<String>,
    pub description: String,
}
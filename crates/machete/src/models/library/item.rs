use super::{GameSystem, Rarity};
use machete_core::{filters::{Filter, FilterType}, ids::InternalId};
use machete_macros::Filterable;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Filterable)]
pub struct LibraryItem {
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
    #[filter(number)]
    pub price: Currency,

    pub url: Option<String>,
    pub description: String,

}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct Currency {
    #[serde(default)]
    pub gold: u32,
    #[serde(default)]
    pub silver: u32,
    #[serde(default)]
    pub copper: u32,
}

impl ToString for Currency {
    fn to_string(&self) -> String {
        let mut s = format!("{}g", self.gold);
        if self.silver > 0 {
            s.push_str(&format!(" {}s", self.silver));
        }
        if self.copper > 0 {
            s.push_str(&format!(" {}c", self.copper));
        }
        s
    }
}

impl Currency {
    pub fn as_base_unit(&self) -> u32 {
        self.gold * 100 + self.silver * 10 + self.copper
    }

    pub fn from_base_unit(value: u32) -> Self {
        let gold = value / 100;
        let silver = (value % 100) / 10;
        let copper = value % 10;
        Currency {
            gold,
            silver,
            copper,
        }
    }
}

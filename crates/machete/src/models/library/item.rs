use super::{GameSystem, Rarity};
use machete_core::filters::{Filter, FilterType};
use machete_macros::Filterable;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Filterable)]
pub struct LibraryItem {
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
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct ItemFilters {
    pub min_level: Option<i8>,
    pub max_level: Option<i8>,
    pub min_price: Option<i32>, // TODO: should this be currency?
    pub max_price: Option<i32>,
    pub name: Option<String>,
    pub rarity: Option<Rarity>,
    pub game_system: Option<GameSystem>,
    #[serde(default)]
    pub tags: Vec<String>,
}

// TODO: bundle with macro?
impl TryFrom<Filter<LibraryItem>> for ItemFilters {
    type Error = String;

    fn try_from(value: Filter<LibraryItem>) -> Result<Self, Self::Error> {
        let mut creature_filters = ItemFilters::default();
        if value.field == "level" {
            match value.filter_type {
                FilterType::GreaterThan(value) => {
                    creature_filters.min_level = Some(value as i8);
                }
                FilterType::LessThan(value) => {
                    creature_filters.max_level = Some(value as i8);
                }
                FilterType::EqualToNumber(value) => {
                    creature_filters.min_level = Some(value as i8);
                    creature_filters.max_level = Some(value as i8);
                }
                _ => {}
            }
        } else if value.field == "name" {
            match value.filter_type {
                FilterType::Contains(value) => {
                    creature_filters.name = Some(value);
                }
                _ => {}
            }
        } else if value.field == "rarity" {
            match value.filter_type {
                FilterType::EqualToChoice(value) => {
                    creature_filters.rarity = Some(Rarity::from_str(&value).unwrap());
                }
                _ => {}
            }
        } else if value.field == "game_system" {
            match value.filter_type {
                FilterType::EqualToChoice(value) => {
                    creature_filters.game_system = Some(GameSystem::from_str(&value).unwrap());
                }
                _ => {}
            }
        } else if value.field == "tags" {
            match value.filter_type {
                FilterType::Contains(value) => {
                    creature_filters.tags.push(value);
                }
                _ => {}
            }
        } else if value.field == "price" {
            match value.filter_type {
                FilterType::GreaterThan(value) => {
                    creature_filters.min_price = Some(value as i32);
                }
                FilterType::LessThan(value) => {
                    creature_filters.max_price = Some(value as i32);
                }
                FilterType::EqualToNumber(value) => {
                    creature_filters.min_price = Some(value as i32);
                    creature_filters.max_price = Some(value as i32);
                }
                _ => {}
            }
        } else {
            return Err(format!("Invalid field: {}", value.field));
        }
        Ok(creature_filters)
    }
}

// TODO: doesn't work for duplicate Some values
impl ItemFilters {
    pub fn merge(self, other: Self) -> Self {
        ItemFilters {
            min_level: self.min_level.or(other.min_level),
            max_level: self.max_level.or(other.max_level),
            name: self.name.or(other.name),
            rarity: self.rarity.or(other.rarity),
            game_system: self.game_system.or(other.game_system),
            min_price: self.min_price.or(other.min_price),
            max_price: self.max_price.or(other.max_price),
            tags: self.tags.into_iter().chain(other.tags).collect(),
        }
    }
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

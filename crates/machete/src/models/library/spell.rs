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
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct SpellFilters {
    pub min_rank: Option<u8>,
    pub max_rank: Option<u8>,
    pub name: Option<String>,
    pub rarity: Option<Rarity>,
    pub game_system: Option<GameSystem>,
    #[serde(default)]
    pub tags: Vec<String>,
}

impl TryFrom<Filter<LibrarySpell>> for SpellFilters {
    type Error = String;

    fn try_from(value: Filter<LibrarySpell>) -> Result<SpellFilters, Self::Error> {
        let mut creature_filters = SpellFilters::default();
        if value.field == "rank" {
            match value.filter_type {
                FilterType::GreaterThan(value) => {
                    creature_filters.min_rank = Some(value as u8);
                }
                FilterType::LessThan(value) => {
                    creature_filters.max_rank = Some(value as u8);
                }
                FilterType::EqualToNumber(value) => {
                    creature_filters.min_rank = Some(value as u8);
                    creature_filters.max_rank = Some(value as u8);
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
        } else {
            return Err(format!("Invalid field: {}", value.field));
        }
        Ok(creature_filters)
    }
}
// TODO: doesn't work for duplicate Some values
impl SpellFilters {
    pub fn merge(self, other: Self) -> Self {
        SpellFilters {
            min_rank: self.min_rank.or(other.min_rank),
            max_rank: self.max_rank.or(other.max_rank),
            name: self.name.or(other.name),
            rarity: self.rarity.or(other.rarity),
            game_system: self.game_system.or(other.game_system),
            tags: self.tags.into_iter().chain(other.tags).collect(),
        }
    }
}

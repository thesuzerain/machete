use std::str::FromStr;

use crate::filters::filter::{Filter, FilterType};

use super::{GameSystem, Rarity};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct LibrarySpell {
    pub name: String,
    pub game_system: GameSystem,
    pub rarity: Rarity,
    pub rank: u8,
    pub tags: Vec<String>,
}

#[derive(Default)]
pub struct SpellFilters {
    pub min_rank: Option<u8>,
    pub max_rank: Option<u8>,
    pub name: Option<String>,
    pub rarity: Option<Rarity>,
    pub game_system: Option<GameSystem>,
    pub tags: Vec<String>,
}

// TODO: mov e these
impl Filter<LibrarySpell> {
    // todo: should these be returning result instead?
    // TODO: &self or self?
    pub fn to_spell_filter(self) -> Option<SpellFilters> {
        let mut creature_filters = SpellFilters::default();
        if self.field == "rank" {
            match self.filter_type {
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
        } else if self.field == "name" {
            match self.filter_type {
                FilterType::Contains(value) => {
                    creature_filters.name = Some(value);
                }
                _ => {}
            }
        } else if self.field == "rarity" {
            match self.filter_type {
                FilterType::EqualToChoice(value) => {
                    creature_filters.rarity = Some(Rarity::from_str(&value).unwrap());
                }
                _ => {}
            }
        } else if self.field == "game_system" {
            match self.filter_type {
                FilterType::EqualToChoice(value) => {
                    creature_filters.game_system = Some(GameSystem::from_str(&value).unwrap());
                }
                _ => {}
            }
        } else if self.field == "tags" {
            match self.filter_type {
                FilterType::Contains(value) => {
                    creature_filters.tags.push(value);
                }
                _ => {}
            }
        } else {
            return None;
        }
        Some(creature_filters)
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
            tags: self
                .tags
                .into_iter()
                .chain(other.tags.into_iter())
                .collect(),
        }
    }
}

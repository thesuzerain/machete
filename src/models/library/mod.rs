use crate::models::ids::InternalId;
use creature::LibraryCreature;
use item::LibraryItem;

use serde::{Deserialize, Serialize};
use spell::LibrarySpell;
use std::{collections::HashMap, str::FromStr};

pub mod classes;
pub mod creature;
pub mod hazard;
pub mod item;
pub mod spell;

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LibraryObjectType {
    Creature,
    Item,
    Hazard,
    Spell,
}


#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum GameSystem {
    #[default]
    PF2E,
    DND5E,
}

impl std::fmt::Display for GameSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameSystem::PF2E => write!(f, "PF2E"),
            GameSystem::DND5E => write!(f, "DND5E"),
        }
    }
}

impl FromStr for GameSystem {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "PF2E" => Ok(GameSystem::PF2E),
            "DND5E" => Ok(GameSystem::DND5E),
            _ => Err(()),
        }
    }
}

impl GameSystem {
    pub fn iter() -> impl Iterator<Item = GameSystem> {
        vec![GameSystem::PF2E, GameSystem::DND5E].into_iter()
    }

    pub fn as_i64(&self) -> i64 {
        match self {
            GameSystem::PF2E => 0,
            GameSystem::DND5E => 1,
        }
    }

    pub fn from_i64(value: i64) -> Self {
        match value {
            0 => GameSystem::PF2E,
            1 => GameSystem::DND5E,
            _ => GameSystem::PF2E,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum Rarity {
    #[default]
    #[serde(alias = "Common")]
    Common,
    #[serde(alias = "Uncommon")]
    Uncommon,
    #[serde(alias = "Rare")]
    Rare,
    #[serde(alias = "Unique")]
    Unique,
}

impl std::fmt::Display for Rarity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Rarity::Common => write!(f, "Common"),
            Rarity::Uncommon => write!(f, "Uncommon"),
            Rarity::Rare => write!(f, "Rare"),
            Rarity::Unique => write!(f, "Unique"),
        }
    }
}

impl FromStr for Rarity {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Common" => Ok(Rarity::Common),
            "Uncommon" => Ok(Rarity::Uncommon),
            "Rare" => Ok(Rarity::Rare),
            "Unique" => Ok(Rarity::Unique),
            _ => Err(()),
        }
    }
}

impl Rarity {
    // TODO: This is essentially a re-implementation of Serialize_repr.
    // Alternatively, should this be i32 for postgres? (Alongside similar as_i64 functions)
    pub fn as_i64(&self) -> i64 {
        match self {
            Rarity::Common => 0,
            Rarity::Uncommon => 1,
            Rarity::Rare => 2,
            Rarity::Unique => 3,
        }
    }

    pub fn from_i64(value: i64) -> Self {
        match value {
            0 => Rarity::Common,
            1 => Rarity::Uncommon,
            2 => Rarity::Rare,
            3 => Rarity::Unique,
            _ => Rarity::Common,
        }
    }
    pub fn iter() -> impl Iterator<Item = Rarity> {
        vec![
            Rarity::Common,
            Rarity::Uncommon,
            Rarity::Rare,
            Rarity::Unique,
        ]
        .into_iter()
    }
}

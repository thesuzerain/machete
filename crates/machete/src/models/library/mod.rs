use creature::LibraryCreature;
use item::LibraryItem;
use machete_core::ids::InternalId;
use serde::{Deserialize, Serialize};
use spell::LibrarySpell;
use std::{collections::HashMap, str::FromStr};

pub mod creature;
pub mod classes;
pub mod hazard;
pub mod item;
pub mod spell;

// TODO: It may be prudent here to remove Clone, to prevent accidental duplication of what may be large data structures.
/// A library of all items that a campaign might reference.
/// This includes creatures, items, spells, etc.
/// Currently serving as an application level model for campaign context.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Library {
    pub items: HashMap<InternalId, LibraryItem>,
    pub creatures: HashMap<InternalId, LibraryCreature>,
    pub spells: HashMap<InternalId, LibrarySpell>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum GameSystem {
    #[default]
    PF2E,
    DND5E,
}

impl ToString for GameSystem {
    fn to_string(&self) -> String {
        match self {
            GameSystem::PF2E => "PF2E".to_string(),
            GameSystem::DND5E => "DND5E".to_string(),
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

impl ToString for Rarity {
    fn to_string(&self) -> String {
        match self {
            Rarity::Common => "Common".to_string(),
            Rarity::Uncommon => "Uncommon".to_string(),
            Rarity::Rare => "Rare".to_string(),
            Rarity::Unique => "Unique".to_string(),
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

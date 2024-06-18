use super::ids::InternalId;
use creature::LibraryCreature;
use item::LibraryItem;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod creature;
pub mod item;

// TODO: It may be prudent here to remove Clone, to prevent accidental duplication of what may be large data structures.
/// A library of all items that a campaign might reference.
/// This includes creatures, items, spells, etc.
/// Currently serving as an application level model for campaign context.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Library {
    pub items: HashMap<InternalId, LibraryItem>,
    pub creatures: HashMap<InternalId, LibraryCreature>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
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

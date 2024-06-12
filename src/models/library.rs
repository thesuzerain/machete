use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::ids::InternalId;

// TODO: It may be prudent here to remove Clone, to prevent accidental duplication of what may be large data structures.
/// A library of all items that a campaign might reference.
/// This includes creatures, items, spells, etc.
/// Currently serving as an application level model for campaign context.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Library {
    pub items: HashMap<InternalId, LibraryItem>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct LibraryItem {
    pub name: String,
    pub price: Currency,
    pub game_system: String,
    pub rarity: Rarity,
    pub level: u8,
    pub tags: Vec<String>,
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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
}

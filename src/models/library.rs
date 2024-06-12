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
    pub price: u32,
    pub game_system: String,
    pub rarity: String,
    pub level: u8,
    pub tags: Vec<String>,
}

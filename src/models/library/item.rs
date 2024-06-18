use super::Rarity;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct LibraryItem {
    pub name: String,
    pub game_system: String,
    pub rarity: Rarity,
    pub level: u8,
    pub tags: Vec<String>,

    pub price: Currency,
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

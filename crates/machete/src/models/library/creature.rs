use super::Rarity;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct LibraryCreature {
    // TODO: Should these be modularized? (Same as LibraryItem)
    pub name: String,
    pub game_system: String,
    pub rarity: Rarity,
    pub level: i8,
    pub tags: Vec<String>,

    pub alignment: Alignment,
    pub size: Size,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Alignment {
    #[serde(rename = "lg")]
    LawfulGood,
    #[serde(rename = "ng")]
    NeutralGood,
    #[serde(rename = "cg")]
    ChaoticGood,
    #[serde(rename = "ln")]
    LawfulNeutral,
    #[serde(rename = "n")]
    TrueNeutral,
    #[serde(rename = "cn")]
    ChaoticNeutral,
    #[serde(rename = "le")]
    LawfulEvil,
    #[serde(rename = "ne")]
    NeutralEvil,
    #[serde(rename = "ce")]
    ChaoticEvil,
    #[serde(other)]
    None,
}

impl ToString for Alignment {
    fn to_string(&self) -> String {
        match self {
            Alignment::LawfulGood => "Lawful Good".to_string(),
            Alignment::NeutralGood => "Neutral Good".to_string(),
            Alignment::ChaoticGood => "Chaotic Good".to_string(),
            Alignment::LawfulNeutral => "Lawful Neutral".to_string(),
            Alignment::TrueNeutral => "True Neutral".to_string(),
            Alignment::ChaoticNeutral => "Chaotic Neutral".to_string(),
            Alignment::LawfulEvil => "Lawful Evil".to_string(),
            Alignment::NeutralEvil => "Neutral Evil".to_string(),
            Alignment::ChaoticEvil => "Chaotic Evil".to_string(),
            Alignment::None => "None".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Size {
    #[serde(alias = "Tiny")]
    Tiny,
    #[serde(alias = "Small")]
    Small,
    #[serde(alias = "Medium")]
    Medium,
    #[serde(alias = "Large")]
    Large,
    #[serde(alias = "Huge")]
    Huge,
    #[serde(alias = "Gargantuan")]
    Gargantuan,
}

impl ToString for Size {
    fn to_string(&self) -> String {
        match self {
            Size::Tiny => "Tiny".to_string(),
            Size::Small => "Small".to_string(),
            Size::Medium => "Medium".to_string(),
            Size::Large => "Large".to_string(),
            Size::Huge => "Huge".to_string(),
            Size::Gargantuan => "Gargantuan".to_string(),
        }
    }
}

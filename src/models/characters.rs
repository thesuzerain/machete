use serde::{Deserialize, Serialize};

use super::ids::InternalId;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Character {
    pub id: InternalId,
    pub name: String,
    pub level: u8,
    pub player: Option<String>,
    pub class: InternalId,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Stat {
    Strength,
    Dexterity,
    Constitution,
    Intelligence,
    Wisdom,
    Charisma,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Skill {
    Acrobatics,
    Arcana,
    Athletics,
    Crafting,
    Deception,
    Diplomacy,
    Intimidation,
    Lore(Option<String>),
    Medicine,
    Nature,
    Occultism,
    Performance,
    Religion,
    Society,
    Stealth,
    Survival,
    Thievery,
}

impl Stat {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Strength" => Some(Self::Strength),
            "Dexterity" => Some(Self::Dexterity),
            "Constitution" => Some(Self::Constitution),
            "Intelligence" => Some(Self::Intelligence),
            "Wisdom" => Some(Self::Wisdom),
            "Charisma" => Some(Self::Charisma),
            _ => None,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Self::Strength => "Strength",
            Self::Dexterity => "Dexterity",
            Self::Constitution => "Constitution",
            Self::Intelligence => "Intelligence",
            Self::Wisdom => "Wisdom",
            Self::Charisma => "Charisma",
        }.to_string()
    }

    pub fn iter() -> impl Iterator<Item = Self> {
        vec![
            Self::Strength,
            Self::Dexterity,
            Self::Constitution,
            Self::Intelligence,
            Self::Wisdom,
            Self::Charisma,
        ]
        .into_iter()
    }
}

impl Skill {

    pub fn to_string(&self) -> String {
        match self {
            Self::Acrobatics => "Acrobatics".to_string(),
            Self::Arcana => "Arcana".to_string(),
            Self::Athletics => "Athletics".to_string(),
            Self::Crafting => "Crafting".to_string(),
            Self::Deception => "Deception".to_string(),
            Self::Diplomacy => "Diplomacy".to_string(),
            Self::Intimidation => "Intimidation".to_string(),
            Self::Lore(None) => "Lore".to_string(),
            Self::Lore(Some(s)) => format!("Lore ({})", s),
            Self::Medicine => "Medicine".to_string(),
            Self::Nature => "Nature".to_string(),
            Self::Occultism => "Occultism".to_string(),
            Self::Performance => "Performance".to_string(),
            Self::Religion => "Religion".to_string(),
            Self::Society => "Society".to_string(),
            Self::Stealth => "Stealth".to_string(),
            Self::Survival => "Survival".to_string(),
            Self::Thievery => "Thievery".to_string(),
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Acrobatics" => Some(Self::Acrobatics),
            "Arcana" => Some(Self::Arcana),
            "Athletics" => Some(Self::Athletics),
            "Crafting" => Some(Self::Crafting),
            "Deception" => Some(Self::Deception),
            "Diplomacy" => Some(Self::Diplomacy),
            "Intimidation" => Some(Self::Intimidation),
            s if s.starts_with("Lore (") && s.ends_with(")") => Some(Self::Lore(Some(s[6..s.len() - 1].to_string()))),
            "Lore" => Some(Self::Lore(None)),
            "Medicine" => Some(Self::Medicine),
            "Nature" => Some(Self::Nature),
            "Occultism" => Some(Self::Occultism),
            "Performance" => Some(Self::Performance),
            "Religion" => Some(Self::Religion),
            "Society" => Some(Self::Society),
            "Stealth" => Some(Self::Stealth),
            "Survival" => Some(Self::Survival),
            "Thievery" => Some(Self::Thievery),
            _ => None,
        }
    }

    pub fn iter() -> impl Iterator<Item = Self> {
        vec![
            Self::Acrobatics,
            Self::Arcana,
            Self::Athletics,
            Self::Crafting,
            Self::Deception,
            Self::Diplomacy,
            Self::Intimidation,
            Self::Lore(None),
            Self::Medicine,
            Self::Nature,
            Self::Occultism,
            Self::Performance,
            Self::Religion,
            Self::Society,
            Self::Stealth,
            Self::Survival,
            Self::Thievery,
        ]
        .into_iter()
    }
}
use super::{GameSystem, Rarity};
use crate::models::{
    characters::{Skill, Stat},
    ids::InternalId,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct LibraryItem {
    pub id: InternalId,
    pub name: String,
    pub game_system: GameSystem,
    pub rarity: Rarity,
    pub level: i8,
    pub tags: Vec<String>,

    // Almost always set. But an item *can* be priceless (usually Unique items)
    pub price: Option<f64>,

    pub url: Option<String>,
    pub description: String,

    pub item_categories: Vec<String>,
    pub traits: Vec<String>,

    pub consumable: bool,
    pub magical: bool,
    // TODO: MAy need to be combined with GameSystem, as its part of the game system
    pub legacy: bool,

    pub item_type: RuneItemType,
    pub skill_boosts: Vec<SkillPotency>,
    pub runes: Vec<Rune>,
    pub apex_stat: Option<Stat>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SkillPotency {
    // TODO: This should, eventually, be not an Option
    pub skill: Option<Skill>,
    pub bonus: i8,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(tag = "type")]
pub enum Rune {
    Resilient {
        potency: i8,
    },
    ArmorPotency {
        potency: i8,
    },
    Striking {
        potency: i8,
    },
    WeaponPotency {
        potency: i8,
    },
    ShieldPotency {
        potency: i8,
    },
    PropertyRune {
        property: String,
        applied_to: RuneItemType, // The item type this rune is applied to
        potency: i8, // For a property rune, the potency = the rune's level (to uniquely identify variant runes of the same name)
    },
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Copy)]
pub enum RuneItemType {
    FundamentalRune,
    PropertyRune,
    Armor,
    Weapon,
    Shield,
    None,
}

impl Rune {
    pub fn from_parts(s: &str, potency: i8, applied_to: RuneItemType) -> Self {
        match s {
            "Resilient" => Self::Resilient { potency },
            "Armor Potency" => Self::ArmorPotency { potency },
            "Striking" => Self::Striking { potency },
            "Weapon Potency" => Self::WeaponPotency { potency },
            "Shield Potency" => Self::ShieldPotency { potency },
            _ => Self::PropertyRune {
                property: s.to_string(),
                potency,
                applied_to,
            },
        }
    }

    pub fn to_parts(&self) -> (String, i8, RuneItemType) {
        match self {
            Self::Resilient { potency: p } => ("Resilient".to_string(), *p, RuneItemType::Armor),
            Self::ArmorPotency { potency: p } => {
                ("Armor Potency".to_string(), *p, RuneItemType::Armor)
            }
            Self::Striking { potency: p } => ("Striking".to_string(), *p, RuneItemType::Weapon),
            Self::WeaponPotency { potency: p } => {
                ("Weapon Potency".to_string(), *p, RuneItemType::Weapon)
            }
            Self::ShieldPotency { potency: p } => {
                ("Shield Potency".to_string(), *p, RuneItemType::Shield)
            }
            Self::PropertyRune {
                property: s,
                potency: p,
                applied_to: a,
            } => (s.to_string(), *p, *a),
        }
    }

    pub fn to_full_name(&self) -> String {
        let get_adjective = |p: i8| {
            match p {
                0 => "", // Or "minor"
                1 => "(Lesser)",
                2 => "(Moderate)",
                3 => "(Greater)",
                4 => "(Major)",
                5 => "(Superior)", // Or mythic, or 'true'
                _ => "(Unknown)",
            }
        };

        match self {
            Self::Resilient { potency: p } => format!("Resilient Rune {}", get_adjective(*p)),
            Self::ArmorPotency { potency: p } => format!("Armor Potency Rune +{}", p),
            Self::Striking { potency: p } => format!("Striking Rune {}", get_adjective(*p)),
            Self::WeaponPotency { potency: p } => format!("Weapon Potency Rune +{}", p),
            Self::ShieldPotency { potency: p } => format!("Shield Potency Rune +{}", p),
            Self::PropertyRune {
                property: s,
                potency: p,
                ..
            } => format!("{} {}", s, get_adjective(*p)),
        }
    }

    pub fn get_potency(&self) -> i8 {
        match self {
            Self::Resilient { potency: p } => *p,
            Self::ArmorPotency { potency: p } => *p,
            Self::Striking { potency: p } => *p,
            Self::WeaponPotency { potency: p } => *p,
            Self::ShieldPotency { potency: p } => *p,
            Self::PropertyRune { potency: p, .. } => *p,
        }
    }
}

impl RuneItemType {
    pub fn from_str(s: &str) -> Self {
        match s {
            "Fundamental Rune" => Self::FundamentalRune,
            "Property Rune" => Self::PropertyRune,
            "Armor" => Self::Armor,
            "Weapon" => Self::Weapon,
            "Shield" => Self::Shield,
            _ => Self::None,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Self::FundamentalRune => "Fundamental Rune",
            Self::PropertyRune => "Property Rune",
            Self::Armor => "Armor",
            Self::Weapon => "Weapon",
            Self::Shield => "Shield",
            Self::None => "None",
        }
        .to_string()
    }
}

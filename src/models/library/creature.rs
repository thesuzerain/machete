use std::str::FromStr;

use super::{GameSystem, Rarity};
use crate::models::ids::InternalId;
use serde::{Deserialize, Serialize};

// TODO: 'Filterable' is kind of a mess
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct LibraryCreature {
    pub id: InternalId,
    pub name: String,
    pub game_system: GameSystem,
    pub rarity: Rarity,
    pub level: i8,
    pub tags: Vec<String>,
    pub alignment: Alignment,
    pub size: Size,
    pub traits: Vec<String>,
    pub legacy: bool,
    pub remastering_alt_id: Option<InternalId>,

    pub url: Option<String>,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub enum Alignment {
    #[serde(rename = "LG")]
    LawfulGood,
    #[serde(rename = "NG")]
    NeutralGood,
    #[serde(rename = "CG")]
    ChaoticGood,
    #[serde(rename = "LN")]
    LawfulNeutral,
    #[serde(rename = "N")]
    #[default]
    TrueNeutral,
    #[serde(rename = "CN")]
    ChaoticNeutral,
    #[serde(rename = "LE")]
    LawfulEvil,
    #[serde(rename = "NE")]
    NeutralEvil,
    #[serde(rename = "CE")]
    ChaoticEvil,
    #[serde(other)]
    None,
}

impl Alignment {
    pub fn iter() -> impl Iterator<Item = Alignment> {
        vec![
            Alignment::LawfulGood,
            Alignment::NeutralGood,
            Alignment::ChaoticGood,
            Alignment::LawfulNeutral,
            Alignment::TrueNeutral,
            Alignment::ChaoticNeutral,
            Alignment::LawfulEvil,
            Alignment::NeutralEvil,
            Alignment::ChaoticEvil,
            Alignment::None,
        ]
        .into_iter()
    }

    // TODO: This is essentially a re-implementation of Serialize_repr.
    // Alternatively, should this be i32 for postgres? (Alongside similar as_i64 functions)
    pub fn as_i64(&self) -> i64 {
        match self {
            Alignment::LawfulGood => 0,
            Alignment::NeutralGood => 1,
            Alignment::ChaoticGood => 2,
            Alignment::LawfulNeutral => 3,
            Alignment::TrueNeutral => 4,
            Alignment::ChaoticNeutral => 5,
            Alignment::LawfulEvil => 6,
            Alignment::NeutralEvil => 7,
            Alignment::ChaoticEvil => 8,
            Alignment::None => 9,
        }
    }

    pub fn from_i64(i: i64) -> Alignment {
        match i {
            0 => Alignment::LawfulGood,
            1 => Alignment::NeutralGood,
            2 => Alignment::ChaoticGood,
            3 => Alignment::LawfulNeutral,
            4 => Alignment::TrueNeutral,
            5 => Alignment::ChaoticNeutral,
            6 => Alignment::LawfulEvil,
            7 => Alignment::NeutralEvil,
            8 => Alignment::ChaoticEvil,
            _ => Alignment::None,
        }
    }
}

impl std::fmt::Display for Alignment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Alignment::LawfulGood => write!(f, "Lawful Good"),
            Alignment::NeutralGood => write!(f, "Neutral Good"),
            Alignment::ChaoticGood => write!(f, "Chaotic Good"),
            Alignment::LawfulNeutral => write!(f, "Lawful Neutral"),
            Alignment::TrueNeutral => write!(f, "True Neutral"),
            Alignment::ChaoticNeutral => write!(f, "Chaotic Neutral"),
            Alignment::LawfulEvil => write!(f, "Lawful Evil"),
            Alignment::NeutralEvil => write!(f, "Neutral Evil"),
            Alignment::ChaoticEvil => write!(f, "Chaotic Evil"),
            Alignment::None => write!(f, "None"),
        }
    }
}

impl FromStr for Alignment {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "lawful good" | "lg" => Ok(Alignment::LawfulGood),
            "neutral good" | "ng" => Ok(Alignment::NeutralGood),
            "chaotic good" | "cg" => Ok(Alignment::ChaoticGood),
            "lawful neutral" | "ln" => Ok(Alignment::LawfulNeutral),
            "true neutral" | "tn" => Ok(Alignment::TrueNeutral),
            "chaotic neutral" | "cn" => Ok(Alignment::ChaoticNeutral),
            "lawful evil" | "le" => Ok(Alignment::LawfulEvil),
            "neutral evil" | "ne" => Ok(Alignment::NeutralEvil),
            "chaotic evil" | "ce" => Ok(Alignment::ChaoticEvil),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum Size {
    #[serde(alias = "Tiny")]
    Tiny,
    #[serde(alias = "Small")]
    Small,
    #[default]
    #[serde(alias = "Medium")]
    Medium,
    #[serde(alias = "Large")]
    Large,
    #[serde(alias = "Huge")]
    Huge,
    #[serde(alias = "Gargantuan")]
    Gargantuan,
}

impl std::fmt::Display for Size {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Size::Tiny => write!(f, "Tiny"),
            Size::Small => write!(f, "Small"),
            Size::Medium => write!(f, "Medium"),
            Size::Large => write!(f, "Large"),
            Size::Huge => write!(f, "Huge"),
            Size::Gargantuan => write!(f, "Gargantuan"),
        }
    }
}

impl FromStr for Size {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Tiny" => Ok(Size::Tiny),
            "Small" => Ok(Size::Small),
            "Medium" => Ok(Size::Medium),
            "Large" => Ok(Size::Large),
            "Huge" => Ok(Size::Huge),
            "Gargantuan" => Ok(Size::Gargantuan),
            _ => Err(()),
        }
    }
}

impl Size {
    pub fn iter() -> impl Iterator<Item = Size> {
        vec![
            Size::Tiny,
            Size::Small,
            Size::Medium,
            Size::Large,
            Size::Huge,
            Size::Gargantuan,
        ]
        .into_iter()
    }

    // TODO: This is essentially a re-implementation of Serialize_repr.
    // Alternatively, should this be i32 for postgres? (Alongside similar as_i64 functions)
    pub fn as_i64(&self) -> i64 {
        match self {
            Size::Tiny => 0,
            Size::Small => 1,
            Size::Medium => 2,
            Size::Large => 3,
            Size::Huge => 4,
            Size::Gargantuan => 5,
        }
    }

    pub fn from_i64(i: i64) -> Size {
        match i {
            0 => Size::Tiny,
            1 => Size::Small,
            2 => Size::Medium,
            3 => Size::Large,
            4 => Size::Huge,
            5 => Size::Gargantuan,
            _ => Size::Medium,
        }
    }
}

use std::str::FromStr;

use super::{GameSystem, Rarity};
use machete_core::{filters::{Filter, FilterType}, ids::InternalId};
use machete_macros::Filterable;
use serde::{Deserialize, Serialize};

// TODO: 'Filterable' is kind of a mess
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Filterable)]
pub struct LibraryCreature {
    pub id : InternalId,
    #[filter(default, string)]
    pub name: String,
    #[filter(iter(GameSystem))]
    pub game_system: GameSystem,
    #[filter(iter(Rarity))]
    pub rarity: Rarity,
    #[filter(number)]
    pub level: i8,
    #[filter(string)]
    pub tags: Vec<String>,

    #[filter(iter(Alignment))]
    pub alignment: Alignment,
    #[filter(iter(Size))]
    pub size: Size,

    pub url: Option<String>,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
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
    #[default]
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

impl FromStr for Alignment {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Lawful Good" => Ok(Alignment::LawfulGood),
            "Neutral Good" => Ok(Alignment::NeutralGood),
            "Chaotic Good" => Ok(Alignment::ChaoticGood),
            "Lawful Neutral" => Ok(Alignment::LawfulNeutral),
            "True Neutral" => Ok(Alignment::TrueNeutral),
            "Chaotic Neutral" => Ok(Alignment::ChaoticNeutral),
            "Lawful Evil" => Ok(Alignment::LawfulEvil),
            "Neutral Evil" => Ok(Alignment::NeutralEvil),
            "Chaotic Evil" => Ok(Alignment::ChaoticEvil),
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

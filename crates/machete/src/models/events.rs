use chrono::{DateTime, Utc};
use machete_core::ids::InternalId;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Display};

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct EventLog {
    pub event_groups: HashMap<InternalId, EventGroup>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EventGroup {
    // TODO: Should all of these ids be referenced inside the object as well as the containing map?
    pub id: InternalId,
    pub name: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub timestamp: DateTime<Utc>,
    pub description: Option<String>,
    pub events: Vec<InternalId>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(tag = "event_group_type", content = "data")]
pub enum EventGroupType {
    Encounter {
        // TODO: Encounter metadata
    },
    #[default]
    Miscellaneous,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Event {
    pub id: InternalId,
    pub log: Option<InternalId>,
    pub character: Option<InternalId>,
    pub timestamp: DateTime<Utc>,
    #[serde(flatten)]
    pub event_type: EventType,
}

impl Default for Event {
    fn default() -> Self {
        Event {
            id: InternalId::new(),
            log: None,
            timestamp: Utc::now(),
            character: None,
            event_type: EventType::CurrencyGain { currency: 0 },
        }
    }
}

/// Metadata for an event type.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(tag = "event_type", content = "data")]
pub enum EventType {
    CurrencyGain { currency: u64 },
    ExperienceGain { experience: u64 },
    // TODO: EnemyDefeated, HazardDefeated, ItemGain, etc should be by ID.
    EnemyDefeated {
        name: String,
    },
    HazardDefeated {
        name: String,
    },
    ItemGain {
        name: String,
    },
    // TODO: Some kind of custom event type.
}

impl Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventType::CurrencyGain { currency } => write!(f, "Currency Gain: {}", currency),
            EventType::ExperienceGain { experience } => {
                write!(f, "Experience Gain: {}", experience)
            }
            EventType::EnemyDefeated { name } => write!(f, "Enemy Defeated: {}", name),
            EventType::HazardDefeated { name } => write!(f, "Hazard Defeated: {}", name),
            EventType::ItemGain { name } => write!(f, "Item Gain: {}", name),
        }
    }
}

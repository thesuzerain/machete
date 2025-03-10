use chrono::{DateTime, Utc};

use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Display};

use super::ids::InternalId;

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
            event_type: EventType::CurrencyGain { currency: 0.0 },
        }
    }
}

/// Metadata for an event type.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(tag = "event_type", content = "data")]
pub enum EventType {
    CurrencyGain {
        currency: f64,
    },
    ExperienceGain {
        experience: u64,
    },
    // TODO: EnemyDefeated, HazardDefeated, ItemGain, etc should be by ID.
    EnemyDefeated {
        id: InternalId,
        #[serde(default)]
        level_adjustment: i16,
    },
    HazardDefeated {
        id: InternalId,
    },
    ItemGain {
        id: InternalId,
    },
    // TODO: Some kind of custom event type.
}

impl Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventType::CurrencyGain { currency } => {
                write!(f, "Currency Gain: {}", currency)
            }
            EventType::ExperienceGain { experience } => {
                write!(f, "Experience Gain: {}", experience)
            }
            EventType::EnemyDefeated {
                id,
                level_adjustment,
            } => match level_adjustment {
                0 => write!(f, "Enemy Defeated: {}", id),
                1 => write!(f, "Elite Enemy Defeated: {}", id),
                -1 => write!(f, "Weak Enemy Defeated: {}", id),
                _ => write!(
                    f,
                    "Enemy Defeated: {} (Level Adjustment: {})",
                    id, level_adjustment
                ),
            },
            EventType::HazardDefeated { id } => write!(f, "Hazard Defeated: {}", id),
            EventType::ItemGain { id } => write!(f, "Item Gain: {}", id),
        }
    }
}

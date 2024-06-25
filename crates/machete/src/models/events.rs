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
    pub events: HashMap<InternalId, Event>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Event {
    pub id: InternalId,
    pub character: Option<String>,
    #[serde(flatten)]
    pub event_type: EventType,
}

impl Default for Event {
    fn default() -> Self {
        Event {
            id: InternalId::new(),
            character: None,
            event_type: EventType::CurrencyGain { currency: 0 },
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(tag = "event_type")]
pub enum EventType {
    CurrencyGain { currency: u64 },
    ExperienceGain { experience: u64 },
}

impl Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventType::CurrencyGain { currency } => write!(f, "Currency Gain: {}", currency),
            EventType::ExperienceGain { experience } => {
                write!(f, "Experience Gain: {}", experience)
            }
        }
    }
}

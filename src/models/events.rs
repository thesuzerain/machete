use std::{collections::HashMap, fmt::Display};

use chrono::{DateTime, Utc};
use egui::Ui;
use serde::{Deserialize, Serialize};

use crate::utils::SelectableOption;

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct EventLog {
    pub event_groups: HashMap<u64, EventGroup>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EventGroup {
    // TODO: A more robust ID system (uuid, base62, etc.)
    pub id: u64,
    pub name: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub timestamp: DateTime<Utc>,
    pub events: HashMap<u64, Event>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Event {
    // TODO: A more robust ID system (uuid, base62, etc.)
    pub id: u64,
    pub character: Option<String>,
    #[serde(flatten)]
    pub event_type: EventType,
}

impl Default for Event {
    fn default() -> Self {
        Event {
            id: 0,
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

impl SelectableOption for EventType {
    fn as_selectable_str(&self) -> &'static str {
        match self {
            EventType::CurrencyGain { .. } => "Currency Gain",
            EventType::ExperienceGain { .. } => "Experience Gain",
        }
    }

    fn iter_options() -> Vec<Self> {
        vec![
            EventType::CurrencyGain { currency: 0 },
            EventType::ExperienceGain { experience: 0 },
        ]
    }
}

impl EventType {
    // TODO: Turn into a trait
    // Also, shouldn't be stored in 'models', so it can `models` can be modular
    pub fn display_fields(&mut self, ui: &mut Ui) -> bool {
        let mut updated = false;
        match self {
            EventType::ExperienceGain { experience } => {
                ui.horizontal(|ui| {
                    ui.label("Experience:");
                    // TODO: Awful pattern here. Make a new widget for this.
                    // https://github.com/emilk/egui/issues/1348

                    let mut experience_string = experience.to_string();
                    let response = ui.text_edit_singleline(&mut experience_string);
                    if response.changed() {
                        *experience = match experience_string.parse() {
                            Ok(e) => e,
                            Err(_) => *experience,
                        };
                        updated = true;
                    }
                });
            }
            EventType::CurrencyGain { currency } => {
                ui.horizontal(|ui| {
                    ui.label("Currency:");
                    let mut currency_string = currency.to_string();
                    let response = ui.text_edit_singleline(&mut currency_string);

                    if response.changed() {
                        *currency = match currency_string.parse() {
                            Ok(c) => c,
                            Err(_) => *currency,
                        };
                        updated = true;
                    }
                });
            }
        }
        updated
    }
}

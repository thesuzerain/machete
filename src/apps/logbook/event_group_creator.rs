use crate::{
    models::{
        campaign::Campaign,
        events::{Event, EventGroup, EventType},
    },
    ui_models::DisplayFields,
    utils::SelectableOption,
};
use chrono::{DateTime, Utc};
use egui::{ahash::HashMap, ComboBox, Ui};
use itertools::Itertools;

use super::log::LogDisplayUiContext;

/// Add an EventGroup to the log.
/// TODO: This should be separated later when the UI is extricated from the core processing lib.
pub struct EventGroupCreator {
    pub name: String,
    pub custom_name: bool,

    pub datetime: DateTime<Utc>,

    pub template: EventGroupTemplate,
    // TODO: should this be within Template? Should it be an id?
    pub characters: HashMap<String, bool>,

    /// Current event group being edited.
    pub event_group: EventGroup,

    /// Event group display context (allows for editing events in the log)
    pub event_group_log_context: LogDisplayUiContext,
}

// TODO: Maybe these should be *more* broad, like 'WonBattle', but those might be better used from the battle itself.
// TODO: Does having fields mess up PartialEq?
#[derive(Debug, Clone, Default, PartialEq)]
pub enum EventGroupTemplate {
    #[default]
    None,
    ExperienceGain {
        experience: u64,
    },
    CurrencyGain {
        // TODO: Currency should be its own struct, allowing for different types of currency.
        currency: u64,
    }, // TODO: Add 'custom' for a custom added template
}

// TODO: This can all be in a models struct.
impl EventGroupTemplate {
    fn requires_separate_characters(&self) -> bool {
        match self {
            EventGroupTemplate::None => false,

            EventGroupTemplate::ExperienceGain { .. } => true,
            EventGroupTemplate::CurrencyGain { .. } => true,
        }
    }

    fn generate(&self, characters: &[String]) -> EventGroup {
        let mut event_group = EventGroup {
            // TODO: Id generation.
            id: 0,
            name: "New Event Group".to_string(),
            timestamp: Utc::now(),
            events: Default::default(),
        };
        match self {
            EventGroupTemplate::ExperienceGain { experience } => {
                for (i, character) in characters.iter().enumerate() {
                    event_group.events.insert(
                        i as u64,
                        Event {
                            id: i as u64,
                            event_type: EventType::ExperienceGain {
                                experience: *experience,
                            },
                            character: Some(character.clone()),
                        },
                    );
                }
            }
            EventGroupTemplate::CurrencyGain { currency } => {
                for (i, character) in characters.iter().enumerate() {
                    event_group.events.insert(
                        i as u64,
                        Event {
                            id: i as u64,
                            event_type: EventType::CurrencyGain {
                                currency: *currency,
                            },
                            character: Some(character.clone()),
                        },
                    );
                }
            }
            EventGroupTemplate::None => {}
        }
        event_group
    }
}

impl SelectableOption for EventGroupTemplate {
    fn as_selectable_str(&self) -> &'static str {
        match self {
            EventGroupTemplate::CurrencyGain { .. } => "Currency Gain",
            EventGroupTemplate::ExperienceGain { .. } => "Experience Gain",
            EventGroupTemplate::None => "None",
        }
    }

    fn iter_options() -> Vec<Self> {
        vec![
            EventGroupTemplate::None,
            EventGroupTemplate::CurrencyGain { currency: 0 },
            EventGroupTemplate::ExperienceGain { experience: 0 },
        ]
    }
}

impl EventGroupCreator {
    pub fn start(campaign: &Campaign) -> Self {
        let template = EventGroupTemplate::default();
        // TODO: See above note with 'character' -> should it be an id?
        let characters = campaign
            .party
            .iter()
            .map(|character| character.name.clone())
            .collect_vec();
        EventGroupCreator {
            name: "New Event Group".to_string(),
            custom_name: false,
            datetime: Utc::now(),
            event_group: template.generate(&characters),
            characters: characters
                .into_iter()
                .map(|character| (character, true))
                .collect(),
            template,
            event_group_log_context: LogDisplayUiContext::default(),
        }
    }

    pub fn ui(&mut self, ui: &mut Ui, campaign: &mut Campaign) {
        let mut updated_template = false;

        ui.horizontal(|ui| {
            ui.label("Add event:");

            if ui.button("Add").clicked() {
                // Add event as it is now.
                // TODO: Id generation. Search for '(0' and 'insert('
                campaign
                    .log
                    .event_groups
                    .insert(0, self.event_group.clone());
            }
        });

        ui.horizontal(|ui| {
            ui.label("Template:");
            let response = ComboBox::from_id_source("Template")
                .selected_text(self.template.as_selectable_str())
                .show_ui(ui, |ui| {
                    let mut any_selected = false;
                    for template in EventGroupTemplate::iter_options() {
                        let s = template.as_selectable_str().to_string();
                        any_selected |= ui
                            .selectable_value(&mut self.template, template, s)
                            .changed()
                    }
                    any_selected
                })
                .inner;
            updated_template |= response.unwrap_or(false);
        });

        ui.horizontal(|ui| {
            ui.label("Name:");
            ui.text_edit_singleline(&mut self.name);
        });

        ui.horizontal(|ui| {
            ui.label("Date:");
            // TODO: datetime editor struct
            let mut datetime_string = self.datetime.to_rfc3339();
            ui.text_edit_singleline(&mut datetime_string);
            self.datetime = match datetime_string.parse() {
                Ok(dt) => dt,
                Err(_) => self.datetime, // tODO: Better error handling here
            };
            if ui.button("Now").clicked() {
                self.datetime = Utc::now();
            }
        });

        // TODO: This looks stronger than I wanted it to
        ui.separator();

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label("Fields:");
                let response_updated = self.template.display_fields(ui);
                updated_template |= response_updated;
            });

            if self.template.requires_separate_characters() {
                ui.vertical(|ui| {
                    ui.label("Characters:");
                    for (character, is_active) in &mut self.characters {
                        let response = ui.checkbox(is_active, character);
                        updated_template |= response.changed();
                    }
                });
            }

            // TODO: This looks stronger than I wanted it to
            ui.separator();

            ui.vertical(|ui| {
                // TODO: Ensure this runs last. Does it still work?
                let characters = self
                    .characters
                    .iter()
                    .filter_map(|(character, is_active)| {
                        if *is_active {
                            Some(character.clone())
                        } else {
                            None
                        }
                    })
                    .collect_vec();
                let basic_template_group = self.template.generate(&characters);
                if updated_template {
                    self.event_group = basic_template_group.clone();
                }

                // tODO: This is hacky.
                self.event_group_log_context.display_event_group(
                    ui,
                    &mut self.event_group,
                    Some(&basic_template_group),
                    campaign,
                );
            });
        });
    }
}

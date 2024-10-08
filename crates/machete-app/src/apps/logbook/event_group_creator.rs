use crate::{
    ui_models::{events::EventGroupTemplateDisplayWrapper, DisplayFields},
    utils::SelectableOption,
    widgets::restricted_text_edit::RestrictedTextEdit,
};
use chrono::{DateTime, Utc};
use egui::{ahash::HashMap, Color32, ComboBox, Ui, Widget};
use itertools::Itertools;
use machete::models::{
    campaign::Campaign,
    events::{Event, EventGroup, EventType},
};
use machete_core::ids::InternalId;

use super::log::LogDisplayUiContext;

/// Add an EventGroup to the log.
/// TODO: This should be separated later when the UI is extricated from the core processing lib.
pub struct EventGroupCreator {
    pub name: String,
    pub custom_name: bool,

    /// Date and time of the event group.
    pub datetime: DateTime<Utc>,
    /// String representation of the datetime for editing (allowing intermediate incorrect values while typing)
    pub datetime_editing_string: String,

    pub template: EventGroupTemplate,
    pub event_group_template_editing_string: String,

    // TODO: Should this be within the Template? Should it be an id?
    pub characters: HashMap<String, bool>,

    /// Current event group being edited.
    pub event_group: EventGroup,

    /// Event group display context (allows for editing events in the log)
    pub event_group_log_context: LogDisplayUiContext,
}

// TODO: Maybe these should be *more* broad, like 'WonBattle', but those might be better used from the battle itself.
#[derive(Debug, Clone, Default, PartialEq)]
pub enum EventGroupTemplate {
    #[default]
    None,
    ExperienceGain {
        experience: u64,
    },
    CurrencyGain {
        // TODO: Currency should be 'Currency' and not u64, allowing for different types of currency.
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
            id: InternalId::new(),
            name: "New Event Group".to_string(),
            timestamp: Utc::now(),
            events: Default::default(),
        };
        match self {
            EventGroupTemplate::ExperienceGain { experience } => {
                for character in characters.iter() {
                    let id = InternalId::new();
                    event_group.events.insert(
                        id,
                        Event {
                            id,
                            event_type: EventType::ExperienceGain {
                                experience: *experience,
                            },
                            character: Some(character.clone()),
                        },
                    );
                }
            }
            EventGroupTemplate::CurrencyGain { currency } => {
                for character in characters.iter() {
                    let id = InternalId::new();
                    event_group.events.insert(
                        id,
                        Event {
                            id,
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
            datetime_editing_string: Utc::now().to_string(),
            event_group: template.generate(&characters),
            event_group_template_editing_string: "".to_string(),
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
                campaign
                    .log
                    .event_groups
                    .insert(self.event_group.id, self.event_group.clone());
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
            // TODO: Use existing datetime editor struct
            RestrictedTextEdit::new_from_persistent_string(
                &mut self.datetime,
                &mut self.datetime_editing_string,
            )
            .allow_failure(Some(Color32::RED))
            .ui(ui);
            if ui.button("Now").clicked() {
                self.datetime = Utc::now();
                self.datetime_editing_string = self.datetime.to_string();
            }
        });

        ui.separator();

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label("Fields:");
                let response_updated = EventGroupTemplateDisplayWrapper {
                    event_group_template: &mut self.template,
                    editable_string: &mut self.event_group_template_editing_string,
                }
                .display_fields(ui);
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

            ui.separator();

            ui.vertical(|ui| {
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

                // TODO: This is hacky.
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

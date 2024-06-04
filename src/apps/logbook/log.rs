use crate::{
    models::{
        campaign::Campaign,
        events::{Event, EventGroup, EventLog},
    },
    widgets::hidden_combo_box::HiddenComboBox,
};
use egui::{ComboBox, RichText, Ui};
use itertools::Itertools;

/// Display a list of all events in the log.
pub struct LogDisplay {
    // A clone of the event log, which is displayed and may be modified.
    // It's not saved back to the campaign until the user confirms the changes.
    // TODO: With increasing size of the log, the cloning may become a performance issue (or we may need to implement partitioning)
    modified_log: EventLog,

    //TODO: comment
    ui_context: LogDisplayUiContext,

    // TODO: Is this necessary?
    frozen: bool,
}

#[derive(Default)]
pub struct LogDisplayUiContext {
    pub editing: Option<EditorSelection>,
}

impl LogDisplayUiContext {
    pub fn is_editing(&self, event_group_id: u64, event_id: u64, field: EditingField) -> bool {
        self.editing
            .as_ref()
            .map(|e| {
                e.event_group_id == event_group_id && e.event_id == event_id && e.field == field
            })
            .unwrap_or(false)
    }
}

pub struct EditorSelection {
    pub event_group_id: u64,
    pub event_id: u64,
    pub field: EditingField,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditingField {
    EventType,
    Character,
}

impl LogDisplay {
    pub fn start(log: &EventLog) -> Self {
        LogDisplay {
            modified_log: log.clone(),
            frozen: false,
            ui_context: Default::default(),
        }
    }

    pub fn ui(&mut self, ui: &mut Ui, campaign: &mut Campaign) {
        ui.label("Logbook:");

        ui.horizontal(|ui| {
            ui.checkbox(&mut self.frozen, "Frozen");

            let response = ui.button("Save").on_hover_text("Save changes to the log.");
            if response.clicked() {
                campaign.log = self.modified_log.clone();
            }

            let response = ui
                .button("Reset")
                .on_hover_text("Reset the log to its original state.");
            if response.clicked() {
                self.modified_log = campaign.log.clone();
            }
        });

        // Event display.
        ui.vertical(|ui| {
            for (id, event_group) in self.modified_log.event_groups.iter_mut() {
                let original_event_group = campaign.log.event_groups.get(id);
                self.ui_context.display_event_group(
                    ui,
                    event_group,
                    original_event_group,
                    campaign,
                );
            }
        });
    }
}

impl LogDisplayUiContext {
    // TODO: Reorganize these functions, the &mut makes this tricky.
    // TODO: Should this be pub?
    pub fn display_event_group(
        &mut self,
        ui: &mut Ui,
        event_group: &mut EventGroup,
        original_event_group: Option<&EventGroup>,
        campaign: &Campaign,
    ) {
        ui.collapsing(event_group.name.clone(), |ui: &mut Ui| {
            ui.label(&event_group.timestamp.to_string());
            ui.vertical(|ui| {
                // TODO: Is sorting a hashmap here ideal?
                for (id, event) in &mut event_group.events.iter_mut().sorted_by_key(|(k, _)| *k) {
                    let original_event = original_event_group.and_then(|og| og.events.get(id));
                    self.display_event(ui, event, original_event, campaign);
                }

                // 'Add event' button.
                let add_event = ui.button("Add event");
                if add_event.clicked() {
                    // Favour copying the last event over a default.
                    // TODO: Ugly- uses sorted, doen't need to be sorted.
                    let mut new_event = event_group
                        .events
                        .iter()
                        .sorted_by_key(|e| e.0)
                        .map(|e| e.1)
                        .last()
                        .cloned()
                        .unwrap_or_default();

                    // TODO: Find a better method of id generation rather than incrementing.
                    new_event.id += 1;
                    event_group.events.insert(new_event.id, new_event);
                }
            });
        });
    }

    fn display_event(
        &mut self,
        ui: &mut Ui,
        event: &mut Event,
        original_event: Option<&Event>,
        campaign: &Campaign,
    ) {
        let character = event.character.clone().unwrap_or("None".to_string());
        let collapsing_header_name = format!("{character} - {}", event.event_type);

        let is_modified = original_event.map(|oe| oe != event).unwrap_or(true);
        ui.collapsing(
            {
                RichText::new(collapsing_header_name).color(if is_modified {
                    egui::Color32::GREEN
                } else {
                    egui::Color32::WHITE
                })
            },
            |ui| {
                ui.horizontal(|ui| {
                    // Event type display
                    let is_editing = self.is_editing(event.id, event.id, EditingField::EventType); // TODO: This is wrong.
                    let is_modified = original_event
                        .map(|oe| oe.event_type != event.event_type)
                        .unwrap_or(true);
                    // TODO: When you add ID, it can have some kidn of 'hash with' function to make calling these easier.
                    let dropdown =
                        HiddenComboBox::new(event.id, &mut event.event_type, is_editing, |e| {
                            if e {
                                self.editing = Some(EditorSelection {
                                    event_group_id: event.id, // TODO: This is wrong.
                                    event_id: event.id,
                                    field: EditingField::EventType,
                                });
                            } else if is_editing {
                                self.editing = None;
                            }
                        });
                    if is_modified {
                        ui.add(dropdown.with_rich_text(|rt| rt.color(egui::Color32::GREEN).weak()));
                    } else {
                        ui.add(dropdown);
                    }

                    // Character display
                    let is_modified = original_event
                        .map(|oe| oe.character != event.character)
                        .unwrap_or(true);
                    let character_display = if is_modified {
                        RichText::new(character).color(egui::Color32::GREEN).weak()
                    } else {
                        RichText::new(character)
                    };
                    // TODO: Can we make this a HiddenComboBox with SelectableOption?
                    // TODO: Better hashing here. Search all 'from_id_source' type things
                    ComboBox::from_id_source(event.id + 50)
                        .selected_text(character_display)
                        .show_ui(ui, |ui| {
                            let options = vec!["None".to_string()].into_iter().chain(
                                campaign
                                    .party
                                    .iter()
                                    .map(|character| character.name.clone()),
                            ); // TODO: smell
                            for character in options {
                                if ui
                                    .selectable_label(
                                        character == event.character.as_deref().unwrap_or("None"),
                                        &character,
                                    )
                                    .clicked()
                                {
                                    // TODO: smell
                                    if character == "None" {
                                        event.character = None;
                                    } else {
                                        event.character = Some(character);
                                    }
                                }
                            }
                        });
                });

                // Display fields for the event type.
                event.event_type.display_fields(ui);
            },
        );
    }
}

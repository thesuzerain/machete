use std::collections::HashMap;
use egui::{Ui, Widget};
use crate::{
    apps::logbook::event_group_creator::EventGroupTemplate, utils::SelectableOption, widgets::restricted_text_edit::RestrictedTextEdit
};
use machete::models::{events::EventType, ids::InternalId};

use super::DisplayFields;

pub struct EventTypeDisplayWrapper<'a> {
    pub event_type: &'a mut EventType,
    pub id: InternalId,
    // TODO: not a great datatype
    pub editable_strings: &'a mut HashMap<u64, String>,
}

impl DisplayFields for EventTypeDisplayWrapper<'_> {
    fn display_fields(&mut self, ui: &mut Ui) -> bool {
        let mut updated = false;
        match self.event_type {
            EventType::ExperienceGain { ref mut experience } => {
                ui.horizontal(|ui| {
                    ui.label("Experience:");
                    // TODO: condense this hashmap pattern into a possible implementation for RestrictedText (like how string, &mut string, etc)
                    let editable_string = self
                        .editable_strings
                        .entry(self.id.hash_with("experience"))
                        .or_insert(experience.to_string());
                    let response =
                        RestrictedTextEdit::new_from_persistent_string(experience, editable_string)
                            .ui(ui);
                    if response.changed() {
                        updated = true;
                    }
                });
            }
            EventType::CurrencyGain { ref mut currency } => {
                ui.horizontal(|ui| {
                    ui.label("Currency:");
                    let editable_string = self
                        .editable_strings
                        .entry(self.id.hash_with("currency"))
                        .or_insert(currency.to_string());
                    let response =
                        RestrictedTextEdit::new_from_persistent_string(currency, editable_string)
                            .ui(ui);
                    if response.changed() {
                        updated = true;
                    }
                });
            }
        }
        updated
    }
}

pub struct EventGroupTemplateDisplayWrapper<'a> {
    pub event_group_template: &'a mut EventGroupTemplate,
    pub editable_string: &'a mut String,
}

impl DisplayFields for EventGroupTemplateDisplayWrapper<'_> {
    // TODO: Turn into a trait
    fn display_fields(&mut self, ui: &mut Ui) -> bool {
        let mut updated = false;
        match self.event_group_template {
            EventGroupTemplate::ExperienceGain { ref mut experience } => {
                ui.horizontal(|ui| {
                    ui.label("Experience:");
                    let response = RestrictedTextEdit::new_from_persistent_string(
                        experience,
                        self.editable_string,
                    )
                    .ui(ui);
                    if response.changed() {
                        updated = true;
                    }
                });
            }
            EventGroupTemplate::CurrencyGain { ref mut currency } => {
                ui.horizontal(|ui| {
                    ui.label("Currency:");
                    let response = RestrictedTextEdit::new_from_persistent_string(
                        currency,
                        self.editable_string,
                    )
                    .ui(ui);
                    if response.changed() {
                        updated = true;
                    }
                });
            }
            EventGroupTemplate::None => {}
        }
        updated
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

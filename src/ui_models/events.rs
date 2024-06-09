use egui::{Ui, Widget};

use crate::{
    apps::logbook::event_group_creator::EventGroupTemplate, models::events::EventType,
    widgets::restricted_text_edit::RestrictedTextEdit,
};

use super::DisplayFields;

impl DisplayFields for EventType {
    fn display_fields(&mut self, ui: &mut Ui) -> bool {
        let mut updated = false;
        match self {
            EventType::ExperienceGain { experience } => {
                ui.horizontal(|ui| {
                    ui.label("Experience:");
                    let response = RestrictedTextEdit::new(experience).ui(ui);
                    if response.changed() {
                        updated = true;
                    }
                });
            }
            EventType::CurrencyGain { currency } => {
                ui.horizontal(|ui| {
                    ui.label("Currency:");
                    let response = RestrictedTextEdit::new(currency).ui(ui);
                    if response.changed() {
                        updated = true;
                    }
                });
            }
        }
        updated
    }
}

impl DisplayFields for EventGroupTemplate {
    // TODO: Turn into a trait
    fn display_fields(&mut self, ui: &mut Ui) -> bool {
        let mut updated = false;
        match self {
            EventGroupTemplate::ExperienceGain { experience } => {
                ui.horizontal(|ui| {
                    ui.label("Experience:");
                    let response = RestrictedTextEdit::new(experience).ui(ui);
                    if response.changed() {
                        updated = true;
                    }
                });
            }
            EventGroupTemplate::CurrencyGain { currency } => {
                ui.horizontal(|ui| {
                    ui.label("Currency:");
                    let response = RestrictedTextEdit::new(currency).ui(ui);
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

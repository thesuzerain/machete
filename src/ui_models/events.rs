use egui::Ui;

use crate::{apps::logbook::event_group_creator::EventGroupTemplate, models::events::EventType};

use super::DisplayFields;

impl DisplayFields for EventType {
    fn display_fields(&mut self, ui: &mut Ui) -> bool {
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

impl DisplayFields for EventGroupTemplate {
    // TODO: Turn into a trait
    fn display_fields(&mut self, ui: &mut Ui) -> bool {
        let mut updated = false;
        match self {
            EventGroupTemplate::ExperienceGain { experience } => {
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
            EventGroupTemplate::CurrencyGain { currency } => {
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
            EventGroupTemplate::None => {}
        }
        updated
    }
}

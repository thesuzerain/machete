use egui::Ui;
use machete::database::QueryableStruct;
use machete_core::filters::FilterableStruct;

use crate::ui_models::{filters::DisplayableStruct, DisplayFields};

use super::FilteredLibrary;

/// Display a list of all filters and queryable fields for the library data.
pub struct FilterDisplay {}

impl FilterDisplay {
    pub fn start() -> Self {
        FilterDisplay {}
    }

    // TODO: Are these traits combinable? Should they be?
    // TODO: fix these traits
    // TODO: remove debug
    pub fn ui<T: FilterableStruct + DisplayableStruct + QueryableStruct + std::fmt::Debug>(
        &mut self,
        ui: &mut Ui,
        filters: &mut FilteredLibrary<T>,
    ) {
        ui.label("Filters:");

        let mut remove = None;
        filters.apply_to_filters_mut(|filters| {
            for filter in filters.iter_mut() {
                ui.horizontal(|ui| {
                    filter.display_fields(ui);

                    if ui.button("Remove").clicked() {
                        remove = Some(filter.id);
                    }
                });
            }

            if let Some(id) = remove {
                filters.retain(|filter| filter.id != id);
            }

            if ui.button("Add filter").clicked() {
                filters.push(T::create_default_filter());
            }
        });
    }
}

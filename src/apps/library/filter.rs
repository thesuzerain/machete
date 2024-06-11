use crate::models::library::LibraryItem;
use crate::ui_models::filters::{Filter, FilterableStruct};
use crate::ui_models::DisplayFields;
use egui::Ui;

/// Display a list of all filters and queryable fields for the library data.
pub struct FilterDisplay {}

impl FilterDisplay {
    pub fn start() -> Self {
        FilterDisplay {}
    }

    pub fn ui(&mut self, ui: &mut Ui, filters: &mut Vec<Filter<LibraryItem>>) {
        ui.label("Filters:");

        let mut remove = None;
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
            filters.push(LibraryItem::create_default_filter());
        }
    }
}

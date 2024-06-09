use crate::models::library::LibraryItem;
use crate::ui_models::{DisplayFields, Filter, FilterType};
use egui::Ui;

/// Display a list of all filters and queryable fields for the library data.
pub struct FilterDisplay {
    pub adding_filter: Filter<LibraryItem>,
}

impl FilterDisplay {
    pub fn start() -> Self {
        FilterDisplay {
            // TODO: smell
            adding_filter: Filter::new("name".to_string(), FilterType::Contains("".to_string())),
        }
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
            filters.push(Filter::new(
                "name".to_string(),
                FilterType::Contains("".to_string()),
            ));
        }
    }
}

use egui::Ui;
use itertools::Itertools;
use machete_core::filters::FilterableStruct;

use crate::ui_models::filters::DisplayableStruct;

/// Display a list of all items  in the library.
pub struct LibraryDisplay {}

// TODO: Should this use the same display as the campaign display?
impl LibraryDisplay {
    pub fn start() -> LibraryDisplay {
        LibraryDisplay {}
    }

    pub fn ui<T: FilterableStruct + DisplayableStruct>(
        &mut self,
        ui: &mut Ui,
        filtered_library_items: &[&T],
    ) {
        ui.label("Library:");

        // Define table for display
        let filtered_items = filtered_library_items.iter().copied().collect_vec();
        T::display_table(ui, filtered_items);
    }
}

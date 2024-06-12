use crate::{models::library::LibraryItem, ui_models::filters::FilterableStruct};
use egui::Ui;
use itertools::Itertools;

/// Display a list of all items  in the library.
pub struct LibraryDisplay {}

// TODO: Should this mimic the campaign display? (That one can use tables too for events)
impl LibraryDisplay {
    pub fn start() -> LibraryDisplay {
        LibraryDisplay {}
    }

    pub fn ui(&mut self, ui: &mut Ui, filtered_library_items: &[&LibraryItem]) {
        ui.label("Library:");

        // Define table for display
        // TODO: May be able to make modular and not have two definitions one after the other
        let filtered_items = filtered_library_items.iter().copied().collect_vec();
        LibraryItem::display_table(ui, filtered_items);
    }
}

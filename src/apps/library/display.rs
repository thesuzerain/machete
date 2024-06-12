use crate::models::library::LibraryItem;
use egui::Ui;
use egui_extras::{Column, TableBuilder};
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
        let available_height = ui.available_height();
        let table = TableBuilder::new(ui)
            .striped(true)
            .resizable(false)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            // Name
            .column(Column::auto().at_least(150.0))
            // Game System
            .column(Column::auto().at_least(100.0))
            // Level
            .column(Column::auto().at_least(50.0))
            // Price
            .column(Column::auto().at_least(50.0))
            // Rarity
            .column(Column::auto().at_least(100.0))
            // Tags
            .column(Column::remainder())
            .min_scrolled_height(0.0)
            .max_scroll_height(available_height);

        let filtered_items = filtered_library_items.iter().collect_vec();
        let num_items = filtered_items.len();
        let mut items = filtered_items.iter();

        table
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.strong("Name");
                });
                header.col(|ui| {
                    ui.strong("Game System");
                });
                header.col(|ui| {
                    ui.strong("Level");
                });
                header.col(|ui| {
                    ui.strong("Price");
                });
                header.col(|ui| {
                    ui.strong("Rarity");
                });
                header.col(|ui| {
                    ui.strong("Tags");
                });
            })
            .body(|body| {
                body.rows(32.0, num_items, |mut row| {
                    let item = items.next().unwrap();

                    row.col(|ui| {
                        ui.label(item.name.clone());
                    });
                    row.col(|ui| {
                        ui.label(item.game_system.clone());
                    });
                    row.col(|ui| {
                        ui.label(item.level.to_string());
                    });
                    row.col(|ui| {
                        ui.label(item.price.to_string());
                    });
                    row.col(|ui| {
                        ui.label(item.rarity.clone());
                    });
                    row.col(|ui| {
                        ui.label(item.tags.join(", "));
                    });
                });
            });
    }
}

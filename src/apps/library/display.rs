use crate::{
    models::library::{Library, LibraryItem},
    ui_models::{Filter, FilterableFields},
};
use egui::Ui;
use egui_extras::{Column, TableBuilder};

/// Display a list of all items  in the library.
pub struct LibraryDisplay {}

// TODO: Should this mimic the campaign display? (That one can use tables too for events)
impl LibraryDisplay {
    pub fn start() -> Self {
        LibraryDisplay {}
    }

    pub fn ui(&mut self, ui: &mut Ui, library: &mut Library, filters: &[Filter<LibraryItem>]) {
        ui.label("Library:");

        let available_height = ui.available_height();
        // TODO: column sizes
        let table = TableBuilder::new(ui)
            .striped(true)
            .resizable(false)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::auto().at_least(150.0))
            .column(Column::auto())
            .column(Column::auto())
            .column(Column::auto())
            .column(Column::auto())
            .column(Column::remainder())
            .min_scrolled_height(0.0)
            .max_scroll_height(available_height);

        // TODO: We do NOT want to re-apply filters every frame.
        // This is just a placeholder for now.
        let filtered_items = library
            .items
            .clone()
            .into_iter()
            .filter(|item| filters.iter().all(|filter| item.filter(filter)))
            .collect::<Vec<_>>();

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

        // TODO: Seems like an element after the table is needed to render
        ui.separator();
    }
}

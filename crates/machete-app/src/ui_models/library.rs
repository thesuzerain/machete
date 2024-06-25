use egui::Ui;
use egui_extras::{Column, TableBuilder};
use machete::models::library::{creature::LibraryCreature, item::LibraryItem, spell::LibrarySpell};

use super::filters::DisplayableStruct;

/// Implements 'filters.rs' traits on library structs.
/// TODO: May be in the wrong file.
// TODO: This might be worth making a derive macro for to ensure implementation consistency and auto-updating if the struct changes. A lot of the functions were intentionally designed with this in mind.
impl DisplayableStruct for LibraryItem {
    fn display_table(ui: &mut Ui, filtered_items: Vec<&Self>) {
        let num_items = filtered_items.len();

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
                    let Some(item) = filtered_items.get(row.index()) else {
                        return;
                    };

                    row.col(|ui| {
                        ui.label(item.name.clone());
                    });
                    row.col(|ui| {
                        ui.label(item.game_system.to_string());
                    });
                    row.col(|ui| {
                        ui.label(item.level.to_string());
                    });
                    row.col(|ui| {
                        ui.label(item.price.to_string());
                    });
                    row.col(|ui| {
                        ui.label(item.rarity.clone().to_string());
                    });
                    row.col(|ui| {
                        ui.label(item.tags.join(", "));
                    });
                });
            })
    }
}

impl DisplayableStruct for LibraryCreature {
    fn display_table(ui: &mut Ui, filtered_items: Vec<&Self>) {
        let num_items = filtered_items.len();

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
            // Alignment
            .column(Column::auto().at_least(50.0))
            // Size
            .column(Column::auto().at_least(50.0))
            // Rarity
            .column(Column::auto().at_least(100.0))
            // Tags
            .column(Column::remainder())
            .min_scrolled_height(0.0)
            .max_scroll_height(available_height);

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
                    ui.strong("Rarity");
                });
                header.col(|ui| {
                    ui.strong("Alignment");
                });
                header.col(|ui| {
                    ui.strong("Size");
                });
                header.col(|ui| {
                    ui.strong("Tags");
                });
            })
            .body(|body| {
                body.rows(32.0, num_items, |mut row| {
                    let Some(item) = filtered_items.get(row.index()) else {
                        return;
                    };

                    row.col(|ui| {
                        ui.label(item.name.clone());
                    });
                    row.col(|ui| {
                        ui.label(item.game_system.to_string());
                    });
                    row.col(|ui| {
                        ui.label(item.level.to_string());
                    });
                    row.col(|ui| {
                        ui.label(item.rarity.clone().to_string());
                    });
                    row.col(|ui| {
                        ui.label(item.alignment.clone().to_string());
                    });
                    row.col(|ui| {
                        ui.label(item.size.clone().to_string());
                    });
                    row.col(|ui| {
                        ui.label(item.tags.join(", "));
                    });
                });
            })
    }
}

impl DisplayableStruct for LibrarySpell {
    fn display_table(ui: &mut Ui, filtered_items: Vec<&Self>) {
        let num_items = filtered_items.len();

        let available_height = ui.available_height();
        let table = TableBuilder::new(ui)
            .striped(true)
            .resizable(false)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            // Name
            .column(Column::auto().at_least(150.0))
            // Game System
            .column(Column::auto().at_least(100.0))
            // Rarity
            .column(Column::auto().at_least(100.0))
            // Rank
            .column(Column::auto().at_least(50.0))
            // Tags
            .column(Column::remainder())
            .min_scrolled_height(0.0)
            .max_scroll_height(available_height);

        table
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.strong("Name");
                });
                header.col(|ui| {
                    ui.strong("Game System");
                });
                header.col(|ui| {
                    ui.strong("Rarity");
                });
                header.col(|ui| {
                    ui.strong("Rank");
                });
                header.col(|ui| {
                    ui.strong("Tags");
                });
            })
            .body(|body| {
                body.rows(32.0, num_items, |mut row| {
                    let Some(item) = filtered_items.get(row.index()) else {
                        return;
                    };

                    row.col(|ui| {
                        ui.label(item.name.clone());
                    });
                    row.col(|ui| {
                        ui.label(item.game_system.to_string());
                    });
                    row.col(|ui| {
                        ui.label(item.rarity.clone().to_string());
                    });
                    row.col(|ui| {
                        ui.label(item.rank.to_string());
                    });
                    row.col(|ui| {
                        ui.label(item.tags.join(", "));
                    });
                });
            })
    }
}

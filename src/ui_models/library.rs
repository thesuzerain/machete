use std::collections::HashMap;

use egui::Ui;
use egui_extras::{Column, TableBuilder};

use crate::models::{
    ids::InternalId,
    library::{
        creature::{Alignment, LibraryCreature, Size},
        item::{Currency, LibraryItem},
        Rarity,
    },
};

use super::filters::{FilterableDataType, FilterableStruct};

/// Implements 'filters.rs' traits on library structs.
/// TODO: May be in the wrong file.
// TODO: This might be worth making a derive macro for to ensure implementation consistency and auto-updating if the struct changes. A lot of the functions were intentionally designed with this in mind.
impl FilterableStruct for LibraryItem {
    fn create_default_filter() -> super::filters::Filter<Self> {
        super::filters::Filter {
            id: InternalId::new(),
            field: "name".to_string(),
            filter_type: super::filters::FilterType::String(
                super::filters::StringFilter::Contains("".to_string()),
            ),
            _phantom: std::marker::PhantomData,
        }
    }

    fn iter_fields() -> Vec<&'static str> {
        ["name", "price", "game_system", "rarity", "level", "tags"].to_vec()
    }

    fn get_field_numerics(&self, field: &str) -> Option<Vec<f32>> {
        match field {
            "name" => self.name.as_numerics(),
            "level" => self.level.as_numerics(),
            "price" => self.price.as_numerics(),
            "game_system" => self.game_system.as_numerics(),
            "rarity" => self.rarity.as_numerics(),
            "tags" => self.tags.as_numerics(),
            _ => None,
        }
    }

    fn get_field_strings(&self, field: &str) -> Option<Vec<String>> {
        // TODO: clones, smell
        match field {
            "name" => self.name.as_strings(),
            "level" => self.level.as_strings(),
            "price" => self.price.as_strings(),
            "game_system" => self.game_system.as_strings(),
            "rarity" => self.rarity.as_strings(),
            "tags" => self.tags.as_strings(),
            _ => None,
        }
    }

    fn is_field_numeric(field: &str) -> bool {
        match field {
            "name" => String::is_numeric(),
            "level" => u8::is_numeric(),
            "price" => Currency::is_numeric(),
            "game_system" => String::is_numeric(),
            "rarity" => Rarity::is_numeric(),
            "tags" => Vec::<String>::is_numeric(),
            _ => false,
        }
    }

    fn is_field_string(field: &str) -> bool {
        match field {
            "name" => String::is_string(),
            "level" => u8::is_string(),
            "price" => Currency::is_string(),
            "game_system" => String::is_string(),
            "rarity" => Rarity::is_string(),
            "tags" => Vec::<String>::is_string(),
            _ => false,
        }
    }

    fn items(library: &crate::models::library::Library) -> &HashMap<InternalId, LibraryItem> {
        &library.items
    }

    fn display_table(ui: &mut Ui, filtered_items: Vec<&Self>) {
        let num_items = filtered_items.len();
        let mut items = filtered_items.iter();

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
                        ui.label(item.rarity.clone().to_string());
                    });
                    row.col(|ui| {
                        ui.label(item.tags.join(", "));
                    });
                });
            })
    }
}

impl FilterableStruct for LibraryCreature {
    fn create_default_filter() -> super::filters::Filter<Self> {
        super::filters::Filter {
            id: InternalId::new(),
            field: "name".to_string(),
            filter_type: super::filters::FilterType::String(
                super::filters::StringFilter::Contains("".to_string()),
            ),
            _phantom: std::marker::PhantomData,
        }
    }

    fn iter_fields() -> Vec<&'static str> {
        [
            "name",
            "game_system",
            "rarity",
            "level",
            "tags",
            "size",
            "alignment",
        ]
        .to_vec()
    }

    fn get_field_numerics(&self, field: &str) -> Option<Vec<f32>> {
        match field {
            "name" => self.name.as_numerics(),
            "level" => self.level.as_numerics(),
            "game_system" => self.game_system.as_numerics(),
            "rarity" => self.rarity.as_numerics(),
            "tags" => self.tags.as_numerics(),
            "size" => self.size.as_numerics(),
            "alignment" => self.alignment.as_numerics(),
            _ => None,
        }
    }

    fn get_field_strings(&self, field: &str) -> Option<Vec<String>> {
        // TODO: clones, smell
        match field {
            "name" => self.name.as_strings(),
            "level" => self.level.as_strings(),
            "game_system" => self.game_system.as_strings(),
            "rarity" => self.rarity.as_strings(),
            "tags" => self.tags.as_strings(),
            "size" => self.size.as_strings(),
            "alignment" => self.alignment.as_strings(),
            _ => None,
        }
    }

    fn is_field_numeric(field: &str) -> bool {
        match field {
            "name" => String::is_numeric(),
            "level" => u8::is_numeric(),
            "game_system" => String::is_numeric(),
            "rarity" => Rarity::is_numeric(),
            "tags" => Vec::<String>::is_numeric(),
            "size" => Size::is_numeric(),
            "alignment" => Alignment::is_numeric(),
            _ => false,
        }
    }

    fn is_field_string(field: &str) -> bool {
        match field {
            "name" => String::is_string(),
            "level" => u8::is_string(),
            "price" => Currency::is_string(),
            "game_system" => String::is_string(),
            "rarity" => Rarity::is_string(),
            "tags" => Vec::<String>::is_string(),
            "size" => Size::is_string(),
            "alignment" => Alignment::is_string(),
            _ => false,
        }
    }

    fn items(library: &crate::models::library::Library) -> &HashMap<InternalId, LibraryCreature> {
        &library.creatures
    }

    fn display_table(ui: &mut Ui, filtered_items: Vec<&Self>) {
        let num_items = filtered_items.len();
        let mut items = filtered_items.iter();

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

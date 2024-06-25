use std::hash::{DefaultHasher, Hash, Hasher};

use crate::{app::StateContext, update_context::UpdateWithContext};
use display::LibraryDisplay;
use filter::FilterDisplay;
use itertools::Itertools;
use machete::{
    database::QueryableStruct,
    models::library::{creature::LibraryCreature, item::LibraryItem, spell::LibrarySpell},
};
use machete_core::filters::{Filter, FilterableStruct};

pub mod display;
pub mod filter;

/// Library application, for viewing and managing the user's library of creatures, items, etc.
/// This is for non-campaign-specific data.
pub struct LibraryApp {
    pub filters_display: FilterDisplay,
    pub viewer: LibraryDisplay,

    pub collection_showing: LibraryCollectionType,

    pub filtered_library_items: FilteredLibrary<LibraryItem>,
    pub filtered_library_creatures: FilteredLibrary<LibraryCreature>,
    pub filtered_library_spells: FilteredLibrary<LibrarySpell>,
}

impl LibraryApp {
    pub fn start() -> LibraryApp {
        LibraryApp {
            filters_display: FilterDisplay::start(),
            viewer: LibraryDisplay::start(),

            collection_showing: LibraryCollectionType::Items,

            filtered_library_items: FilteredLibrary::new(),
            filtered_library_creatures: FilteredLibrary::new(),
            filtered_library_spells: FilteredLibrary::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LibraryCollectionType {
    Items,
    Creatures,
    Spells,
}

impl UpdateWithContext for LibraryApp {
    fn update(
        &mut self,
        ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        _context: &mut StateContext,
    ) {
        // TODO: remove  clones here- beter system for this
        let filtered_items = self.filtered_library_items.items().clone();
        let filtered_creatures = self.filtered_library_creatures.items().clone();
        let filtered_spells = self.filtered_library_spells.items().clone();

        egui::TopBottomPanel::top("Collection").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Items").clicked() {
                    self.collection_showing = LibraryCollectionType::Items;
                }
                if ui.button("Creatures").clicked() {
                    self.collection_showing = LibraryCollectionType::Creatures;
                }
                if ui.button("Spells").clicked() {
                    self.collection_showing = LibraryCollectionType::Spells;
                }
            });
        });

        egui::TopBottomPanel::top("Filters").show(ctx, |ui| match self.collection_showing {
            LibraryCollectionType::Items => self
                .filters_display
                .ui(ui, &mut self.filtered_library_items),
            LibraryCollectionType::Creatures => self
                .filters_display
                .ui(ui, &mut self.filtered_library_creatures),
            LibraryCollectionType::Spells => self
                .filters_display
                .ui(ui, &mut self.filtered_library_spells),
        });

        egui::CentralPanel::default().show(ctx, |ui| match self.collection_showing {
            LibraryCollectionType::Items => {
                self.viewer.ui(ui, &filtered_items.iter().collect_vec())
            }
            LibraryCollectionType::Creatures => {
                self.viewer.ui(ui, &filtered_creatures.iter().collect_vec())
            }
            LibraryCollectionType::Spells => {
                self.viewer.ui(ui, &filtered_spells.iter().collect_vec())
            }
        });
    }
}

/// A struct for maintaining persistence of filters and filtered items.
/// This is kept as a separate struct to allow auto-updating of the filtered items when filters are modified.
/// This assumes that the library's id-to-item mapping will not change.
// TODO: Simplify traits
// TODO: remove deubg
pub struct FilteredLibrary<T: FilterableStruct + QueryableStruct + std::fmt::Debug> {
    // TODO: Is there a better pattern for this?
    filters: Vec<Filter<T>>,
    // TODO: is there a better way to remember filter updates?
    filters_hash: u64,
    values: Vec<T>,
}

impl<T: FilterableStruct + QueryableStruct + std::fmt::Debug> Default for FilteredLibrary<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: FilterableStruct + QueryableStruct + std::fmt::Debug> FilteredLibrary<T> {
    pub fn new() -> Self {
        let default_filters = vec![T::create_default_filter()];
        let default_filters_hash = {
            let mut hasher = DefaultHasher::new();
            default_filters.hash(&mut hasher);
            hasher.finish()
        };
        FilteredLibrary {
            filters: vec![T::create_default_filter()],
            filters_hash: default_filters_hash,
            // TODO: initialize with data
            values: vec![],
        }
    }

    /// Get the list of all items filtered from the library.
    /// TODO: Rename to 'ids'?
    pub fn items(&self) -> &Vec<T> {
        // TODO: Is there a faster way to do this?
        &self.values
    }

    /// Apply a closure to the filters, then recalculate the filtered table.
    pub fn apply_to_filters_mut(&mut self, closure: impl FnOnce(&mut Vec<Filter<T>>)) {
        closure(&mut self.filters);
        self.recalculate_filtered_table();
    }

    /// Recalculate the filtered table based on the current filters.
    fn recalculate_filtered_table(&mut self) {
        // Hash the filters to see if they have changed.
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.filters.hash(&mut hasher);
        let new_hash = hasher.finish();
        if new_hash == self.filters_hash {
            return;
        }
        self.filters_hash = new_hash;

        // TODO: get this connection elsewhere and reuse it
        // TODO better tokio async- not a blocking thread
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async {
            let pool = machete::database::connect().await.unwrap();
            self.values = T::query_get(&pool, &self.filters).await.unwrap();
        });
    }
}

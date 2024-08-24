use std::{
    hash::{DefaultHasher, Hash, Hasher},
    sync::Arc,
};

use crate::{app::StateContext, fetch::FetchableStruct, update_context::UpdateWithContext};
use display::LibraryDisplay;
use egui::mutex::RwLock;
use filter::FilterDisplay;
use itertools::Itertools;
use machete::models::library::{creature::LibraryCreature, item::LibraryItem, spell::LibrarySpell};
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
    pub fn start() -> Self {
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
        // TODO: Remove clones here- poor system. May be better to use tokio async RwLock or similar.
        let filtered_items = self.filtered_library_items.values.read().clone();
        let filtered_creatures = self.filtered_library_creatures.values.read().clone();
        let filtered_spells = self.filtered_library_spells.values.read().clone();

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
// TODO: These traits are combinable/simplifiable
// TODO: Remove debug
pub struct FilteredLibrary<
    T: FilterableStruct + FetchableStruct + std::fmt::Debug + Send + std::marker::Sync + 'static,
> {
    // TODO: Is there a better pattern for this?
    filters: Vec<Filter<T>>,
    filters_hash: Arc<u64>,
    // TODO: It might be better for this to be a tokio RwLock rather than an egui one to use async/await
    values: Arc<RwLock<Vec<T>>>,
}

impl<
        T: FilterableStruct + FetchableStruct + std::fmt::Debug + Send + std::marker::Sync + 'static,
    > Default for FilteredLibrary<T>
{
    fn default() -> Self {
        Self::new()
    }
}

// TODO: These traits are combinable/simplifiable
impl<
        T: FilterableStruct + FetchableStruct + std::fmt::Debug + Send + std::marker::Sync + 'static,
    > FilteredLibrary<T>
{
    pub fn new() -> Self {
        let default_filters = vec![T::create_default_filter()];
        let default_filters_hash = {
            let mut hasher = DefaultHasher::new();
            default_filters.hash(&mut hasher);
            hasher.finish()
        };
        FilteredLibrary {
            filters: vec![T::create_default_filter()],
            filters_hash: Arc::new(default_filters_hash),
            // TODO: initialize with data
            values: Arc::new(RwLock::new(vec![])),
        }
    }

    /// Apply a closure to the filters, then recalculate the filtered table.
    pub fn apply_to_filters_mut(
        &mut self,
        closure: impl FnOnce(&mut Vec<Filter<T>>),
        ctx: egui::Context,
    ) {
        closure(&mut self.filters);
        self.recalculate_filtered_table(ctx);
    }

    /// Recalculate the filtered table based on the current filters.
    /// Spawns a new thread to query the database and update the values.
    /// If multiple calls are made to this function before the query is complete, only the one with the 'latest' filters will be used.
    /// (By checking the hash of the filters.)
    fn recalculate_filtered_table(&mut self, ctx: egui::Context) {
        // Hash the filters to see if they have changed.
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.filters.hash(&mut hasher);
        let new_hash = hasher.finish();
        if new_hash == *self.filters_hash {
            return;
        }
        self.filters_hash = Arc::new(new_hash);

        let values_clone = self.values.clone();
        let filters_hash_clone = self.filters_hash.clone();
        let filters_clone = self.filters.clone(); // TODO: We can avoid clone here. Maybe just move?
        let fut = async move {
            let filters_clone = filters_clone.clone();
            let values = T::fetch_backend(&filters_clone).await.unwrap();

            let mut values_clone = values_clone.write();

            // Only write to the values if the filters have not changed since the query was made.
            if *filters_hash_clone == new_hash {
                values_clone.clone_from(&values);
                ctx.request_repaint();
            }
        };

        #[cfg(feature = "offline")]
        {
            tokio::spawn(fut);
        }
        #[cfg(feature = "web_app")]
        {
            wasm_bindgen_futures::spawn_local(fut);
        }
    }
}

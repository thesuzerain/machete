use crate::{
    app::StateContext,
    ui_models::filters::{Filter, FilterableStruct},
    update_context::UpdateWithContext,
};
use machete::models::{
    ids::InternalId,
    library::{creature::LibraryCreature, item::LibraryItem, spell::LibrarySpell, Library},
};
use display::LibraryDisplay;
use filter::FilterDisplay;
use itertools::Itertools;

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
    pub fn start(library: &Library) -> LibraryApp {
        LibraryApp {
            filters_display: FilterDisplay::start(),
            viewer: LibraryDisplay::start(),

            collection_showing: LibraryCollectionType::Items,

            filtered_library_items: FilteredLibrary::new(library),
            filtered_library_creatures: FilteredLibrary::new(library),
            filtered_library_spells: FilteredLibrary::new(library),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LibraryCollectionType {
    Items,
    Creatures,
    Spells
}

impl UpdateWithContext for LibraryApp {
    fn update(
        &mut self,
        ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        context: &mut StateContext,
    ) {
        let filtered_items = self.filtered_library_items.items(&context.library);
        let filtered_creatures = self.filtered_library_creatures.items(&context.library);
        let filtered_spells = self.filtered_library_spells.items(&context.library);

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
            LibraryCollectionType::Items => {
                self.filters_display
                    .ui(ui, &mut self.filtered_library_items, &context.library)
            }
            LibraryCollectionType::Creatures => {
                self.filters_display
                    .ui(ui, &mut self.filtered_library_creatures, &context.library)
            },
            LibraryCollectionType::Spells => {
                self.filters_display
                    .ui(ui, &mut self.filtered_library_spells, &context.library)
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| match self.collection_showing {
            LibraryCollectionType::Items => self.viewer.ui(ui, &filtered_items),
            LibraryCollectionType::Creatures => self.viewer.ui(ui, &filtered_creatures),
            LibraryCollectionType::Spells => self.viewer.ui(ui, &filtered_spells),
        });
    }
}

/// A struct for maintaining persistence of filters and filtered items.
/// This is kept as a separate struct to allow auto-updating of the filtered items when filters are modified.
/// This assumes that the library's id-to-item mapping will not change.
pub struct FilteredLibrary<T: FilterableStruct> {
    // TODO: Is there a better pattern for this?
    filters: Vec<Filter<T>>,
    ids: Vec<InternalId>,
}

impl<T: FilterableStruct> FilteredLibrary<T> {
    pub fn new(library: &Library) -> Self {
        FilteredLibrary {
            filters: vec![T::create_default_filter()],
            ids: library.items.keys().copied().collect(),
        }
    }

    /// Get the list of all items filtered from the library.
    /// TODO: Rename to 'ids'?
    pub fn items<'a>(&self, library: &'a Library) -> Vec<&'a T> {
        // TODO: Is there a faster way to do this?
        self.ids
            .iter()
            .filter_map(|id| T::items(library).get(id))
            .collect_vec()
    }

    /// Apply a closure to the filters, then recalculate the filtered table.
    pub fn apply_to_filters_mut(
        &mut self,
        closure: impl FnOnce(&mut Vec<Filter<T>>),
        library: &Library,
    ) {
        closure(&mut self.filters);
        self.recalculate_filtered_table(library);
    }

    /// Recalculate the filtered table based on the current filters.
    fn recalculate_filtered_table(&mut self, library: &Library) {
        let items = T::items(library)
            .iter()
            .filter(|(_, item)| {
                self.filters
                    .iter()
                    .all(|filter| filter.apply_filter(item).unwrap())
            })
            .map(|(k, _)| *k)
            .collect();
        self.ids = items;
    }
}

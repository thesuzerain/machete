use crate::{
    app::StateContext,
    models::{
        ids::InternalId,
        library::{Library, LibraryItem},
    },
    ui_models::filters::Filter,
    update_context::UpdateWithContext,
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

    pub filtered_library: FilteredLibrary,
}

impl LibraryApp {
    pub fn start(library: &Library) -> LibraryApp {
        LibraryApp {
            filters_display: FilterDisplay::start(),
            viewer: LibraryDisplay::start(),

            filtered_library: FilteredLibrary::new(library),
        }
    }
}

impl UpdateWithContext for LibraryApp {
    fn update(
        &mut self,
        ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        context: &mut StateContext,
    ) {
        let filtered_items = self.filtered_library.items(&context.library);
        egui::TopBottomPanel::top("Filters").show(ctx, |ui| {
            self.filters_display
                .ui(ui, &mut self.filtered_library, &context.library);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.viewer.ui(ui, &filtered_items);
        });
    }
}

/// A struct for maintaining persistence of filters and filtered items.
/// This is kept as a separate struct to allow auto-updating of the filtered items when filters are modified.
/// This assumes that the library's id-to-item mapping will not change.
pub struct FilteredLibrary {
    filters: Vec<Filter<LibraryItem>>,
    items: Vec<InternalId>,
}

impl FilteredLibrary {
    pub fn new(library: &Library) -> FilteredLibrary {
        FilteredLibrary {
            filters: Vec::new(),
            items: library.items.keys().copied().collect(),
        }
    }

    /// Get the list of all items filtered from the library.
    pub fn items<'a>(&self, library: &'a Library) -> Vec<&'a LibraryItem> {
        // TODO: Is there a faster way to do this?
        self.items
            .iter()
            .filter_map(|id| library.items.get(id))
            .collect_vec()
    }

    /// Apply a closure to the filters, then recalculate the filtered table.
    pub fn apply_to_filters_mut(
        &mut self,
        closure: impl FnOnce(&mut Vec<Filter<LibraryItem>>),
        library: &Library,
    ) {
        closure(&mut self.filters);
        self.recalculate_filtered_table(library);
    }

    /// Recalculate the filtered table based on the current filters.
    fn recalculate_filtered_table(&mut self, library: &Library) {
        let items = library
            .items
            .iter()
            .filter(|(_, item)| {
                self.filters
                    .iter()
                    .all(|filter| filter.apply_filter(item).unwrap())
            })
            .map(|(k, _)| *k)
            .collect();
        self.items = items;
    }
}

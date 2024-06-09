use crate::{
    app::StateContext, models::library::LibraryItem, ui_models::Filter,
    update_context::UpdateWithContext,
};
use display::LibraryDisplay;
use filter::FilterDisplay;

pub mod display;
pub mod filter;

/// Library application, for viewing and managing the user's library of creatures, items, etc.
/// This is for non-campaign-specific data.
pub struct LibraryApp {
    pub filters_display: FilterDisplay,
    pub viewer: LibraryDisplay,

    pub filters: Vec<Filter<LibraryItem>>,
}

impl LibraryApp {
    pub fn start() -> Self {
        LibraryApp {
            filters_display: FilterDisplay::start(),
            viewer: LibraryDisplay::start(),
            filters: vec![],
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
        egui::TopBottomPanel::top("Filters").show(ctx, |ui| {
            self.filters_display.ui(ui, &mut self.filters);
        });

        egui::TopBottomPanel::top("Library").show(ctx, |ui| {
            self.viewer.ui(ui, &mut context.library, &self.filters);
        });

        egui::CentralPanel::default()
            .frame(egui::Frame::dark_canvas(&ctx.style()))
            .show(ctx, |_ui| {
                // TODO: More panels in the summary app.
            });
    }
}

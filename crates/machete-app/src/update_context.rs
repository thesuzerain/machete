use crate::app::StateContext;

/// A trait for any struct that can be updated (like egui::App) with the state context.
pub trait UpdateWithContext {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame, state: &mut StateContext);
}

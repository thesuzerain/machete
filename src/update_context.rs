use crate::models::campaign::Campaign;

/// a trait for any struct that can be updated (like egui::App) with the campaign context.
pub trait UpdateWithContext {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame, campaign: &mut Campaign);
}

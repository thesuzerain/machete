use egui::Ui;

// ui_models: UI relevant traits/implementations for models
// TODO: These are kept separate for now, in anticipation of it helping with the UI code organization as we add crates,
// but it may be better to move these into the models structure.

pub mod events;
pub mod filters;
pub mod library;

/// A trait representing an object representable in the UI with multiple fields.
/// To allow for editing these fields, implement the DisplayFields trait.
pub trait DisplayFields {
    /// Display the fields of the object in the UI.
    /// Returns true if any of the fields were updated.
    fn display_fields(&mut self, ui: &mut Ui) -> bool;
}

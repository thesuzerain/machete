use egui::{ComboBox, Ui};
use machete_core::filters::{Filter, FilterType, FilterableStruct};

use super::DisplayFields;

// TODO: This will need to be dynamic or changed. This is just a placeholder.
const MAX_SLIDER: f32 = 1000.0;

/// A struct that can be displayed as a table in egui.
/// (A list of said struct)
// TODO: This might be worth making a derive macro for to ensure implementation consistency and auto-updating if the struct changes. A lot of the functions were intentionally designed with this in mind.
pub trait DisplayableStruct
where
    Self: Sized + Clone + PartialEq,
{
    /// Display a table of items in egui.
    fn display_table(ui: &mut Ui, filtered_items: Vec<&Self>);
}

impl<F: FilterableStruct> DisplayFields for Filter<F> {
    fn display_fields(&mut self, ui: &mut egui::Ui) -> bool {
        let id = self.id;

        let mut field_editable = self.field.clone();
        ComboBox::from_id_source(id.hash_with("display_filterable_fields"))
            .selected_text(field_editable.to_string())
            .show_ui(ui, |ui| {
                for field in F::iter_fields() {
                    ui.selectable_value(&mut field_editable, field.to_string(), field.to_string());
                }
            });

        if self.field != field_editable {
            // TODO: unneeded- can change directly in above
            self.field = field_editable;
            // If changed, we need to update the filter type to match the default for the new field.
            self.filter_type = F::iter_filter_types_for_field(&self.field)
                .unwrap()
                .first()
                .unwrap()
                .clone(); // TODO: unwrap
        }

        let mut edited: bool = false;

        ComboBox::from_id_source(self.id.hash_with("display_filter_type"))
            .selected_text(self.filter_type.as_str())
            .show_ui(ui, |ui| {
                for filter in F::iter_filter_types_for_field(&self.field).unwrap() {
                    // TODO: unwrap
                    ui.selectable_value(&mut self.filter_type, filter.clone(), filter.as_str());
                }
            });

        // TODO: doing two branches for get_field_string and get_field_numeric is a smell
        match self.filter_type {
            FilterType::Contains(ref mut string) => {
                edited |= ui.text_edit_singleline(string).changed();
            }
            FilterType::GreaterThan(ref mut f) => {
                edited |= ui.add(egui::Slider::new(f, 0.0..=MAX_SLIDER)).changed();
            }
            FilterType::LessThan(ref mut f) => {
                edited |= ui.add(egui::Slider::new(f, 0.0..=MAX_SLIDER)).changed();
            }
            FilterType::EqualToNumber(ref mut f) => {
                edited |= ui.add(egui::Slider::new(f, 0.0..=MAX_SLIDER)).changed();
            }
            FilterType::EqualToChoice(ref mut string) => {
                // todo: not it
                let choice =
                    ComboBox::from_id_source(self.id.hash_with("display_filter_choice_variant"))
                        .selected_text(string.to_string())
                        .show_ui(ui, |ui| {
                            for choice in F::iter_filter_variants_for_field(&self.field).unwrap() {
                                // TODO: unwrap
                                ui.selectable_value(string, choice.to_string(), choice.to_string());
                            }
                        });
                edited |= choice.response.changed();
            }
        }

        edited
    }
}

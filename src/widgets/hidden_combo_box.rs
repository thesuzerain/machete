use std::hash::Hash;

use egui::{Response, RichText, Ui, Widget};

use crate::utils::SelectableOption;

pub struct HiddenComboBox<'a, F: FnOnce(bool), T: SelectableOption, Id: Hash> {
    id: Id,
    selected: &'a mut T,

    is_active: bool,
    set_bool: F,

    modify_rich_text: Option<fn(RichText) -> RichText>,
}

impl<'a, F: FnOnce(bool), T: SelectableOption, Id: Hash> HiddenComboBox<'a, F, T, Id> {
    pub fn new(id: Id, selected: &'a mut T, is_active: bool, set_bool: F) -> Self {
        Self {
            id,
            is_active,
            selected,
            set_bool,
            modify_rich_text: None,
        }
    }

    pub fn with_rich_text(self, modify_rich_text: fn(RichText) -> RichText) -> Self {
        Self {
            modify_rich_text: Some(modify_rich_text),
            ..self
        }
    }
}

impl<F: FnOnce(bool), T: SelectableOption, Id: Hash> Widget for HiddenComboBox<'_, F, T, Id> {
    fn ui(self, ui: &mut Ui) -> Response {
        let (response, on_option) = if self.is_active {
            let frame = egui::ComboBox::from_id_source(self.id)
                .selected_text(self.selected.as_selectable_str())
                .show_ui(ui, |ui| {
                    for option in T::iter_options() {
                        let option_str = option.as_selectable_str();
                        ui.selectable_value(self.selected, option, option_str);
                    }
                });
            (frame.response, frame.inner.is_some())
        } else {
            // TODO: Customizable label
            let mut rich_text = RichText::new(self.selected.as_selectable_str());
            if let Some(modify_rich_text) = self.modify_rich_text {
                rich_text = modify_rich_text(rich_text);
            }
            let label = egui::Label::new(rich_text);
            (ui.add(label), false)
        };

        let rect = response.rect;
        if ui.rect_contains_pointer(rect) || on_option {
            (self.set_bool)(true);
        } else {
            (self.set_bool)(false);
        }

        response
    }
}

use egui::{Response, TextEdit, Ui, Widget};
use std::str::FromStr;

/// TextEdit field that allows a non-string type to be edited (while functioning as a string editor).
/// For example, typing in an arbitrary number into a text field and parsing it into a number type.
pub struct RestrictedTextEdit<'a, S: ToString + FromStr> {
    val: &'a mut S,
    val_string: String,

    failure_string: Option<&'a mut String>,
    failure_color: Option<egui::Color32>,
}

impl<'a, S: ToString + FromStr> RestrictedTextEdit<'a, S> {
    pub fn new(text: &'a mut S) -> Self {
        let val_string = text.to_string();

        Self {
            val: text,
            val_string,
            failure_string: None,
            failure_color: None,
        }
    }

    /// Set the text and color when the value is not parseable.
    /// If if not set, when invalid text is entered, the changes will be reverted immediately.
    /// This allows for 'intermediate' states where the text is invalid, but the user can still see what they typed, and correct or continue typing.
    ///
    /// 'failure_string' is used to store partial values, and is modified internally by the widget during parsing (should not be used externally).
    /// It should be set to the same initial value as the value being edited (.to_string()).
    pub fn allow_failure(mut self, failure_string: &'a mut String, color: egui::Color32) -> Self {
        self.failure_string = Some(failure_string);
        self.failure_color = Some(color);
        self
    }
}

impl<S: ToString + FromStr> Widget for RestrictedTextEdit<'_, S> {
    fn ui(mut self, ui: &mut Ui) -> Response {
        let display_text = match self.failure_string {
            Some(f) => f,
            None => &mut self.val_string,
        };
        let is_parsable = display_text.parse::<S>().is_ok();

        let response = {
            let mut text_edit = TextEdit::singleline(display_text);

            // If current text is unparseable, color it to indicate failure to parse.
            if let Some(color) = self.failure_color {
                if !is_parsable {
                    text_edit = text_edit.text_color(color);
                }
            }

            text_edit.ui(ui)
        };

        if response.changed() {
            if let Ok(e) = display_text.parse() {
                // Only update the 'real' value if the text is parseable.
                *self.val = e;
            }
        }
        response
    }
}

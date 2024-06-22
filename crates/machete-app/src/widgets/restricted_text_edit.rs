use egui::{Response, TextEdit, Ui, Widget};
use std::str::FromStr;

/// TextEdit field that allows a non-string type to be edited (while functioning as a string editor).
/// For example, typing in an arbitrary number into a text field and parsing it into a number type.
pub struct RestrictedTextEdit<'a, S: ToString + FromStr> {
    /// The last valid value of the type being edited.
    /// This corresponds to the variable that will be updated when the text is successfully parsed.
    valid_value: &'a mut S,

    /// The string representation of the value being edited.
    /// This only needs to be set if we are allowing intermediate states where the text is not parseable.
    /// (e.g. in the progress of writing a DateTime, the text may be '20', which is not yet a valid DateTime).
    editable_string: RestrictedTextType<'a>,

    /// If true, the text will not revert to the last valid value when the text is not parseable.
    /// For allow_failure to be true, the text type must be 'MutableReference' (a persistent string must exist outside the widget).
    allow_failure: bool,
    /// The color to use when the text is not parseable. If None, the text will not be colored.
    failure_color: Option<egui::Color32>,
}

/// The storage structure for the value being edited.
enum RestrictedTextType<'a> {
    /// The text string is owned by the widget.
    /// If this is being used, the text cannot have an intermediate non-parseable state.
    Owned { s: String },
    /// The text string is a mutable reference to a string outside the widget.
    MutableReference { s: &'a mut String },
}

impl RestrictedTextType<'_> {
    /// Get the text string as a reference.
    pub fn get_text_mut(&mut self) -> &mut String {
        match self {
            RestrictedTextType::Owned { ref mut s } => s,
            RestrictedTextType::MutableReference { ref mut s } => s,
        }
    }
}

impl<'a, S: ToString + FromStr> RestrictedTextEdit<'a, S> {
    /// Create a new RestrictedTextEdit widget where the text is owned by the widget.
    /// The 'valid_value' is the value that will be updated when the text is successfully parsed.
    /// If the value is not parseable, the text will immediately revert to the last valid value.
    /// You cannot `allow_failure` with this constructor, as the text is not persistent outside the widget.
    pub fn new(valid_value: &'a mut S) -> Self {
        let editable_string = RestrictedTextType::Owned {
            s: valid_value.to_string(),
        };
        Self {
            valid_value,
            editable_string,
            allow_failure: false,
            failure_color: None,
        }
    }

    /// Create a new RestrictedTextEdit widget with a persistent string.
    /// The 'valid_value' is the value that will be updated when the text is successfully parsed.
    /// The 'string' is a mutable reference to a string that will be used to store the text.
    /// This allows for intermediate states where the text is not parseable.
    pub fn new_from_persistent_string(valid_value: &'a mut S, string: &'a mut String) -> Self {
        Self {
            valid_value,
            editable_string: RestrictedTextType::MutableReference { s: string },
            allow_failure: false,
            failure_color: None,
        }
    }

    /// Set the text and color when the value is not parseable.
    /// If this not set, when invalid text is entered, the changes will be reverted immediately.
    /// This allows for 'intermediate' states where the text is invalid, but the user can still see what they typed, and correct or continue typing.
    ///
    /// RestrictedText 'editable_string' is used to store partial values, and is modified internally by the widget during parsing (should not be used externally).
    /// 'color' is the color to use when the text is invalid- if None, the unparsable text will not have a changed color.
    ///
    /// PANICS: `self` must be created with `new_from_persistent_string` to use this function.
    pub fn allow_failure(mut self, color: Option<egui::Color32>) -> Self {
        self.allow_failure = true;
        self.failure_color = color;

        // Forbid allow_failure with Owned text, as it doesn't make sense to allow failure with a text that doesn't persist outside the widget.
        assert!(
            matches!(
                self.editable_string,
                RestrictedTextType::MutableReference { .. }
            ),
            "allow_failure can only be used with `new_from_persistent_string`"
        );
        self
    }
}

impl<S: ToString + FromStr> Widget for RestrictedTextEdit<'_, S> {
    fn ui(mut self, ui: &mut Ui) -> Response {
        let editable_string = self.editable_string.get_text_mut();
        let is_parsable = editable_string.parse::<S>().is_ok();

        let response = {
            let mut text_edit = TextEdit::singleline(editable_string);

            // If current text is unparseable, color it to indicate failure to parse.
            if let Some(color) = self.failure_color {
                if !is_parsable {
                    text_edit = text_edit.text_color(color);
                }
            }

            text_edit.ui(ui)
        };

        if response.changed() {
            if let Ok(e) = editable_string.parse::<S>() {
                // Only update the 'real' value if the text is parseable.
                *self.valid_value = e;
            }

            // If the text is not parseable, and we're not allowing failure, revert the text to the last valid value.
            if !is_parsable && !self.allow_failure {
                *editable_string = self.valid_value.to_string();
            }
        }
        response
    }
}

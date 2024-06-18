use egui::{ComboBox, Ui};
use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
};

use crate::models::{
    ids::InternalId,
    library::{
        creature::{Alignment, Size},
        item::Currency,
        Library, Rarity,
    },
};

use super::DisplayFields;

// TODO: This will need to be dynamic or changed. This is just a placeholder.
const MAX_SLIDER: f32 = 1000.0;

/// A filter over structure fields.
/// For example: "Name contains 'Bob'" or "Level is greater than 5".
#[derive(Debug)]
pub struct Filter<F: FilterableStruct> {
    /// Unique identifier for this filter.
    pub id: InternalId,
    /// The field to filter on by name.
    /// TODO: This is a code smell to use a String here (and the FilterableStruct `is_numeric` etc.) and is worth coming back to.
    pub field: String,
    /// The nature of the filter: "less than 5", for instance.
    /// This must match the type of the field in 'field'.
    // TODO: This same code smell.
    pub filter_type: FilterType,

    pub _phantom: std::marker::PhantomData<F>,
}

/// A struct that Filter can be applied to.
// TODO: This might be worth making a derive macro for to ensure implementation consistency and auto-updating if the struct changes. A lot of the functions were intentionally designed with this in mind.
pub trait FilterableStruct
where
    Self: Sized + Clone + PartialEq,
{
    /// Creates a default filter for this struct.
    /// The Filter must be consistent with the FilterType and fields of the struct.
    /// This is used to create a new filter when the user requests one (by switching to this struct, for instance)
    fn create_default_filter() -> Filter<Self>;

    /// Iterate over the fields that can be filtered on.
    /// These are returned as strings for display purposes.
    fn iter_fields() -> Vec<&'static str>;

    /// Get the values of a field as strings.
    /// This returns a Vec of strings for the field, even if there is only one value (to allow for consistency in the return type).
    /// This allows for `get_field_strings` to be used on String and Vec<String> fields (etc.), allowing for a simple trait definition.
    /// This should return None if the field does not exist or is not a String.
    /// These should correspond to the values provided by `iter_fields`.
    fn get_field_strings(&self, field: &str) -> Option<Vec<String>>;

    /// Get the values of a field as numerics.
    /// This returns a Vec of f32 for the field, even if there is only one value (to allow for consistency in the return type).
    /// This allows for `get_field_numerics` to be used on f32 and Vec<f32> fields (etc.), allowing for a simple trait definition.
    /// This should return None if the field does not exist or is not a numeric type.
    /// These should correspond to the values provided by `iter_fields`.
    fn get_field_numerics(&self, field: &str) -> Option<Vec<f32>>;

    // TODO: redundant?
    /// Returns whether the given field is a string.
    /// This should return false if the field does not exist.
    fn is_field_string(field: &str) -> bool;

    /// Returns whether the given field is numeric.
    /// This should return false if the field does not exist.
    fn is_field_numeric(field: &str) -> bool;

    /// Gets all items that can be filtered on from a Library.
    /// TODO: May make the macro difficult.
    fn items(library: &Library) -> &HashMap<InternalId, Self>;

    /// Display a table of items in egui.
    fn display_table(ui: &mut Ui, filtered_items: Vec<&Self>);
}

impl<F: FilterableStruct> Hash for Filter<F> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

/// Types of filters based on the data type of the filter's field.
#[derive(Debug)]
pub enum FilterType {
    Number(NumberFilter),
    String(StringFilter),
}

impl<F: FilterableStruct> Filter<F> {
    /// Applies a given filter to the struct, returning whether the filter passes.
    /// It returns None if the field does not exist in the struct, is not properly implemented by FilterableStruct,
    /// or if the filter type does not match the field type.
    pub fn apply_filter(&self, field_struct: &F) -> Option<bool> {
        match self.filter_type {
            FilterType::Number(ref number_filter) => {
                if let Some(value) = field_struct.get_field_numerics(&self.field) {
                    return Some(number_filter.filter_any(value));
                }
            }
            FilterType::String(ref string_filter) => {
                if let Some(value) = field_struct.get_field_strings(&self.field) {
                    return Some(
                        string_filter.filter_any(value.iter().map(|s| s.as_str()).collect()),
                    );
                }
            }
        }
        None
    }

    /// Returns a list of all possible filters for numerical fields.
    /// Filters are listed in their default state.
    pub fn iter_number_filters() -> Vec<NumberFilter> {
        vec![
            NumberFilter::LessThan(0.0),
            NumberFilter::GreaterThan(0.0),
            NumberFilter::EqualTo(0.0),
        ]
    }

    /// Returns a list of all possible filters for string fields.
    /// Filters are listed in their default state.
    pub fn iter_string_filters() -> Vec<StringFilter> {
        vec![StringFilter::Contains("".to_string())]
    }
}

/// A filter for numerical fields.
/// Numerical fields will be converted to f32 when compared to the filter.
#[derive(Clone, PartialEq, Debug)]
pub enum NumberFilter {
    LessThan(f32),
    GreaterThan(f32),
    EqualTo(f32),
}

impl Default for NumberFilter {
    fn default() -> Self {
        NumberFilter::LessThan(0.0)
    }
}

impl From<NumberFilter> for FilterType {
    fn from(filter: NumberFilter) -> Self {
        FilterType::Number(filter)
    }
}

impl NumberFilter {
    /// Returns whether the given value passes the filter.
    /// TODO: This is currently unused.
    pub fn filter(&self, value: f32) -> bool {
        self.filter_any(vec![value])
    }

    /// Returns whether any of the given values pass the filter.
    pub fn filter_any(&self, value: Vec<f32>) -> bool {
        match self {
            NumberFilter::LessThan(ref number) => value.iter().any(|v| v < number),
            NumberFilter::GreaterThan(ref number) => value.iter().any(|v| v > number),
            NumberFilter::EqualTo(ref number) => value.iter().any(|v| v == number),
        }
    }

    /// The string representation of the type of the filter (absent the value itself)
    pub fn as_str(&self) -> &str {
        match self {
            NumberFilter::LessThan(_) => "Less than",
            NumberFilter::GreaterThan(_) => "Greater than",
            NumberFilter::EqualTo(_) => "Equal to",
        }
    }
}

/// A filter for string fields.
#[derive(Clone, PartialEq, Debug)]
pub enum StringFilter {
    Contains(String),
}

impl Default for StringFilter {
    fn default() -> Self {
        StringFilter::Contains("".to_string())
    }
}

impl From<StringFilter> for FilterType {
    fn from(filter: StringFilter) -> Self {
        FilterType::String(filter)
    }
}

impl StringFilter {
    /// Returns whether the given value passes the filter.
    /// TODO: This is currently unused.
    pub fn filter(&self, value: &str) -> bool {
        self.filter_any(vec![value])
    }

    /// Returns whether any of the given values pass the filter.
    /// This is useful for filtering on fields that may have a list of values (ie: 'traits').
    pub fn filter_any(&self, value: Vec<&str>) -> bool {
        match self {
            StringFilter::Contains(ref string) => value
                .iter()
                .any(|v| v.to_lowercase().contains(&string.to_lowercase())),
        }
    }

    /// The string representation of the type of the filter (absent the value itself)
    pub fn as_str(&self) -> &str {
        match self {
            StringFilter::Contains(_) => "Contains",
        }
    }
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
            // If field changed, update the filter type to match the new field
            if F::is_field_string(&field_editable) {
                self.filter_type = StringFilter::default().into();
                self.field = field_editable;
            } else if F::is_field_numeric(&field_editable) {
                self.filter_type = NumberFilter::default().into();
                self.field = field_editable;
            } else {
                panic!();
            }
        }

        let mut edited: bool = false;

        // TODO: doing two branches for get_field_string and get_field_numeric is a smell
        match self.filter_type {
            FilterType::String(ref mut string_filter) => {
                // TODO: may be able ot make most of this combo box modular between variants
                ComboBox::from_id_source(self.id.hash_with("display_filter_type"))
                    .selected_text(string_filter.as_str())
                    .show_ui(ui, |ui| {
                        for filter in Filter::<F>::iter_string_filters() {
                            ui.selectable_value(string_filter, filter.clone(), filter.as_str());
                        }
                    });

                match string_filter {
                    StringFilter::Contains(string) => {
                        edited |= ui.text_edit_singleline(string).changed();
                    }
                }
            }
            FilterType::Number(ref mut number_filter) => {
                ComboBox::from_id_source(self.id.hash_with("display_filter_type"))
                    .selected_text(number_filter.as_str())
                    .show_ui(ui, |ui| {
                        for filter in Filter::<F>::iter_number_filters() {
                            ui.selectable_value(number_filter, filter.clone(), filter.as_str());
                        }
                    });

                match number_filter {
                    NumberFilter::LessThan(f) => {
                        // TODO: maybe use the RestrictedText for this
                        edited |= ui.add(egui::Slider::new(f, 0.0..=MAX_SLIDER)).changed();
                    }
                    NumberFilter::GreaterThan(f) => {
                        edited |= ui.add(egui::Slider::new(f, 0.0..=MAX_SLIDER)).changed();
                    }
                    NumberFilter::EqualTo(f) => {
                        edited |= ui.add(egui::Slider::new(f, 0.0..=MAX_SLIDER)).changed();
                    }
                }
            }
        }

        edited
    }
}

// TODO: Separate into another crate probably
// TODO: avoid clones in these
pub trait FilterableDataType {
    fn as_numerics(&self) -> Option<Vec<f32>>;
    fn as_strings(&self) -> Option<Vec<String>>;

    fn is_numeric() -> bool;
    fn is_string() -> bool;
}

impl FilterableDataType for String {
    fn as_numerics(&self) -> Option<Vec<f32>> {
        None
    }

    fn as_strings(&self) -> Option<Vec<String>> {
        Some(vec![self.clone()])
    }

    fn is_numeric() -> bool {
        false
    }

    fn is_string() -> bool {
        true
    }
}

impl FilterableDataType for f32 {
    fn as_numerics(&self) -> Option<Vec<f32>> {
        Some(vec![*self])
    }

    fn as_strings(&self) -> Option<Vec<String>> {
        None
    }

    fn is_numeric() -> bool {
        true
    }

    fn is_string() -> bool {
        false
    }
}

impl FilterableDataType for Vec<String> {
    fn as_numerics(&self) -> Option<Vec<f32>> {
        None
    }

    fn as_strings(&self) -> Option<Vec<String>> {
        Some(self.clone())
    }

    fn is_numeric() -> bool {
        false
    }

    fn is_string() -> bool {
        true
    }
}

impl FilterableDataType for u8 {
    fn as_numerics(&self) -> Option<Vec<f32>> {
        Some(vec![*self as f32])
    }

    fn as_strings(&self) -> Option<Vec<String>> {
        None
    }

    fn is_numeric() -> bool {
        true
    }

    fn is_string() -> bool {
        false
    }
}

impl FilterableDataType for Currency {
    fn as_numerics(&self) -> Option<Vec<f32>> {
        Some(vec![self.as_base_unit() as f32])
    }

    fn as_strings(&self) -> Option<Vec<String>> {
        Some(vec![self.to_string()])
    }

    fn is_numeric() -> bool {
        true
    }

    fn is_string() -> bool {
        true
    }
}

impl FilterableDataType for Rarity {
    fn as_numerics(&self) -> Option<Vec<f32>> {
        None
    }

    fn as_strings(&self) -> Option<Vec<String>> {
        Some(vec![self.to_string()])
    }

    fn is_numeric() -> bool {
        false
    }

    fn is_string() -> bool {
        true
    }
}

impl FilterableDataType for Alignment {
    fn as_numerics(&self) -> Option<Vec<f32>> {
        None
    }

    fn as_strings(&self) -> Option<Vec<String>> {
        Some(vec![self.to_string()])
    }

    fn is_numeric() -> bool {
        false
    }

    fn is_string() -> bool {
        true
    }
}

impl FilterableDataType for Size {
    fn as_numerics(&self) -> Option<Vec<f32>> {
        None
    }

    fn as_strings(&self) -> Option<Vec<String>> {
        Some(vec![self.to_string()])
    }

    fn is_numeric() -> bool {
        false
    }

    fn is_string() -> bool {
        true
    }
}

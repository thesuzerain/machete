use egui::{ComboBox, Ui};

use crate::models::ids::InternalId;

// ui_models: UI relevant traits/implementations for models
// TODO: These are kept separate for now in anticipation of it helping with the UI code organization as we add crates,
// but it may be better to move these into the models structure.

pub mod events;
pub mod library;

/// A trait representing an object representable in the UI with multiple fields.
/// To allow for editing these fields, implement the DisplayFields trait.
pub trait DisplayFields {
    /// Display the fields of the object in the UI.
    /// Returns true if any of the fields were updated.
    fn display_fields(&mut self, ui: &mut Ui) -> bool;
}
// TODO: A lot of the code in here needs refactoring
// TODO: Move this to another module?
// TODO: This might also be prudent to keep in 'main' crate, as it's going to be used in multiple places.
/// A trait representing an object that can be filtered by a set of its fields.
// TODO: I dont understand why clone and partial eq are needed here if its just the generic and not actually in use
pub trait FilterableFields
where
    Self: Sized + Clone + PartialEq,
{
    fn iter_fields() -> Vec<&'static str>;
    fn get_field(&self, field: &str) -> Option<String>;

    fn filter(&self, filter: &Filter<Self>) -> bool {
        match &filter.filter_type {
            FilterType::LessThan(filter_value) => {
                if let Some(value_str) = self.get_field(&filter.field) {
                    if let Ok(value) = value_str.parse::<f32>() {
                        if let Ok(v) = filter_value.parse::<f32>() {
                            // TODO: Two parses here. Can we do this better?
                            return value < v;
                        }
                    }
                }
            }
            FilterType::GreaterThan(filter_value) => {
                if let Some(value_str) = self.get_field(&filter.field) {
                    if let Ok(inner_value) = value_str.parse::<f32>() {
                        if let Ok(v) = filter_value.parse::<f32>() {
                            // TODO: Two parses here. Can we do this better?
                            return inner_value > v;
                        }
                    }
                }
            }
            FilterType::EqualTo(filter_value) => {
                if let Some(value_str) = self.get_field(&filter.field) {
                    if let Ok(value) = value_str.parse::<f32>() {
                        if let Ok(filter_value) = filter_value.parse::<f32>() {
                            // TODO: Two parses here. Can we do this better?
                            return value == filter_value;
                        }
                    }
                }
            }
            FilterType::Contains(filter_value) => {
                if let Some(value_str) = self.get_field(&filter.field) {
                    // TODO: Needs to better handle tags
                    return value_str
                        .to_lowercase()
                        .contains(filter_value.to_lowercase().as_str());
                }
            }
            FilterType::True => {
                if let Some(value_str) = self.get_field(&filter.field) {
                    return value_str == "true";
                }
            }
            FilterType::Not(filter) => {
                return !self.filter(filter);
            }
            FilterType::Or(filter1, filter2) => {
                return self.filter(filter1) || self.filter(filter2);
            }
        }

        false
    }
}

// TODO: I dont understand why clone and partial eq are needed here if its just the generic and not actually in use
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Filter<T: FilterableFields + Clone + PartialEq> {
    pub id: InternalId, // tODO: this is purely to avoid widget clashes. Is there a better way?
    pub field: String,
    pub filter_type: FilterType<T>,

    _phantom: std::marker::PhantomData<T>,
}

impl<T: FilterableFields + Clone + PartialEq> Filter<T> {
    pub fn new(field: String, filter_type: FilterType<T>) -> Self {
        Filter {
            field,
            filter_type,
            id: InternalId::new(),
            _phantom: std::marker::PhantomData,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FilterType<T: FilterableFields + Clone + PartialEq> {
    LessThan(String), // TODO: IS there a way to do this with f32?
    GreaterThan(String),
    EqualTo(String),
    Contains(String),
    True,

    Not(Box<Filter<T>>),
    Or(Box<Filter<T>>, Box<Filter<T>>),
}

impl<T: FilterableFields + Clone + PartialEq> FilterType<T> {
    // TODO: str != string, etc .bad naming and maybe not needed
    pub fn as_str(&self) -> String {
        match self {
            FilterType::LessThan(_) => "Less than".to_string(),
            FilterType::GreaterThan(_) => "Greater than".to_string(),
            FilterType::EqualTo(_) => "Equal to".to_string(),
            FilterType::Contains(_) => "Contains".to_string(),
            FilterType::True => "True".to_string(),
            FilterType::Not(_) => "Not".to_string(),
            FilterType::Or(_, _) => "Or".to_string(),
        }
    }
}

impl<T: FilterableFields + Clone + PartialEq> Filter<T> {
    pub fn iter_filters() -> Vec<FilterType<T>> {
        vec![
            FilterType::<T>::LessThan("".to_string()),
            FilterType::<T>::GreaterThan("".to_string()),
            FilterType::<T>::EqualTo("".to_string()),
            FilterType::<T>::Contains("".to_string()),
            FilterType::<T>::True,
        ]
    }

    // TODO: still necessary?
    pub fn field_mut(&mut self) -> &mut String {
        &mut self.field
    }
}

impl<T: FilterableFields + Clone + PartialEq> DisplayFields for Filter<T> {
    fn display_fields(&mut self, ui: &mut Ui) -> bool {
        let id = self.id;

        let field_editable = self.field_mut();
        ComboBox::from_id_source(id.hash_with("display_filterable_fields"))
            .selected_text(field_editable.to_string())
            .show_ui(ui, |ui| {
                for field in T::iter_fields() {
                    ui.selectable_value(field_editable, field.to_string(), field.to_string());
                }
            });

        ComboBox::from_id_source(id.hash_with("display_filter_type"))
            .selected_text(self.filter_type.as_str())
            .show_ui(ui, |ui| {
                for filter in Filter::<T>::iter_filters() {
                    ui.selectable_value(&mut self.filter_type, filter.clone(), filter.as_str());
                }
            });

        match &mut self.filter_type {
            FilterType::LessThan(value) => {
                ui.label("Less than:");
                // TODO: Awful pattern here. Make a new widget for this.
                // https://github.com/emilk/egui/issues/1348
                // (In another PR, I made a widget for this, but it's not merged yet.)
                let mut value_str = value.to_string();
                ui.text_edit_singleline(&mut value_str);
                if value_str.parse::<f32>().is_ok() {
                    *value = value_str;
                }
                true
            }
            FilterType::GreaterThan(value) => {
                ui.label("Greater than:");
                let mut value_str = value.to_string(); // TODO: can get rid oft hese if its a string
                ui.text_edit_singleline(&mut value_str);
                if value_str.parse::<f32>().is_ok() {
                    *value = value_str;
                }
                true
            }
            FilterType::EqualTo(value) => {
                ui.label("Equal to:");
                let mut value_str = value.to_string();
                ui.text_edit_singleline(&mut value_str);
                if value_str.parse::<f32>().is_ok() {
                    *value = value_str;
                }
                true
            }
            FilterType::Contains(value) => {
                ui.label("Contains:");
                ui.text_edit_singleline(value);
                true
            }
            FilterType::True => {
                ui.label("True:");
                true
            }
            FilterType::Not(filter) => {
                ui.label("Not:");
                filter.display_fields(ui)
            }
            FilterType::Or(filter1, filter2) => {
                ui.label("Or:");
                filter1.display_fields(ui) || filter2.display_fields(ui)
            }
        }
    }
}

use egui::ComboBox;

use crate::models::ids::InternalId;

use super::DisplayFields;

// TODO: This will need to be dynamic or changed. This is just a placeholder.
const MAX_SCROLLBAR: f32 = 1000.0;

#[derive(Clone, PartialEq)]
pub struct Example {
    pub level: f32,
    pub name: String,
}

pub trait FilterableStruct
where
    Self: Sized + Clone + PartialEq,
{
    fn create_default_filter() -> Filter<Self>;

    fn iter_fields() -> Vec<&'static str>;

    // TODO: this pattern is a smell
    fn get_field_string(&self, field: &str) -> Option<String>;
    fn get_field_numeric(&self, field: &str) -> Option<f32>;

    // TODO: redundant?
    fn is_field_numeric(field: &str) -> bool;
    fn is_field_string(field: &str) -> bool;
}

impl FilterableStruct for Example {
    // TODO: this is a smell
    fn create_default_filter() -> Filter<Self> {
        Filter {
            id: InternalId::new(),
            field: "name".to_string(),
            filter_type: FilterType::String(StringFilter::Contains("".to_string())),
            _phantom: std::marker::PhantomData,
        }
    }

    fn iter_fields() -> Vec<&'static str> {
        vec!["level", "name"]
    }

    fn get_field_string(&self, field: &str) -> Option<String> {
        match field {
            "name" => Some(self.name.clone()),
            _ => None,
        }
    }

    fn get_field_numeric(&self, field: &str) -> Option<f32> {
        match field {
            "level" => Some(self.level),
            _ => None,
        }
    }

    fn is_field_numeric(field: &str) -> bool {
        match field {
            "level" => true,
            _ => false,
        }
    }

    fn is_field_string(field: &str) -> bool {
        match field {
            "name" => true,
            _ => false,
        }
    }
}

#[derive(Debug)]
pub struct Filter<F: FilterableStruct> {
    pub id: InternalId,
    pub field: String, // TODO: Not a string
    pub filter_type: FilterType,

    // TODO: is this needed
    pub _phantom: std::marker::PhantomData<F>,
}

#[derive(Debug)]
pub enum FilterType {
    Number(NumberFilter),
    String(StringFilter),
}

impl<F: FilterableStruct> Filter<F> {
    // TODO: Should this be an option? currently is an option if fields dont match- maybe can isolate and make a result or panic
    pub fn apply_filter(&self, field_struct: &F) -> Option<bool> {
        match self.filter_type {
            FilterType::Number(ref number_filter) => {
                if let Some(value) = field_struct.get_field_numeric(&self.field) {
                    return Some(number_filter.filter(value));
                }
            }
            FilterType::String(ref string_filter) => {
                if let Some(value) = field_struct.get_field_string(&self.field) {
                    return Some(string_filter.filter(&value));
                }
            }
        }
        None
    }

    pub fn iter_number_filters() -> Vec<NumberFilter> {
        vec![
            NumberFilter::LessThan(0.0),
            NumberFilter::GreaterThan(0.0),
            NumberFilter::EqualTo(0.0),
        ]
    }

    pub fn iter_string_filters() -> Vec<StringFilter> {
        vec![StringFilter::Contains("".to_string())]
    }
}

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
    pub fn filter(&self, value: f32) -> bool {
        match self {
            NumberFilter::LessThan(ref number) => value < *number,
            NumberFilter::GreaterThan(ref number) => value > *number,
            NumberFilter::EqualTo(ref number) => value == *number,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            NumberFilter::LessThan(_) => "Less than",
            NumberFilter::GreaterThan(_) => "Greater than",
            NumberFilter::EqualTo(_) => "Equal to",
        }
    }
}

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
    pub fn filter(&self, value: &str) -> bool {
        match self {
            StringFilter::Contains(ref string) => value.contains(string),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            StringFilter::Contains(_) => "Contains",
        }
    }
}

impl<F: FilterableStruct> DisplayFields for Filter<F> {
    fn display_fields(&mut self, ui: &mut egui::Ui) -> bool {
        let id = self.id;

        let mut field_editable = self.field.clone(); //todo: needed?
        ComboBox::from_id_source(id.hash_with("display_filterable_fields"))
            .selected_text(field_editable.to_string())
            .show_ui(ui, |ui| {
                for field in F::iter_fields() {
                    ui.selectable_value(&mut field_editable, field.to_string(), field.to_string());
                }
            });
        // todo: smell?
        if self.field != field_editable {
            // If field changed, update the filter type to match the new field
            // TODO: smell here, awkward,
            // TODO: doing two if let fields for get_field_string and get_field_numeric is a smell
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
                        edited |= ui.add(egui::Slider::new(f, 0.0..=MAX_SCROLLBAR)).changed();
                    }
                    NumberFilter::GreaterThan(f) => {
                        edited |= ui.add(egui::Slider::new(f, 0.0..=MAX_SCROLLBAR)).changed();
                    }
                    NumberFilter::EqualTo(f) => {
                        edited |= ui.add(egui::Slider::new(f, 0.0..=MAX_SCROLLBAR)).changed();
                    }
                }
            }
        }

        edited
    }
}

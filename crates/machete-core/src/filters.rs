use std::hash::{Hash, Hasher};

use crate::ids::InternalId;

/// A filter over structure fields.
/// For example: "Name contains 'Bob'" or "Level is greater than 5".
#[derive(Debug, Clone)]
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

    /// Iterate over the possible filter types for a field.
    /// For example, if the field is "level", this would return a list of all possible filters for level.
    fn iter_filter_types_for_field(field: &str) -> Option<Vec<FilterType>>;

    /// Iterate over the possible filter variants for a variant field.
    /// For example, if the field is "rarity", this would return a list of all possible rarities.
    /// These must correspond to the iterable fields in the struct, and are only where a EqualToChoice filter is used.
    fn iter_filter_variants_for_field(field: &str) -> Option<Vec<String>>;
}

impl<F: FilterableStruct> Hash for Filter<F> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.field.hash(state);
        self.filter_type.hash(state);
    }
}

/// Types of filters based on the data type of the filter's field.
#[derive(Debug, PartialEq, Clone)]
pub enum FilterType {
    GreaterThan(f32),
    LessThan(f32),
    /// Whether a number is equal to a given number.
    EqualToNumber(f32),
    /// Whether a variant matches a specific choice.
    // TODO: This might be better to use integers as variant codes and combine with EqualToNumber.
    // TODO: Or, even better, find a way to use variants directly (Box<> wasn't working initially.)
    EqualToChoice(String),
    Contains(String),
}

impl Hash for FilterType {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            FilterType::GreaterThan(f) => {
                0.hash(state);
                f.to_bits().hash(state);
            }
            FilterType::LessThan(f) => {
                1.hash(state);
                f.to_bits().hash(state);
            }
            FilterType::EqualToNumber(f) => {
                2.hash(state);
                f.to_bits().hash(state);
            }
            FilterType::EqualToChoice(s) => {
                3.hash(state);
                s.hash(state);
            }
            FilterType::Contains(s) => {
                4.hash(state);
                s.hash(state);
            }
        }
    }
}

impl FilterType {
    /// The string representation of the type of the filter (absent the value itself)
    pub fn as_str(&self) -> &str {
        match self {
            FilterType::LessThan(_) => "Less than",
            FilterType::GreaterThan(_) => "Greater than",
            FilterType::EqualToNumber(_) => "Equal to",
            FilterType::EqualToChoice(_) => "Equal to",
            FilterType::Contains(_) => "Contains",
        }
    }
}

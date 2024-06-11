use super::filters::FilterableStruct;
use crate::models::{ids::InternalId, library::LibraryItem};

impl FilterableStruct for LibraryItem {
    fn create_default_filter() -> super::filters::Filter<Self> {
        super::filters::Filter {
            id: InternalId::new(),
            field: "name".to_string(),
            filter_type: super::filters::FilterType::String(
                super::filters::StringFilter::Contains("".to_string()),
            ),
            _phantom: std::marker::PhantomData,
        }
    }

    fn iter_fields() -> Vec<&'static str> {
        ["name", "price", "game_system", "rarity", "level", "tags"].to_vec()
    }

    fn get_field_numeric(&self, field: &str) -> Option<f32> {
        match field {
            "price" => Some(self.price as f32),
            "level" => Some(self.level as f32),
            _ => None,
        }
    }

    fn get_field_string(&self, field: &str) -> Option<String> {
        match field {
            "name" => Some(self.name.clone()),
            "game_system" => Some(self.game_system.clone()),
            "rarity" => Some(self.rarity.clone()),
            "tags" => Some(self.tags.join(", ")),
            _ => None,
        }
    }

    fn is_field_numeric(field: &str) -> bool {
        match field {
            "price" | "level" => true,
            _ => false,
        }
    }

    fn is_field_string(field: &str) -> bool {
        match field {
            "name" | "game_system" | "rarity" | "tags" => true,
            _ => false,
        }
    }
}

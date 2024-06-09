use crate::models::library::LibraryItem;

use super::FilterableFields;

impl FilterableFields for LibraryItem {
    fn iter_fields() -> Vec<&'static str> {
        ["name", "price", "game_system", "rarity", "level", "tags"].to_vec()
    }

    // TODO: String should be &str
    fn get_field(&self, field: &str) -> Option<String> {
        match field {
            "name" => Some(self.name.clone()),
            "price" => Some(self.price.to_string()),
            "game_system" => Some(self.game_system.clone()),
            "rarity" => Some(self.rarity.clone()),
            "level" => Some(self.level.to_string()),
            "tags" => Some(self.tags.join(", ")),
            _ => None,
        }
    }
}

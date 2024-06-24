use crate::models::{
    ids::InternalId,
    library::{
        creature::{Alignment, LibraryCreature, Size},
        item::LibraryItem,
        spell::LibrarySpell,
        GameSystem, Library, Rarity,
    },
};
use std::collections::HashMap;

use super::filter::{Filter, FilterType, FilterableStruct};

/// Implements 'filters.rs' traits on library structs.
/// TODO: May be in the wrong file.
// TODO: This might be worth making a derive macro for to ensure implementation consistency and auto-updating if the struct changes. A lot of the functions were intentionally designed with this in mind.
impl FilterableStruct for LibraryItem {
    fn create_default_filter() -> Filter<Self> {
        Filter {
            id: InternalId::new(),
            field: "name".to_string(),
            filter_type: FilterType::Contains("".to_string()),
            _phantom: std::marker::PhantomData,
        }
    }

    fn iter_fields() -> Vec<&'static str> {
        ["name", "price", "game_system", "rarity", "level", "tags"].to_vec()
    }

    fn iter_filter_variants_for_field(field: &str) -> Option<Vec<String>> {
        match field {
            "rarity" => Some(Rarity::iter().map(|r| r.to_string()).collect()),
            "game_system" => Some(GameSystem::iter().map(|gs| gs.to_string()).collect()),
            _ => None,
        }
    }

    // todo: should these be returning result instead?
    fn iter_filter_types_for_field(field: &str) -> Option<Vec<FilterType>> {
        let mut filter_types = Vec::new();
        match field {
            "level" => {
                filter_types.push(FilterType::GreaterThan(0.0));
                filter_types.push(FilterType::LessThan(0.0));
                filter_types.push(FilterType::EqualToNumber(0.0));
            }
            "name" => {
                filter_types.push(FilterType::Contains("".to_string()));
            }
            // TODO: price should be searchable
            "price" => {
                filter_types.push(FilterType::GreaterThan(0.0));
                filter_types.push(FilterType::LessThan(0.0));
                filter_types.push(FilterType::EqualToNumber(0.0));
            }
            "rarity" => {
                filter_types.push(FilterType::EqualToChoice(Rarity::default().to_string()));
            }
            "game_system" => {
                filter_types.push(FilterType::EqualToChoice(GameSystem::default().to_string()));
            }
            "tags" => {
                filter_types.push(FilterType::Contains("".to_string()));
            }
            _ => {
                return None;
            }
        }
        Some(filter_types)
    }

    fn items(library: &Library) -> &HashMap<InternalId, LibraryItem> {
        &library.items
    }
}

impl FilterableStruct for LibraryCreature {
    fn create_default_filter() -> Filter<Self> {
        Filter {
            id: InternalId::new(),
            field: "name".to_string(),
            filter_type: FilterType::Contains("".to_string()),
            _phantom: std::marker::PhantomData,
        }
    }

    fn iter_fields() -> Vec<&'static str> {
        [
            "name",
            "game_system",
            "rarity",
            "level",
            "tags",
            "size",
            "alignment",
        ]
        .to_vec()
    }

    fn iter_filter_variants_for_field(field: &str) -> Option<Vec<String>> {
        match field {
            "rarity" => Some(Rarity::iter().map(|r| r.to_string()).collect()),
            "size" => Some(Size::iter().map(|s| s.to_string()).collect()),
            "alignment" => Some(Alignment::iter().map(|a| a.to_string()).collect()),
            "game_system" => Some(GameSystem::iter().map(|gs| gs.to_string()).collect()),
            _ => None,
        }
    }

    // todo: should these be returning result instead?
    fn iter_filter_types_for_field(field: &str) -> Option<Vec<FilterType>> {
        let mut filter_types = Vec::new();
        match field {
            "level" => {
                filter_types.push(FilterType::GreaterThan(0.0));
                filter_types.push(FilterType::LessThan(0.0));
                filter_types.push(FilterType::EqualToNumber(0.0));
            }
            "name" => {
                filter_types.push(FilterType::Contains("".to_string()));
            }
            "rarity" => {
                filter_types.push(FilterType::EqualToChoice(Rarity::default().to_string()));
            }
            "game_system" => {
                filter_types.push(FilterType::EqualToChoice(GameSystem::default().to_string()));
            }
            "alignment" => {
                filter_types.push(FilterType::EqualToChoice(Alignment::default().to_string()));
            }
            "size" => {
                filter_types.push(FilterType::EqualToChoice(Size::default().to_string()));
            }
            "tags" => {
                filter_types.push(FilterType::Contains("".to_string()));
            }
            _ => {
                return None;
            }
        }
        Some(filter_types)
    }

    fn items(library: &Library) -> &HashMap<InternalId, LibraryCreature> {
        &library.creatures
    }
}

impl FilterableStruct for LibrarySpell {
    fn create_default_filter() -> Filter<Self> {
        Filter {
            id: InternalId::new(),
            field: "name".to_string(),
            filter_type: FilterType::Contains("".to_string()),
            _phantom: std::marker::PhantomData,
        }
    }

    fn iter_filter_variants_for_field(field: &str) -> Option<Vec<String>> {
        match field {
            "rarity" => Some(Rarity::iter().map(|r| r.to_string()).collect()),
            "game_system" => Some(GameSystem::iter().map(|gs| gs.to_string()).collect()),
            _ => None,
        }
    }

    fn iter_fields() -> Vec<&'static str> {
        ["name", "game_system", "rarity", "rank", "tags"].to_vec()
    }

    // todo: should these be returning result instead?
    fn iter_filter_types_for_field(field: &str) -> Option<Vec<FilterType>> {
        let mut filter_types = Vec::new();
        match field {
            "rank" => {
                filter_types.push(FilterType::GreaterThan(0.0));
                filter_types.push(FilterType::LessThan(0.0));
                filter_types.push(FilterType::EqualToNumber(0.0));
            }
            "name" => {
                filter_types.push(FilterType::Contains("".to_string()));
            }
            "rarity" => {
                filter_types.push(FilterType::EqualToChoice(Rarity::default().to_string()));
            }
            "game_system" => {
                filter_types.push(FilterType::EqualToChoice(GameSystem::default().to_string()));
            }
            "tags" => {
                filter_types.push(FilterType::Contains("".to_string()));
            }
            _ => {
                return None;
            }
        }
        Some(filter_types)
    }

    fn items(library: &Library) -> &HashMap<InternalId, LibrarySpell> {
        &library.spells
    }
}

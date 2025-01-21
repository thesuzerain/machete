use std::{collections::HashMap, sync::Arc};

use crate::{auth::extract_user_from_cookies, database::{creatures::{CreatureFilters, CreatureSearch}, hazards::{HazardFilters, HazardSearch}, items::{ItemFilters, ItemSearch}, spells::{SpellFilters, SpellSearch}, DEFAULT_MAX_GROUP_LIMIT}, intelligent::noun_phrases, models::{ids::InternalId, library::{creature::LibraryCreature, hazard::LibraryHazard, item::LibraryItem, spell::LibrarySpell}}, AppState};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};

use axum_extra::extract::CookieJar;
use itertools::Itertools;
use nlprule::Tokenizer;
use serde_json::json;
use sqlx::{PgPool, Pool};

use crate::{
    database::{
        self,
        encounters::{EncounterFilters, InsertEncounter, ModifyEncounter},
    },
    ServerError,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/augmented-nlp", post(parse_augmented_noun_phrases))
}

#[derive(serde::Deserialize)]   
pub struct ParseAugmentedNounPhrases {
    pub text : String,
    pub min_similarity : Option<f32>,
    pub creature_filters : Option<CreatureFilters>,
    pub item_filters :  Option<ItemFilters>,
    pub spell_filters :  Option<SpellFilters>,
    pub hazard_filters : Option< HazardFilters>,
}

// TODO: Move things to models
#[derive(serde::Serialize)]
pub struct AugmentedNounPhrase {
    pub text : String,
    pub span : (usize, usize),
    pub similarity : f32,
    pub augment : AugmentedNounPhraseAugment,
}

#[derive(serde::Serialize)]
#[serde(tag = "type")]
pub enum AugmentedNounPhraseAugment {
    Creature(LibraryCreature),
    Spell(LibrarySpell),
    Item(LibraryItem),
    Hazard(LibraryHazard),
    None
}

async fn parse_augmented_noun_phrases(
    State(tokenizer): State<Arc<Tokenizer>>,
    State(pool): State<PgPool>,
    Json(data): Json<ParseAugmentedNounPhrases>,
) -> Result<impl IntoResponse, ServerError> {
    // Get noun phrases from text
    let noun_phrases = noun_phrases::extract_noun_phrases_from_text(&tokenizer, &data.text);
    let noun_phrase_strings = noun_phrases.iter().map(|np| np.to_string()).collect::<Vec<String>>();

    let min_similarity = data.min_similarity.unwrap_or(0.4);

    // Do searches for each noun phrase
    // Creatures
    let creature_search = CreatureSearch {
        filters: data.creature_filters.unwrap_or_default(),
        query: noun_phrase_strings.clone(),
        min_similarity: Some(min_similarity),
    };
    let hazard_search = HazardSearch {
        filters: data.hazard_filters.unwrap_or_default(),
        query: noun_phrase_strings.clone(),
        min_similarity: Some(min_similarity),
    };
    let item_search = ItemSearch {
        filters: data.item_filters.unwrap_or_default(),
        query: noun_phrase_strings.clone(),
        min_similarity: Some(min_similarity),
    };
    let spell_search = SpellSearch {
        filters: data.spell_filters.unwrap_or_default(),
        query: noun_phrase_strings.clone(),
        min_similarity: Some(min_similarity),
    };

    // Fetch all the data
    // TODO: these need to return similarities so you can cross compare here
    let (creatures, hazards, items, spells) = tokio::try_join!{
        database::creatures::get_creatures_search(&pool, &creature_search, DEFAULT_MAX_GROUP_LIMIT),
        database::hazards::get_hazards_search(&pool, &hazard_search, DEFAULT_MAX_GROUP_LIMIT),
        database::items::get_items_search(&pool, &item_search, DEFAULT_MAX_GROUP_LIMIT),
        database::spells::get_spells_search(&pool, &spell_search, DEFAULT_MAX_GROUP_LIMIT),
    }?;

    let mut augmented_noun_phrases = vec![];
    for np in noun_phrases {
        let np_text = np.to_string();
        let np_span = (np.start, np.end);
        let mut np_data = AugmentedNounPhrase {
            text: np_text.clone(),
            span: np_span,
            similarity: 0.0,
            augment: AugmentedNounPhraseAugment::None,
        };

        // All values are sorted, so use first from each
        if let Some(creatures) = creatures.get(&np_text) {
            if let Some((similarity, lib)) = creatures.first() {
                if *similarity > np_data.similarity {
                    np_data.similarity = *similarity;
                    np_data.augment = AugmentedNounPhraseAugment::Creature(lib.clone());
                }
            }
        }

        if let Some(hazards) = hazards.get(&np_text) {
            if let Some((similarity, lib)) = hazards.first() {
                if *similarity > np_data.similarity {
                    np_data.similarity = *similarity;
                    np_data.augment = AugmentedNounPhraseAugment::Hazard(lib.clone());
                }
            }
        }

        if let Some(items) = items.get(&np_text) {
            if let Some((similarity, lib)) = items.first() {
                if *similarity > np_data.similarity {
                    np_data.similarity = *similarity;
                    np_data.augment = AugmentedNounPhraseAugment::Item(lib.clone());
                }
            }
        }

        if let Some(spells) = spells.get(&np_text) {
            if let Some((similarity, lib)) = spells.first() {
                if *similarity > np_data.similarity {
                    np_data.similarity = *similarity;
                    np_data.augment = AugmentedNounPhraseAugment::Spell(lib.clone());
                }
            }
        }

        augmented_noun_phrases.push(np_data);
    }

    // TODO: Handle overlaps
    
    Ok(Json(augmented_noun_phrases))
}


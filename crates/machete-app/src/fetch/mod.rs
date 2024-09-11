use async_trait::async_trait;
use machete::models::library::{
    creature::{CreatureFilters, LibraryCreature},
    item::{ItemFilters, LibraryItem},
    spell::{LibrarySpell, SpellFilters},
};
use machete_core::filters::{Filter, FilterableStruct};

lazy_static::lazy_static! {
    static ref REQWEST_CLIENT: reqwest::Client = reqwest::Client::new();
}
// TODO: Is there a coherent way to make this into an environment variable, given WASM?
// WASI may be an option
pub const SERVER_URL: &str = "https://machete-api.wyattverchere.com";

#[cfg_attr(feature = "offline", async_trait)]
#[cfg_attr(feature = "web_app", async_trait(?Send))]
pub trait FetchableStruct: FilterableStruct {
    async fn fetch_backend(filters: &[Filter<Self>]) -> crate::Result<Vec<Self>>;
}

#[cfg_attr(feature = "offline", async_trait)]
#[cfg_attr(feature = "web_app", async_trait(?Send))]
impl FetchableStruct for LibraryItem {
    async fn fetch_backend(filters: &[Filter<Self>]) -> crate::Result<Vec<Self>> {
        let mut item_filters = ItemFilters::default();
        for filter in filters {
            let filter = (*filter).clone();
            // TODO: These should be included with the macro for these fetchable objects.
            if let Ok(f) = ItemFilters::try_from(filter) {
                item_filters = item_filters.merge(f);
            }
        }

        let query = serde_qs::to_string(&item_filters).unwrap();
        let result = reqwest::Client::new()
            .get(format!("{}/items?{query}", SERVER_URL).as_str())
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        Ok(result)
    }
}

#[cfg_attr(feature = "offline", async_trait)]
#[cfg_attr(feature = "web_app", async_trait(?Send))]
impl FetchableStruct for LibrarySpell {
    async fn fetch_backend(filters: &[Filter<Self>]) -> crate::Result<Vec<Self>> {
        let mut spell_filters = SpellFilters::default();
        for filter in filters {
            let filter = (*filter).clone();

            if let Ok(f) = SpellFilters::try_from(filter) {
                spell_filters = spell_filters.merge(f);
            }
        }
        let query = serde_qs::to_string(&spell_filters).unwrap();
        let result = REQWEST_CLIENT
            .get(format!("{}/spells?{query}", SERVER_URL).as_str())
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        Ok(result)
    }
}

#[cfg_attr(feature = "offline", async_trait)]
#[cfg_attr(feature = "web_app", async_trait(?Send))]
impl FetchableStruct for LibraryCreature {
    async fn fetch_backend(filters: &[Filter<Self>]) -> crate::Result<Vec<Self>> {
        let mut creature_filters = CreatureFilters::default();
        for filter in filters {
            let filter = (*filter).clone();

            if let Ok(f) = CreatureFilters::try_from(filter) {
                creature_filters = creature_filters.merge(f);
            }
        }

        let query = serde_qs::to_string(&creature_filters).unwrap();
        let result = REQWEST_CLIENT
            .get(format!("{}/creatures?{query}", SERVER_URL).as_str())
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        Ok(result)
    }
}

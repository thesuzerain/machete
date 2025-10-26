use std::collections::HashMap;
use std::collections::HashSet;

use itertools::Itertools;
use serde::Deserialize;
use serde::Serialize;

use crate::models::ids::InternalId;
use crate::models::library::LibraryObjectType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertTag {
    pub id: InternalId,
    pub tag: String,
    pub r#trait: bool,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagList {
    pub combined_tags: Vec<String>,
    pub combined_traits: Vec<String>,

    #[serde(flatten)]
    pub subsets: HashMap<LibraryObjectType, TagSubset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagSubset {
    pub tags: Vec<String>,
    pub traits: Vec<String>,
}

// TODO: not sure if I like this
pub async fn insert_tags(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    items: &Vec<InsertTag>,
) -> crate::Result<()> {
    if items.is_empty() {
        return Ok(());
    }

    // Deduplicate the items
    let mut seen = HashSet::new();
    let items: Vec<InsertTag> = items
        .iter()
        .filter(|x| seen.insert(x.tag.clone()))
        .sorted_by_key(|x| x.id.0)
        .cloned()
        .collect();

    sqlx::query!(
        r#"
        INSERT INTO library_tags (id, tag, "trait")
        SELECT * FROM UNNEST (
            $1::int[],
            $2::text[],
            $3::boolean[]
        )
        ON CONFLICT (id) DO UPDATE
        SET tag = EXCLUDED.tag,
            "trait" = EXCLUDED."trait"
        "#,
        &items.iter().map(|x| x.id.0 as i32).collect::<Vec<i32>>(),
        &items.iter().map(|x| x.tag.clone()).collect::<Vec<String>>(),
        &items.iter().map(|x| x.r#trait).collect::<Vec<bool>>(),
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}

#[derive(Debug)]
pub struct TagMatches {
    pub all_traits: Option<Vec<String>>,
    pub any_traits: Option<Vec<String>>,
}

pub async fn get_tag_matches(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    all_traits: &Option<Vec<String>>,
    any_traits: &Option<Vec<String>>,
) -> crate::Result<TagMatches> {
    let combined = all_traits
        .iter()
        .flatten()
        .cloned()
        .chain(any_traits.iter().flatten().cloned())
        .collect::<Vec<String>>();

    // TODO: This can be expanded/slightly refactored for close semantic matches
    let tag_matches : Vec<String> = sqlx::query!(
        r#"
        SELECT tag
        FROM library_tags
        WHERE tag = ANY($1::text[])
        "#,
        &combined as _,
    )
    .fetch_all(exec)
    .await?
    .into_iter()
    .map(|x| x.tag)
    .collect();

    let all_traits = all_traits.as_ref().map(|x: &Vec<String>| {
        x.iter()
            .filter(|x| tag_matches.contains(x))
            .cloned()
            .collect::<Vec<String>>()
    });
    let any_traits = any_traits.as_ref().map(|x| {
        x.iter()
            .filter(|x| tag_matches.contains(x))
            .cloned()
            .collect::<Vec<String>>()
    });

    Ok(TagMatches {
        all_traits,
        any_traits,
    })
}

pub async fn get_tags(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
) -> crate::Result<TagList> {

    struct FetchRow {
        tag: String,
        r#trait: bool,
        any_creature: bool,
        any_item: bool,
        any_hazard: bool,
        any_spell: bool,
    }

    let traits : (Vec<FetchRow>, Vec<FetchRow>) = sqlx::query!(
        r#"
        SELECT
            lt.id, tag, "trait" as "trait!",
            any_value(lc.id) IS NOT NULL AS "any_creature!",
            any_value(li.id) IS NOT NULL AS "any_item!",
            any_value(ls.id) IS NOT NULL AS "any_spell!",
            any_value(lh.id) IS NOT NULL AS "any_hazard!"
        FROM library_tags lt
        LEFT JOIN library_objects_tags lot ON lt.id = lot.tag_id
        LEFT JOIN library_creatures lc ON lot.library_object_id = lc.id
        LEFT JOIN library_items li ON lot.library_object_id = li.id
        LEFT JOIN library_spells ls ON lot.library_object_id = ls.id
        LEFT JOIN library_hazards lh ON lot.library_object_id = lh.id
        GROUP BY lt.id
        "#,
    )
    .fetch_all(exec)
    .await?
    .into_iter()
    .map(|x| FetchRow {
        tag: x.tag,
        r#trait: x.r#trait,
        any_creature: x.any_creature,
        any_item: x.any_item,
        any_hazard: x.any_hazard,
        any_spell: x.any_spell,
    })
    .partition(|x| x.r#trait);

    let creature_traits = traits.0.iter().filter(|x| x.any_creature).map(|x| x.tag.clone()).collect_vec();
    let creature_tags = traits.1.iter().filter(|x| x.any_creature).map(|x| x.tag.clone()).collect_vec();
    let item_traits = traits.0.iter().filter(|x| x.any_item).map(|x| x.tag.clone()).collect_vec();
    let item_tags = traits.1.iter().filter(|x| x.any_item).map(|x| x.tag.clone()).collect_vec();
    let hazard_traits = traits.0.iter().filter(|x| x.any_hazard).map(|x| x.tag.clone()).collect_vec();
    let hazard_tags = traits.1.iter().filter(|x| x.any_hazard).map(|x| x.tag.clone()).collect_vec();
    let spell_traits = traits.0.iter().filter(|x| x.any_spell).map(|x| x.tag.clone()).collect_vec();
    let spell_tags = traits.1.iter().filter(|x| x.any_spell).map(|x| x.tag.clone()).collect_vec();

    Ok(TagList {
        combined_tags: traits.1.iter().map(|x| x.tag.clone()).collect(),
        combined_traits: traits.0.iter().map(|x| x.tag.clone()).collect(),
        subsets: HashMap::from([
            (LibraryObjectType::Creature, TagSubset { tags: creature_tags, traits: creature_traits }),
            (LibraryObjectType::Item, TagSubset { tags: item_tags, traits: item_traits }),
            (LibraryObjectType::Hazard, TagSubset { tags: hazard_tags, traits: hazard_traits }),
            (LibraryObjectType::Spell, TagSubset { tags: spell_tags, traits: spell_traits }),
        ]),
    })
}

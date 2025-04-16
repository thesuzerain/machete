use std::collections::HashMap;

use crate::{
    auth::extract_admin_from_headers,
    database::{
        classes::{ClassSearch, InsertLibraryClass},
        creatures::{CreatureFiltering, CreatureSearch, InsertLibraryCreature},
        hazards::{HazardFiltering, HazardSearch, InsertLibraryHazard},
        items::{InsertLibraryItem, ItemFiltering, ItemSearch},
        spells::{InsertLibrarySpell, SpellFiltering, SpellSearch},
        DEFAULT_MAX_GROUP_LIMIT, DEFAULT_MAX_LIMIT,
    },
    models::library::{
        classes::LibraryClass, creature::LibraryCreature, hazard::LibraryHazard, item::LibraryItem,
        spell::LibrarySpell,
    },
    AppState,
};
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use axum_extra::extract::CookieJar;
use axum_extra::extract::Query;
use sqlx::PgPool;

use crate::{
    database::{self, classes::ClassFilters},
    ServerError,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/creatures", get(get_creatures))
        .route("/creatures/search", get(get_creatures_search))
        .route("/creatures/:id", get(get_creature_id))
        .route("/creatures", post(insert_creatures))
        .route("/items", get(get_items))
        .route("/items/search", get(get_items_search))
        .route("/items/:id", get(get_item_id))
        .route("/items", post(insert_items))
        .route("/spells", get(get_spells))
        .route("/spells/search", get(get_spells_search))
        .route("/spells/:id", get(get_spell_id))
        .route("/spells", post(insert_spells))
        .route("/hazards", get(get_hazards))
        .route("/hazards/search", get(get_hazards_search))
        .route("/hazards/:id", get(get_hazard_id))
        .route("/hazards", post(insert_hazards))
        .route("/classes", get(get_classes))
        .route("/classes/search", get(get_classes_search))
        .route("/classes", post(insert_classes))
}

async fn get_creatures(
    Query(payload): Query<CreatureFiltering>,
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, ServerError> {
    if payload.limit.unwrap_or(0) > DEFAULT_MAX_LIMIT {
        return Err(ServerError::BadRequest(format!(
            "Limit exceeds maximum of {}",
            DEFAULT_MAX_LIMIT
        )));
    }

    let creatures = database::creatures::get_creatures(&pool, &payload).await?;
    Ok(Json(creatures))
}

async fn get_creatures_search(
    Query(payload): Query<CreatureSearch>,
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, ServerError> {
    if payload.limit.unwrap_or(0) > DEFAULT_MAX_GROUP_LIMIT {
        return Err(ServerError::BadRequest(format!(
            "Limit exceeds maximum of {}",
            DEFAULT_MAX_GROUP_LIMIT
        )));
    }

    let creatures: HashMap<String, Vec<LibraryCreature>> =
        database::creatures::get_creatures_search(&pool, &payload, DEFAULT_MAX_GROUP_LIMIT)
            .await?
            .into_iter()
            .map(|(k, v)| (k, v.into_iter().map(|(_, v)| v).collect()))
            .collect();
    Ok(Json(creatures))
}

async fn get_creature_id(
    State(pool): State<PgPool>,
    Path(id): Path<u32>,
) -> Result<impl IntoResponse, ServerError> {
    let payload = CreatureFiltering::from_id(id);
    let creature = database::creatures::get_creatures(&pool, &payload)
        .await?
        .pop()
        .ok_or(ServerError::NotFound)?;
    Ok(Json(creature))
}

async fn insert_creatures(
    State(pool): State<PgPool>,
    jar: CookieJar,
    headers: HeaderMap,
    Json(payload): Json<Vec<InsertLibraryCreature>>,
) -> Result<impl IntoResponse, ServerError> {
    extract_admin_from_headers(&jar, &headers, &pool).await?;
    let mut tx = pool.begin().await?;
    database::creatures::insert_creatures(&mut tx, &payload).await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn get_items(
    Query(payload): Query<ItemFiltering>,
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, ServerError> {
    if payload.limit.unwrap_or(0) > DEFAULT_MAX_LIMIT {
        return Err(ServerError::BadRequest(format!(
            "Limit exceeds maximum of {}",
            DEFAULT_MAX_LIMIT
        )));
    }
    let items = database::items::get_items(&pool, &payload).await?;
    Ok(Json(items))
}

async fn get_items_search(
    Query(payload): Query<ItemSearch>,
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, ServerError> {
    if payload.limit.unwrap_or(0) > DEFAULT_MAX_GROUP_LIMIT {
        return Err(ServerError::BadRequest(format!(
            "Limit exceeds maximum of {}",
            DEFAULT_MAX_LIMIT
        )));
    }
    let items: HashMap<String, Vec<LibraryItem>> =
        database::items::get_items_search(&pool, &payload, DEFAULT_MAX_GROUP_LIMIT)
            .await?
            .into_iter()
            .map(|(k, v)| (k, v.into_iter().map(|(_, v)| v).collect()))
            .collect();
    Ok(Json(items))
}

async fn get_item_id(
    State(pool): State<PgPool>,
    Path(id): Path<u32>,
) -> Result<impl IntoResponse, ServerError> {
    let payload = ItemFiltering::from_id(id);
    let item = database::items::get_items(&pool, &payload)
        .await?
        .pop()
        .ok_or(ServerError::NotFound)?;
    Ok(Json(item))
}

async fn insert_items(
    State(pool): State<PgPool>,
    jar: CookieJar,
    headers: HeaderMap,
    Json(payload): Json<Vec<InsertLibraryItem>>,
) -> Result<impl IntoResponse, ServerError> {
    extract_admin_from_headers(&jar, &headers, &pool).await?;
    let mut tx = pool.begin().await?;
    database::items::insert_items(&mut tx, &payload).await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn get_spells(
    Query(payload): Query<SpellFiltering>,
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, ServerError> {
    if payload.limit.unwrap_or(0) > DEFAULT_MAX_LIMIT {
        return Err(ServerError::BadRequest(format!(
            "Limit exceeds maximum of {}",
            DEFAULT_MAX_LIMIT
        )));
    };
    let spells = database::spells::get_spells(&pool, &payload).await?;
    Ok(Json(spells))
}

async fn get_spells_search(
    Query(payload): Query<SpellSearch>,
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, ServerError> {
    if payload.limit.unwrap_or(0) > DEFAULT_MAX_GROUP_LIMIT {
        return Err(ServerError::BadRequest(format!(
            "Limit exceeds maximum of {}",
            DEFAULT_MAX_LIMIT
        )));
    };
    let spells: HashMap<String, Vec<LibrarySpell>> =
        database::spells::get_spells_search(&pool, &payload, DEFAULT_MAX_GROUP_LIMIT)
            .await?
            .into_iter()
            .map(|(k, v)| (k, v.into_iter().map(|(_, v)| v).collect()))
            .collect();
    Ok(Json(spells))
}

async fn get_spell_id(
    State(pool): State<PgPool>,
    Path(id): Path<u32>,
) -> Result<impl IntoResponse, ServerError> {
    let payload = SpellFiltering::from_id(id);
    let spell = database::spells::get_spells(&pool, &payload)
        .await?
        .pop()
        .ok_or(ServerError::NotFound)?;
    Ok(Json(spell))
}

async fn insert_spells(
    State(pool): State<PgPool>,
    jar: CookieJar,
    headers: HeaderMap,
    Json(payload): Json<Vec<InsertLibrarySpell>>,
) -> Result<impl IntoResponse, ServerError> {
    extract_admin_from_headers(&jar, &headers, &pool).await?;
    let mut tx = pool.begin().await?;
    database::spells::insert_spells(&mut tx, &payload).await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn get_hazards(
    Query(payload): Query<HazardFiltering>,
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, ServerError> {
    if payload.limit.unwrap_or(0) > DEFAULT_MAX_LIMIT {
        return Err(ServerError::BadRequest(format!(
            "Limit exceeds maximum of {}",
            DEFAULT_MAX_LIMIT
        )));
    }
    let hazards = database::hazards::get_hazards(&pool, &payload).await?;
    Ok(Json(hazards))
}

async fn get_hazards_search(
    Query(payload): Query<HazardSearch>,
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, ServerError> {
    if payload.limit.unwrap_or(0) > DEFAULT_MAX_GROUP_LIMIT {
        return Err(ServerError::BadRequest(format!(
            "Limit exceeds maximum of {}",
            DEFAULT_MAX_LIMIT
        )));
    }
    let hazards: HashMap<String, Vec<LibraryHazard>> =
        database::hazards::get_hazards_search(&pool, &payload, DEFAULT_MAX_GROUP_LIMIT)
            .await?
            .into_iter()
            .map(|(k, v)| (k, v.into_iter().map(|(_, v)| v).collect()))
            .collect();
    Ok(Json(hazards))
}

async fn get_hazard_id(
    State(pool): State<PgPool>,
    Path(id): Path<u32>,
) -> Result<impl IntoResponse, ServerError> {
    let payload = HazardFiltering::from_id(id);
    let hazard = database::hazards::get_hazards(&pool, &payload)
        .await?
        .pop()
        .ok_or(ServerError::NotFound)?;
    Ok(Json(hazard))
}

async fn insert_hazards(
    State(pool): State<PgPool>,
    jar: CookieJar,
    headers: HeaderMap,
    Json(payload): Json<Vec<InsertLibraryHazard>>,
) -> Result<impl IntoResponse, ServerError> {
    extract_admin_from_headers(&jar, &headers, &pool).await?;
    let mut tx = pool.begin().await?;
    database::hazards::insert_hazards(&mut tx, &payload).await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn get_classes(
    Query(payload): Query<ClassFilters>,
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, ServerError> {
    let classes = database::classes::get_classes(&pool, &payload).await?;
    Ok(Json(classes))
}

async fn get_classes_search(
    Query(payload): Query<ClassSearch>,
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, ServerError> {
    if payload.limit.unwrap_or(0) > DEFAULT_MAX_GROUP_LIMIT {
        return Err(ServerError::BadRequest(format!(
            "Limit exceeds maximum of {}",
            DEFAULT_MAX_LIMIT
        )));
    }

    let classes: HashMap<String, Vec<LibraryClass>> =
        database::classes::get_classes_search(&pool, &payload, DEFAULT_MAX_GROUP_LIMIT)
            .await?
            .into_iter()
            .map(|(k, v)| (k, v.into_iter().map(|(_, v)| v).collect()))
            .collect();

    Ok(Json(classes))
}

async fn insert_classes(
    State(pool): State<PgPool>,
    jar: CookieJar,
    headers: HeaderMap,
    Json(payload): Json<Vec<InsertLibraryClass>>,
) -> Result<impl IntoResponse, ServerError> {
    extract_admin_from_headers(&jar, &headers, &pool).await?;
    let mut tx = pool.begin().await?;
    database::classes::insert_classes(&mut tx, &payload).await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

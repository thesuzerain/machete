use crate::{
    auth::extract_admin_from_cookies,
    models::library::{
        classes::LibraryClass, creature::LibraryCreature, hazard::LibraryHazard, item::LibraryItem,
        spell::LibrarySpell,
    },
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use axum_extra::{
    extract::CookieJar,
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use sqlx::{PgPool, Pool};

use crate::{
    database::{
        self, classes::ClassFilters, creatures::CreatureFilters, hazards::HazardFilters,
        items::ItemFilters, spells::SpellFilters,
    },
    ServerError,
};

pub fn router() -> Router<Pool<sqlx::Postgres>> {
    Router::new()
        .route("/creatures", get(get_creatures))
        .route("/creatures/:id", get(get_creature_id))
        .route("/creatures", post(insert_creatures))
        .route("/items", get(get_items))
        .route("/items/:id", get(get_item_id))
        .route("/items", post(insert_items))
        .route("/spells", get(get_spells))
        .route("/spells/:id", get(get_spell_id))
        .route("/spells", post(insert_spells))
        .route("/hazards", get(get_hazards))
        .route("/hazards/:id", get(get_hazard_id))
        .route("/hazards", post(insert_hazards))
        .route("/classes", get(get_classes))
        .route("/classes", post(insert_classes))
}

async fn get_creatures(
    Query(payload): Query<CreatureFilters>,
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, ServerError> {
    let creatures = database::creatures::get_creatures(&pool, &payload).await?;
    Ok(Json(creatures))
}

async fn get_creature_id(
    State(pool): State<PgPool>,
    Path(id): Path<u32>,
) -> Result<impl IntoResponse, ServerError> {
    let payload = CreatureFilters::from_id(id);
    let creature = database::creatures::get_creatures(&pool, &payload)
        .await?
        .pop()
        .ok_or(ServerError::NotFound)?;
    Ok(Json(creature))
}

async fn insert_creatures(
    bearer: Option<TypedHeader<Authorization<Bearer>>>,
    State(pool): State<PgPool>,
    jar: CookieJar,
    Json(payload): Json<Vec<LibraryCreature>>,
) -> Result<impl IntoResponse, ServerError> {
    extract_admin_from_cookies(&jar, bearer, &pool).await?;
    database::creatures::insert_creatures(&pool, &payload).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn get_items(
    Query(payload): Query<ItemFilters>,
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, ServerError> {
    let items = database::items::get_items(&pool, &payload).await?;
    Ok(Json(items))
}

async fn get_item_id(
    State(pool): State<PgPool>,
    Path(id): Path<u32>,
) -> Result<impl IntoResponse, ServerError> {
    let payload = ItemFilters::from_id(id);
    let item = database::items::get_items(&pool, &payload)
        .await?
        .pop()
        .ok_or(ServerError::NotFound)?;
    Ok(Json(item))
}

async fn insert_items(
    bearer: Option<TypedHeader<Authorization<Bearer>>>,
    State(pool): State<PgPool>,
    jar: CookieJar,
    Json(payload): Json<Vec<LibraryItem>>,
) -> Result<impl IntoResponse, ServerError> {
    extract_admin_from_cookies(&jar, bearer, &pool).await?;

    database::items::insert_items(&pool, &payload).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn get_spells(
    Query(payload): Query<SpellFilters>,
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, ServerError> {
    let spells = database::spells::get_spells(&pool, &payload).await?;
    Ok(Json(spells))
}

async fn get_spell_id(
    State(pool): State<PgPool>,
    Path(id): Path<u32>,
) -> Result<impl IntoResponse, ServerError> {
    let payload = SpellFilters::from_id(id);
    let spell = database::spells::get_spells(&pool, &payload)
        .await?
        .pop()
        .ok_or(ServerError::NotFound)?;
    Ok(Json(spell))
}

async fn insert_spells(
    bearer: Option<TypedHeader<Authorization<Bearer>>>,
    State(pool): State<PgPool>,
    jar: CookieJar,
    Json(payload): Json<Vec<LibrarySpell>>,
) -> Result<impl IntoResponse, ServerError> {
    extract_admin_from_cookies(&jar, bearer, &pool).await?;

    database::spells::insert_spells(&pool, &payload).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn get_hazards(
    Query(payload): Query<HazardFilters>,
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, ServerError> {
    let hazards = database::hazards::get_hazards(&pool, &payload).await?;
    Ok(Json(hazards))
}

async fn get_hazard_id(
    State(pool): State<PgPool>,
    Path(id): Path<u32>,
) -> Result<impl IntoResponse, ServerError> {
    let payload = HazardFilters::from_id(id);
    let hazard = database::hazards::get_hazards(&pool, &payload)
        .await?
        .pop()
        .ok_or(ServerError::NotFound)?;
    Ok(Json(hazard))
}

async fn insert_hazards(
    bearer: Option<TypedHeader<Authorization<Bearer>>>,
    State(pool): State<PgPool>,
    jar: CookieJar,
    Json(payload): Json<Vec<LibraryHazard>>,
) -> Result<impl IntoResponse, ServerError> {
    extract_admin_from_cookies(&jar, bearer, &pool).await?;

    database::hazards::insert_hazards(&pool, &payload).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn get_classes(
    Query(payload): Query<ClassFilters>,
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, ServerError> {
    let classes = database::classes::get_classes(&pool, &payload).await?;
    Ok(Json(classes))
}

async fn insert_classes(
    bearer: Option<TypedHeader<Authorization<Bearer>>>,
    State(pool): State<PgPool>,
    jar: CookieJar,
    Json(payload): Json<Vec<LibraryClass>>,
) -> Result<impl IntoResponse, ServerError> {
    extract_admin_from_cookies(&jar, bearer, &pool).await?;

    database::classes::insert_classes(&pool, &payload).await?;
    Ok(StatusCode::NO_CONTENT)
}

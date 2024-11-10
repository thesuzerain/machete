use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use machete::models::library::{
    creature::{CreatureFilters, LibraryCreature},
    item::{ItemFilters, LibraryItem},
    spell::{LibrarySpell, SpellFilters},
};
use sqlx::{PgPool, Pool};

use crate::{database, ServerError};

pub fn router() -> Router<Pool<sqlx::Postgres>> {
    Router::new()
        .route("/creatures", get(get_creatures))
        .route("/creatures", post(insert_creatures))
        .route("/items", get(get_items))
        .route("/items", post(insert_items))
        .route("/spells", get(get_spells))
        .route("/spells", post(insert_spells))
}

async fn get_creatures(
    Query(payload): Query<CreatureFilters>,
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, ServerError> {
    let creatures = database::creatures::get_creatures(&pool, &payload).await?;
    Ok(Json(creatures))
}

async fn insert_creatures(
    State(pool): State<PgPool>,
    Json(payload): Json<Vec<LibraryCreature>>,
) -> Result<impl IntoResponse, ServerError> {
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

async fn insert_items(
    State(pool): State<PgPool>,
    Json(payload): Json<Vec<LibraryItem>>,
) -> Result<impl IntoResponse, ServerError> {
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

async fn insert_spells(
    State(pool): State<PgPool>,
    Json(payload): Json<Vec<LibrarySpell>>,
) -> Result<impl IntoResponse, ServerError> {
    database::spells::insert_spells(&pool, &payload).await?;
    Ok(StatusCode::NO_CONTENT)
}

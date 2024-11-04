use axum::{extract::{Query, State}, http::StatusCode, routing::{get, post}, Json, Router};
use machete::models::library::{creature::{CreatureFilters, LibraryCreature}, item::{ItemFilters, LibraryItem}, spell::{LibrarySpell, SpellFilters}};
use sqlx::{PgPool, Pool};

use crate::database;

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
) -> (StatusCode, Json<Vec<LibraryCreature>>) {
    let creatures = database::creatures::get_creatures(&pool, &payload)
        .await
        .unwrap();
    (StatusCode::OK, Json(creatures))
}

async fn insert_creatures(
    State(pool): State<PgPool>,
    Json(payload): Json<Vec<LibraryCreature>>,
) -> (StatusCode, ()) {
    println!("Adding {} creatures", payload.len());
    database::creatures::insert_creatures(&pool, &payload)
        .await
        .unwrap();
    (StatusCode::NO_CONTENT, ())
}

async fn get_items(
    Query(payload): Query<ItemFilters>,
    State(pool): State<PgPool>,
) -> (StatusCode, Json<Vec<LibraryItem>>) {
    let items = database::items::get_items(&pool, &payload).await.unwrap();
    (StatusCode::OK, Json(items))
}

async fn insert_items(
    State(pool): State<PgPool>,
    Json(payload): Json<Vec<LibraryItem>>,
) -> (StatusCode, ()) {
    database::items::insert_items(&pool, &payload).await.unwrap();
    (StatusCode::NO_CONTENT, ())
}

async fn get_spells(
    Query(payload): Query<SpellFilters>,
    State(pool): State<PgPool>,
) -> (StatusCode, Json<Vec<LibrarySpell>>) {
    let spells = database::spells::get_spells(&pool, &payload).await.unwrap();
    (StatusCode::OK, Json(spells))
}

async fn insert_spells(
    State(pool): State<PgPool>,
    Json(payload): Json<Vec<LibrarySpell>>,
) -> (StatusCode, ()) {
    database::spells::insert_spells(&pool, &payload).await.unwrap();
    (StatusCode::NO_CONTENT, ())
}

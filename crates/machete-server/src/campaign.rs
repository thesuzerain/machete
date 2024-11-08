use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use machete_core::ids::InternalId;
use sqlx::{PgPool, Pool};

use crate::{
    database::{
        self,
        characters::{CharacterFilters, InsertCharacter},
        events::{EventFilters, InsertEvent},
    },
    dummy_test_user, ServerError,
};

pub fn router() -> Router<Pool<sqlx::Postgres>> {
    Router::new()
        .route("/", get(get_campaigns))
        .route("/", post(insert_campaign))
        .route("/:id/characters", get(get_characters))
        .route("/:id/characters", post(insert_characters))
        .route("/:id/events", get(get_events))
        .route("/:id/events", post(insert_events))
}

async fn get_campaigns(State(pool): State<PgPool>) -> Result<impl IntoResponse, ServerError> {
    let campaigns = database::campaigns::get_campaign(&pool, dummy_test_user()).await?;
    Ok(Json(campaigns))
}

#[derive(serde::Deserialize)]
pub struct InsertCampaign {
    pub name: String,
}
async fn insert_campaign(
    State(pool): State<PgPool>,
    Json(campaign): Json<InsertCampaign>,
) -> Result<impl IntoResponse, ServerError> {
    database::campaigns::insert_campaign(&pool, &campaign.name, dummy_test_user()).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn get_characters(
    Query(filters): Query<CharacterFilters>,
    Path(id): Path<InternalId>,
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, ServerError> {
    let characters =
        database::characters::get_characters(&pool, dummy_test_user(), id, &filters).await?;
    Ok(Json(characters))
}

async fn insert_characters(
    State(pool): State<PgPool>,
    Path(id): Path<InternalId>,
    Json(characters): Json<Vec<InsertCharacter>>,
) -> Result<impl IntoResponse, ServerError> {
    database::characters::insert_characters(&pool, dummy_test_user(), id, &characters).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn get_events(
    Query(filters): Query<EventFilters>,
    Path(id): Path<InternalId>,
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, ServerError> {
    let events = database::events::get_campaigns(&pool, dummy_test_user(), id, &filters).await?;
    Ok(Json(events))
}

async fn insert_events(
    State(pool): State<PgPool>,
    Path(id): Path<InternalId>,
    Json(events): Json<Vec<InsertEvent>>,
) -> Result<impl IntoResponse, ServerError> {
    database::events::insert_events(&pool, dummy_test_user(), id, &events).await?;
    Ok(StatusCode::NO_CONTENT)
}

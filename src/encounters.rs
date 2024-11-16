use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, patch, post},
    Json, Router,
};
use crate::models::ids::InternalId;


use sqlx::{PgPool, Pool};

use crate::{
    database::{
        self,
        encounters::{EncounterFilters, InsertEncounter, ModifyEncounter},
    },
    dummy_test_user, ServerError,
};

pub fn router() -> Router<Pool<sqlx::Postgres>> {
    Router::new()
        .route("/", get(get_encounters))
        .route("/", post(insert_encounter))
        .route("/draft", get(get_encounter_draft))
        .route("/draft", post(insert_encounter_draft))
        .route("/draft", delete(clear_encounter_draft))
        .route("/:id", patch(edit_encounter))
        .route("/:id/", delete(delete_encounter))
}

async fn get_encounters(
    Query(filters): Query<EncounterFilters>,
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, ServerError> {
    let encounters =
        database::encounters::get_encounters(&pool, dummy_test_user(), &filters).await?;
    Ok(Json(encounters))
}

async fn insert_encounter(
    State(pool): State<PgPool>,
    Json(events): Json<Vec<InsertEncounter>>,
) -> Result<impl IntoResponse, ServerError> {
    database::encounters::insert_encounters(&pool, dummy_test_user(), &events).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn edit_encounter(
    State(pool): State<PgPool>,
    Path(event_id): Path<InternalId>,
    Json(event): Json<ModifyEncounter>,
) -> Result<impl IntoResponse, ServerError> {
    database::encounters::edit_encounter(&pool, dummy_test_user(), event_id, &event).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn delete_encounter(
    State(pool): State<PgPool>,
    Path(event_id): Path<InternalId>,
) -> Result<impl IntoResponse, ServerError> {
    database::encounters::delete_encounters(&pool, dummy_test_user(), &vec![event_id]).await?;
    Ok(StatusCode::NO_CONTENT)
}


async fn get_encounter_draft(
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, ServerError> {
    let encounter = database::encounters::get_encounter_draft(&pool, dummy_test_user()).await?;
    Ok(Json(encounter))
}

async fn insert_encounter_draft(
    State(pool): State<PgPool>,
    Json(event): Json<InsertEncounter>,
) -> Result<impl IntoResponse, ServerError> {
    database::encounters::insert_encounter_draft(&pool, dummy_test_user(), &event).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn clear_encounter_draft(
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, ServerError> {
    database::encounters::clear_encounter_draft(&pool, dummy_test_user()).await?;
    Ok(StatusCode::NO_CONTENT)
}

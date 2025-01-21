use crate::{auth::extract_user_from_cookies, models::ids::InternalId, AppState};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};

use axum_extra::extract::CookieJar;
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
        .route("/", get(get_encounters))
        .route("/", post(insert_encounter))
        .route("/draft", get(get_encounter_draft))
        .route("/draft", post(insert_encounter_draft))
        .route("/draft", delete(clear_encounter_draft))
        .route("/:id", put(edit_encounter))
        .route("/:id/", delete(delete_encounter))
}

async fn get_encounters(
    Query(filters): Query<EncounterFilters>,
    jar: CookieJar,
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, ServerError> {
    let user = extract_user_from_cookies(&jar, &pool).await?;

    let encounters = database::encounters::get_encounters(&pool, user.id, &filters).await?;
    Ok(Json(encounters))
}

async fn insert_encounter(
    State(pool): State<PgPool>,
    jar: CookieJar,
    Json(encounters): Json<Vec<InsertEncounter>>,
) -> Result<impl IntoResponse, ServerError> {
    let user = extract_user_from_cookies(&jar, &pool).await?;

    database::encounters::insert_encounters(&pool, user.id, &encounters).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn edit_encounter(
    State(pool): State<PgPool>,
    jar: CookieJar,
    Path(encounter_id): Path<InternalId>,
    Json(encounter): Json<ModifyEncounter>,
) -> Result<impl IntoResponse, ServerError> {
    let user = extract_user_from_cookies(&jar, &pool).await?;

    // Check if user has access to the encounter
    if database::encounters::get_owned_encounter_ids(&pool, &[encounter_id], user.id)
        .await?
        .is_empty()
    {
        return Err(ServerError::NotFound);
    }

    database::encounters::edit_encounter(&pool, encounter_id, &encounter).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn delete_encounter(
    State(pool): State<PgPool>,
    jar: CookieJar,
    Path(encounter_id): Path<InternalId>,
) -> Result<impl IntoResponse, ServerError> {
    let user = extract_user_from_cookies(&jar, &pool).await?;

    // Check if user has access to the encounter
    if database::encounters::get_owned_encounter_ids(&pool, &[encounter_id], user.id)
        .await?
        .is_empty()
    {
        return Err(ServerError::NotFound);
    }

    database::encounters::delete_encounters(&pool, &[encounter_id]).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn get_encounter_draft(
    State(pool): State<PgPool>,
    jar: CookieJar,
) -> Result<impl IntoResponse, ServerError> {
    let user = extract_user_from_cookies(&jar, &pool).await?;

    let encounter = database::encounters::get_encounter_draft(&pool, user.id).await?;
    Ok(Json(encounter))
}

async fn insert_encounter_draft(
    State(pool): State<PgPool>,
    jar: CookieJar,
    Json(event): Json<InsertEncounter>,
) -> Result<impl IntoResponse, ServerError> {
    let user = extract_user_from_cookies(&jar, &pool).await?;

    database::encounters::insert_user_encounter_draft(&pool, user.id, &event).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn clear_encounter_draft(
    State(pool): State<PgPool>,
    jar: CookieJar,
) -> Result<impl IntoResponse, ServerError> {
    let user = extract_user_from_cookies(&jar, &pool).await?;

    database::encounters::clear_user_encounter_draft(&pool, user.id).await?;
    Ok(StatusCode::NO_CONTENT)
}

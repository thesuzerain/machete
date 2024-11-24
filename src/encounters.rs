use crate::{auth::extract_user_from_cookies, models::ids::InternalId};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, patch, post},
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
    jar: CookieJar,
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, ServerError> {
    let user = extract_user_from_cookies(&jar, &pool).await?;

    let encounters =
        database::encounters::get_encounters(&pool, user.id, &filters).await?;
    Ok(Json(encounters))
}

async fn insert_encounter(
    State(pool): State<PgPool>,
    jar: CookieJar,
    Json(events): Json<Vec<InsertEncounter>>,
) -> Result<impl IntoResponse, ServerError> {
    let user = extract_user_from_cookies(&jar, &pool).await?;

    database::encounters::insert_encounters(&pool, user.id, &events).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn edit_encounter(
    State(pool): State<PgPool>,
    jar: CookieJar,
    Path(event_id): Path<InternalId>,
    Json(event): Json<ModifyEncounter>,
) -> Result<impl IntoResponse, ServerError> {
    let user = extract_user_from_cookies(&jar, &pool).await?;

    database::encounters::edit_encounter(&pool, user.id, event_id, &event).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn delete_encounter(
    State(pool): State<PgPool>,
    jar: CookieJar,
    Path(event_id): Path<InternalId>,
) -> Result<impl IntoResponse, ServerError> {
    let user = extract_user_from_cookies(&jar, &pool).await?;

    database::encounters::delete_encounters(&pool, user.id, &vec![event_id]).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn get_encounter_draft(State(pool): State<PgPool>,     jar: CookieJar) -> Result<impl IntoResponse, ServerError> {
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

    database::encounters::insert_encounter_draft(&pool, user.id, &event).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn clear_encounter_draft(
    State(pool): State<PgPool>,
    jar: CookieJar,
) -> Result<impl IntoResponse, ServerError> {
    let user = extract_user_from_cookies(&jar, &pool).await?;

    database::encounters::clear_encounter_draft(&pool, user.id).await?;
    Ok(StatusCode::NO_CONTENT)
}

use crate::{auth::extract_user_from_cookies, models::ids::InternalId, AppState};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, patch, post},
    Json, Router,
};

use axum_extra::extract::CookieJar;
use sqlx::PgPool;

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
        .route("/:id", get(get_encounter))
        .route("/:id", patch(edit_encounter))
        .route("/:id", delete(delete_encounter))
        .route("/:id/session", delete(delete_session_link))
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

async fn get_encounter(
    State(pool): State<PgPool>,
    jar: CookieJar,
    Path(encounter_id): Path<InternalId>,
) -> Result<impl IntoResponse, ServerError> {
    let user = extract_user_from_cookies(&jar, &pool).await?;

    let filters = EncounterFilters::from_ids(&[encounter_id]);
    let encounters = database::encounters::get_encounters(&pool, user.id, &filters).await?;
    if encounters.is_empty() {
        return Err(ServerError::NotFound);
    }

    Ok(Json(encounters[0].clone()))
}

async fn insert_encounter(
    State(pool): State<PgPool>,
    jar: CookieJar,
    Json(encounters): Json<Vec<InsertEncounter>>,
) -> Result<impl IntoResponse, ServerError> {
    let user = extract_user_from_cookies(&jar, &pool).await?;
    let mut tx = pool.begin().await?;
    let ids = database::encounters::insert_encounters(&mut tx, user.id, &encounters).await?;
    let encounters =
        database::encounters::get_encounters(&mut *tx, user.id, &EncounterFilters::from_ids(&ids))
            .await?;
    tx.commit().await?;
    Ok(Json(encounters))
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

    let mut tx = pool.begin().await?;
    database::encounters::edit_encounter(&mut tx, encounter_id, user.id, &encounter).await?;
    tx.commit().await?;

    Ok(StatusCode::NO_CONTENT)
}

async fn delete_encounter(
    State(pool): State<PgPool>,
    jar: CookieJar,
    Path(encounter_id): Path<InternalId>,
) -> Result<impl IntoResponse, ServerError> {
    let user = extract_user_from_cookies(&jar, &pool).await?;
    println!("Deleting encounter: {}", encounter_id);
    // Check if user has access to the encounter
    if database::encounters::get_owned_encounter_ids(&pool, &[encounter_id], user.id)
        .await?
        .is_empty()
    {
        return Err(ServerError::NotFound);
    }

    let mut tx = pool.begin().await?;
    database::encounters::delete_encounters(&mut tx, &[encounter_id]).await?;
    tx.commit().await?;

    Ok(StatusCode::NO_CONTENT)
}

async fn delete_session_link(
    State(pool): State<PgPool>,
    jar: CookieJar,
    Path(encounter_id): Path<InternalId>,
) -> Result<impl IntoResponse, ServerError> {
    let user = extract_user_from_cookies(&jar, &pool).await?;

    println!("Deleting session link for encounter: {}", encounter_id);
    // Check if user has access to the encounter
    if database::encounters::get_owned_encounter_ids(&pool, &[encounter_id], user.id)
        .await?
        .is_empty()
    {
        return Err(ServerError::NotFound);
    }

    // Unlink the encounter from the session
    let mut tx = pool.begin().await?;
    database::sessions::unlink_encounter_from_session(&mut tx, encounter_id).await?;
    tx.commit().await?;

    Ok(StatusCode::NO_CONTENT)
}

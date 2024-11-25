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
        campaigns::InsertCampaign,
        characters::{CharacterFilters, InsertCharacter, ModifyCharacter},
        events::{EditEvent, EventFilters, InsertEvent},
        logs::{InsertLog, LogFilters},
    },
    ServerError,
};

pub fn router() -> Router<Pool<sqlx::Postgres>> {
    Router::new()
        .route("/", get(get_campaigns))
        .route("/", post(insert_campaign))
        .route("/:id/characters", get(get_characters))
        .route("/:id/characters", post(insert_characters))
        .route("/:id/characters/:id", patch(edit_character))
        .route("/:id/events", get(get_events))
        .route("/:id/events", post(insert_events))
        .route("/:id/events/:id", patch(edit_event))
        .route("/:id/events/:id", delete(delete_event))
        .route("/:id/events", delete(delete_events))
        .route("/:id/logs", get(get_logs))
        .route("/:id/logs", post(insert_log))
        .route("/:id/logs/:id", patch(edit_log))
        .route("/:id/logs/:id", delete(delete_log))
}

async fn get_campaigns(
    State(pool): State<PgPool>,
    jar: CookieJar,
) -> Result<impl IntoResponse, ServerError> {
    let user = extract_user_from_cookies(&jar, &pool).await?;
    let campaigns = database::campaigns::get_campaign(&pool, user.id).await?;
    Ok(Json(campaigns))
}

async fn insert_campaign(
    State(pool): State<PgPool>,
    jar: CookieJar,
    Json(campaign): Json<InsertCampaign>,
) -> Result<impl IntoResponse, ServerError> {
    let user = extract_user_from_cookies(&jar, &pool).await?;
    database::campaigns::insert_campaign(&pool, &campaign.name, user.id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn get_characters(
    Query(filters): Query<CharacterFilters>,
    jar: CookieJar,
    Path(id): Path<InternalId>,
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, ServerError> {
    let user = extract_user_from_cookies(&jar, &pool).await?;

    // Check if user has access to the campaign
    if database::campaigns::get_owned_campaign_id(&pool, id, user.id)
        .await?
        .is_none()
    {
        return Err(ServerError::NotFound);
    }

    let characters = database::characters::get_characters(&pool, user.id, id, &filters).await?;
    Ok(Json(characters))
}

async fn insert_characters(
    State(pool): State<PgPool>,
    jar: CookieJar,
    Path(id): Path<InternalId>,
    Json(characters): Json<Vec<InsertCharacter>>,
) -> Result<impl IntoResponse, ServerError> {
    let user = extract_user_from_cookies(&jar, &pool).await?;

    // Check if user has access to the campaign
    if database::campaigns::get_owned_campaign_id(&pool, id, user.id)
        .await?
        .is_none()
    {
        return Err(ServerError::NotFound);
    }

    database::characters::insert_characters(&pool, id, &characters).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn edit_character(
    State(pool): State<PgPool>,
    jar: CookieJar,
    Path((_, character_id)): Path<(InternalId, InternalId)>,
    Json(character): Json<ModifyCharacter>,
) -> Result<impl IntoResponse, ServerError> {
    let user = extract_user_from_cookies(&jar, &pool).await?;

    // Check if user has access to the chracter
    if database::characters::get_chracter_id(&pool, character_id, user.id)
        .await?
        .is_none()
    {
        return Err(ServerError::NotFound);
    }

    database::characters::edit_character(&pool, character_id, &character).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn get_logs(
    Query(filters): Query<LogFilters>,
    jar: CookieJar,
    Path(id): Path<InternalId>,
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, ServerError> {
    let user = extract_user_from_cookies(&jar, &pool).await?;

    let logs = database::logs::get_logs(&pool, user.id, id, &filters).await?;
    Ok(Json(logs))
}

async fn insert_log(
    State(pool): State<PgPool>,
    jar: CookieJar,
    Path(id): Path<InternalId>,
    Json(log): Json<InsertLog>,
) -> Result<impl IntoResponse, ServerError> {
    let user = extract_user_from_cookies(&jar, &pool).await?;

    // Check if user has access to the campaign
    if database::campaigns::get_owned_campaign_id(&pool, id, user.id)
        .await?
        .is_none()
    {
        return Err(ServerError::NotFound);
    }

    database::logs::insert_log(&pool, id, &log).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn edit_log(
    State(pool): State<PgPool>,
    jar: CookieJar,
    Path((_, log_id)): Path<(InternalId, InternalId)>,
    Json(log): Json<InsertLog>,
) -> Result<impl IntoResponse, ServerError> {
    let user = extract_user_from_cookies(&jar, &pool).await?;

    // Check if user has access to the logs
    if !database::logs::get_owned_logs_ids(&pool, &[log_id], user.id)
        .await?
        .is_empty()
    {
        return Err(ServerError::NotFound);
    }

    database::logs::edit_log(&pool, user.id, log_id, &log).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn delete_log(
    State(pool): State<PgPool>,
    jar: CookieJar,
    Path((_, log_id)): Path<(InternalId, InternalId)>,
) -> Result<impl IntoResponse, ServerError> {
    let user = extract_user_from_cookies(&jar, &pool).await?;

    // Check if user has access to the logs
    if !database::logs::get_owned_logs_ids(&pool, &[log_id], user.id)
        .await?
        .is_empty()
    {
        return Err(ServerError::NotFound);
    }

    database::logs::delete_log(&pool, user.id, log_id)
        .await
        .unwrap();
    Ok(StatusCode::NO_CONTENT)
}

async fn get_events(
    Query(filters): Query<EventFilters>,
    Path(id): Path<InternalId>,
    State(pool): State<PgPool>,
    jar: CookieJar,
) -> Result<impl IntoResponse, ServerError> {
    let user = extract_user_from_cookies(&jar, &pool).await?;

    let events = database::events::get_events(&pool, user.id, id, &filters).await?;
    Ok(Json(events))
}

async fn insert_events(
    State(pool): State<PgPool>,
    jar: CookieJar,
    Path(id): Path<InternalId>,
    Json(events): Json<Vec<InsertEvent>>,
) -> Result<impl IntoResponse, ServerError> {
    let user = extract_user_from_cookies(&jar, &pool).await?;

    // Check if user has access to the campaign
    if database::campaigns::get_owned_campaign_id(&pool, id, user.id)
        .await?
        .is_none()
    {
        return Err(ServerError::NotFound);
    }

    database::events::insert_events(&pool, id, None, &events).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn edit_event(
    State(pool): State<PgPool>,
    jar: CookieJar,
    Path((_, event_id)): Path<(InternalId, InternalId)>,
    Json(event): Json<EditEvent>,
) -> Result<impl IntoResponse, ServerError> {
    let user = extract_user_from_cookies(&jar, &pool).await?;

    // Check if user has access to the event
    if !database::events::get_owned_events_ids(&pool, &[event_id], user.id)
        .await?
        .is_empty()
    {
        return Err(ServerError::NotFound);
    }

    database::events::edit_event(&pool, user.id, event_id, &event).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn delete_event(
    State(pool): State<PgPool>,
    jar: CookieJar,
    Path((_, event_id)): Path<(InternalId, InternalId)>,
) -> Result<impl IntoResponse, ServerError> {
    let user = extract_user_from_cookies(&jar, &pool).await?;

    // Check if user has access to the event
    if !database::events::get_owned_events_ids(&pool, &[event_id], user.id)
        .await?
        .is_empty()
    {
        return Err(ServerError::NotFound);
    }

    database::events::delete_events(&pool, user.id, &[event_id]).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn delete_events(
    State(pool): State<PgPool>,
    jar: CookieJar,
    Json(ids): Json<Vec<InternalId>>,
) -> Result<impl IntoResponse, ServerError> {
    let user = extract_user_from_cookies(&jar, &pool).await?;

    // Check if user has access to the events
    if database::events::get_events_ids(&pool, &ids).await?.len() != ids.len() {
        return Err(ServerError::NotFound);
    }

    database::events::delete_events(&pool, user.id, &ids).await?;
    Ok(StatusCode::NO_CONTENT)
}

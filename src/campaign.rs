use std::collections::HashMap;

use crate::{
    auth::extract_user_from_cookies,
    database::{
        campaigns::ModifyCampaign,
        import::ImportCampaign,
        sessions::{InsertSession, LinkEncounterSession, ModifySession, UpdateCharacterSessions},
    },
    models::ids::InternalId,
    AppState,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, patch, post, put},
    Json, Router,
};

use axum_extra::extract::CookieJar;
use sqlx::PgPool;

use crate::{
    database::{
        self,
        campaigns::InsertCampaign,
        characters::{CharacterFilters, InsertCharacter, ModifyCharacter},
    },
    ServerError,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_campaigns))
        .route("/", post(insert_campaign))
        .route("/:id", patch(edit_campaign))
        .route("/:id", delete(delete_campaign))
        .route("/import", post(import_campaign)) // TODO: Does this need to differ from generic 'insert'?
        .route("/:id/export", get(export_campaign))
        .route("/:id/stats", get(get_stats))
        .route("/:id/characters", get(get_characters))
        .route("/:id/characters", post(insert_characters))
        .route("/:id/characters/:id", put(edit_character))
        .route("/:id/characters/:id", delete(delete_character))
        .route("/:id/sessions", get(get_sessions))
        .route("/:id/sessions", post(insert_sessions))
        .route("/:id/sessions", patch(edit_sessions))
        .route("/:id/sessions/:id", delete(delete_session))
        .route(
            "/:id/sessions/:session_id/encounters",
            post(link_sessions_encounters),
        )
        .route(
            "/:id/sessions/:session_id/encounters",
            patch(update_link_session_encounters),
        )
        .route(
            "/:id/sessions/:session_id/encounters/:encounter_id",
            delete(unlink_session_encounters),
        )
}

async fn get_campaigns(
    State(pool): State<PgPool>,
    jar: CookieJar,
) -> Result<impl IntoResponse, ServerError> {
    let user = extract_user_from_cookies(&jar, &pool).await?;
    let campaigns = database::campaigns::get_campaigns_owner(&pool, user.id).await?;
    Ok(Json(campaigns))
}

async fn edit_campaign(
    State(pool): State<PgPool>,
    jar: CookieJar,
    Path(id): Path<InternalId>,
    Json(campaign): Json<ModifyCampaign>,
) -> Result<impl IntoResponse, ServerError> {
    let user = extract_user_from_cookies(&jar, &pool).await?;

    // Check if user has access to the campaign
    if database::campaigns::get_owned_campaign_id(&pool, id, user.id)
        .await?
        .is_none()
    {
        return Err(ServerError::NotFound);
    }

    let mut tx = pool.begin().await?;
    database::campaigns::edit_campaign(&mut tx, id, &campaign).await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn delete_campaign(
    State(pool): State<PgPool>,
    jar: CookieJar,
    Path(id): Path<InternalId>,
) -> Result<impl IntoResponse, ServerError> {
    let user = extract_user_from_cookies(&jar, &pool).await?;

    // Check if user has access to the campaign
    if database::campaigns::get_owned_campaign_id(&pool, id, user.id)
        .await?
        .is_none()
    {
        return Err(ServerError::NotFound);
    }

    let mut tx = pool.begin().await?;
    database::campaigns::delete_campaign(&mut tx, id).await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn insert_campaign(
    State(pool): State<PgPool>,
    jar: CookieJar,
    Json(campaign): Json<InsertCampaign>,
) -> Result<impl IntoResponse, ServerError> {
    let user = extract_user_from_cookies(&jar, &pool).await?;
    let mut tx = pool.begin().await?;
    let campaign_id =
        database::campaigns::insert_campaign(&mut tx, &campaign, true, user.id).await?;
    let owned_campaigns = database::campaigns::get_campaigns_owner(&mut *tx, user.id).await?;
    tx.commit().await?;

    let campaign = owned_campaigns
        .into_iter()
        .find(|c| c.id == campaign_id)
        .ok_or(ServerError::NotFound)?;
    Ok(Json(campaign))
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

    let mut tx = pool.begin().await?;
    database::characters::insert_characters(&mut tx, id, &characters).await?;
    tx.commit().await?;

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

    let mut tx = pool.begin().await?;
    database::characters::edit_character(&mut tx, character_id, &character).await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn delete_character(
    State(pool): State<PgPool>,
    jar: CookieJar,
    Path((_, character_id)): Path<(InternalId, InternalId)>,
) -> Result<impl IntoResponse, ServerError> {
    let user = extract_user_from_cookies(&jar, &pool).await?;

    // Check if user has access to the chracter
    if database::characters::get_chracter_id(&pool, character_id, user.id)
        .await?
        .is_none()
    {
        return Err(ServerError::NotFound);
    }
    let mut tx = pool.begin().await?;
    database::characters::delete_character(&mut tx, character_id).await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn get_sessions(
    State(pool): State<PgPool>,
    jar: CookieJar,
    Path(id): Path<InternalId>,
) -> Result<impl IntoResponse, ServerError> {
    let user = extract_user_from_cookies(&jar, &pool).await?;

    let sessions = database::sessions::get_sessions(&pool, user.id, id).await?;
    Ok(Json(sessions))
}

async fn insert_sessions(
    State(pool): State<PgPool>,
    jar: CookieJar,
    Path(id): Path<InternalId>,
    Json(session): Json<Vec<InsertSession>>,
) -> Result<impl IntoResponse, ServerError> {
    let user = extract_user_from_cookies(&jar, &pool).await?;

    // Check if user has access to the campaign
    if database::campaigns::get_owned_campaign_id(&pool, id, user.id)
        .await?
        .is_none()
    {
        return Err(ServerError::NotFound);
    }

    let mut tx = pool.begin().await?;
    database::sessions::insert_sessions(&mut tx, id, &session).await?;
    tx.commit().await?;

    Ok(StatusCode::NO_CONTENT)
}

// TODO: Is PATCH the most reasonable way to do this? (w.r.t a hashmap for a bulk update)
async fn edit_sessions(
    State(pool): State<PgPool>,
    jar: CookieJar,
    Json(session): Json<HashMap<InternalId, ModifySession>>,
) -> Result<impl IntoResponse, ServerError> {
    let user = extract_user_from_cookies(&jar, &pool).await?;

    // Check if user has access to the session
    let session_ids = session.keys().cloned().collect::<Vec<_>>();
    if database::sessions::get_owned_session_ids(&pool, &session_ids, user.id)
        .await?
        .is_empty()
    {
        return Err(ServerError::NotFound);
    }

    let mut tx = pool.begin().await?;
    database::sessions::update_sessions(&mut tx, &session).await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn delete_session(
    State(pool): State<PgPool>,
    jar: CookieJar,
    Path((_, session_id)): Path<(InternalId, InternalId)>,
) -> Result<impl IntoResponse, ServerError> {
    let user = extract_user_from_cookies(&jar, &pool).await?;

    // Check if user has access to the session
    if database::sessions::get_owned_session_ids(&pool, &[session_id], user.id)
        .await?
        .is_empty()
    {
        return Err(ServerError::NotFound);
    }

    let mut tx = pool.begin().await?;
    database::sessions::delete_session(&mut tx, session_id).await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn link_sessions_encounters(
    State(pool): State<PgPool>,
    jar: CookieJar,
    Path((_campaign_id, session_id)): Path<(InternalId, InternalId)>,
    Json(link): Json<LinkEncounterSession>,
) -> Result<impl IntoResponse, ServerError> {
    let user = extract_user_from_cookies(&jar, &pool).await?;

    // Check if user has access to the session
    if database::sessions::get_owned_session_ids(&pool, &[session_id], user.id)
        .await?
        .is_empty()
    {
        return Err(ServerError::NotFound);
    }

    let mut tx = pool.begin().await?;

    // Unlink first
    database::sessions::unlink_encounter_from_session(&mut tx, link.encounter_id).await?;
    database::sessions::link_encounter_to_session(&mut tx, link.encounter_id, session_id).await?;
    tx.commit().await?;

    Ok(StatusCode::NO_CONTENT)
}

async fn update_link_session_encounters(
    State(pool): State<PgPool>,
    jar: CookieJar,
    Path((_campaign_id, session_id)): Path<(InternalId, InternalId)>,
    Json(session): Json<UpdateCharacterSessions>,
) -> Result<impl IntoResponse, ServerError> {
    let user = extract_user_from_cookies(&jar, &pool).await?;

    // Check if user has access to the session
    if database::sessions::get_owned_session_ids(&pool, &[session_id], user.id)
        .await?
        .is_empty()
    {
        return Err(ServerError::NotFound);
    }

    let mut tx = pool.begin().await?;

    database::sessions::edit_encounter_session_character_assignments(&mut tx, session_id, &session)
        .await?;

    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn unlink_session_encounters(
    State(pool): State<PgPool>,
    jar: CookieJar,
    Path((_campaign_id, session_id, encounter_id)): Path<(InternalId, InternalId, InternalId)>,
) -> Result<impl IntoResponse, ServerError> {
    let user = extract_user_from_cookies(&jar, &pool).await?;

    // Check if user has access to the session
    if database::sessions::get_owned_session_ids(&pool, &[session_id], user.id)
        .await?
        .is_empty()
    {
        return Err(ServerError::NotFound);
    }

    let mut tx = pool.begin().await?;
    database::sessions::unlink_encounter_from_session(&mut tx, encounter_id).await?;
    tx.commit().await?;

    Ok(StatusCode::NO_CONTENT)
}

async fn import_campaign(
    State(pool): State<PgPool>,
    jar: CookieJar,
    Json(campaign): Json<ImportCampaign>,
) -> Result<impl IntoResponse, ServerError> {
    let user = extract_user_from_cookies(&jar, &pool).await?;
    let mut tx = pool.begin().await?;

    let campaign_id = database::import::import_with_functions(campaign, &mut tx, user.id).await?;
    let owned_campaigns = database::campaigns::get_campaigns_owner(&mut *tx, user.id).await?;
    tx.commit().await?;

    let campaign = owned_campaigns
        .into_iter()
        .find(|c| c.id == campaign_id)
        .ok_or(ServerError::NotFound)?;
    Ok(Json(campaign))
}

async fn export_campaign(
    State(pool): State<PgPool>,
    jar: CookieJar,
    Path(id): Path<InternalId>,
) -> Result<impl IntoResponse, ServerError> {
    let user = extract_user_from_cookies(&jar, &pool).await?;

    // Ensure owned
    if database::campaigns::get_owned_campaign_id(&pool, id, user.id)
        .await?
        .is_none()
    {
        return Err(ServerError::NotFound);
    }

    let campaign = database::import::export(id, &pool, user.id).await?;

    Ok(Json(campaign))
}

async fn get_stats(
    State(pool): State<PgPool>,
    jar: CookieJar,
    Path(id): Path<InternalId>,
) -> Result<impl IntoResponse, ServerError> {
    let user = extract_user_from_cookies(&jar, &pool).await?;

    // Ensure owned
    if database::campaigns::get_owned_campaign_id(&pool, id, user.id)
        .await?
        .is_none()
    {
        return Err(ServerError::NotFound);
    }

    let stats = database::stats::get_campaign_stats(&pool, user.id, id).await?;
    Ok(Json(stats))
}

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, patch, post},
    Json, Router,
};
use machete::models::{
    campaign::CampaignPartial,
    characters::Character,
    events::{Event, EventGroup, EventType},
};
use machete_core::ids::InternalId;
use sqlx::{PgPool, Pool};

use crate::{
    database::{
        self,
        campaigns::InsertCampaign,
        characters::{CharacterFilters, InsertCharacter, ModifyCharacter},
        encounters::{EncounterFilters, InsertEncounter, ModifyEncounter},
        events::{EventFilters, InsertEvent},
        logs::{InsertLog, LogFilters},
    },
    dummy_test_user, ServerError,
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
        .route("/:id/encounters", get(get_encounters))
        .route("/:id/encounters", post(insert_encounter))
        .route("/:id/encounters/:id", patch(edit_encounter))
        .route("/:id/encounters/:id", delete(delete_encounter))
}

async fn get_campaigns(State(pool): State<PgPool>) -> Result<impl IntoResponse, ServerError> {
    let campaigns = database::campaigns::get_campaign(&pool, dummy_test_user()).await?;
    Ok(Json(campaigns))
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

async fn edit_character(
    State(pool): State<PgPool>,
    Path((campaign_id, character_id)): Path<(InternalId, InternalId)>,
    Json(character): Json<ModifyCharacter>,
) -> Result<impl IntoResponse, ServerError> {
    database::characters::edit_character(&pool, character_id, dummy_test_user(), &character)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn get_logs(
    Query(filters): Query<LogFilters>,
    Path(id): Path<InternalId>,
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, ServerError> {
    let logs = database::logs::get_logs(&pool, dummy_test_user(), id, &filters).await?;
    Ok(Json(logs))
}

async fn insert_log(
    State(pool): State<PgPool>,
    Path(id): Path<InternalId>,
    Json(log): Json<InsertLog>,
) -> Result<impl IntoResponse, ServerError> {
    database::logs::insert_log(&pool, dummy_test_user(), id, &log).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn edit_log(
    State(pool): State<PgPool>,
    Path((campaign_id, log_id)): Path<(InternalId, InternalId)>,
    Json(log): Json<InsertLog>,
) -> Result<impl IntoResponse, ServerError> {
    database::logs::edit_log(&pool, dummy_test_user(), log_id, &log).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn delete_log(
    State(pool): State<PgPool>,
    Path((campaign_id, log_id)): Path<(InternalId, InternalId)>,
) -> Result<impl IntoResponse, ServerError> {
    database::logs::delete_log(&pool, dummy_test_user(), log_id)
        .await
        .unwrap();
    Ok(StatusCode::NO_CONTENT)
}

async fn get_events(
    Query(filters): Query<EventFilters>,
    Path(id): Path<InternalId>,
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, ServerError> {
    let events = database::events::get_events(&pool, dummy_test_user(), id, &filters).await?;
    Ok(Json(events))
}

async fn insert_events(
    State(pool): State<PgPool>,
    Path(id): Path<InternalId>,
    Json(events): Json<Vec<InsertEvent>>,
) -> Result<impl IntoResponse, ServerError> {
    database::events::insert_events(&pool, dummy_test_user(), id, None, &events).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn edit_event(
    State(pool): State<PgPool>,
    Path((campaign_id, event_id)): Path<(InternalId, InternalId)>,
    Json(event): Json<EventType>,
) -> Result<impl IntoResponse, ServerError> {
    database::events::edit_event(&pool, dummy_test_user(), event_id, &event).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn delete_event(
    State(pool): State<PgPool>,
    Path((campaign_id, event_id)): Path<(InternalId, InternalId)>,
) -> Result<impl IntoResponse, ServerError> {
    database::events::delete_events(&pool, dummy_test_user(), &vec![event_id]).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn delete_events(
    State(pool): State<PgPool>,
    Json(ids): Json<Vec<InternalId>>,
) -> Result<impl IntoResponse, ServerError> {
    database::events::delete_events(&pool, dummy_test_user(), &ids).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn get_encounters(
    Query(filters): Query<EncounterFilters>,
    Path(id): Path<InternalId>,
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, ServerError> {
    let encounters =
        database::encounters::get_encounters(&pool, dummy_test_user(), id, &filters).await?;
    Ok(Json(encounters))
}

async fn insert_encounter(
    State(pool): State<PgPool>,
    Path(id): Path<InternalId>,
    Json(events): Json<Vec<InsertEncounter>>,
) -> Result<impl IntoResponse, ServerError> {
    database::encounters::insert_encounters(&pool, dummy_test_user(), &events).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn edit_encounter(
    State(pool): State<PgPool>,
    Path((campaign_id, event_id)): Path<(InternalId, InternalId)>,
    Json(event): Json<ModifyEncounter>,
) -> Result<impl IntoResponse, ServerError> {
    database::encounters::edit_encounter(&pool, dummy_test_user(), event_id, &event).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn delete_encounter(
    State(pool): State<PgPool>,
    Path((campaign_id, event_id)): Path<(InternalId, InternalId)>,
) -> Result<impl IntoResponse, ServerError> {
    database::encounters::delete_encounters(&pool, dummy_test_user(), &vec![event_id]).await?;
    Ok(StatusCode::NO_CONTENT)
}

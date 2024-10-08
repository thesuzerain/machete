use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use machete::models::library::{
    creature::{CreatureFilters, LibraryCreature},
    item::{ItemFilters, LibraryItem},
    spell::{LibrarySpell, SpellFilters},
};

use sqlx::PgPool;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;

pub mod database;

pub async fn run_server() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .parse_default_env()
        .init();

    let pool = database::connect().await.unwrap();
    log::info!("Connected to database");

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .route("/creatures", get(get_creatures))
        .route("/items", get(get_items))
        .route("/spells", get(get_spells))
        .with_state(pool)
        .layer(ServiceBuilder::new().layer(CorsLayer::permissive()));

    // run our app with hyper, listening globally on port 3000
    let bind_addr = dotenvy::var("BIND_URL").expect("BIND_URL must be set");
    let listener = tokio::net::TcpListener::bind(&bind_addr).await.unwrap();

    log::info!("Listening on: {}", bind_addr);
    axum::serve(listener, app).await.unwrap();
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
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

async fn get_items(
    Query(payload): Query<ItemFilters>,
    State(pool): State<PgPool>,
) -> (StatusCode, Json<Vec<LibraryItem>>) {
    let items = database::items::get_items(&pool, &payload).await.unwrap();
    (StatusCode::OK, Json(items))
}

async fn get_spells(
    Query(payload): Query<SpellFilters>,
    State(pool): State<PgPool>,
) -> (StatusCode, Json<Vec<LibrarySpell>>) {
    let spells = database::spells::get_spells(&pool, &payload).await.unwrap();
    (StatusCode::OK, Json(spells))
}

pub type Result<T> = std::result::Result<T, ServerError>;

#[derive(thiserror::Error, Debug)]
pub enum ServerError {
    #[error("Internal error: {0}")]
    SqlxError(#[from] sqlx::Error),
}

use axum::{http, response::IntoResponse, routing::get, Router};
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;

pub mod campaign;
pub mod models;
pub mod database;
pub mod library;

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
        .nest("/library", library::router())
        .nest("/campaign", campaign::router())
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

// TODO: users are not added yet. all users use this same id.
pub fn dummy_test_user() -> machete_core::ids::InternalId {
    machete_core::ids::InternalId(1)
}

pub type Result<T> = std::result::Result<T, ServerError>;

#[derive(thiserror::Error, Debug)]
pub enum ServerError {
    #[error("Internal error: {0}")]
    SqlxError(#[from] sqlx::Error),
    #[error("Internal error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),
}

impl ServerError {
    pub fn status_code(&self) -> http::StatusCode {
        match self {
            ServerError::SqlxError(_) => http::StatusCode::INTERNAL_SERVER_ERROR,
            ServerError::SerdeJsonError(_) => http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn body(&self) -> String {
        self.to_string()
    }
}

impl IntoResponse for ServerError {
    fn into_response(self) -> http::Response<axum::body::Body> {
        let mut res = http::Response::new(self.body().into());
        *res.status_mut() = self.status_code();
        res
    }
}

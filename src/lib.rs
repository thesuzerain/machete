use axum::{
    extract::FromRef,
    http::{self, HeaderValue},
    response::IntoResponse,
    routing::get,
    Router,
};
use models::ids::InternalId;
use reqwest::Method;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;

pub mod auth;
pub mod campaign;
pub mod database;
pub mod encounters;
pub mod library;
pub mod models;

pub async fn run_server() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .parse_default_env()
        .init();

    // Check for required environment variables and stop if any are missing
    check_env();

    // Connect to database
    let pool = database::connect().await.unwrap();
    log::info!("Connected to database");

    let app_state = AppState {
        pool: pool.clone(),
    };

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .nest("/auth", auth::router())
        .nest("/library", library::router())
        .nest("/campaign", campaign::router())
        .nest("/encounters", encounters::router())
        .with_state(app_state)
        .layer(
            ServiceBuilder::new().layer(
                CorsLayer::permissive()
                    .allow_credentials(true)
                    .allow_headers(vec![
                        http::header::AUTHORIZATION,
                        http::header::CONTENT_TYPE,
                        http::header::COOKIE,
                        http::header::SET_COOKIE,
                    ])
                    .allow_methods(vec![
                        Method::GET,
                        Method::POST,
                        Method::DELETE,
                        Method::PUT,
                        Method::PATCH,
                    ])
                    .expose_headers(vec![http::header::AUTHORIZATION, http::header::SET_COOKIE])
                    .allow_origin([
                        "http://localhost:8123".parse::<HeaderValue>().unwrap(),
                        "http://localhost:8080".parse::<HeaderValue>().unwrap(),
                        "http://localhost:5173".parse::<HeaderValue>().unwrap(),
                    ]),
            ),
        );

    // run our app with hyper, listening globally on port 3000
    let bind_addr = dotenvy::var("BIND_URL").expect("BIND_URL must be set");
    let listener = tokio::net::TcpListener::bind(&bind_addr).await.unwrap();

    log::info!("Listening on: {}", bind_addr);
    axum::serve(listener, app).await.unwrap();
}

#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::PgPool,
}

impl FromRef<AppState> for sqlx::PgPool {
    fn from_ref(state: &AppState) -> sqlx::PgPool {
        state.pool.clone()
    }
}

// Check for required environment variables
// Panics if any are missing
fn check_env() {
    dotenvy::dotenv().ok();
    dotenvy::var("DATABASE_URL").expect("DATABASE_URL must be set");
    dotenvy::var("BIND_URL").expect("BIND_URL must be set");
    dotenvy::var("ADMIN_API_KEY").expect("ADMIN_API_KEY must be set");
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
}

// TODO: users are not added yet. all users use this same id.
pub fn dummy_test_user() -> InternalId {
    InternalId(1)
}

pub type Result<T> = std::result::Result<T, ServerError>;

#[derive(thiserror::Error, Debug)]
pub enum ServerError {
    #[error("Internal error: {0}")]
    SqlxError(#[from] sqlx::Error),
    #[error("Internal error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),
    #[error("Internal error: {0}")]
    InternalError(String),
    #[error("Not found")]
    NotFound,
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Bad request: {0}")]
    BadRequest(String),
}

impl ServerError {
    pub fn status_code(&self) -> http::StatusCode {
        match self {
            ServerError::SqlxError(_) => http::StatusCode::INTERNAL_SERVER_ERROR,
            ServerError::SerdeJsonError(_) => http::StatusCode::INTERNAL_SERVER_ERROR,
            ServerError::InternalError(_) => http::StatusCode::INTERNAL_SERVER_ERROR,

            ServerError::NotFound => http::StatusCode::NOT_FOUND,

            ServerError::Unauthorized => http::StatusCode::UNAUTHORIZED,

            ServerError::BadRequest(_) => http::StatusCode::BAD_REQUEST,
        }
    }

    pub fn body(&self) -> String {
        self.to_string()
    }
}

impl IntoResponse for ServerError {
    fn into_response(self) -> http::Response<axum::body::Body> {
        // Log the error
        log::error!("{}", self);

        let mut res = http::Response::new(self.body().into());
        *res.status_mut() = self.status_code();
        res
    }
}

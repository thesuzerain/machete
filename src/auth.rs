use axum::{
    extract::State,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use axum_extra::{extract::cookie::{Cookie, CookieJar}, headers::{authorization::Bearer, Authorization}, TypedHeader};
use lazy_static::lazy_static;
use rand::Rng;
use reqwest::StatusCode;
use serde::Deserialize;
use sqlx::{PgPool, Pool};
use time::Duration;

use crate::{
    database,
    models::{
        auth::{Session, User},
        ids::InternalId,
    },
};

pub const SESSION_COOKIE_NAME: &str = "session_id";

lazy_static!(
    pub static ref ADMIN_API_KEY: String = std::env::var("ADMIN_API_KEY").expect("`ADMIN_API_KEY` must be set");
);

#[derive(Deserialize, Debug)]
pub struct CreateUser {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, Debug)]
struct LoginRequest {
    username: String,
    password: String,
}

pub fn router() -> Router<Pool<sqlx::Postgres>> {
    Router::new()
        .route("/signup", post(signup))
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/me", get(get_current_user))
}

pub async fn extract_user_from_cookies(
    jar: &CookieJar,
    exec: &sqlx::PgPool,
) -> crate::Result<User> {
    if let Some(session_id) = jar.get(SESSION_COOKIE_NAME) {
        if let Some(user) = database::auth::get_user_for_session(exec, session_id.value()).await? {
            return Ok(user);
        }
    }
    Err(crate::ServerError::Unauthorized)
}

pub async fn extract_admin_from_cookies(
    jar: &CookieJar,
    bearer : Option<TypedHeader<Authorization<Bearer>>>,
    exec: &sqlx::PgPool,
) -> crate::Result<()> {
    // Check for api key as bearer token
    if let Some(bearer) = bearer {
        if bearer.token() == *ADMIN_API_KEY {
            return Ok(());
        }
    }

    // If no api key, check for user and return if admin
    let user = extract_user_from_cookies(jar, exec).await?;
    if user.is_admin {
        Ok(())
    } else {
        Err(crate::ServerError::Unauthorized)
    }
}

async fn get_current_user(
    State(pool): State<Pool<sqlx::Postgres>>,
    cookie_jar: CookieJar,
) -> Result<Json<User>, crate::ServerError> {
    let user = extract_user_from_cookies(&cookie_jar, &pool).await?;
    Ok(Json(user))
}

async fn signup(
    State(pool): State<Pool<sqlx::Postgres>>,
    mut jar: CookieJar,
    Json(user): Json<CreateUser>,
) -> crate::Result<impl IntoResponse> {
    if database::auth::get_user_by_name(&pool, &user.username)
        .await?
        .is_some()
    {
        return Err(crate::ServerError::BadRequest("User already exists".into()));
    }

    let password_hash = bcrypt::hash(user.password, bcrypt::DEFAULT_COST).unwrap();
    let id = database::auth::insert_user(&pool, &user.username, &password_hash).await?;

    let token: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();

    let mut cookie = Cookie::new(SESSION_COOKIE_NAME, token.clone());
    cookie.set_max_age(Some(Duration::days(7)));
    cookie.set_path("/");
    jar = jar.add(cookie);

    database::auth::create_session(&pool, id, &token).await?;
    Ok((
        jar,
        Json(Session {
            token,
            user: User {
                id,
                username: user.username,
                is_admin: false,
            },
        }),
    ))
}

async fn login(
    State(pool): State<Pool<sqlx::Postgres>>,
    mut jar: CookieJar,
    Json(credentials): Json<LoginRequest>,
) -> Result<impl IntoResponse, crate::ServerError> {
    if let Some(user) = database::auth::get_user_by_name(&pool, &credentials.username).await? {
        let user_id = InternalId(user.id as u64);
        if !bcrypt::verify(credentials.password, &user.password_hash).unwrap() {
            return Err(crate::ServerError::Unauthorized);
        }

        let token: String = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();

        database::auth::create_session(&pool, user_id, &token).await?;

        let mut cookie = Cookie::new(SESSION_COOKIE_NAME, token.clone());
        cookie.set_max_age(Some(Duration::days(7)));
        cookie.set_path("/");
        jar = jar.add(cookie);

        return Ok((
            jar,
            Json(Session {
                token,
                user: user.into(),
            }),
        ));
    }
    Err(crate::ServerError::Unauthorized)
}

async fn logout(
    State(pool): State<Pool<sqlx::Postgres>>,
    mut jar: CookieJar,
) -> Result<impl IntoResponse, crate::ServerError> {
    if let Some(session_id) = jar.get(SESSION_COOKIE_NAME) {
        database::auth::delete_session(&pool, session_id.value()).await?;
        jar = jar.remove(Cookie::from(SESSION_COOKIE_NAME));
    }
    Ok((jar, StatusCode::NO_CONTENT))
}
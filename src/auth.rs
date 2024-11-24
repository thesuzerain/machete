use axum::{
    routing::{get, post},
    Router,
    Json,
    extract::State,
    response::{IntoResponse, Response},
};
use axum_extra::extract::cookie::{self, Cookie, CookieJar};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{PgPool, Pool};
use tokio::sync::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use rand::Rng;
use time::Duration;

use crate::{database, models::{auth::{Session, User}, ids::InternalId}};

pub const SESSION_COOKIE_NAME: &str = "session_id";

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

#[derive(Serialize)]
struct UserResponse {
    username: String,
}

pub fn router() -> Router<Pool<sqlx::Postgres>> {
    Router::new()
    .route("/signup", post(signup))
    .route("/login", post(login))
    .route("/logout", post(logout))
    .route("/me", get(get_current_user))
    .route("/campaigns", get(get_campaigns))
}

pub async fn extract_user_from_cookies(
    jar: &CookieJar,
    exec: &sqlx::PgPool,
) -> crate::Result<User> {
    log::info!("Extracting user from cookies");
    if let Some(session_id) = jar.get(SESSION_COOKIE_NAME) {
        log::info!("Got session id: {:?}", session_id);
        if let Some(user) = database::auth::get_user_for_session(exec, session_id.value()).await? {
            log::info!("Got user: {:?}", user);
            return Ok(user);
        }
    }
    log::info!("No user found");
    Err(crate::ServerError::Unauthorized)
}

pub async fn extract_admin_from_cookies(
    jar: &CookieJar,
    exec: &sqlx::PgPool,
) -> crate::Result<User> {
    let user = extract_user_from_cookies(jar, exec).await?;
    if user.is_admin {
        Ok(user)
    } else {
        Err(crate::ServerError::Unauthorized)
    }
}

async fn get_current_user(
    State(pool): State<Pool<sqlx::Postgres>>,
    cookie_jar: CookieJar,
) -> Result<Json<User>, crate::ServerError> {
    log::info!("get_current_user {:?}", cookie_jar);
    let user = extract_user_from_cookies(&cookie_jar, &pool).await?;
    Ok(Json(user))
}

async fn get_campaigns(State(pool): State<PgPool>, 
    jar: CookieJar,
) -> Result<impl IntoResponse, crate::ServerError> {
    log::info!("get_campaigns: {:?}", jar);
    let user = extract_user_from_cookies(&jar, &pool).await?;
    log::info!("Getting campaigns for owner: {:?}", user);
    let campaigns = database::campaigns::get_campaign(&pool, user.id).await?;
    Ok(Json(campaigns))
}


async fn signup(
    State(pool): State<Pool<sqlx::Postgres>>,
    mut jar: CookieJar,
    Json(user): Json<CreateUser>,
) -> crate::Result<impl IntoResponse> {
    if database::auth::get_user_by_name(&pool, &user.username).await?.is_some() {
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
    })))
}

async fn login(
    State(pool): State<Pool<sqlx::Postgres>>,
    mut jar: CookieJar,
    Json(credentials): Json<LoginRequest>,
) -> Result<impl IntoResponse, crate::ServerError> {
   log::info!("Got login request: {:?}", credentials);
   if let Some(user) = database::auth::get_user_by_name(&pool, &credentials.username).await? {
        let user_id = InternalId(user.id as u64);
        let hashed_password = &user.password_hash;
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

        return Ok((jar, Json(Session { token, user: user.into() })));
    } 
    Err(crate::ServerError::Unauthorized)
}

async fn logout(
    State(pool): State<Pool<sqlx::Postgres>>,
    mut jar: CookieJar,
) -> Result<impl IntoResponse, crate::ServerError> {
    if let Some(session_id) = jar.get(SESSION_COOKIE_NAME) {
        database::auth::delete_session(&pool, session_id.value()).await?;
        jar = jar.remove(Cookie::named(SESSION_COOKIE_NAME));
    }
    Ok((jar, StatusCode::NO_CONTENT))
}
use crate::models::auth::User;
use crate::models::ids::InternalId;

#[derive(serde::Deserialize)]
pub struct InsertUser {
    pub username: String,
    pub password_hash: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct DatabaseUser {
    pub id: i32,
    pub username: String,
    pub is_admin: bool,
    pub password_hash: String,
}

pub async fn get_user_by_name(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    username: &str,
) -> crate::Result<Option<DatabaseUser>> {
    let query = sqlx::query!(
        r#"
        SELECT 
            u.id,
            u.username,
            u.password_hash,
            u.is_admin
        FROM users u
        WHERE 
            u.username = $1
    "#,
        username,
    );

    let user = query.fetch_optional(exec).await?.map(|row| DatabaseUser {
        id: row.id,
        username: row.username,
        is_admin: row.is_admin.unwrap_or(false),
        password_hash: row.password_hash,
    });
    Ok(user)
}

pub async fn insert_user(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    username: &str,
    password_hash: &str,
) -> crate::Result<InternalId> {
    let id = sqlx::query!(
        r#"
        INSERT INTO users (username, password_hash)
        VALUES ($1, $2)
        RETURNING id
        "#,
        username,
        password_hash
    )
    .fetch_one(&mut **tx)
    .await?;

    Ok(InternalId(id.id as u32))
}

pub async fn delete_user(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    user_id: InternalId,
) -> crate::Result<()> {
    sqlx::query!(
        r#"
        DELETE FROM users
        WHERE id = $1
        "#,
        user_id.0 as i32,
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}

pub async fn create_session(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    user_id: InternalId,
    session_id: &str,
) -> crate::Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO sessions (user_id, token)
        VALUES ($1, $2)
        "#,
        user_id.0 as i32,
        session_id,
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}

pub async fn get_user_for_session(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    session_id: &str,
) -> crate::Result<Option<User>> {
    let query = sqlx::query!(
        r#"
        SELECT 
            u.id,
            u.username,
            u.is_admin
        FROM users u
        JOIN sessions s ON u.id = s.user_id
        WHERE 
            s.token = $1
            AND s.expires_at > NOW()
    "#,
        session_id,
    );

    let user = query.fetch_optional(exec).await?.map(|row| User {
        id: InternalId(row.id as u32),
        username: row.username,
        is_admin: row.is_admin.unwrap_or(false),
    });
    Ok(user)
}

pub async fn delete_session(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    session_id: &str,
) -> crate::Result<()> {
    sqlx::query!(
        r#"
        DELETE FROM sessions
        WHERE token = $1
        "#,
        session_id,
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}

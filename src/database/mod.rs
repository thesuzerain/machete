use sqlx::PgPool;

pub mod auth;
pub mod campaigns;
pub mod characters;
pub mod classes;
pub mod creatures;
pub mod encounters;
pub mod events;
pub mod hazards;
pub mod items;
pub mod logs;
pub mod spells;
// pub mod stats;
pub mod sessions;
pub mod tags;

pub const DEFAULT_MAX_LIMIT: u64 = 100;
pub const DEFAULT_MAX_GROUP_LIMIT: u64 = 25;

pub async fn connect() -> Result<PgPool, sqlx::Error> {
    let database_url = dotenvy::var("DATABASE_URL").expect("`DATABASE_URL` not in .env");
    // TODO: Num connections, etc
    let pool = PgPool::connect(&database_url).await?;
    Ok(pool)
}

use sqlx::PgPool;

pub mod creatures;
pub mod items;
pub mod spells;
pub mod tags;

pub async fn connect() -> Result<PgPool, sqlx::Error> {
    let database_url = dotenvy::var("DATABASE_URL").expect("`DATABASE_URL` not in .env");
    // TODO: Num connections, etc
    let pool = PgPool::connect(&database_url).await?;
    Ok(pool)
}

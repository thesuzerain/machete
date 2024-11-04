use sqlx::PgPool;

pub mod characters;
pub mod creatures;
pub mod events;
pub mod items;
pub mod spells;
pub mod tags;
pub mod campaigns;

pub async fn connect() -> Result<PgPool, sqlx::Error> {
    let database_url = dotenvy::var("DATABASE_URL").expect("`DATABASE_URL` not in .env");
    // TODO: Num connections, etc
    let pool = PgPool::connect(&database_url).await?;
    Ok(pool)
}

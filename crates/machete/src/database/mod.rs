use std::future::Future;

use machete_core::filters::{Filter, FilterableStruct};
use sqlx::PgPool;
use tokio::sync::OnceCell;

pub mod creatures;
pub mod events;
pub mod items;
pub mod spells;
pub mod tags;

/// OnceCell for the database pool.
/// TODO: This is a naive implementation. As we move away from client-only (into a webserver, etc), this will be replaced.
static DATABASE_ONCE: OnceCell<sqlx::Pool<sqlx::Postgres>> = OnceCell::const_new();
pub async fn get_database_pool() -> sqlx::Pool<sqlx::Postgres> {
    DATABASE_ONCE
        .get_or_init(|| async { crate::database::connect().await.unwrap() })
        .await
        .clone()
}

pub async fn connect() -> Result<PgPool, sqlx::Error> {
    let database_url = dotenvy::var("DATABASE_URL").expect("`DATABASE_URL` not in .env");
    // TODO: Num connections, etc
    let pool = PgPool::connect(&database_url).await?;
    Ok(pool)
}

// TODO: FilterableStruct, DisplayableStruct, QueryableStruct -> redefine these? merge them?
pub trait QueryableStruct: FilterableStruct {
    fn query_get(
        pool: sqlx::PgPool,
        filters: &Vec<Filter<Self>>,
    ) -> impl Future<Output = crate::Result<Vec<Self>>> + Send;
}

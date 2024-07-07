use std::future::Future;

use machete_core::filters::{Filter, FilterableStruct};
use sqlx::PgPool;

pub mod creatures;
pub mod events;
pub mod items;
pub mod spells;
pub mod tags;

pub async fn connect() -> Result<PgPool, sqlx::Error> {
    let database_url = dotenvy::var("DATABASE_URL").expect("`DATABASE_URL` not in .env");
    // TODO: Num connections, etc
    let pool = PgPool::connect(&database_url).await?;

    Ok(pool)
}

// TODO: FilterableStruct, DisplayableStruct, QueryableStruct -> redefine these? merge them?
pub trait QueryableStruct: FilterableStruct {
    fn query_get<'a>(
        exec: impl sqlx::Executor<'a, Database = sqlx::Postgres>,
        filters: &Vec<Filter<Self>>,
    ) -> impl Future<Output = crate::Result<Vec<Self>>> + Send;
}

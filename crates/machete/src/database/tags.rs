use std::collections::HashMap;
use std::collections::HashSet;

use sqlx::Row;
use sqlx::Sqlite;

// TODO: not sure if I like this
pub async fn insert_and_return_tags(
    exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite> + Copy,
    items: Vec<String>,
) -> crate::Result<HashMap<String, i64>> {
    // TODO: This doesn't use sqlx::query! because it needs to be dynamic. Is there a better way to do this?
    // Maybe postgres + unnest as in labrinth?
    // TODO: Do we *need* two tables for this?

    // todo: no duplicate,s but also this fixes the too many variables error
    let items = items.into_iter().collect::<HashSet<String>>();

    let mut builder: sqlx::QueryBuilder<Sqlite> = sqlx::QueryBuilder::new(
        r#"
        INSERT INTO tags (tag)
        VALUES    
    "#,
    );

    // TODO: I really don't like this
    for (i, item) in items.iter().enumerate() {
        builder.push("(");
        builder.push_bind(item);
        builder.push(")");
        if i < items.len() - 1 {
            builder.push(", ");
        }
    }
    let built = builder.build();
    built.execute(exec).await?;

    // Now, fetch all ids for these tags
    let mut builder: sqlx::QueryBuilder<Sqlite> = sqlx::QueryBuilder::new(
        r#"
        SELECT id, tag
        FROM tags
        WHERE tag IN (
    "#,
    );

    for (i, item) in items.iter().enumerate() {
        builder.push_bind(item);
        if i < items.len() - 1 {
            builder.push(", ");
        }
    }

    builder.push(")");

    let built = builder.build();
    let ids = built
        .fetch_all(exec)
        .await?
        .into_iter()
        .map(|row| Ok((row.get(1), row.get(0))))
        .collect::<Result<HashMap<String, i64>, sqlx::Error>>()?;

    Ok(ids)
}

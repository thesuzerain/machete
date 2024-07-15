use std::collections::HashMap;
use std::collections::HashSet;

// TODO: not sure if I like this
pub async fn insert_and_return_tags(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres> + Copy,
    items: Vec<String>,
) -> crate::Result<HashMap<String, i32>> {
    // TODO: This doesn't use sqlx::query! because it needs to be dynamic. Is there a better way to do this?
    // Maybe postgres + unnest as in labrinth?
    // TODO: Do we *need* two tables for this?

    // todo: no duplicate,s but also this fixes the too many variables error
    let items = items.into_iter().collect::<HashSet<String>>();

    sqlx::query!(
        r#"
        INSERT INTO tags (tag)
        SELECT * FROM UNNEST ($1::text[])
        "#,
        // TODO: optimize this too
        &items.iter().map(|x| x.clone()).collect::<Vec<String>>(),
    )
    .execute(exec)
    .await?;

    // Now, fetch all ids for these tags
    // TODO: optimize
    let ids = sqlx::query!(
        r#"
        SELECT id, tag
        FROM tags
        WHERE tag IN (
            SELECT * FROM UNNEST ($1::text[])
        )
    "#,
        // TODO: optimize this too
        &items.iter().map(|x| x.clone()).collect::<Vec<String>>(),
    )
    .fetch_all(exec)
    .await?;

    let ids = ids
        .into_iter()
        .map(|row| Ok((row.tag, row.id)))
        .collect::<Result<HashMap<String, i32>, sqlx::Error>>()?;

    Ok(ids)
}

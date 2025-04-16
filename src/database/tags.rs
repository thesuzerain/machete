use std::collections::HashMap;
use std::collections::HashSet;

// TODO: not sure if I like this
pub async fn insert_and_return_tags(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    items: Vec<String>,
) -> crate::Result<HashMap<String, i32>> {
    if items.is_empty() {
        return Ok(HashMap::new());
    }
    // TODO: Do we *need* two tables for this?
    // todo: No duplicates, but also this fixes the too many variables error
    let items = items.into_iter().collect::<HashSet<String>>();

    sqlx::query!(
        r#"
        INSERT INTO tags (tag)
        SELECT * FROM UNNEST ($1::text[])
        "#,
        // TODO: optimize this too
        &items.iter().map(|x| x.clone()).collect::<Vec<String>>(),
    )
    .execute(&mut **tx)
    .await?;

    // Now, fetch all ids for these tags
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
    .fetch_all(&mut **tx)
    .await?;

    let ids = ids
        .into_iter()
        .map(|row| Ok((row.tag, row.id)))
        .collect::<Result<HashMap<String, i32>, sqlx::Error>>()?;

    Ok(ids)
}

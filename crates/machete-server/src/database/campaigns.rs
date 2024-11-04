use machete::models::campaign::CampaignPartial;
use machete_core::ids::InternalId;

// TODO: May be prudent to make a separate models system for the database.
pub async fn get_campaign(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    owner: InternalId,
) -> crate::Result<Vec<CampaignPartial>> {
    let query = sqlx::query!(
        r#"
        SELECT 
            ca.id,
            ca.name
        FROM campaigns ca
        WHERE 
            ca.owner = $1
    "#,
        owner.0 as i32,
    );

    let campaigns = query
        .fetch_all(exec)
        .await?
        .into_iter()
        .map(|row| {
            Ok(CampaignPartial {
                id: InternalId(row.id as u64),
                name: row.name,
            })
        })
        .collect::<Result<Vec<CampaignPartial>, sqlx::Error>>()?;
    Ok(campaigns)
}

pub async fn insert_campaign(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres> + Copy,
    name: &str,
    owner: InternalId,
) -> crate::Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO campaigns (name, owner)
        VALUES ($1, $2)
        "#,
        name,
        owner.0 as i32,
    )
    .execute(exec)
    .await?;

    Ok(())
}

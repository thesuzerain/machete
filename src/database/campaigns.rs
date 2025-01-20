use crate::models::campaign::CampaignPartial;
use crate::models::ids::InternalId;
use crate::ServerError;

#[derive(serde::Deserialize)]
pub struct InsertCampaign {
    pub name: String,
    pub description: Option<String>,
}

// TODO: May be prudent to make a separate models system for the database.
pub async fn get_campaigns_owner(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    owner: InternalId,
) -> crate::Result<Vec<CampaignPartial>> {
    let query = sqlx::query!(
        r#"
        SELECT 
            ca.id,
            ca.name,
            description
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
                description: row.description,
            })
        })
        .collect::<Result<Vec<CampaignPartial>, sqlx::Error>>()?;
    Ok(campaigns)
}

pub async fn get_owned_campaign_id(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    campaign_id: InternalId,
    owner: InternalId,
) -> crate::Result<Option<InternalId>> {
    let query = sqlx::query!(
        r#"
        SELECT 
            ca.id AS "id!"
        FROM campaigns ca
        WHERE 
            ca.id = $1
            AND ca.owner = $2
    "#,
        campaign_id.0 as i32,
        owner.0 as i32,
    )
    .fetch_optional(exec)
    .await?
    .map(|row| InternalId(row.id as u64));

    Ok(query)
}

pub async fn insert_campaign(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres> + Copy,
    insert: &InsertCampaign,
    owner: InternalId,
) -> crate::Result<InternalId> {
    let id = sqlx::query!(
        r#"
        INSERT INTO campaigns (name, owner, description)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
        &insert.name,
        owner.0 as i32,
        insert.description.as_ref(),
    )
    .fetch_all(exec)
    .await?.into_iter().next().ok_or(ServerError::InternalError("Failed to insert campaign".to_string()))?.id;

    Ok(InternalId(id as u64))
}


pub async fn delete_campaign(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres> + Copy,
    campaign_id: InternalId,
    owner: InternalId,
) -> crate::Result<()> {
    sqlx::query!(
        r#"
        DELETE FROM campaigns
        WHERE id = $1 AND owner = $2
        "#,
        campaign_id.0 as i32,
        owner.0 as i32,
    )
    .execute(exec)
    .await?;

    Ok(())
}

use std::collections::HashMap;

use crate::models::characters::Stat;
use crate::models::ids::InternalId;
use crate::models::library::item::{Rune, RuneItemType, SkillPotency};
use crate::models::library::{item::LibraryItem, GameSystem, Rarity};

use crate::models::query::CommaSeparatedVec;
use crate::ServerError;
use crate::v2::database::models::item_instances::ItemInstance;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use sqlx::PgConnection;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InsertItemInstance {
    pub library_item_id: InternalId,
    pub parent_item_id: Option<InternalId>,
    pub campaign_id: Option<InternalId>,
    pub encounter_id: Option<InternalId>,
    pub character_id: Option<InternalId>,
    pub session_id: Option<InternalId>,
    pub is_reward: bool,
    pub quantity: u16,
    pub nickname: Option<String>,
    pub notes: Option<String>,
}

// TODO: Filters, etc. Currently only used for exporting.
pub async fn get_item_instances(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres> + Copy,
) -> crate::Result<Vec<ItemInstance>> {
    let res = sqlx::query!(
        r#"
        SELECT
            ii.id,
            ii.library_item_id,
            ii.parent_item_id,
            ii.campaign_id,
            ii.encounter_id,
            ii.character_id,
            ii.session_id,
            ii.is_reward,
            ii.quantity,
            ii.nickname,
            ii.notes
        FROM item_instances ii
        "#,
    )
    .fetch_all(exec)
    .await?
    .into_iter()
    .map(|row| ItemInstance {
        id: row.id,
        library_item_id: row.library_item_id,
        parent_item_id: row.parent_item_id,
        campaign_id: row.campaign_id,
        encounter_id: row.encounter_id,
        character_id: row.character_id,
        session_id: row.session_id,
        is_reward: row.is_reward,
        quantity: row.quantity as u16,
        nickname: row.nickname,
        notes: row.notes,
    })
    .collect::<Vec<ItemInstance>>();

    Ok(res)
}


pub async fn insert_item_instances(
    tx : &mut sqlx::Transaction<'_, sqlx::Postgres>,
    item_instances: Vec<InsertItemInstance>,
) -> crate::Result<Vec<InternalId>> {
    let res = sqlx::query!(
        r#"
        INSERT INTO item_instances (
            library_item_id,
            parent_item_id,
            campaign_id,
            encounter_id,
            character_id,
            session_id,
            is_reward,
            quantity,
            nickname,
            notes
        )
        SELECT
            library_item_id, parent_item_id, campaign_id, encounter_id, character_id, session_id, is_reward, quantity, nickname, notes
        FROM UNNEST(
            $1::int[],
            $2::int[],
            $3::int[],
            $4::int[],
            $5::int[],
            $6::int[],
            $7::boolean[],
            $8::smallint[],
            $9::text[],
            $10::text[]
        ) AS i(
            library_item_id,
            parent_item_id,
            campaign_id,
            encounter_id,
            character_id,
            session_id,
            is_reward,
            quantity,
            nickname,
            notes
        )
        RETURNING
            id
        "#
        ,
        &item_instances.iter().map(|x| x.library_item_id.0 as i32).collect::<Vec<i32>>(),
        &item_instances.iter().map(|x| x.parent_item_id.map(|id| id.0 as i32)).collect::<Vec<Option<i32>>>() as _,
        &item_instances.iter().map(|x| x.campaign_id.map(|id| id.0 as i32)).collect::<Vec<Option<i32>>>() as _,
        &item_instances.iter().map(|x| x.encounter_id.map(|id| id.0 as i32)).collect::<Vec<Option<i32>>>() as _,
        &item_instances.iter().map(|x| x.character_id.map(|id| id.0 as i32)).collect::<Vec<Option<i32>>>() as _,
        &item_instances.iter().map(|x| x.session_id.map(|id| id.0 as i32)).collect::<Vec<Option<i32>>>() as _,
        &item_instances.iter().map(|x| x.is_reward).collect::<Vec<bool>>(),
        &item_instances.iter().map(|x| x.quantity as i16).collect::<Vec<i16>>(),
        &item_instances.iter().map(|x| x.nickname.clone()).collect::<Vec<Option<String>>>() as _,
        &item_instances.iter().map(|x| x.notes.clone()).collect::<Vec<Option<String>>>() as _
    )
    .fetch_all(&mut **tx)
    .await?
    .into_iter()
    .map(|row| {
        InternalId(row.id as u32)
    })
    .collect::<Vec<InternalId>>();

    Ok(res)
}

use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::ServerError;

pub mod auth;
pub mod campaigns;
pub mod characters;
pub mod classes;
pub mod creatures;
pub mod encounters;
pub mod events;
pub mod hazards;
pub mod import;
pub mod items;
pub mod logs;
pub mod sessions;
pub mod spells;
pub mod stats;
pub mod tags;
pub mod sorts;

pub const DEFAULT_MAX_LIMIT: u64 = 100;
pub const DEFAULT_MAX_GROUP_LIMIT: u64 = 25;

pub async fn connect() -> Result<PgPool, sqlx::Error> {
    let database_url = dotenvy::var("DATABASE_URL").expect("`DATABASE_URL` not in .env");
    // TODO: Num connections, etc
    let pool = PgPool::connect(&database_url).await?;
    Ok(pool)
}

// todo: export to mod.rs, so all other routes that can use it
// also add those- for spells, etc. should consider legacy items
#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq)]
#[serde(rename_all = "snake_case")] // TODO: Camelcase when everything is converted to camelcase
pub enum LegacyStatus {
    #[default]
    All,        // Includes both (duplicates)
    Legacy,     // Includes legacy then remaster (but only where no duplicates)
    LegacyOnly, // Only legacy, no remaster
    Remaster, // Includes remaster then legacy (but only where no duplicates)
    RemasterOnly, // Only remaster, no legacy
}

impl LegacyStatus {
    pub fn include_legacy(&self) -> bool {
        matches!(
            self,
            Self::All | Self::LegacyOnly | Self::Remaster | Self::Legacy
        )
    }

    pub fn include_remaster(&self) -> bool {
        matches!(
            self,
            Self::All | Self::Remaster | Self::RemasterOnly | Self::Legacy
        )
    }

    pub fn favor_remaster(&self) -> bool {
        matches!(self, Self::Remaster | Self::RemasterOnly)
    }

    pub fn favor_legacy(&self) -> bool {
        matches!(self, Self::Legacy | Self::LegacyOnly)
    }
}

pub async fn check_library_requested_ids(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    requested_ids: &[i32],
) -> Result<Vec<i32>, ServerError> {
    let existing_ids = sqlx::query!(
        r#"
       SELECT id FROM library_objects WHERE id = ANY($1::int[])
   "#,
        &requested_ids.iter().map(|id| *id).collect::<Vec<i32>>(),
    )
    .fetch_all(exec)
    .await?;

    if !existing_ids.is_empty() {
        return Err(ServerError::BadRequest(format!(
            "Requested IDs already in use: {:?}",
            existing_ids
        )));
    }

    Ok(existing_ids.iter().map(|id| id.id).collect())
}

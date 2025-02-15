use serde::{Deserialize, Serialize};
use sqlx::PgPool;

pub mod auth;
pub mod campaigns;
pub mod characters;
pub mod classes;
pub mod creatures;
pub mod encounters;
pub mod events;
pub mod hazards;
pub mod items;
pub mod logs;
pub mod spells;
// pub mod stats;
pub mod sessions;
pub mod tags;

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
    All, // Includes both (duplicates)
    LegacyOnly, // Only legacy, no remaster
    #[default]
    Remaster, // Includes remaster then legacy (but only where no duplicates)
    RemasterOnly, // Only remaster, no legacy
}

impl LegacyStatus {
    pub fn include_legacy(&self) -> bool {
        matches!(self, Self::All | Self::LegacyOnly | Self::Remaster)
    }

    pub fn include_remaster(&self) -> bool {
        matches!(self, Self::All | Self::Remaster | Self::RemasterOnly)
    }

    pub fn favor_remaster(&self) -> bool {
        matches!(self, Self::Remaster)
    }
}

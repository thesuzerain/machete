use serde::{Deserialize, Serialize};

use crate::database::auth::DatabaseUser;

use super::ids::InternalId;
#[derive(serde::Deserialize, serde::Serialize)]
pub struct Session {
    pub token: String,
    pub user: User,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct User {
    pub id: InternalId,
    pub username: String,
    pub is_admin: bool,
}

impl From<DatabaseUser> for User {
    fn from(user: DatabaseUser) -> Self {
        User {
            id: InternalId(user.id as u32),
            username: user.username,
            is_admin: user.is_admin,
        }
    }
}

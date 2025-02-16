use std::{
    fmt::Display,
    hash::{DefaultHasher, Hasher},
};

use rand::Rng;
use serde::{Deserialize, Serialize};

/// An internal identifier for a an object for user-local usage.
/// These are randomly generated and risk hash collisions, and are not used online.
/// TODO: Should these be randomly generated? Should just use uuids?
/// TODO: sqlx derivation for easy database storage.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialOrd, Ord, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct InternalId(pub u32);

impl Default for InternalId {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for InternalId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "InternalId({})", self.0)
    }
}

impl InternalId {
    /// Generate a new random internal ID.
    pub fn new() -> Self {
        let mut bytes = [0u8; 4];
        rand::thread_rng().fill(&mut bytes);
        let id = u32::from_ne_bytes(bytes);
        InternalId(id)
    }

    /// Create a hashed internal id from an object.
    /// Useful for generating a likely-unique ID from outside the API
    pub fn new_from_hash(souce: impl std::hash::Hash) -> InternalId {
        let mut hasher = DefaultHasher::default();
        souce.hash(&mut hasher);
        InternalId(hasher.finish() as u32)
    }
}

use std::hash::{DefaultHasher, Hasher};

use getrandom::getrandom;
use serde::{Deserialize, Serialize};

/// An internal identifier for a an object for user-local usage.
/// These are randomly generated and risk hash collisions, and are not used online.
/// TODO: Should these be randomly generated? Should just use uuids?
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialOrd, Ord, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct InternalId(pub u64);

impl Default for InternalId {
    fn default() -> Self {
        Self::new()
    }
}

impl InternalId {
    /// Generate a new random internal ID.
    pub fn new() -> Self {
        let mut bytes = [0u8; 8];
        getrandom(&mut bytes).expect("Failed to generate random bytes from the OS");
        let id = u64::from_ne_bytes(bytes);
        InternalId(id)
    }

    /// Hash this ID with another hashable object.
    /// This is useful for consistent but unique hashing for UI elements.
    pub fn hash_with(self, other: impl std::hash::Hash) -> u64 {
        let mut hasher = DefaultHasher::default();
        hasher.write_u64(self.0);
        other.hash(&mut hasher);
        hasher.finish()
    }
}

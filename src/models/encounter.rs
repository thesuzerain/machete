
use serde::{Deserialize, Serialize};

use super::{ids::InternalId, library::item::Currency};

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct Encounter {
    pub id: InternalId,
    pub status : CompletionStatus,
    pub name: String,
    pub description: Option<String>,

    pub party_level: u32,
    pub party_size: u32,

    pub enemies: Vec<InternalId>,
    pub hazards: Vec<InternalId>,

    pub treasure_items: Vec<InternalId>,
    pub treasure_currency: Currency,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub enum CompletionStatus {
    Draft,
    #[default]
    Prepared,
    Archived,
    Success,
    Failure,
}

impl CompletionStatus {
    pub fn as_i32(&self) -> i32 {
        match self {
            CompletionStatus::Draft => 0,
            CompletionStatus::Prepared => 1,
            CompletionStatus::Archived => 2,
            CompletionStatus::Success => 3,
            CompletionStatus::Failure => 4,
        }
    }

    pub fn from_i32(i: i32) -> Self {
        match i {
            0 => CompletionStatus::Draft,
            1 => CompletionStatus::Prepared,
            2 => CompletionStatus::Archived,
            3 => CompletionStatus::Success,
            4 => CompletionStatus::Failure,
            _ => panic!("Invalid status"),
        }
    }
}
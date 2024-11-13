use machete_core::ids::InternalId;
use serde::{Deserialize, Serialize};

use super::library::item::Currency;

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct Encounter {
    pub id: InternalId,
    pub status : CompletionStatus,
    pub name: String,
    pub description: Option<String>,

    pub enemies: Vec<InternalId>,
    pub hazards: Vec<InternalId>,

    pub treasure_items: Vec<InternalId>,
    pub treasure_currency: Currency,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub enum CompletionStatus {
    #[default]
    Prepared,
    Archived,
    Success,
    Failure,
}

impl CompletionStatus {
    pub fn as_i32(&self) -> i32 {
        match self {
            CompletionStatus::Prepared => 0,
            CompletionStatus::Archived => 1,
            CompletionStatus::Success => 2,
            CompletionStatus::Failure => 3,
        }
    }

    pub fn from_i32(i: i32) -> Self {
        match i {
            0 => CompletionStatus::Prepared,
            1 => CompletionStatus::Archived,
            2 => CompletionStatus::Success,
            3 => CompletionStatus::Failure,
            _ => panic!("Invalid status"),
        }
    }
}
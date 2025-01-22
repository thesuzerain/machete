use serde::{Deserialize, Serialize};

use super::ids::InternalId;

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct Encounter {
    pub id: InternalId,
    pub status: CompletionStatus,
    pub name: String,
    pub description: Option<String>,

    pub owner: InternalId,

    pub party_level: u32,
    pub party_size: u32,

    pub enemies: Vec<EncounterEnemy>,
    pub hazards: Vec<InternalId>,

    pub treasure_items: Vec<InternalId>,
    pub treasure_currency: f32,
    pub extra_experience: i32,

    // Derived values
    pub total_experience: i32,
    pub total_treasure_value: i32,
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct EncounterEnemy {
    pub id: InternalId,
    pub level_adjustment: i16,
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

pub enum EncounterDifficulty {
    Trivial,
    Low,
    Moderate,
    Severe,
    Extreme,
}

impl EncounterDifficulty {
    pub fn as_i32(&self) -> i32 {
        match self {
            EncounterDifficulty::Trivial => 0,
            EncounterDifficulty::Low => 1,
            EncounterDifficulty::Moderate => 2,
            EncounterDifficulty::Severe => 3,
            EncounterDifficulty::Extreme => 4,
        }
    }

    pub fn from_i32(i: i32) -> Self {
        match i {
            0 => EncounterDifficulty::Trivial,
            1 => EncounterDifficulty::Low,
            2 => EncounterDifficulty::Moderate,
            3 => EncounterDifficulty::Severe,
            4 => EncounterDifficulty::Extreme,
            _ => panic!("Invalid difficulty"),
        }
    }

    pub fn extra_player_experience_delta(&self) -> i32 {
        match self {
            EncounterDifficulty::Trivial => 0,
            EncounterDifficulty::Low => 10,
            EncounterDifficulty::Moderate => 20,
            EncounterDifficulty::Severe => 30,
            EncounterDifficulty::Extreme => 40,
        }
    }

    pub fn experience_cutoff(&self) -> i32 {
        match self {
            EncounterDifficulty::Trivial => 40,
            EncounterDifficulty::Low => 60,
            EncounterDifficulty::Moderate => 80,
            EncounterDifficulty::Severe => 120,
            EncounterDifficulty::Extreme => 160,
        }
    }
}

pub fn calculate_total_adjusted_experience(
    enemy_levels: &[i8],
    hazard_levels: &[i8],
    party_level: u8,
    party_size: u8,
) -> i32 {
    let mut total_experience: i32 = 0;
    for level in enemy_levels {
        total_experience += calculate_enemy_experience(*level, party_level);
    }
    for level in hazard_levels {
        total_experience += calculate_enemy_experience(*level, party_level);
    }

    let diff_off = party_size as i32 - 4;
    if total_experience - 40 - 10 * diff_off >= 160 {
        return total_experience;
    }
    if total_experience - 30 * diff_off >= 120 {
        return total_experience;
    }
    if total_experience - 20 * diff_off >= 80 {
        return total_experience;
    }
    if total_experience - 20 * diff_off >= 60 {
        return total_experience;
    }
    total_experience
}

pub fn calculate_enemy_experience(level: i8, party_level: u8) -> i32 {
    let level_diff = level as i32 - party_level as i32;
    match level_diff {
        ..=-4 => 10,
        -3 => 15,
        -2 => 20,
        -1 => 30,
        0 => 40,
        1 => 60,
        2 => 80,
        3 => 120,
        4.. => 160,
    }
}

use serde::{Deserialize, Serialize};

use super::{characters::Skill, ids::InternalId};

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct Encounter {
    pub id: InternalId,
    pub status: CompletionStatus,
    pub name: String,
    pub description: Option<String>,

    pub session_id: Option<InternalId>,

    pub owner: InternalId,

    #[serde(flatten)]
    pub encounter_type: EncounterType,

    pub party_level: u32,
    pub party_size: u32,

    pub treasure_items: Vec<InternalId>,
    pub treasure_currency: f32,
    pub extra_experience: i32,

    // Derived values
    pub total_experience: i32,
    pub total_items_value: i32,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(tag = "encounter_type", rename_all = "camelCase")]
pub enum EncounterType {
    #[default]
    Unknown,
    Reward,
    RewardInitialization,
    Combat {
        #[serde(default)]
        enemies: Vec<EncounterEnemy>,
        #[serde(default)]
        hazards: Vec<InternalId>,    
    },
    Subsystem {
        subsystem_type: EncounterSubsystemType,
        #[serde(default)]
        subsystem_checks: Vec<EncounterSubsystemCheck>,
    },
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub enum EncounterSubsystemType {
    #[default]
    Unknown,
    Chase,
    Infiltration,
    Research
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EncounterSubsystemCheck {
    pub name: String,
    pub roll_options: Vec<EncounterSubsystemRoll>,
    pub vp : u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EncounterSubsystemRoll {
    pub skill : Skill,
    pub dc : u8,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct EncounterEnemy {
    pub id: InternalId,
    #[serde(default)]
    pub level_adjustment: i16,
}

// TODO: We may want to remove this
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

impl EncounterType {
    pub fn from_id_and_parts(i : i32, enemies : Vec<EncounterEnemy>, hazards: Vec<InternalId>, subsystem : Option<EncounterSubsystemType>, subsystem_rolls : Vec<EncounterSubsystemCheck>) -> EncounterType {
        match i {
            1 => EncounterType::RewardInitialization,
            2 => EncounterType::Reward,
            3 => EncounterType::Combat { enemies, hazards },
            4 => EncounterType::Subsystem { subsystem_type: subsystem.unwrap_or_default(), subsystem_checks: subsystem_rolls },
            _ => EncounterType::Unknown,
        }
    }

    pub fn get_id(&self) -> i32 {
        match self {
            EncounterType::RewardInitialization => 1,
            EncounterType::Reward => 2,
            EncounterType::Combat { .. } => 3,
            EncounterType::Subsystem { .. } => 4,
            EncounterType::Unknown => 0,
        }
    }

    pub fn get_enemies(&self) -> Vec<EncounterEnemy> {
        match self {
            EncounterType::Combat { enemies, .. } => enemies.clone(),
            _ => Vec::new(),
        }
    }

    pub fn get_hazards(&self) -> Vec<InternalId> {
        match self {
            EncounterType::Combat { hazards, .. } => hazards.clone(),
            _ => Vec::new(),
        }
    }

    pub fn get_subsystem_type(&self) -> EncounterSubsystemType {
        match self {
            EncounterType::Subsystem { subsystem_type, .. } => subsystem_type.clone(),
            _ => EncounterSubsystemType::Unknown,
        }
    }

    pub fn get_subsystem_checks(&self) -> Vec<EncounterSubsystemCheck> {
        match self {
            EncounterType::Subsystem { subsystem_checks, .. } => subsystem_checks.clone(),
            _ => Vec::new(),
        }
    }
}

impl EncounterSubsystemType {
    pub fn as_i32(&self) -> i32 {
        match self {
            EncounterSubsystemType::Unknown => 0,
            EncounterSubsystemType::Chase => 1,
            EncounterSubsystemType::Infiltration => 2,
            EncounterSubsystemType::Research => 3,
        }
    }

    pub fn from_i32(i: i32) -> Self {
        match i {
            1 => EncounterSubsystemType::Chase,
            2 => EncounterSubsystemType::Infiltration,
            3 => EncounterSubsystemType::Research,
            _ => EncounterSubsystemType::Unknown,
        }
    }
}

pub fn calculate_total_adjusted_experience(
    enemy_levels: &[i16],
    hazard_levels: &[i16],
    party_level: u8,
    party_size: u8,
) -> i32 {
    let mut total_experience: i32 = 0;
    for level in enemy_levels {
        total_experience += calculate_enemy_experience(*level as i8, party_level);
    }
    for level in hazard_levels {
        total_experience += calculate_enemy_experience(*level as i8, party_level);
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


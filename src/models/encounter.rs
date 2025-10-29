use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{characters::Skill, ids::InternalId};
use crate::models::characters::skill_serialize;

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct Encounter {
    pub id: InternalId,
    pub name: String,
    pub description: Option<String>,

    // Only if this is linked to a session
    pub session_id: Option<InternalId>,
    pub campaign_id: Option<InternalId>,

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
    Accomplishment,
    RewardInitialization,
    // TODO: We may want to move party level and size to the combat part? It's only relevant for it- but we may want to keep it otherwise as its in a different section.
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
    Research,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EncounterSubsystemCheck {
    pub name: String,
    pub roll_options: Vec<EncounterSubsystemRoll>,
    pub vp: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EncounterSubsystemRoll {
    #[serde(with = "skill_serialize")]
    pub skill: Skill,
    pub dc: u8,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct EncounterEnemy {
    pub id: InternalId,
    #[serde(default)]
    pub level_adjustment: i16,
}

#[derive(Serialize, Deserialize, Hash, PartialEq, Eq, Debug, Clone, Copy)]
pub enum EncounterDifficulty {
    Trivial,
    Low,
    Moderate,
    Severe,
    Extreme,
}

impl EncounterDifficulty {
    pub fn extra_player_experience_delta(&self) -> i32 {
        match self {
            EncounterDifficulty::Trivial => 10,
            EncounterDifficulty::Low => 20,
            EncounterDifficulty::Moderate => 20,
            EncounterDifficulty::Severe => 30,
            EncounterDifficulty::Extreme => 40,
        }
    }

    pub fn budget(&self) -> i32 {
        match self {
            EncounterDifficulty::Trivial => 40,
            EncounterDifficulty::Low => 60,
            EncounterDifficulty::Moderate => 80,
            EncounterDifficulty::Severe => 120,
            EncounterDifficulty::Extreme => 160,
        }
    }

    pub fn iter() -> impl Iterator<Item = EncounterDifficulty> {
        vec![
            EncounterDifficulty::Trivial,
            EncounterDifficulty::Low,
            EncounterDifficulty::Moderate,
            EncounterDifficulty::Severe,
            EncounterDifficulty::Extreme,
        ]
        .into_iter()
    }

    pub fn get_severity_boundaries(party_size: u8) -> HashMap<EncounterDifficulty, (i32, i32)> {
        let difficulties = EncounterDifficulty::iter().collect::<Vec<_>>();
        let budgets = difficulties
            .iter()
            .map(|d| {
                let mut budget = d.budget();
                // Adjust the budget based on party size
                if party_size > 4 {
                    budget += (party_size as i32 - 4) * d.extra_player_experience_delta();
                } else if party_size < 4 {
                    budget -= (4 - party_size as i32) * d.extra_player_experience_delta();
                }
                budget
            })
            .collect::<Vec<_>>();

        // Budgets are halfway between the difficulties
        let boundary_points = std::iter::once(0)
            .chain(budgets.windows(2).map(|w| (w[0] + w[1]) / 2))
            .chain(std::iter::once(i32::MAX))
            .collect::<Vec<_>>();

        difficulties
            .iter()
            .zip(boundary_points.iter().zip(boundary_points.iter().skip(1)))
            .map(|(d, (start, end))| (*d, (*start, *end)))
            .collect()
    }

    fn get_difficulty_from_raw_experience(
        raw_experience: i32,
        party_size: u8,
    ) -> EncounterDifficulty {
        let boundaries = EncounterDifficulty::get_severity_boundaries(party_size);
        for (difficulty, (start, end)) in boundaries.iter() {
            if raw_experience >= *start && raw_experience < *end {
                return *difficulty;
            }
        }
        EncounterDifficulty::Extreme
    }
}

impl EncounterType {
    pub fn from_id_and_parts(
        i: i32,
        enemies: Vec<EncounterEnemy>,
        hazards: Vec<InternalId>,
        subsystem: Option<EncounterSubsystemType>,
        subsystem_rolls: Vec<EncounterSubsystemCheck>,
    ) -> EncounterType {
        match i {
            1 => EncounterType::RewardInitialization,
            2 => EncounterType::Accomplishment,
            3 => EncounterType::Combat { enemies, hazards },
            4 => EncounterType::Subsystem {
                subsystem_type: subsystem.unwrap_or_default(),
                subsystem_checks: subsystem_rolls,
            },
            _ => EncounterType::Unknown,
        }
    }

    pub fn get_id(&self) -> i32 {
        match self {
            EncounterType::RewardInitialization => 1,
            EncounterType::Accomplishment => 2,
            EncounterType::Combat { .. } => 3,
            EncounterType::Subsystem { .. } => 4,
            EncounterType::Unknown => 0,
        }
    }

    pub fn string_from_id(i: i32) -> String {
        match i {
            1 => "Reward Initialization".to_string(),
            2 => "Accomplishment".to_string(),
            3 => "Combat".to_string(),
            4 => "Subsystem".to_string(),
            _ => "Unknown".to_string(),
        }
    }

    pub fn id_from_string(s: &str) -> i32 {
        match s {
            "Reward Initialization" => 1,
            "Accomplishment" => 2,
            "Combat" => 3,
            "Subsystem" => 4,
            _ => 0,
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
            EncounterType::Subsystem {
                subsystem_checks, ..
            } => subsystem_checks.clone(),
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
    hazard_level_complexities: &[(i16, bool)],
    party_level: u8,
    party_size: u8,
) -> i32 {
    if (enemy_levels.is_empty() && hazard_level_complexities.is_empty())
        || party_level == 0
        || party_size == 0
    {
        return 0;
    }

    let mut total_experience: i32 = 0;
    for level in enemy_levels {
        total_experience += calculate_enemy_experience(*level as i8, party_level);
    }
    for (level, complex) in hazard_level_complexities {
        let exp = calculate_enemy_experience(*level as i8, party_level);
        if *complex {
            total_experience += exp;
        } else {
            // Hazards are worth 1/5 of the experience of enemies if they are not complex
            total_experience += exp / 5;
        }
    }

    let difficulty =
        EncounterDifficulty::get_difficulty_from_raw_experience(total_experience, party_size);

    // Adjust the experience based on the difficulty
    let diff_off = party_size as i32 - 4;
    total_experience - diff_off * difficulty.extra_player_experience_delta()
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

#[cfg(test)]
mod tests {
    use super::{calculate_total_adjusted_experience, EncounterDifficulty};

    #[test]
    fn test_experience_calculation() {
        // Basic test- simple when 4 players
        assert_eq!(
            calculate_total_adjusted_experience(&[5, 5, 5, 5], &[], 5, 4),
            160
        );
        assert_eq!(
            calculate_total_adjusted_experience(&[7, 1, 1, 1, 1], &[], 5, 4),
            120
        );
        assert_eq!(calculate_total_adjusted_experience(&[7, 5], &[], 5, 4), 120);
        assert_eq!(
            calculate_total_adjusted_experience(&[5, 5, 5], &[], 5, 4),
            120
        );
        assert_eq!(calculate_total_adjusted_experience(&[5, 5], &[], 5, 4), 80);
        assert_eq!(
            calculate_total_adjusted_experience(&[5, 1, 1, 1, 1], &[], 5, 4),
            80
        );
        assert_eq!(
            calculate_total_adjusted_experience(&[5, 3, 3], &[], 5, 4),
            80
        );
        assert_eq!(
            calculate_total_adjusted_experience(&[1, 1, 1, 1, 1, 1], &[], 5, 4),
            60
        );

        // The basic budgets of each difficulty level for 5 players
        // 5 players increases budget by 40, 30, 20, 20
        assert_eq!(
            calculate_total_adjusted_experience(&[5, 5, 5, 5, 5], &[], 5, 5),
            160
        ); // New extreme budget: 200, results in 160
        assert_eq!(
            calculate_total_adjusted_experience(&[7, 5, 4], &[], 5, 5),
            120
        ); // New severe budget: 150, results in 120
        assert_eq!(
            calculate_total_adjusted_experience(&[5, 5, 3], &[], 5, 5),
            80
        ); // New moderate budget: 100, results in 80

        // The basic budgets of each difficulty level for 6 players
        assert_eq!(
            calculate_total_adjusted_experience(&[5, 5, 5, 5, 5, 5], &[], 5, 6),
            160
        ); // New extreme budget: 240, results in 160
        assert_eq!(
            calculate_total_adjusted_experience(&[7, 5, 4, 4], &[], 5, 6),
            120
        ); // New severe budget: 200, results in 120
        assert_eq!(
            calculate_total_adjusted_experience(&[5, 5, 3, 3], &[], 5, 6),
            80
        ); // New moderate budget: 160, results in 80

        // The basic budgets of each difficulty level for 3 players
        // 3 players decreases budget by 40, 30, 20, 20
        assert_eq!(
            calculate_total_adjusted_experience(&[5, 5, 5], &[], 5, 3),
            160
        ); // New extreme budget: 120, results in 160
        assert_eq!(calculate_total_adjusted_experience(&[7, 1], &[], 5, 3), 120); // New severe budget: 90, results in 120
        assert_eq!(calculate_total_adjusted_experience(&[5, 3], &[], 5, 3), 80); // New moderate budget: 60, results in 80

        // The basic budgets of each difficulty level for 2 players
        assert_eq!(calculate_total_adjusted_experience(&[5, 5], &[], 5, 2), 160); // New extreme budget: 80, results in 160
        assert_eq!(calculate_total_adjusted_experience(&[6], &[], 5, 2), 120); // New severe budget: 60, results in 120
        assert_eq!(calculate_total_adjusted_experience(&[5], &[], 5, 2), 80); // New moderate budget: 40, results in 80

        // Now, we do the *edge cases*. We allow for any experience, not just the budgeted ones.
        // We define the shifting points as the halfway point between the budgets, rounded up.
        // So, for 4 players, the budgets with boundaries are:
        // 40 (0-50), 60 (50-70), 80 (70-100), 120 (100-140), 160 (140-170)
        // Notably, the center of the budget range may not be the same as the budget itself.

        // 5 player tests on the edge of the budgets
        // 50 (0-65), 80 (65-90), 100 (90-125), 150 (125-175), 200 (175+)
        assert_eq!(
            calculate_total_adjusted_experience(&[5, 5, 5, 5, 3], &[], 5, 5),
            140
        ); // 180 raw- classified as extreme, meaning -40 penalty
        assert_eq!(
            calculate_total_adjusted_experience(&[5, 5, 5, 5, 2], &[], 5, 5),
            135
        ); // 175 raw- classified as extreme, meaning -40 penalty
        assert_eq!(
            calculate_total_adjusted_experience(&[5, 5, 5, 5, 1], &[], 5, 5),
            140
        ); // 170 raw- classified as severe, meaning -30 penalty, and amusingly, a higher total

        assert_eq!(
            calculate_total_adjusted_experience(&[5, 5, 4, 3], &[], 5, 5),
            100
        ); // 130 raw- classified as severe, meaning -30 penalty
        assert_eq!(
            calculate_total_adjusted_experience(&[5, 5, 4, 2], &[], 5, 5),
            95
        ); // 125 raw- classified as severe, meaning -30 penalty
        assert_eq!(
            calculate_total_adjusted_experience(&[5, 5, 3, 3], &[], 5, 5),
            100
        ); // 120 raw- classified as moderate, meaning -20 penalty

        // 3 player tests on the edge of the budgets
        // 30, 40, 60, 90, 120
        // So:
        // 30 (0-35), 40 (35-50), 60 (50-75), 90 (75-105), 120 (105+)
        assert_eq!(
            calculate_total_adjusted_experience(&[5, 5, 2, 2], &[], 5, 3),
            150
        ); // 110 raw- classified as extreme, meaning +40 reward
        assert_eq!(
            calculate_total_adjusted_experience(&[5, 5, 2, 1], &[], 5, 3),
            145
        ); // 105 raw- classified as extreme, meaning +40 reward
        assert_eq!(
            calculate_total_adjusted_experience(&[5, 5, 1, 1], &[], 5, 3),
            130
        ); // 100 raw- classified as severe, meaning +30 reward

        assert_eq!(
            calculate_total_adjusted_experience(&[5, 3, 3], &[], 5, 3),
            110
        ); // 80 raw- classified as severe, meaning +30 reward
        assert_eq!(
            calculate_total_adjusted_experience(&[5, 3, 2], &[], 5, 3),
            105
        ); // 75 raw- classified as severe, meaning +30 reward
        assert_eq!(
            calculate_total_adjusted_experience(&[5, 2, 2], &[], 5, 3),
            90
        ); // 70 raw- classified as severe, meaning +20 reward

        // Some specific cases
        assert_eq!(
            calculate_total_adjusted_experience(&[5, 3, 3], &[], 4, 3),
            160
        ); // 120 raw- classified as extreme, meaning 40 reward
    }

    #[test]
    fn test_experience_boundaries() {
        let party_4 = EncounterDifficulty::get_severity_boundaries(4);
        assert_eq!(party_4.get(&EncounterDifficulty::Trivial), Some(&(0, 50)));
        assert_eq!(party_4.get(&EncounterDifficulty::Low), Some(&(50, 70)));
        assert_eq!(
            party_4.get(&EncounterDifficulty::Moderate),
            Some(&(70, 100))
        );
        assert_eq!(party_4.get(&EncounterDifficulty::Severe), Some(&(100, 140)));
        assert_eq!(
            party_4.get(&EncounterDifficulty::Extreme),
            Some(&(140, i32::MAX))
        );

        let party_5 = EncounterDifficulty::get_severity_boundaries(5);
        assert_eq!(party_5.get(&EncounterDifficulty::Trivial), Some(&(0, 65)));
        assert_eq!(party_5.get(&EncounterDifficulty::Low), Some(&(65, 90)));
        assert_eq!(
            party_5.get(&EncounterDifficulty::Moderate),
            Some(&(90, 125))
        );
        assert_eq!(party_5.get(&EncounterDifficulty::Severe), Some(&(125, 175)));
        assert_eq!(
            party_5.get(&EncounterDifficulty::Extreme),
            Some(&(175, i32::MAX))
        );

        let party_3 = EncounterDifficulty::get_severity_boundaries(3);
        assert_eq!(party_3.get(&EncounterDifficulty::Trivial), Some(&(0, 35)));
        assert_eq!(party_3.get(&EncounterDifficulty::Low), Some(&(35, 50)));
        assert_eq!(party_3.get(&EncounterDifficulty::Moderate), Some(&(50, 75)));
        assert_eq!(party_3.get(&EncounterDifficulty::Severe), Some(&(75, 105)));
        assert_eq!(
            party_3.get(&EncounterDifficulty::Extreme),
            Some(&(105, i32::MAX))
        );
    }
}

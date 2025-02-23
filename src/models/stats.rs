use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::ids::InternalId;

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct CampaignStats {
    pub num_encounters: u32,
    pub num_sessions: u32,

    // Total experience, and experience this level. (Every level has 1000 XP)
    pub level: u32,
    pub total_xp: u32,
    pub experience_this_level: u32,

    // Treasure totals over all the campaign, ever rewarded.
    pub total_combined_treasure: u32,
    pub total_treasure_items_value: u32,
    pub total_gold: u32,

    pub total_expected_combined_treasure: f32,
    pub total_expected_combined_treasure_start_of_level: f32,
    pub total_expected_combined_treasure_end_of_level: f32,

    // level of item -> number of items given at that level
    pub total_permanent_items_by_level: HashMap<u32, u32>,
    pub expected_permanent_items_by_end_of_level: HashMap<u32, u32>,

    pub total_consumable_items_by_level: HashMap<u32, u32>,
    pub expected_consumable_items_by_end_of_level: HashMap<u32, u32>,

    pub encounters: Vec<EncounterStats>,
    pub character_stats: HashMap<InternalId, CharacterStats>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CharacterStats {
    pub total_combined_treasure: f64,
    pub total_treasure_items_value: f64,
    pub total_gold: f64,

    pub available_boosts: Vec<AssignedBoost>,
    pub expected_boosts: Vec<AssignedBoost>,

    pub rewards_per_session: Vec<AssignedRewardsSession>,

    pub total_permanent_items: Vec<InternalId>,
    pub total_consumable_items: Vec<InternalId>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AssignedBoost {
    pub boost_category_id: u32,
    pub boost_category_name: String,
    pub potency: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AssignedRewardsSession {
    pub session_id: InternalId,
    pub treasure_gold: f64,
    pub treasure_item_value: f64,
    pub treasure_items_group: Vec<InternalId>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EncounterStats {
    pub encounter_ix: u32,
    pub session_id: u32,
    pub session_ix: u32,

    pub accumulated_items_treasure: f32,
    pub accumulated_gold_treasure: f32,
    pub accumulated_xp: u32,
    pub calculated_expected_total_treasure: f32,
    pub pf_expected_total_treasure: f32,
}

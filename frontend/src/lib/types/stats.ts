import type { InternalId } from './types';

export interface CampaignStats {
    num_encounters: number;
    num_sessions: number;
    level: number;
    total_xp: number;
    experience_this_level: number;

    total_combined_treasure: number;
    total_treasure_items_value: number;
    total_gold: number;

    total_expected_combined_treasure: number;
    total_expected_combined_treasure_start_of_level: number;
    total_expected_combined_treasure_end_of_level: number;

    total_permanent_items_by_level: Record<number, number>;
    expected_permanent_items_by_end_of_level: Record<number, number>;

    total_consumable_items_by_level: Record<number, number>;
    expected_consumable_items_by_end_of_level: Record<number, number>;

    encounters: EncounterStats[];
    character_stats: Record<InternalId, CharacterStats>;
}

export interface CharacterStats {
    total_combined_treasure: number;
    total_treasure_items_value: number;
    total_gold: number;

    available_boosts: AssignedBoost[];
    expected_boosts: AssignedBoost[];

    rewards_per_session: AssignedRewardsSession[];

    total_permanent_items: InternalId[];
    total_consumable_items: InternalId[];
}

export interface AssignedBoost {
    boost_category_id: number;
    boost_category_name: string;
    potency: number;
}

export interface AssignedRewardsSession {
    session_id: InternalId;
    treasure_gold: number;
    treasure_item_value: number;
    treasure_items_group: InternalId[];
}

export interface EncounterStats {
    encounter_ix: number;
    session_id: number;
    session_ix: number;
    accumulated_items_treasure: number;
    accumulated_gold_treasure: number;
    accumulated_xp: number;
    calculated_expected_total_treasure: number;
    pf_expected_total_treasure: number;
} 
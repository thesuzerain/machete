
// TODO: Ensure these line up with the things currently being exported from the backend
export interface Campaign {
    id: number;
    name: string;
    experience: number;
    level: number;
    description?: string;
}

export interface InsertInitialCampaignData {
    gold: number;
    items: number[];
    characters: {
        name: string;
        class: number;
        gold: number;
        items: number[];
    }[];
}

export interface CampaignSession {
    id: number;
    session_order: number;
    name: string;
    description?: string;
    encounter_ids: number[];
    play_date: Date;

    level_at_end: number;
    experience_at_end: number;

    // Derived values for aggregation. Not modified directly.
    total_experience: number;
    total_combined_treasure_value: number;

    // Character-specific awards
    // These are assumed to be correct, and are derived values.
    //  character_id -> {gold and items}
    // `compiled_rewards` is modifiable, and the unassigned ones are derived in the backend.
    compiled_rewards: Record<number, CompiledRewards>;
    unassigned_item_rewards: number[];
    unassigned_gold_rewards: number;
}

export interface InsertCampaignSession {
    session_order: number;
    name: string;
    description?: string;
    characters: number[];
}

export interface CompiledRewards {
    gold: number;
    items: number[];
    present: boolean;
}

export interface InsertCampaignSessionEncounterLinksMetadata {
    compiled_rewards: Record<number, CompiledRewards>;
}

export interface Character {
    id: number;
    name: string;
    experience: number;
    class: number;
    class_name: string;
    campaign_id: number;
}
export interface Event {
    id: number;
    character?: number;
    timestamp: Date;
    event_type: string;
    data: Record<string, unknown>;
}

// Types for inserting new records
export interface InsertCampaign {
    name: string;
}

export interface InsertCharacter {
    name: string;
    class: number;
    level: number;
}

export interface UpdateCharacter {
    id: number;
    name: string;
    class: number;
    level: number;
}

export interface InsertEvent {
    character?: number;
    event_type: string;
    description: string;
    data: Record<string, unknown>;
}

export interface Log {
    id: number;
    campaign: number;
    name: string;
    timestamp: string;
    description: string;
    events: Event[];
}

export interface InsertLog {
    name: string;
    description: string;
    events: InsertEvent[];
}

export interface WIPInsertLog {
    name: string;
    description: string;
    extra_experience: number;
    characterIds: number[];
    enemies: WIPLogEnemy[];
    treasures: WIPLogTreasure[];
    current_manual_events: InsertEvent[];
}

// TODO: Revisit these- only for log creation
export interface WIPLogEnemy {   
    id: number;
    count: number;
    level?: number;
    type: 'enemy' | 'hazard';
}

export interface WIPLogTreasure {
    type: 'currency' | 'item';
    amount?: number; // TODO: Standardize this
    itemId?: number; // TODO: Standardize this
}

export interface LibraryClass {
    id: number;
    name: string;
    game_system: string;
    rarity: string;
    tags: string[];
} 

export interface User {
    id: number;
    username: string;
    is_admin: boolean;
}

export interface LibraryEntity {
    id: number;
    name: string;
    level?: number;
    // ... other properties
    // TODO: Think of a better way to do this. LibraryClass expand sthis, etc?
}

export interface Enemy {
    id: number;
    count: number;
    type: 'enemy' | 'hazard';
    level?: number;  // Add this
}

export type Skill = 'Acrobatics' | 'Arcana' | 'Athletics' | 'Crafting' | 'Deception' | 'Diplomacy' | 'Intimidation' | 'Lore' | 'Medicine' | 'Nature' | 'Occultism' | 'Performance' | 'Religion' | 'Society' | 'Stealth' | 'Survival' | 'Thievery' | 'Unknown';
export const skills: Skill[] = ['Acrobatics', 'Arcana', 'Athletics', 'Crafting', 'Deception', 'Diplomacy', 'Intimidation', 'Lore', 'Medicine', 'Nature', 'Occultism', 'Performance', 'Religion', 'Society', 'Stealth', 'Survival', 'Thievery', 'Unknown'];
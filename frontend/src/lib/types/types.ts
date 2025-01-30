
// TODO: Ensure these line up with the things currently being exported from the backend
export interface Campaign {
    id: number;
    name: string;
    description?: string;
}

export interface CampaignSession {
    id: number;
    session_order: number;
    name: string;
    description?: string;
    encounter_ids: number[];
    play_date: Date;

    // These are assumed to be correct, and are derived values.
    // TODO: Handle unlinking encounters from a sessions- these values will need to be changed or reset?
    //  encounter_id -> character_id -> item_id -> #
    compiled_item_rewards: Record<number, Record<number, number[]>>;
    compiled_gold_rewards: Record<number, Record<number, number>>;
    unassigned_item_rewards: Record<number, number[]>;
    unassigned_gold_rewards: Record<number, number>;
}

export interface InsertCampaignSession {
    session_order: number;
    name: string;
    description?: string;
    encounter_ids: number[];
}

export interface InsertCampaignSessionEncounterLinksMetadata {
    encounter_ids: number[];

    // TODO: Should we split these out into separate routes (ie with /:encounterId)? Shoudl we coalesce t he down into just the campaign session route
    compiled_item_rewards: Record<number, Record<number, number[]>>;
    compiled_gold_rewards: Record<number, Record<number, number>>;
    unassigned_item_rewards: Record<number, number[]>;
    unassigned_gold_rewards: Record<number, number>;
}

export interface Character {
    id: number;
    name: string;
    level: number;
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
}

export interface Enemy {
    id: number;
    count: number;
    type: 'enemy' | 'hazard';
    level?: number;  // Add this
}

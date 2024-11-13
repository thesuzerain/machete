export interface Campaign {
    id: number;
    name: string;
}

export interface Character {
    id: number;
    campaign_id: number;
    name: string;
    class: number;
    level: number;
    race: string;
    background: string;
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
    character_id?: number;
    event_type: string;
    description: string;
    value?: number;
}

export interface Log {
    id: number;
    campaign: number;
    name: string;
    timestamp: string;
    description: string;
    events: number[];
}

export interface InsertLog {
    campaign: number;
    template_id: string;
    description: string;
    events: InsertEvent[];
}

export interface LibraryClass {
    id: number;
    name: string;
    game_system: string;
    rarity: string;
    tags: string[];
} 
export interface Campaign {
    id: number;
    name: string;
}

export interface Character {
    id: number;
    campaign_id: number;
    name: string;
    class: string;
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
    class: string;
    level: number;
    race: string;
    background: string;
}

export interface InsertEvent {
    character_id?: number;
    event_type: string;
    description: string;
    value?: number;
} 
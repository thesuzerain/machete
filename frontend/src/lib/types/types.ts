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
    campaign_id: number;
    character_id?: number;
    date: Date;
    event_type: 'CURRENCY' | 'ITEM' | 'LEVEL_UP' | 'STORY' | 'OTHER';
    description: string;
    value?: number;
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
    event_type: 'CURRENCY' | 'ITEM' | 'LEVEL_UP' | 'STORY' | 'OTHER';
    description: string;
    value?: number;
} 
export type EncounterStatus = 'InProgress' | 'Prepared' | 'Archived' | 'Success' | 'Failure';

export interface Encounter {
    id: number;
    status: EncounterStatus;
    name: string;
    description: string;
    enemies: number[];
    hazards: number[];
    treasure_items: number[];
    treasure_currency: number;
    party_level: number;
    party_size: number;
}

export interface CreateEncounter {
    name: string;
    description: string;
    enemies: number[];
    hazards: number[];
    treasure_items: number[];
    treasure_currency: number;
    party_level: number;
    party_size: number;
} 
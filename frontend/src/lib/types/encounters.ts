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
}

export interface CreateEncounter {
    name: string;
    description: string;
    enemies: number[];
    hazards: number[];
    treasure_items: number[];
    treasure_currency: number;
} 
export type EncounterStatus = 'Draft' | 'Prepared' | 'Completed';

export interface Encounter {
    id: number;
    session_id: number | null;
    status: EncounterStatus;
    name: string;
    description: string;
    enemies: EncounterEnemy[];
    enemy_level_adjustments: number[];
    hazards: number[];
    treasure_items: number[];
    treasure_currency: number;
    party_level: number;
    party_size: number;
    extra_experience: number;

    // Derived fields
    total_experience: number;
    total_currency: number;
}

export interface CreateEncounter {
    name: string;
    description: string;
    enemies: EncounterEnemy[];
    hazards: number[];
    treasure_items: number[];
    treasure_currency: number;
    extra_experience: number;
    party_level: number;
    party_size: number;

    status: EncounterStatus;
} 

export interface EncounterEnemy {
    id: number;
    level_adjustment: number;
}

export interface CreateEncounterFinalized extends CreateEncounter {    
    // On creation, these are optionally omitted and, if so, are calculated by the backend
    total_experience: number;
    total_currency: number;
    session_id: number | null;
} 
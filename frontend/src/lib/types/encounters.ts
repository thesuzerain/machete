import type { Skill } from "./types";

export type EncounterType = 'combat' | 'accomplishment' | 'rewardInitialization' | 'subsystem' | 'unknown';
export type SubsystemCategory = 'chase' | 'infiltration' | 'research' | 'unknown';

export interface Encounter {
    id: number;
    name: string;
    description: string;

    // Value or null together
    session_id: number | null;
    campaign_id: number | null;

    encounter_type: EncounterType;
    
    // Combat encounter fields
    enemies?: EncounterEnemy[];
    enemy_level_adjustments?: number[];
    hazards?: number[];
    
    // Reward fields (used by all encounter types)
    treasure_items: number[];
    treasure_currency: number;
    
    // Common fields
    party_level: number;
    party_size: number;
    extra_experience: number;
    
    // Subsystem fields
    subsystem_type?: SubsystemCategory;
    subsystem_checks?: SkillCheck[];

    // Derived fields
    total_experience: number;
    total_items_value: number;
}


export interface SkillRollOption {
    skill: Skill;
    dc: number;
}

export interface SkillCheck {
    name: string;
    roll_options: SkillRollOption[];
    vp: number;

}

export interface CreateOrReplaceEncounter {
    name: string;
    description: string;
    encounter_type: EncounterType;
    
    // Combat fields
    enemies?: EncounterEnemy[];
    hazards?: number[];
    
    // Reward fields
    treasure_items: number[];
    treasure_currency: number;
    extra_experience: number;
    
    // Common fields
    party_level: number;
    party_size: number;
    
    // Subsystem fields
    subsystem_type?: SubsystemCategory;
    subsystem_checks?: SkillCheck[];
} 

export interface EncounterEnemy {
    id: number;
    level_adjustment: number;
}

export interface CreateOrReplaceEncounterExtended extends CreateOrReplaceEncounter {    
    // On creation, these are optionally omitted and, if so, are calculated by the backend
    total_experience: number | null;
    total_items_value: number | null;
} 

export interface CreateEncounterFinalized extends CreateOrReplaceEncounterExtended {    
    // session_id can only be set on creation
    session_id: number | null;
} 

export type AccomplishmentLevel = 'minor' | 'moderate' | 'major';
export function experienceForAccomplishment(level : AccomplishmentLevel | null) : number {
    switch (level) {
        case null: return 0;
        case 'minor': return 10;
        case 'moderate': return 30;
        case 'major': return 80;
    }
}
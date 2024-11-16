import type { Currency } from "$lib/types/library";

export enum EncounterDifficulty {
    Trivial = "Trivial",
    Low = "Low",
    Moderate = "Moderate",
    Severe = "Severe",
    Extreme = "Extreme"
}

interface PartyConfig {
    party_size: number;
    party_level: number;
}

export function getExperienceFromLevel(partyLevel: number, creatureLevel: number): number {
    const levelDifference = creatureLevel - partyLevel;

    // Base XP values from the Pathfinder 2e rules
    const xpByDifference: { [key: number]: number } = {
        '-4': 10,
        '-3': 15,
        '-2': 20,
        '-1': 30,
        '0': 40,
        '1': 60,
        '2': 80,
        '3': 120,
        '4': 160
    };
    
    // Clamp the level difference to our known values
    const clampedDiff = Math.max(-4, Math.min(4, levelDifference));
    return xpByDifference[clampedDiff] || 40; // Default to level 0 if something goes wrong
}

export function getSeverityFromExperience(totalXP: number, partySize: number): EncounterDifficulty {
    // XP thresholds based on the party size
    const baseThresholds = {
        trivial: 40,
        low: 60,
        moderate: 80,
        severe: 120,
        extreme: 160
    };

    const playerAdjustmentThresholds = {
        trivial: 10,
        low: 20,
        moderate: 20,
        severe: 30,
        extreme: 40
    }

    const diffOff = partySize - 4;
    if (totalXP - playerAdjustmentThresholds.extreme*diffOff >= baseThresholds.extreme) return EncounterDifficulty.Extreme;
    if (totalXP - playerAdjustmentThresholds.severe*diffOff >= baseThresholds.severe) return EncounterDifficulty.Severe;
    if (totalXP - playerAdjustmentThresholds.moderate*diffOff >= baseThresholds.moderate) return EncounterDifficulty.Moderate;
    if (totalXP - playerAdjustmentThresholds.low*diffOff >= baseThresholds.low) return EncounterDifficulty.Low;
    return EncounterDifficulty.Trivial;
}

export function getRewardForLevelSeverity(level: number, severity: EncounterDifficulty): { 
    xp: number,
    currency: Currency
} {
    // Base XP rewards
    const xpRewards = {
        [EncounterDifficulty.Trivial]: 40,
        [EncounterDifficulty.Low]: 60,
        [EncounterDifficulty.Moderate]: 80,
        [EncounterDifficulty.Severe]: 120,
        [EncounterDifficulty.Extreme]: 160
    };

    // Currency rewards based on level and severity
    // These are rough estimates and should be adjusted based on your game's economy
    const baseCurrency = Math.pow(2, level - 1); // Exponential growth with level
    const currencyMultiplier = {
        [EncounterDifficulty.Trivial]: 0.5,
        [EncounterDifficulty.Low]: 1,
        [EncounterDifficulty.Moderate]: 2,
        [EncounterDifficulty.Severe]: 4,
        [EncounterDifficulty.Extreme]: 8
    };

    const totalCurrency = baseCurrency * currencyMultiplier[severity];

    return {
        xp: xpRewards[severity],
        currency: {
            gold: Math.floor(totalCurrency),
            silver: Math.floor((totalCurrency % 1) * 10),
            copper: Math.floor((totalCurrency * 10 % 1) * 10)
        }
    };
} 
export enum EncounterDifficulty {
    Trivial = "Trivial",
    Low = "Low",
    Moderate = "Moderate",
    Severe = "Severe",
    Extreme = "Extreme"
}

interface PartyConfig {
    playerCount: number;
    partyLevel: number;
}

export function getExperienceFromLevel(partyLevel: number, creatureLevel: number): number {
    const levelDifference = creatureLevel - partyLevel;

    console.log("calculating xp from level difference", levelDifference, partyLevel, creatureLevel);
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

export function getSeverityFromExperience(totalXP: number, partyConfig: PartyConfig): EncounterDifficulty {
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

    const diffOff = partyConfig.playerCount - 4;

    console.log("totalXP", totalXP);
    console.log("diffOff", diffOff);
    console.log("Extreme check", totalXP + playerAdjustmentThresholds.extreme*diffOff, baseThresholds.extreme);
    console.log("Severe check", totalXP + playerAdjustmentThresholds.severe*diffOff, baseThresholds.severe);
    console.log("Moderate check", totalXP + playerAdjustmentThresholds.moderate*diffOff, baseThresholds.moderate);
    console.log("Low check", totalXP + playerAdjustmentThresholds.low*diffOff, baseThresholds.low);

    if (totalXP - playerAdjustmentThresholds.extreme*diffOff >= baseThresholds.extreme) return EncounterDifficulty.Extreme;
    if (totalXP - playerAdjustmentThresholds.severe*diffOff >= baseThresholds.severe) return EncounterDifficulty.Severe;
    if (totalXP - playerAdjustmentThresholds.moderate*diffOff >= baseThresholds.moderate) return EncounterDifficulty.Moderate;
    if (totalXP - playerAdjustmentThresholds.low*diffOff >= baseThresholds.low) return EncounterDifficulty.Low;
    return EncounterDifficulty.Trivial;
}

export function getRewardForLevelSeverity(level: number, severity: EncounterDifficulty): { 
    xp: number,
    currency: { gold?: number, silver?: number, copper?: number }
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
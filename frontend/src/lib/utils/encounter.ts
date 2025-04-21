
export enum EncounterDifficulty {
    Trivial = "Trivial",
    Low = "Low",
    Moderate = "Moderate",
    Severe = "Severe",
    Extreme = "Extreme",
    Unknown = "Unknown"
}

export function getHazardExperienceFromLevel(partyLevel: number, hazardLevel: number, isComplex: boolean): number {
    // First, get as if it was a creature
    const baseXP = getCreatureExperienceFromLevel(partyLevel, hazardLevel);

    // Complex hazards keep as is, simple hazards are 1/5th of the XP
    return isComplex ? baseXP : baseXP / 5;
}

export function getCreatureExperienceFromLevel(partyLevel: number, creatureLevel: number): number {
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

type Boundaries = Map<EncounterDifficulty, [number, number]>;

type Thresholds = Map<EncounterDifficulty, number>;

export function getEncounterDifficultyThresholds(partySize: number): Thresholds {
   // XP thresholds based on the party size
   const playerAdjustmentThresholds = {
    trivial: 10,
    low: 20,
    moderate: 20,
    severe: 30,
    extreme: 40
}

const diffOff = partySize - 4;

const baseThresholds : Thresholds = new Map([
    [EncounterDifficulty.Trivial, 40 + playerAdjustmentThresholds.trivial*diffOff],
    [EncounterDifficulty.Low, 60 + playerAdjustmentThresholds.low*diffOff],
    [EncounterDifficulty.Moderate, 80 + playerAdjustmentThresholds.moderate*diffOff],
    [EncounterDifficulty.Severe, 120 + playerAdjustmentThresholds.severe*diffOff],
    [EncounterDifficulty.Extreme, 160 + playerAdjustmentThresholds.extreme*diffOff]
]);

return baseThresholds;
}

export function getEncounterExperienceBoundaries(partySize: number): Boundaries {
    const baseThresholds = getEncounterDifficultyThresholds(partySize);
    const baseBoundaries : Boundaries = new Map([
        [EncounterDifficulty.Trivial, [0, (baseThresholds.get(EncounterDifficulty.Trivial!)! + baseThresholds.get(EncounterDifficulty.Low)!) / 2]],
        [EncounterDifficulty.Low, [(baseThresholds.get(EncounterDifficulty.Trivial)! + baseThresholds.get(EncounterDifficulty.Low)!) / 2, (baseThresholds.get(EncounterDifficulty.Low)! + baseThresholds.get(EncounterDifficulty.Moderate)!) / 2]],
        [EncounterDifficulty.Moderate, [(baseThresholds.get(EncounterDifficulty.Low)! + baseThresholds.get(EncounterDifficulty.Moderate)!) / 2, (baseThresholds.get(EncounterDifficulty.Moderate)! + baseThresholds.get(EncounterDifficulty.Severe)!) / 2]],
        [EncounterDifficulty.Severe, [(baseThresholds.get(EncounterDifficulty.Moderate)! + baseThresholds.get(EncounterDifficulty.Severe)!) / 2, (baseThresholds.get(EncounterDifficulty.Severe)! + baseThresholds.get(EncounterDifficulty.Extreme)!) / 2]],
        [EncounterDifficulty.Extreme, [(baseThresholds.get(EncounterDifficulty.Severe)! + baseThresholds.get(EncounterDifficulty.Extreme)!) / 2, Infinity]]
    ]);
    
    return baseBoundaries;
}

export function getSeverityFromRawExperience(rawTotalXP: number, partySize: number): EncounterDifficulty {
    // XP thresholds based on the party size
    const baseBoundaries = getEncounterExperienceBoundaries(partySize);

    // Check which range the rawTotalXP falls into
    if (rawTotalXP < baseBoundaries.get(EncounterDifficulty.Trivial)![1]) return EncounterDifficulty.Trivial;
    if (rawTotalXP < baseBoundaries.get(EncounterDifficulty.Low)![1]) return EncounterDifficulty.Low;
    if (rawTotalXP < baseBoundaries.get(EncounterDifficulty.Moderate)![1]) return EncounterDifficulty.Moderate;
    if (rawTotalXP < baseBoundaries.get(EncounterDifficulty.Severe)![1]) return EncounterDifficulty.Severe;
    return EncounterDifficulty.Extreme;
}

// This function is used to calculate the severity of an encounter based on the final XP value
// extraExperience is NOT included in severity calculation, but is included in totalXP, so we have to subtract it
export function getSeverityFromFinalExperience(totalXP: number, extraExperience : number): EncounterDifficulty {
    // TODO: Check this
    const totalXPWithoutExtra = totalXP - extraExperience;
    // XP thresholds based on the party size
    const baseBoundaries = getEncounterExperienceBoundaries(4); // 4 is default - no modifications, representing final value.

    // Check which range the rawTotalXP falls into
    if (totalXPWithoutExtra < baseBoundaries.get(EncounterDifficulty.Trivial)![1]) return EncounterDifficulty.Trivial;
    if (totalXPWithoutExtra < baseBoundaries.get(EncounterDifficulty.Low)![1]) return EncounterDifficulty.Low;
    if (totalXPWithoutExtra < baseBoundaries.get(EncounterDifficulty.Moderate)![1]) return EncounterDifficulty.Moderate;
    if (totalXPWithoutExtra < baseBoundaries.get(EncounterDifficulty.Severe)![1]) return EncounterDifficulty.Severe;
    return EncounterDifficulty.Extreme;
}

export function getAdjustedExperienceFromPartySize(rawTotalXP: number, partySize: number): number {
    if (rawTotalXP === 0) return rawTotalXP; // TODO: Added this becuase empty ones were causing  issues- but ensure the math isn't generally wrong
    const severity = getSeverityFromRawExperience(rawTotalXP, partySize);

    const playerAdjustmentThresholds = {
        trivial: 10,
        low: 20,
        moderate: 20,
        severe: 30,
        extreme: 40
    }

    const diffOff = partySize - 4;
    if (severity === EncounterDifficulty.Extreme) return rawTotalXP - playerAdjustmentThresholds.extreme*diffOff;
    if (severity === EncounterDifficulty.Severe) return rawTotalXP - playerAdjustmentThresholds.severe*diffOff;
    if (severity === EncounterDifficulty.Moderate) return rawTotalXP - playerAdjustmentThresholds.moderate*diffOff;
    if (severity === EncounterDifficulty.Low) return rawTotalXP - playerAdjustmentThresholds.low*diffOff;
    return rawTotalXP;
}

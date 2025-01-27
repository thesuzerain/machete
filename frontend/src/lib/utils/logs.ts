import type { Character, InsertEvent, WIPLogEnemy, WIPLogTreasure } from "$lib/types/types";
import { getExperienceFromLevel } from "./encounter";

export function generateEventsFromData(characterIds: number[], characters:Character[], enemies: WIPLogEnemy[], treasures: WIPLogTreasure[]): InsertEvent[] {
    const events: InsertEvent[] = [];
    
    // Generate defeat and experience events for each enemy/hazard
    for (const enemy of enemies) {
        for (const characterId of characterIds) {
            events.push({
                character: characterId,
                event_type: enemy.type === 'enemy' ? 'EnemyDefeated' : 'HazardDefeated',
                description: `Defeated ${enemy.count} ${enemy.type}`,
                data: {
                    id: enemy.id,
                    count: enemy.count
                }
            });

            // Add experience event
            events.push({
                character: characterId,
                event_type: 'ExperienceGain',
                description: `Gained experience from ${enemy.type}`,
                data: {
                    experience: getExperienceFromLevel(enemy.level || 0, characters.find(c => c.id === characterId)?.level || 0)
                }
            });
        }
    }

    // Generate treasure events
    for (const treasure of treasures) {
        for (const characterId of characterIds) {
            events.push({
                character: characterId,
                event_type: treasure.type === 'currency' ? 'CurrencyGain' : 'ItemGain',
                description: treasure.type === 'currency' 
                    ? `Gained ${treasure.amount} currency`
                    : `Gained item`,
                data: treasure.type === 'currency' 
                    ? { currency: { gold: treasure.amount } }
                    : { id: treasure.itemId }
            });
        }
    }

    return events;
}
export type LogTemplate = {
    id: string;
    name: string;
    description: string;
    eventGenerators: EventGenerator[];
}

type EventGenerator = {
    event_type: string;
    generateData: (formData: FormData, characterId: number) => any;
}

export const LOG_TEMPLATES: Record<string, LogTemplate> = {
    WonBattle: {
        id: 'WonBattle',
        name: 'Won Battle',
        description: 'Record winning a battle, including experience and loot gained',
        eventGenerators: [
            {
                event_type: 'ExperienceGain',
                generateData: (formData, _) => ({
                    experience: parseInt(formData.get('experience') as string)
                })
            },
            {
                event_type: 'CurrencyGain',
                generateData: (formData, _) => ({
                    currency: parseInt(formData.get('currency') as string)
                })
            },
            {
                event_type: 'EnemyDefeated',
                generateData: (formData, _) => ({
                    enemy_name: formData.get('enemy_name'),
                    count: parseInt(formData.get('enemy_count') as string)
                })
            }
        ]
    },
    // Add more templates as needed
} 
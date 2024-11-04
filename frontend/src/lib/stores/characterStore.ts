import { writable } from 'svelte/store';
import type { Character } from '$lib/types/types';

function createCharacterStore() {
    const { subscribe, set, update } = writable<Character[]>([]);

    return {
        subscribe,
        set,
        add: (character: Character) => update((characters: Character[]) => [...characters, character]),
        remove: (id: number) => update((characters: Character[]) => 
            characters.filter((character: Character) => character.id !== id)
        ),
        update: (updatedCharacter: Character) => update((characters: Character[]) =>
            characters.map((character: Character) => 
                character.id === updatedCharacter.id ? updatedCharacter : character
            )
        ),
        getByCampaign: (campaignId: number) => {
            let characters: Character[] = [];
            subscribe((chars: Character[]) => {
                characters = chars.filter((char: Character) => char.campaign_id === campaignId);
            });
            return characters;
        }
    };
}

export const characters = createCharacterStore();
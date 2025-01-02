import { writable } from 'svelte/store';
import type { Character } from '$lib/types/types';
import { API_URL } from '$lib/config';
import { auth } from './auth';

function createCharacterStore() {
    const { subscribe, set, update } = writable<Record<string, Character[]>>({});

    return {
        subscribe,
        fetchCharacters: async (campaignId: string) => {
            try {
                const response = await fetch(`${API_URL}/campaign/${campaignId}/characters`, {
                    credentials: 'include',
                });
                if (!response.ok) throw new Error('Failed to fetch characters');
                const characters = await response.json();
                update(store => ({ ...store, [campaignId]: characters }));
                return characters;
            } catch (e) {
                console.error('Error fetching characters:', e);
                return [];
            }
        },
        addCharacter: async (campaignId: string, character: Omit<Character, 'id'>) => {
            try {
                const response = await fetch(`${API_URL}/campaign/${campaignId}/characters`, {
                    method: 'POST',
                    credentials: 'include',
                    headers: {
                        'Content-Type': 'application/json',
                    },
                    body: JSON.stringify([character]),
                });
                if (!response.ok) throw new Error('Failed to create character');
                await characterStore.fetchCharacters(campaignId);
            } catch (e) {
                console.error('Error adding character:', e);
                throw e;
            }
        },
        updateCharacter: async (campaignId: string, character: Character) => {
            try {
                const response = await fetch(`${API_URL}/campaign/${campaignId}/characters/${character.id}`, {
                    method: 'PUT',
                    credentials: 'include',
                    headers: {
                        'Content-Type': 'application/json',
                    },
                    body: JSON.stringify(character),
                });
                if (!response.ok) throw new Error('Failed to update character');
                await characterStore.fetchCharacters(campaignId);
            } catch (e) {
                console.error('Error updating character:', e);
                throw e;
            }
        },
        reset: () => set({}),
    };
}

export const characterStore = createCharacterStore();

auth.subscribe(($auth) => {
    if (!$auth.user) {
        characterStore.reset();
    }
}); 
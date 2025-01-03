import { writable } from 'svelte/store';
import type { Character } from '$lib/types/types';
import { API_URL } from '$lib/config';

function createCharacterStore() {
    const { subscribe, set, update } = writable<Character[]>([]);

    return {
        subscribe,
        fetchCharacters: async (campaignId : number) => {
            try {
                const response = await fetch(`${API_URL}/campaign/${campaignId}/characters`, {
                    credentials: 'include'
                });
                if (!response.ok) throw new Error('Failed to fetch characters');
                const characters = await response.json();
                set(characters);
            } catch (error) {
                console.error('Error fetching characters:', error);
                throw error;
            }
        },
        addCharacter: async (campaignId : number, character: Omit<Character, 'id'>) => {
            try {
                const response = await fetch(`${API_URL}/campaign/${campaignId}/characters`, {
                    method: 'POST',
                    credentials: 'include',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify(character),
                });
                if (!response.ok) throw new Error('Failed to add character');
                const newCharacter = await response.json();
                update(chars => [...chars, newCharacter]);
                return newCharacter;
            } catch (error) {
                console.error('Error adding character:', error);
                throw error;
            }
        },
        updateCharacter: async (campaignId : number, id: number, character: Partial<Character>) => {
            try {
                const response = await fetch(`${API_URL}/campaign/${campaignId}/characters/${id}`, {
                    method: 'PUT',
                    credentials: 'include',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify(character),
                });
                if (!response.ok) throw new Error('Failed to update character');
                const updatedCharacter = await response.json();
                update(chars => chars.map(char => 
                    char.id === id ? updatedCharacter : char
                ));
                return updatedCharacter;
            } catch (error) {
                console.error('Error updating character:', error);
                throw error;
            }
        },
        deleteCharacter: async (campaignId : number, id: number) => {
            try {
                const response = await fetch(`${API_URL}/campaign/${campaignId}/characters/${id}`, {
                    method: 'DELETE',
                    credentials: 'include',
                });
                if (!response.ok) throw new Error('Failed to delete character');
                update(chars => chars.filter(char => char.id !== id));
            } catch (error) {
                console.error('Error deleting character:', error);
                throw error;
            }
        }
    };
}

export const characterStore = createCharacterStore(); 
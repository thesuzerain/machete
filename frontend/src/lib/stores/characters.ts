import { writable } from 'svelte/store';
import type { Character } from '$lib/types/types';
import { API_URL } from '$lib/config';

function createCharacterStore() {
    const { subscribe, set, update } = writable<Map<number,Character[]>>(new Map());

    return {
        subscribe,
        fetchCharacters: async (campaignId : number) => {
            try {
                const response = await fetch(`${API_URL}/campaign/${campaignId}/characters`, {
                    credentials: 'include'
                });
                if (!response.ok) throw new Error('Failed to fetch characters');
                const characters = await response.json();
                
                update(chars => {
                    chars.set(campaignId, characters);
                    return new Map(chars);
                });
            } catch (error) {
                console.error('Error fetching characters:', error);
                throw error;
            }
        },
        addCharacters: async (campaignId : number, newCharacters: Omit<Character, 'id'>[]) => {
            try {
                const response = await fetch(`${API_URL}/campaign/${campaignId}/characters`, {
                    method: 'POST',
                    credentials: 'include',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify(newCharacters),
                });
                if (!response.ok) {
                    console.error('Failed to add characters:', response);
                    throw new Error('Failed to add characters');
                }
                
                // Refresh characters after adding
                await characterStore.fetchCharacters(campaignId);
            } catch (error) {
                console.error('Error adding characters:', error);
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
                
                // Refresh characters after updating
                await characterStore.fetchCharacters(campaignId);
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
                update(chars => {
                    chars.set(campaignId, chars.get(campaignId)!.filter(c => c.id !== id));
                    return new Map(chars);
                });
            } catch (error) {
                console.error('Error deleting character:', error);
                throw error;
            }
        }
    };
}

export const characterStore = createCharacterStore(); 
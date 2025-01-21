import { writable } from 'svelte/store';
import type { Encounter, CreateEncounter } from '$lib/types/types';
import { API_URL } from '$lib/config';
import { auth } from './auth';

function createEncounterStore() {
    const { subscribe, set, update } = writable<Encounter[]>([]);
    let currentDraft: CreateEncounter | null = null;

    return {
        subscribe,
        fetchEncounters: async () => {
            // TODO: Cache these- don't fetch if we already have them.
            // More generally, do a better caching solution for most of these. all of them could use it except for library (which is ALWAYS cached, more or less)
            try {
                const response = await fetch(`${API_URL}/encounters`, {
                    credentials: 'include',
                });
                if (!response.ok) throw new Error('Failed to fetch encounters');
                const encounters = await response.json();
                set(encounters);
            } catch (e) {
                console.error('Error fetching encounters:', e);
                set([]);
            }
        },
        getDraft: async () => {
            try {
                const response = await fetch(`${API_URL}/encounters/draft`, {
                    credentials: 'include',
                });
                if (!response.ok) return null;
                currentDraft = await response.json();
                return currentDraft;
            } catch (e) {
                console.error('Error fetching draft:', e);
                return null;
            }
        },
        updateDraft: async (draft: CreateEncounter) => {
            try {
                const response = await fetch(`${API_URL}/encounters/draft`, {
                    method: 'POST',
                    credentials: 'include',
                    headers: {
                        'Content-Type': 'application/json',
                    },
                    body: JSON.stringify(draft),
                });
                if (!response.ok) throw new Error('Failed to update draft');
                currentDraft = draft;
            } catch (e) {
                console.error('Error updating draft:', e);
                throw e;
            }
        },
        reset: () => {
            set([]);
            currentDraft = null;
        }
    };
}

export const encounterStore = createEncounterStore();

// Subscribe to auth changes to reset store on logout
auth.subscribe(($auth) => {
    if (!$auth.user) {
        encounterStore.reset();
    }
}); 
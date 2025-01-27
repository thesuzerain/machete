import { writable } from 'svelte/store';
import type { Encounter, CreateEncounter, CreateEncounterFinalized } from '$lib/types/encounters';
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
        addEncounter: async (encounter: CreateEncounterFinalized) => {
            try {
                const response = await fetch(`${API_URL}/encounters`, {
                    method: 'POST',
                    credentials: 'include',
                    headers: {
                        'Content-Type': 'application/json',
                    },
                    body: JSON.stringify([{
                        ...encounter,
                    }]),
                });

                if (!response.ok) throw new Error('Failed to create encounter');
                await encounterStore.fetchEncounters();
            } catch (e) {
                console.error('Error adding encounter:', e);
                throw e;
            }
        },
        replaceEncounter: async (id : number, encounter: CreateEncounterFinalized) => {
            try {
                const response = await fetch(`${API_URL}/encounters/${id}`, {
                    method: 'PUT',
                    credentials: 'include',
                    headers: {
                        'Content-Type': 'application/json',
                    },
                    body: JSON.stringify(encounter),
                });

                if (!response.ok) throw new Error('Failed to update encounter');
                await encounterStore.fetchEncounters();
            } catch (e) {
                console.error('Error updating encounter:', e);
                throw e;
            }
        },
        updateEncounter: async (id : number, encounter: Partial<Encounter>) => {
            try {
                const response = await fetch(`${API_URL}/encounters/${id}`, {
                    method: 'PATCH',
                    credentials: 'include',
                    headers: {
                        'Content-Type': 'application/json',
                    },
                    body: JSON.stringify(encounter),
                });

                if (!response.ok) throw new Error('Failed to update encounter');
                await encounterStore.fetchEncounters();

                // TODO: May be helpful to have a refresh for campaign sessions as well here.
            } catch (e) {
                console.error('Error updating encounter:', e);
                throw e;
            }
        },
        deleteEncounter: async (id : number) => {
            try {
                const response = await fetch(`${API_URL}/encounters/${id}`, {
                    method: 'DELETE',
                    credentials: 'include',
                });

                if (!response.ok) throw new Error('Failed to delete encounter');
                await encounterStore.fetchEncounters();
            } catch (e) {
                console.error('Error deleting encounter:', e);
                throw e;
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
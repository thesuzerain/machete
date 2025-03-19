import { get, writable } from 'svelte/store';
import { type Encounter, type CreateOrReplaceEncounterExtended, type CreateEncounterFinalized, type CreateOrReplaceEncounter, type AccomplishmentLevel, experienceForAccomplishment } from '$lib/types/encounters';
import { API_URL } from '$lib/config';
import { auth } from './auth';
import { campaignSessionStore } from './campaignSessions';
import { campaignStore, selectedCampaignStore } from './campaigns';

function createEncounterStore() {
    const { subscribe, set, update } = writable<Encounter[]>([]);
    let currentDraft: CreateOrReplaceEncounter | null = null;

    return {
        subscribe,
        fetchEncounters: async () => {
            // TODO: Cache these- don't fetch if we already have them.
            // More generally, do a better caching solution for most of these. all of them could use it except for library (which is ALWAYS cached, more or less)
            try {
                const response = await fetch(`${API_URL}/encounters`, {
                    credentials: 'include',
                });
                if (!response.ok) throw new Error(`Failed to fetch encounters: ${response.status}`);
                const encounters = await response.json();
                set(encounters);
            } catch (e) {
                console.error('Error fetching encounters:', e);
                set([]);
            }
        },
        // addEncounter
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
        addQuickAccomplishment: async (campaignId : number, campaignSessionId : number, name : string, xp : number | AccomplishmentLevel) => {
            if (typeof xp === 'string') {
                xp = experienceForAccomplishment(xp);
            }
            const encounter : CreateEncounterFinalized = {
                name: name,
                description: '',
                session_id: campaignSessionId,
                extra_experience: xp,
                treasure_items: [],
                encounter_type: 'accomplishment',
                status: 'Prepared',
                treasure_currency: 0,
                party_level: 0, // TODO: I believe this doesn't matter for accomplishments. Revisit if it does.
                party_size: 0, // TODO: ^^

                total_experience: xp,
                total_items_value: 0,
            };
            await encounterStore.addEncounter(encounter);
            await campaignSessionStore.fetchCampaignSessions(campaignId);
        },
        updateEncounter: async (id : number, encounter: Partial<CreateOrReplaceEncounterExtended>) => {
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

                // Update encounters (and sessions if we're in a campaign)
                let campaignId = get(selectedCampaignStore);
                if (campaignId) {
                await Promise.all([
                    campaignSessionStore.fetchCampaignSessions(campaignId),
                    await encounterStore.fetchEncounters()
                ]);
            } else {
                await encounterStore.fetchEncounters();
            }
                
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
        unlinkEncounterFromSession: async (encounterId: number) => {
            try {
                const response = await fetch(`${API_URL}/encounters/${encounterId}/session`, {
                    method: 'DELETE',
                    credentials: 'include',
                });

                if (!response.ok) throw new Error('Failed to unlink encounter');

                // Refresh session and encounter data after unlinking
                const selectedCampaign = get(selectedCampaignStore);
                if (selectedCampaign) {
                    await Promise.all([
                        campaignSessionStore.fetchCampaignSessions(selectedCampaign),
                        encounterStore.fetchEncounters(),
                    ]);
                } else {
                    await encounterStore.fetchEncounters();
                }

            } catch (e) {
                console.error('Error unlinking encounter:', e);
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
        updateDraft: async (draft: CreateOrReplaceEncounter) => {
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
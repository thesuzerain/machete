import { writable } from 'svelte/store';
import type { Campaign } from '$lib/types/types';
import { API_URL } from '$lib/config';
import { auth } from './auth';

function createCampaignStore() {
    const { subscribe, set, update } = writable<Campaign[]>([]);

    return {
        subscribe,
        fetchCampaigns: async () => {
            try {
                const response = await fetch(`${API_URL}/campaign`, {
                    credentials: 'include',
                });
                if (!response.ok) throw new Error('Failed to fetch campaigns');
                const campaigns = await response.json();
                set(campaigns);
            } catch (e) {
                console.error('Error fetching campaigns:', e);
                set([]);
            }
        },
        reset: () => set([]),
        addCampaign: async (campaign: Campaign) => {
            try {
                const response = await fetch(`${API_URL}/campaign`, {
                    method: 'POST',
                    credentials: 'include',
                    headers: {
                        'Content-Type': 'application/json',
                    },
                    body: JSON.stringify(campaign),
                });
                if (!response.ok) throw new Error('Failed to create campaign');
                
                // Refresh campaigns after adding
                await campaignStore.fetchCampaigns();
            } catch (e) {
                console.error('Error adding campaign:', e);
                throw e;
            }
        }
    };
}

export const campaignStore = createCampaignStore();

// Subscribe to auth changes to reset store on logout
auth.subscribe(($auth) => {
    if (!$auth.user) {
        campaignStore.reset();
    }
}); 
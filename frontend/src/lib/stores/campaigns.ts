import { writable } from 'svelte/store';
import type { Campaign, InsertInitialCampaignData } from '$lib/types/types';
import { API_URL } from '$lib/config';
import { auth } from './auth';

function createCampaignStore() {
    const { subscribe, set, update } = writable<Map<number,Campaign>>(new Map());

    return {
        subscribe,
        fetchCampaigns: async () => {
            try {
                const response = await fetch(`${API_URL}/campaign`, {
                    credentials: 'include',
                });
                if (!response.ok) throw new Error('Failed to fetch campaigns');
                const campaigns : Campaign[] = await response.json();

                const campaignMap = new Map();
                campaigns.forEach(campaign => {
                    campaignMap.set(campaign.id, campaign);
                });
                set(campaignMap);
            } catch (e) {
                console.error('Error fetching campaigns:', e);
                set(new Map());
            }
        },
        reset: () => set(new Map()),
        addCampaign: async (campaign: Omit<Campaign, "id">, initialData: InsertInitialCampaignData | null = null) => {
            try {
                const response = await fetch(`${API_URL}/campaign`, {
                    method: 'POST',
                    credentials: 'include',
                    headers: {
                        'Content-Type': 'application/json',
                    },
                    body: JSON.stringify({initialization: initialData, ...campaign}),
                });
                if (!response.ok) throw new Error('Failed to create campaign');
                
                // Refresh campaigns after adding
                await campaignStore.fetchCampaigns();

                return await response.json();
            } catch (e) {
                console.error('Error adding campaign:', e);
                throw e;
            }
        },
        importCampaign: async (rawJsonString : string) : Promise<number> => {
            try {
                const jsonStructure = JSON.parse(rawJsonString);

                const response = await fetch(`${API_URL}/campaign/import`, {
                    method: 'POST',
                    credentials: 'include',
                    headers: {
                        'Content-Type': 'application/json',
                    },
                    body: JSON.stringify(jsonStructure),
                });
                if (!response.ok) throw new Error('Failed to import campaign');
                
                // Returned data id of the new campaign
                const data = await response.json();
                
                // Refresh campaigns after adding
                await campaignStore.fetchCampaigns();

                return data.id;
            } catch (e) {
                console.error('Error importing campaign:', e);
                throw e;
            }
        },
        exportCampaign: async (id: number) : Promise<any> => {
            try {
                const response = await fetch(`${API_URL}/campaign/${id}/export`, {
                    credentials: 'include',
                });
                if (!response.ok) throw new Error('Failed to export campaign');
                
                return await response.json();
            } catch (e) {
                console.error('Error exporting campaign:', e);
                throw e;
            }
        },
    };
}

export const selectedCampaignStore = writable<number | null>(null);
export const campaignStore = createCampaignStore();

// Subscribe to auth changes to reset store on logout
auth.subscribe(($auth) => {
    if (!$auth.user) {
        campaignStore.reset();
    }
}); 
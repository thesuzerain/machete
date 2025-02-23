import { writable } from 'svelte/store';
import type { CampaignStats } from '$lib/types/stats';
import { API_URL } from '$lib/config';

function createStatsStore() {
    const { subscribe, set, update } = writable<Map<number, CampaignStats>>(new Map());

    return {
        subscribe,
        fetchStats: async (campaignId: number) => {
            const response = await fetch(`${API_URL}/campaign/${campaignId}/stats`, {
                credentials: 'include',
            });
            if (!response.ok) throw new Error('Failed to fetch campaign stats');
            const stats = await response.json();
            
            update(map => {
                map.set(campaignId, stats);
                return map;
            });
        },
        clear: () => set(new Map())
    };
}

export const statsStore = createStatsStore(); 
import { writable } from 'svelte/store';
import type { Campaign } from '$lib/types/types';

function createCampaignStore() {
    const { subscribe, set, update } = writable<Campaign[]>([]);

    return {
        subscribe,
        set,
        add: (campaign: Campaign) => update((campaigns: Campaign[]) => [...campaigns, campaign]),
        remove: (id: number) => update((campaigns: Campaign[]) => 
            campaigns.filter((campaign: Campaign) => campaign.id !== id)
        ),
        update: (updatedCampaign: Campaign) => update((campaigns: Campaign[]) =>
            campaigns.map((campaign: Campaign) => 
                campaign.id === updatedCampaign.id ? updatedCampaign : campaign
            )
        )
    };
}

export const campaigns = createCampaignStore(); 
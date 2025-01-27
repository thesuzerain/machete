import { writable } from 'svelte/store';
import type { CampaignSession } from '$lib/types/types';
import { API_URL } from '$lib/config';

function createCampaignSessionStore() {
    const { subscribe, set, update } = writable<Map<number,CampaignSession[]>>(new Map());

    return {
        subscribe,
        fetchCampaignSessions: async (campaignId : number) => {
            try {

                const response = await fetch(`${API_URL}/campaign/${campaignId}/sessions`, {
                    credentials: 'include'
                });
                if (!response.ok) throw new Error('Failed to fetch sessions');

                // These are ordered by session_order
                let campaignSessions : CampaignSession[] = await response.json();
                
                campaignSessions.sort((a, b) => a.session_order - b.session_order);
                
                update(chars => {
                    chars.set(campaignId, campaignSessions);
                    return new Map(chars);
                });
            } catch (error) {
                console.error('Error fetching sessions:', error);
                throw error;
            }
        },
        addCampaignSessions: async (campaignId : number, newCampaignSessions: Omit<CampaignSession, 'id' | 'play_date'>[]) => {
            try {
                const response = await fetch(`${API_URL}/campaign/${campaignId}/sessions`, {
                    method: 'POST',
                    credentials: 'include',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify(newCampaignSessions),
                });
                if (!response.ok) {
                    console.error('Failed to add sessions:', response);
                    throw new Error('Failed to add sessions');
                }
                
                // Refresh sessions after adding
                await campaignSessionStore.fetchCampaignSessions(campaignId);
            } catch (error) {
                console.error('Error adding sessions:', error);
                throw error;
            }
        },
        updateCampaignSession: async (campaignId : number, campaignSession: Partial<CampaignSession>) => {
            await campaignSessionStore.updateCampaignSessions(campaignId, [campaignSession]);
        },
        updateCampaignSessions: async (campaignId : number, campaignSessions: Partial<CampaignSession>[]) => {
            try {
                const campaignSessionsMap = Object.fromEntries(campaignSessions.map(c => [c.id, c]));
                const response = await fetch(`${API_URL}/campaign/${campaignId}/sessions`, {
                    method: 'PATCH',
                    credentials: 'include',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify(campaignSessionsMap),
                });
                if (!response.ok) throw new Error('Failed to update campaign sessions');
                
                // Refresh sessions after updating
                await campaignSessionStore.fetchCampaignSessions(campaignId);
            } catch (error) {
                console.error('Error updating sessions:', error);
                throw error;
            }
        },
        deleteCampaignSessions: async (campaignId : number, id: number) => {
            try {
                const response = await fetch(`${API_URL}/campaign/${campaignId}/sessions/${id}`, {
                    method: 'DELETE',
                    credentials: 'include',
                });
                if (!response.ok) throw new Error('Failed to delete sessions');
                update(chars => {
                    chars.set(campaignId, chars.get(campaignId)!.filter(c => c.id !== id));
                    return new Map(chars);
                });
            } catch (error) {
                console.error('Error deleting sessions:', error);
                throw error;
            }
        }
    };
}

export const campaignSessionStore = createCampaignSessionStore(); 
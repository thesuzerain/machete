import { writable } from 'svelte/store';
import type { Log } from '$lib/types/types';
import { API_URL } from '$lib/config';
import { auth } from './auth';

function createLogStore() {
    const { subscribe, set, update } = writable<Record<string, Log[]>>({});

    return {
        subscribe,
        fetchLogs: async (campaignId: string) => {
            try {
                const response = await fetch(`${API_URL}/campaign/${campaignId}/logs`, {
                    credentials: 'include',
                });
                if (!response.ok) throw new Error('Failed to fetch logs');
                const logs = await response.json();
                update(store => ({ ...store, [campaignId]: logs }));
                return logs;
            } catch (e) {
                console.error('Error fetching logs:', e);
                return [];
            }
        },
        addLog: async (campaignId: string, log: Omit<Log, 'id' | 'timestamp'>) => {
            try {
                const response = await fetch(`${API_URL}/campaign/${campaignId}/logs`, {
                    method: 'POST',
                    credentials: 'include',
                    headers: {
                        'Content-Type': 'application/json',
                    },
                    body: JSON.stringify(log),
                });
                if (!response.ok) throw new Error('Failed to create log');
                await logStore.fetchLogs(campaignId);
            } catch (e) {
                console.error('Error adding log:', e);
                throw e;
            }
        },
        updateLog: async (campaignId: string, logId: number, updates: Partial<Log>) => {
            try {
                const response = await fetch(`${API_URL}/campaign/${campaignId}/logs/${logId}`, {
                    method: 'PUT',
                    credentials: 'include',
                    headers: {
                        'Content-Type': 'application/json',
                    },
                    body: JSON.stringify(updates),
                });
                if (!response.ok) throw new Error('Failed to update log');
                await logStore.fetchLogs(campaignId);
            } catch (e) {
                console.error('Error updating log:', e);
                throw e;
            }
        },
        deleteLog: async (campaignId: string, logId: number) => {
            try {
                const response = await fetch(`${API_URL}/campaign/${campaignId}/logs/${logId}`, {
                    method: 'DELETE',
                    credentials: 'include',
                });
                if (!response.ok) throw new Error('Failed to delete log');
                await logStore.fetchLogs(campaignId);
            } catch (e) {
                console.error('Error deleting log:', e);
                throw e;
            }
        },
        reset: () => set({}),
    };
}

export const logStore = createLogStore();

auth.subscribe(($auth) => {
    if (!$auth.user) {
        logStore.reset();
    }
}); 
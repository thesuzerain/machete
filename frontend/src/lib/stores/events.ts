import { writable } from 'svelte/store';
import type { Event } from '$lib/types/types';
import { API_URL } from '$lib/config';
import { auth } from './auth';

function createEventStore() {
    const { subscribe, set, update } = writable<Record<string, Event[]>>({});

    return {
        subscribe,
        fetchEvents: async (campaignId: string) => {
            try {
                const response = await fetch(`${API_URL}/campaign/${campaignId}/events`, {
                    credentials: 'include',
                });
                if (!response.ok) throw new Error('Failed to fetch events');
                const events = await response.json();
                update(store => ({ ...store, [campaignId]: events }));
                return events;
            } catch (e) {
                console.error('Error fetching events:', e);
                return [];
            }
        },
        addEvent: async (campaignId: string, event: Omit<Event, 'id'>) => {
            try {
                const response = await fetch(`${API_URL}/campaign/${campaignId}/events`, {
                    method: 'POST',
                    credentials: 'include',
                    headers: {
                        'Content-Type': 'application/json',
                    },
                    body: JSON.stringify({
                        event_group: event.event_group,
                        events: [event]
                    }),
                });
                if (!response.ok) throw new Error('Failed to create event');
                await eventStore.fetchEvents(campaignId);
            } catch (e) {
                console.error('Error adding event:', e);
                throw e;
            }
        },
        deleteEvents: async (campaignId: string, eventIds: number[]) => {
            try {
                const response = await fetch(`${API_URL}/campaign/${campaignId}/events`, {
                    method: 'DELETE',
                    credentials: 'include',
                    headers: {
                        'Content-Type': 'application/json',
                    },
                    body: JSON.stringify(eventIds),
                });
                if (!response.ok) throw new Error('Failed to delete events');
                await eventStore.fetchEvents(campaignId);
            } catch (e) {
                console.error('Error deleting events:', e);
                throw e;
            }
        },
        reset: () => set({}),
    };
}

export const eventStore = createEventStore();

auth.subscribe(($auth) => {
    if (!$auth.user) {
        eventStore.reset();
    }
}); 
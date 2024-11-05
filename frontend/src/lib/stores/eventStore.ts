import { writable } from 'svelte/store';
import type { Event } from '$lib/types/types';

function createEventStore() {
    const { subscribe, set, update } = writable<Event[]>([]);

    return {
        subscribe,
        set,
        add: (event: Event) => update((events: Event[]) => [...events, event]),
        remove: (id: number) => update((events: Event[]) => 
            events.filter((evt: Event) => evt.id !== id)
        ),
        update: (updatedEvent: Event) => update((events: Event[]) =>
            events.map((evt: Event) => 
                evt.id === updatedEvent.id ? updatedEvent : evt
            )
        ),
        getByCampaign: (campaignId: number) => {
            let events: Event[] = [];
            subscribe((evts: Event[]) => {
                events = evts.filter((evt: Event) => evt.campaign_id === campaignId);
            });
            return events;
        }
    };
}

export const events = createEventStore();
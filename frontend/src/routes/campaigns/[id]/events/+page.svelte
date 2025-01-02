<script lang="ts">
    import { page } from '$app/stores';
    import { onMount } from 'svelte';
    import type { Event, Character, InsertEvent } from '$lib/types/types';
    import EventManager from '$lib/components/EventManager.svelte';
    import { API_URL } from '$lib/config';
    import { requireAuth } from '$lib/guards/auth';

    const campaignId = parseInt($page.params.id);
    let campaignEvents: Event[] = [];
    let campaignCharacters: Character[] = [];
    let loading = true;
    let error: string | null = null;
    let selectedEventType: string = 'CurrencyGain';
    let selectedCharacterIds: string[] = [];
    // Add filter states
    let filterCharacterId: string = '';
    let filterStartDate: string = '';
    let filterEndDate: string = '';
    
    // Add new state variables
    let selectedEventIds: number[] = [];
    let editingEvent: Event | null = null;
    
    // Helper function to get all unique keys from event_type objects
    function getEventTypeKeys(events: Event[]): string[] {
        const keys = new Set<string>();
        events.forEach(event => {
            Object.keys(event.data).forEach(key => {
                if (key !== 'event_type') keys.add(key);
            });
        });
        return Array.from(keys);
    }
    
    // Function to fetch filtered events
    async function fetchEvents() {
        try {
            let url = `${API_URL}/campaign/${campaignId}/events?`;
            const params = new URLSearchParams();
            
            if (filterCharacterId) params.append('character_id', filterCharacterId);
            if (filterStartDate) params.append('start_date', filterStartDate);
            if (filterEndDate) params.append('end_date', filterEndDate);
            
            const eventsResponse = await fetch(url + params.toString());
            if (!eventsResponse.ok) throw new Error('Failed to fetch events');
            campaignEvents = await eventsResponse.json();
        } catch (e) {
            error = e instanceof Error ? e.message : 'Failed to fetch events';
        }
    }
    
    // Watch filter changes
    $: {
        if (!loading) {
            fetchEvents();
        }
    }

    onMount(async () => {
        requireAuth();

        try {
            // Fetch characters for character selection
            const charactersResponse = await fetch(`${API_URL}/campaign/${campaignId}/characters`, {
                credentials: 'include',
            });
            if (!charactersResponse.ok) throw new Error('Failed to fetch characters');
            campaignCharacters = await charactersResponse.json();

            // Fetch events
            const eventsResponse = await fetch(`${API_URL}/campaign/${campaignId}/events`, {
                credentials: 'include',
            });
            if (!eventsResponse.ok) throw new Error('Failed to fetch events');
            campaignEvents = await eventsResponse.json();
        } catch (e) {
            error = e instanceof Error ? e.message : 'An error occurred';
        } finally {
            loading = false;
        }
    });

    async function addEvent(event: SubmitEvent) {
        event.preventDefault();
        const form = event.target as HTMLFormElement;
        const formData = new FormData(form);
        
        const eventType = formData.get('event_type') as string;

        let eventTypeData: any;
        let description: string;
        
        switch (eventType) {
            case 'CurrencyGain':
                const currency = parseInt(formData.get('currency') as string);
                eventTypeData = { currency };
                description = `Gained ${currency} currency`;
                break;
            case 'ExperienceGain':
                const experience = parseInt(formData.get('experience') as string);
                eventTypeData = { experience };
                description = `Gained ${experience} experience`;
                break;
            default:
                throw new Error('Invalid event type');
        }

        const characterIds = formData.getAll('character_id') as string[];
        const newEvents: InsertEvent[] = characterIds.map(character_id => ({
            character: parseInt(character_id),
            event_type: eventType,
            description,
            data: eventTypeData,
        }));

        try {
            const response = await fetch(`${API_URL}/campaign/${campaignId}/events`, {
                method: 'POST',
                credentials: 'include',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(newEvents),
            });

            if (!response.ok) throw new Error('Failed to create events');
            
            // Refresh events list
            const eventsResponse = await fetch(`${API_URL}/campaign/${campaignId}/events`, {
                credentials: 'include',
            });
            if (!eventsResponse.ok) throw new Error('Failed to fetch events');
            campaignEvents = await eventsResponse.json();
            
            form.reset();
        } catch (e) {
            error = e instanceof Error ? e.message : 'Failed to create events';
        }
    }

    async function deleteSelectedEvents() {
        if (!selectedEventIds.length) return;
        
        try {
            const response = await fetch(`${API_URL}/campaign/${campaignId}/events`, {
                method: 'DELETE',
                credentials: 'include',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(selectedEventIds),
            });

            if (!response.ok) throw new Error('Failed to delete events');
            
            // Refresh events list and clear selection
            await fetchEvents();
            selectedEventIds = [];
        } catch (e) {
            error = e instanceof Error ? e.message : 'Failed to delete events';
        }
    }

    async function updateEvent(eventId: number, newData: any) {
        try {
            const eventType = campaignEvents.find(event => event.id === eventId)?.event_type; 
            const response = await fetch(`${API_URL}/campaign/${campaignId}/events/${eventId}`, {
                method: 'PUT',
                credentials: 'include',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({ 
                    event_type: eventType,
                    data: newData 
                }),
            });

            if (!response.ok) throw new Error('Failed to update event');
            
            // Refresh events list and clear editing state
            await fetchEvents();
            editingEvent = null;
        } catch (e) {
            error = e instanceof Error ? e.message : 'Failed to update event';
        }
    }

    // Helper function to create edit form data based on event type
    function getEditFormData(event: Event) {
        switch (event.event_type) {
            case 'CurrencyGain':
                return { currency: event.data.currency };
            case 'ExperienceGain':
                return { experience: event.data.experience };
            default:
                return {};
        }
    }
</script>

<div class="events-page">
    <h1>Campaign Events</h1>

    {#if error}
        <div class="error">{error}</div>
    {/if}

    <form on:submit={addEvent} class="event-form">
        <h2>Add New Event</h2>
        
        <div class="form-group">
            <label for="event_type">Event Type</label>
            <select 
                name="event_type" 
                id="event_type" 
                bind:value={selectedEventType} 
                required
            >
                <option value="CurrencyGain">Currency Gain</option>
                <option value="ExperienceGain">Experience Gain</option>
            </select>
        </div>

        <div class="form-group">
            <label for="character">Characters</label>
            {#each campaignCharacters as character}
                <div class="character-checkbox">
                    {character.name}
                    <input type="checkbox" id="character-{character.id}" name="character_id" value={character.id} bind:group={selectedCharacterIds} />
                </div>
            {/each}
        </div>

        {#if selectedEventType === 'CurrencyGain'}
            <div class="form-group">
                <label for="currency">Currency Amount</label>
                <input
                    type="number"
                    id="currency"
                    name="currency"
                    required
                    min="0"
                />
            </div>
        {:else if selectedEventType === 'ExperienceGain'}
            <div class="form-group">
                <label for="experience">Experience Amount</label>
                <input
                    type="number"
                    id="experience"
                    name="experience"
                    required
                    min="0"
                />
            </div>
        {/if}

        <button type="submit" disabled={selectedCharacterIds.length === 0}>Add Event</button>
    </form>

    {#if loading}
        <div class="loading">Loading events...</div>
    {:else}
        <EventManager 
            events={campaignEvents}
            characters={campaignCharacters}
            campaignId={campaignId}
            onEventsUpdate={fetchEvents}
        />
    {/if}
</div>

<style>
    .events-page {
        padding: 2rem;
        max-width: 1200px;
        margin: 0 auto;
    }

    .event-form {
        background: #f8f8f8;
        padding: 1.5rem;
        border-radius: 8px;
        margin-bottom: 2rem;
        max-width: 600px;
    }

    .form-group {
        margin-bottom: 1rem;
    }

    .form-group label {
        display: block;
        margin-bottom: 0.5rem;
        font-weight: bold;
    }

    .character-checkbox { display: flex; align-items: center; white-space: nowrap; }

    input, select, textarea {
        width: 100%;
        padding: 0.5rem;
        border: 1px solid #ddd;
        border-radius: 4px;
    }

    .filters {
        display: flex;
        gap: 1rem;
        margin-bottom: 1.5rem;
        background: #f8f8f8;
        padding: 1rem;
        border-radius: 8px;
    }

    .filter-group {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    .table-container {
        overflow-x: auto;
    }

    table {
        width: 100%;
        border-collapse: collapse;
        margin-top: 1rem;
    }

    th, td {
        padding: 0.75rem;
        text-align: left;
        border-bottom: 1px solid #ddd;
    }

    th {
        background-color: #f8f8f8;
        font-weight: bold;
    }

    tr:hover {
        background-color: #f8f8f8;
    }

    .error {
        background: #fee2e2;
        color: #ef4444;
        padding: 1rem;
        border-radius: 4px;
        margin-bottom: 1rem;
    }

    .loading {
        text-align: center;
        color: #666;
        padding: 2rem;
    }

    .bulk-actions {
        margin-bottom: 1rem;
    }

    .delete-button {
        background-color: #ef4444;
        color: white;
        border: none;
        padding: 0.5rem 1rem;
        border-radius: 4px;
        cursor: pointer;
    }

    .delete-button:hover {
        background-color: #dc2626;
    }

    .edit-button {
        background-color: #3b82f6;
        color: white;
        border: none;
        padding: 0.25rem 0.5rem;
        border-radius: 4px;
        cursor: pointer;
    }

    .edit-button:hover {
        background-color: #2563eb;
    }

    .edit-form {
        display: flex;
        gap: 0.5rem;
        align-items: center;
    }

    .edit-form input {
        width: auto;
    }

    input[type="checkbox"] {
        width: auto;
        cursor: pointer;
    }
</style> 
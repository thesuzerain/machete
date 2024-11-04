<script lang="ts">
    import { page } from '$app/stores';
    import { onMount } from 'svelte';
    import { API_URL } from '$lib/config';
    import { events } from '$lib/stores/eventStore';
    import { characters } from '$lib/stores/characterStore';
    import type { Event, Character, InsertEvent } from '$lib/types/types';

    const campaignId = parseInt($page.params.id);
    let campaignEvents: Event[] = [];
    let campaignCharacters: Character[] = [];
    let loading = true;
    let error: string | null = null;

    onMount(async () => {
        try {
            // Fetch characters for character selection
            const charactersResponse = await fetch(`/api/campaign/${campaignId}/characters`);
            if (!charactersResponse.ok) throw new Error('Failed to fetch characters');
            campaignCharacters = await charactersResponse.json();

            // Fetch events
            const eventsResponse = await fetch(`/api/campaign/${campaignId}/events`);
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
        
        const newEvent: InsertEvent = {
            character_id: formData.get('character_id') ? parseInt(formData.get('character_id') as string) : undefined,
            event_type: formData.get('event_type') as Event['event_type'],
            description: formData.get('description') as string,
            value: formData.get('value') ? parseFloat(formData.get('value') as string) : undefined
        };

        try {
            const response = await fetch(`/api/campaign/${campaignId}/events`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify([newEvent]),
            });

            if (!response.ok) throw new Error('Failed to create event');
            
            // Refresh events list
            const eventsResponse = await fetch(`/api/campaign/${campaignId}/events`);
            if (!eventsResponse.ok) throw new Error('Failed to fetch events');
            campaignEvents = await eventsResponse.json();
            
            form.reset();
        } catch (e) {
            error = e instanceof Error ? e.message : 'Failed to create event';
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
            <label for="character">Character (optional)</label>
            <select name="character_id" id="character">
                <option value="">-- Select Character --</option>
                {#each campaignCharacters as character}
                    <option value={character.id}>{character.name}</option>
                {/each}
            </select>
        </div>

        <div class="form-group">
            <label for="event_type">Event Type</label>
            <select name="event_type" id="event_type" required>
                <option value="CURRENCY">Currency</option>
                <option value="ITEM">Item</option>
                <option value="LEVEL_UP">Level Up</option>
                <option value="STORY">Story</option>
                <option value="OTHER">Other</option>
            </select>
        </div>

        <div class="form-group">
            <label for="description">Description</label>
            <textarea
                id="description"
                name="description"
                rows="3"
                required
            ></textarea>
        </div>

        <div class="form-group">
            <label for="value">Value (optional)</label>
            <input
                type="number"
                id="value"
                name="value"
                step="0.01"
            />
        </div>

        <button type="submit">Add Event</button>
    </form>

    {#if loading}
        <div class="loading">Loading events...</div>
    {:else}
        <div class="event-list">
            {#each campaignEvents as event}
                <div class="event-card">
                    <div class="event-header">
                        <span class="event-type">{event.event_type}</span>
                        <time>{new Date(event.date).toLocaleDateString()}</time>
                    </div>
                    
                    {#if event.character_id}
                        <div class="character-name">
                            {campaignCharacters.find(c => c.id === event.character_id)?.name || 'Unknown Character'}
                        </div>
                    {/if}
                    
                    <p class="description">{event.description}</p>
                    
                    {#if event.value !== undefined}
                        <div class="value">
                            Value: {event.value}
                        </div>
                    {/if}
                </div>
            {/each}
        </div>
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

    input, select, textarea {
        width: 100%;
        padding: 0.5rem;
        border: 1px solid #ddd;
        border-radius: 4px;
    }

    .event-list {
        display: grid;
        gap: 1rem;
    }

    .event-card {
        background: white;
        padding: 1.5rem;
        border-radius: 8px;
        box-shadow: 0 2px 4px rgba(0,0,0,0.1);
    }

    .event-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 0.5rem;
    }

    .event-type {
        font-weight: bold;
        color: #3b82f6;
        padding: 0.25rem 0.5rem;
        background: #eff6ff;
        border-radius: 4px;
    }

    .character-name {
        color: #666;
        font-style: italic;
        margin-bottom: 0.5rem;
    }

    .description {
        margin: 0.5rem 0;
    }

    .value {
        font-weight: bold;
        color: #22c55e;
    }

    time {
        color: #666;
        font-size: 0.9rem;
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
</style> 
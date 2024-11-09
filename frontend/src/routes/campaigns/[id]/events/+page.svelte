<script lang="ts">
    import { page } from '$app/stores';
    import { onMount } from 'svelte';
    import type { Event, Character, InsertEvent } from '$lib/types/types';

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
            let url = `/api/campaign/${campaignId}/events?`;
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
        
        const eventType = formData.get('event_type') as string;

        let eventTypeData: any;
        switch (eventType) {
            case 'CurrencyGain':
                eventTypeData = {
                    currency: parseInt(formData.get('currency') as string)
                };
                break;
            case 'ExperienceGain':
                eventTypeData = {
                    experience: parseInt(formData.get('experience') as string)
                };
                break;
            default:
                throw new Error('Invalid event type');
        }

        const characterIds = formData.getAll('character_id') as string[];
        const newEvents: InsertEvent[] = characterIds.map(character_id => ({
            character: parseInt(character_id),
            event_type: eventType,
            data: eventTypeData,
        }));

        try {
            const response = await fetch(`/api/campaign/${campaignId}/events`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(newEvents),
            });

            if (!response.ok) throw new Error('Failed to create events');
            
            // Refresh events list
            const eventsResponse = await fetch(`/api/campaign/${campaignId}/events`);
            if (!eventsResponse.ok) throw new Error('Failed to fetch events');
            campaignEvents = await eventsResponse.json();
            
            form.reset();
        } catch (e) {
            error = e instanceof Error ? e.message : 'Failed to create events';
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
        <div class="filters">
            <div class="filter-group">
                <label for="filter-character">Filter by Character</label>
                <select 
                    id="filter-character" 
                    bind:value={filterCharacterId}
                >
                    <option value="">All Characters</option>
                    {#each campaignCharacters as character}
                        <option value={character.id}>{character.name}</option>
                    {/each}
                </select>
            </div>
            
            <div class="filter-group">
                <label for="filter-start-date">Start Date</label>
                <input 
                    type="datetime-local" 
                    id="filter-start-date" 
                    bind:value={filterStartDate}
                >
            </div>
            
            <div class="filter-group">
                <label for="filter-end-date">End Date</label>
                <input 
                    type="datetime-local" 
                    id="filter-end-date" 
                    bind:value={filterEndDate}
                >
            </div>
        </div>

        <div class="table-container">
            <table>
                <thead>
                    <tr>
                        <th>Timestamp</th>
                        <th>Character</th>
                        <th>Event Type</th>
                        <th>Raw Data</th>
                        {#each getEventTypeKeys(campaignEvents) as key}
                            <th>{key}</th>
                        {/each}
                    </tr>
                </thead>
                <tbody>
                    {#each campaignEvents as event}
                        <tr>
                            <td>{new Date(event.timestamp).toLocaleString()}</td>
                            <td>
                                {campaignCharacters.find(c => c.id === event.character)?.name || 'Unknown'}
                            </td>
                            <td>{event.event_type}</td>
                            <td>{JSON.stringify(event.data)}</td>
                            {#each getEventTypeKeys(campaignEvents) as key}
                                <td>
                                    {event.data[key] !== undefined ? event.data[key] : '-'}
                                </td>
                            {/each}
                        </tr>
                    {/each}
                </tbody>
            </table>
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
</style> 
<script lang="ts">
    import type { Event, Character } from '$lib/types/types';
    
    export let events: Event[] = [];
    export let characters: Character[] = [];
    export let campaignId: number;
    export let groupId: string | undefined; // Optional group ID for log-based events
    export let onEventsUpdate: () => void | undefined; // Callback when events are modified

    let selectedEventIds: number[] = [];
    let editingEvent: Event | null = null;
    let error: string | null = null;

    // Helper function to get all unique keys from event data
    function getEventDataKeys(events: Event[]): string[] {
        const keys = new Set<string>();
        events.forEach(event => {
            Object.keys(event.data).forEach(key => keys.add(key));
        });
        return Array.from(keys);
    }

    async function updateEvent(eventId: number, newData: any) {
        try {
            const response = await fetch(`/api/campaign/${campaignId}/events/${eventId}`, {
                method: 'PATCH',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ 
                    data: newData,
                    event_group: groupId
                }),
            });

            if (!response.ok) throw new Error('Failed to update event');
            
            if (onEventsUpdate) onEventsUpdate();
            editingEvent = null;
        } catch (e) {
            error = e instanceof Error ? e.message : 'Failed to update event';
        }
    }

    async function deleteSelectedEvents() {
        if (!selectedEventIds.length) return;
        
        try {
            const response = await fetch(`/api/campaign/${campaignId}/events`, {
                method: 'DELETE',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(selectedEventIds),
            });

            if (!response.ok) throw new Error('Failed to delete events');
            
            if (onEventsUpdate) onEventsUpdate();
            selectedEventIds = [];
        } catch (e) {
            error = e instanceof Error ? e.message : 'Failed to delete events';
        }
    }

    // Helper function to create edit form data based on event type
    function getEditFormData(event: Event) {
        switch (event.event_type) {
            case 'CurrencyGain':
                return { currency: event.data.currency };
            case 'ExperienceGain':
                return { experience: event.data.experience };
            case 'ItemGain':
                return { name: event.data.name };
            case 'EnemyDefeated':
            case 'HazardDefeated':
                return { 
                    name: event.data.name,
                    count: event.data.count
                };
            default:
                return event.data;
        }
    }
</script>

{#if error}
    <div class="error">{error}</div>
{/if}

{#if events.length > 0}
    {#if selectedEventIds.length > 0}
        <div class="bulk-actions">
            <button class="delete-button" on:click={deleteSelectedEvents}>
                Delete Selected ({selectedEventIds.length})
            </button>
        </div>
    {/if}

    <div class="table-container">
        <table>
            <thead>
                <tr>
                    <th>
                        <input 
                            type="checkbox" 
                            on:change={(e) => {
                                if (e.currentTarget.checked) {
                                    selectedEventIds = events.map(event => event.id);
                                } else {
                                    selectedEventIds = [];
                                }
                            }}
                            checked={selectedEventIds.length === events.length}
                        >
                    </th>
                    <th>Timestamp</th>
                    <th>Character</th>
                    <th>Event Type</th>
                    {#each getEventDataKeys(events) as key}
                        <th>{key}</th>
                    {/each}
                    <th>Actions</th>
                </tr>
            </thead>
            <tbody>
                {#each events as event}
                    <tr>
                        <td>
                            <input 
                                type="checkbox" 
                                checked={selectedEventIds.includes(event.id)}
                                on:change={(e) => {
                                    if (e.currentTarget.checked) {
                                        selectedEventIds = [...selectedEventIds, event.id];
                                    } else {
                                        selectedEventIds = selectedEventIds.filter(id => id !== event.id);
                                    }
                                }}
                            >
                        </td>
                        <td>{new Date(event.timestamp).toLocaleString()}</td>
                        <td>
                            {characters.find(c => c.id === event.character)?.name || 'Unknown'}
                        </td>
                        <td>{event.event_type}</td>
                        {#if editingEvent?.id === event.id}
                            <td colspan={getEventDataKeys(events).length + 1}>
                                <form 
                                    on:submit|preventDefault={() => {
                                        const formData = getEditFormData(event);
                                        updateEvent(event.id, formData);
                                    }}
                                    class="edit-form"
                                >
                                    {#each Object.entries(getEditFormData(event)) as [key, value]}
                                        <div class="edit-field">
                                            <label>{key}</label>
                                            <input 
                                                type={typeof value === 'number' ? 'number' : 'text'}
                                                bind:value={event.data[key]}
                                                required
                                            />
                                        </div>
                                    {/each}
                                    <button type="submit">Save</button>
                                    <button type="button" on:click={() => editingEvent = null}>Cancel</button>
                                </form>
                            </td>
                        {:else}
                            {#each getEventDataKeys(events) as key}
                                <td>
                                    {event.data[key] !== undefined ? event.data[key] : '-'}
                                </td>
                            {/each}
                            <td>
                                <button class="edit-button" on:click={() => editingEvent = event}>
                                    Edit
                                </button>
                            </td>
                        {/if}
                    </tr>
                {/each}
            </tbody>
        </table>
    </div>
{:else}
    <p>No events to display.</p>
{/if}

<style>
    .table-container {
        overflow-x: auto;
        margin-top: 1rem;
    }

    table {
        width: 100%;
        border-collapse: collapse;
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

    .edit-form {
        display: flex;
        gap: 1rem;
        align-items: center;
        padding: 0.5rem;
    }

    .edit-field {
        display: flex;
        flex-direction: column;
        gap: 0.25rem;
    }

    .error {
        background: #fee2e2;
        color: #ef4444;
        padding: 1rem;
        border-radius: 4px;
        margin-bottom: 1rem;
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

    .edit-button {
        background-color: #3b82f6;
        color: white;
        border: none;
        padding: 0.25rem 0.5rem;
        border-radius: 4px;
        cursor: pointer;
    }
</style> 
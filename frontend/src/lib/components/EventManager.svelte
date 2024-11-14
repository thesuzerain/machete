<script lang="ts">
    import type { Event, Character } from '$lib/types/types';
    import LibrarySelector from '$lib/components/LibrarySelector.svelte';
    import { formatCurrency } from '$lib/types/library';
    import LibraryEntityName from './LibraryEntityName.svelte';
    
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

    async function updateEvent(eventId: number, formData: Record<string, any>) {
        try {
            // Reconstruct nested data structure
            const newData = Object.entries(formData).reduce((acc: any, [key, value]) => {
                const parts = key.split('.');
                let current = acc;
                
                for (let i = 0; i < parts.length - 1; i++) {
                    current[parts[i]] = current[parts[i]] || {};
                    current = current[parts[i]];
                }
                
                current[parts[parts.length - 1]] = value;
                return acc;
            }, {});

            const response = await fetch(`/api/campaign/${campaignId}/events/${eventId}`, {
                method: 'PATCH',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    event_type: editingEvent?.event_type,
                    data: newData,
                    event_group: groupId,
                    character: editingEvent?.character
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
        // Return appropriate form fields based on event type
        switch (event.event_type) {
            case 'CurrencyGain':
                return {
                    type: 'CurrencyGain',
                    fields: {
                        currency: {
                            gold: event.data.currency.gold || 0,
                            silver: event.data.currency.silver || 0,
                            copper: event.data.currency.copper || 0
                        }
                    }
                };
            case 'ExperienceGain':
                return {
                    type: 'ExperienceGain',
                    fields: {
                        experience: event.data.experience
                    }
                };
            case 'EnemyDefeated':
                return {
                    type: 'EnemyDefeated',
                    fields: {
                        id: event.data.id
                    }
                };
            case 'HazardDefeated':
                return {
                    type: 'HazardDefeated',
                    fields: {
                        id: event.data.id
                    }
                };
            case 'ItemGain':
                return {
                    type: 'ItemGain',
                    fields: {
                        id: event.data.id
                    }
                };
            default:
                return { type: 'Unknown', fields: event.data };
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
                    <th>Data</th>
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
                            <td colspan="2">
                                {#if editingEvent}
                                    {@const formData = getEditFormData(editingEvent)}
                                    <form 
                                        on:submit|preventDefault={() => {
                                            updateEvent(event.id, formData.fields);
                                        }}
                                        class="edit-form"
                                    >
                                        <div class="edit-field">
                                            <label>Character</label>
                                            <select 
                                                bind:value={editingEvent.character}
                                                required
                                            >
                                                {#each characters as character}
                                                    <option value={character.id}>
                                                        {character.name}
                                                    </option>
                                                {/each}
                                            </select>
                                        </div>

                                        {#if formData.type === 'CurrencyGain'}
                                            <div class="currency-fields">
                                                <div class="edit-field">
                                                    <label>Gold</label>
                                                    <input 
                                                        type="number"
                                                        bind:value={editingEvent.data.currency.gold}
                                                        min="0"
                                                    />
                                                </div>
                                                <div class="edit-field">
                                                    <label>Silver</label>
                                                    <input 
                                                        type="number"
                                                        bind:value={editingEvent.data.currency.silver}
                                                        min="0"
                                                    />
                                                </div>
                                                <div class="edit-field">
                                                    <label>Copper</label>
                                                    <input 
                                                        type="number"
                                                        bind:value={editingEvent.data.currency.copper}
                                                        min="0"
                                                    />
                                                </div>
                                            </div>
                                        {:else if formData.type === 'ExperienceGain'}
                                            <div class="edit-field">
                                                <label>Experience</label>
                                                <input 
                                                    type="number"
                                                    bind:value={editingEvent.data.experience}
                                                    min="0"
                                                    required
                                                />
                                            </div>
                                        {:else if formData.type === 'EnemyDefeated'}
                                            <div class="edit-field">
                                                <label>Enemy</label>
                                                <LibrarySelector
                                                    entityType="creature"
                                                    onSelect={(id) => editingEvent.data.id = id}
                                                    placeholder="Select enemy..."
                                                    initialIds={[editingEvent.data.id]}
                                                />
                                            </div>
                                        {:else if formData.type === 'HazardDefeated'}
                                            <div class="edit-field">
                                                <label>Hazard</label>
                                                <LibrarySelector
                                                    entityType="hazard"
                                                    onSelect={(id) => editingEvent.data.id = id}
                                                    placeholder="Select hazard..."
                                                    initialIds={[editingEvent.data.id]}
                                                />
                                            </div>
                                        {:else if formData.type === 'ItemGain'}
                                            <div class="edit-field">
                                                <label>Item</label>
                                                <LibrarySelector
                                                    entityType="item"
                                                    onSelect={(id) => editingEvent.data.id = id}
                                                    placeholder="Select item..."
                                                    initialIds={[editingEvent.data.id]}
                                                />
                                            </div>
                                        {/if}

                                        <div class="form-actions">
                                            <button type="submit">Save</button>
                                            <button type="button" on:click={() => editingEvent = null}>Cancel</button>
                                        </div>
                                    </form>
                                {/if}
                            </td>
                        {:else}
                            <td>
                                {#if event.event_type === 'CurrencyGain'}
                                    {formatCurrency(event.data.currency)}
                                {:else if event.event_type === 'ExperienceGain'}
                                    {event.data.experience} XP
                                {:else if event.event_type === 'EnemyDefeated'}
                                    <LibraryEntityName 
                                        entityType="creature"
                                        entityId={event.data.id}
                                    />
                                {:else if event.event_type === 'HazardDefeated'}
                                    <LibraryEntityName 
                                        entityType="hazard"
                                        entityId={event.data.id}
                                    />
                                {:else if event.event_type === 'ItemGain'}
                                    <LibraryEntityName 
                                        entityType="item"
                                        entityId={event.data.id}
                                    />
                                {:else}
                                    {JSON.stringify(event.data)}
                                {/if}
                            </td>
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
        flex-direction: column;
        gap: 1rem;
        padding: 1rem;
        background: #f8f8f8;
        border-radius: 4px;
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

    select {
        padding: 0.25rem;
        border-radius: 4px;
        border: 1px solid #ddd;
    }

    .currency-fields {
        display: grid;
        grid-template-columns: repeat(3, 1fr);
        gap: 1rem;
    }

    .form-actions {
        display: flex;
        gap: 0.5rem;
        margin-top: 1rem;
    }
</style> 
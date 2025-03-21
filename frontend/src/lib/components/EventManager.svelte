<script lang="ts">
    import type { Event, Character } from '$lib/types/types';
    import LibrarySelector from '$lib/components/LibrarySelector.svelte';
    import { formatCurrency } from '$lib/types/library';
    import LibraryEntityName from './LibraryEntityName.svelte';
    import { API_URL } from '$lib/config';
    
    export let events: Event[] = [];
    export let characters: Character[] = [];
    export let campaignId: number;
    export let groupId: string | undefined; // Optional group ID for log-based events
    export let onEventsUpdate: () => Promise<void>;

    let selectedEventIds: number[] = [];
    let editingEvent: Event | null = null;
    let error: string | null = null;

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

            const response = await fetch(`${API_URL}/campaign/${campaignId}/events/${eventId}`, {
                method: 'PUT',
                headers: { 'Content-Type': 'application/json' },
                credentials: 'include',
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
            const response = await fetch(`${API_URL}/campaign/${campaignId}/events`, {
                method: 'DELETE',
                credentials: 'include',
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
                        currency: event.data.currency
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
                        id: event.data.id,
                        level_adjustment: event.data.level_adjustment || 0
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
                                                    <input 
                                                        type="number"
                                                        bind:value={editingEvent.data.currency}
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
                                                    onSelect={(id) => {
                                                        if (editingEvent) editingEvent!.data.id = id
                                                    }}
                                                    placeholder="Select enemy..."
                                                    initialIds={[editingEvent!.data.id as number]}
                                                />
                                                <!-- dropdown of -1, 0, +1, level adjustment-->
                                                <label>Level Adjustment</label>
                                                <select 
                                                    bind:value={editingEvent.data.level_adjustment}
                                                >
                                                    <option value={-1}>-1</option>
                                                    <option value={0}>0</option>
                                                    <option value={1}>+1</option>
                                                </select>
                                            </div>
                                        {:else if formData.type === 'HazardDefeated'}
                                            <div class="edit-field">
                                                <label>Hazard</label>
                                                <LibrarySelector
                                                    entityType="hazard"
                                                    onSelect={(id) => editingEvent!.data.id = id}
                                                    placeholder="Select hazard..."
                                                    initialIds={[editingEvent.data.id as number]}
                                                />
                                            </div>
                                        {:else if formData.type === 'ItemGain'}
                                            <div class="edit-field">
                                                <label>Item</label>
                                                <LibrarySelector
                                                    entityType="item"
                                                    onSelect={(id) => editingEvent!.data.id = id}
                                                    placeholder="Select item..."
                                                    initialIds={[editingEvent.data.id  as number]}
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
                                    {formatCurrency(event.data.currency as number)}
                                {:else if event.event_type === 'ExperienceGain'}
                                    {event.data.experience} XP
                                {:else if event.event_type === 'EnemyDefeated'}
                                    <LibraryEntityName 
                                        entityType="creature"
                                        entityId={event.data.id as number}
                                    />
                                    {#if event.data.level_adjustment && event.data.level_adjustment as number !== 0}
                                        <span>({event.data.level_adjustment as number > 0 ? '+' : ''}{event.data.level_adjustment})</span>
                                    {/if}
                
                                {:else if event.event_type === 'HazardDefeated'}
                                    <LibraryEntityName 
                                        entityType="hazard"
                                        entityId={event.data.id as number}
                                    />
                                {:else if event.event_type === 'ItemGain'}
                                    <LibraryEntityName 
                                        entityType="item"
                                        entityId={event.data.id as number}
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
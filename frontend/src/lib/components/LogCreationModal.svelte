<script lang="ts">
    import type { Character, InsertEvent, InsertLog, LibraryEntity, Log, WIPInsertLog, WIPLogEnemy, WIPLogTreasure } from '$lib/types/types';
    import LibrarySelector from './LibrarySelector.svelte';
    import { API_URL } from '$lib/config';
    import { getExperienceFromLevel } from '$lib/utils/encounter';
    import EventCreator from './EventCreator.svelte';
    import LibraryEntityName from './LibraryEntityName.svelte';
    import { formatCurrency } from '$lib/types/library';
    import { creatureStore, hazardStore, itemStore } from '$lib/stores/libraryStore';
    import { generateEventsFromData } from '$lib/utils/logs';


    export let show = false;
    export let selectedCampaignId: number;
    export let characters: Character[];
    export let fetchLogs: () => Promise<void>;
    export let initialData : WIPInsertLog | null = null;
    export let updateOnlyCallback: ((log: WIPInsertLog) => void) | null = null; // TODO: probably better practice to use runes or on:xxx dispatch

    let error: string | null = null;
    let selectedCharacterIds: number[] = [];
    let enemies: WIPLogEnemy[] = [];
    let treasures: WIPLogTreasure[] = [];

    // Subscribe to the stores
    let libraryEnemies: Map<number, LibraryEntity>;
    let libraryHazards: Map<number, LibraryEntity>;
    let libraryItems: Map<number, LibraryEntity>;

    creatureStore.subscribe(state => libraryEnemies = state.entities);
    hazardStore.subscribe(state => libraryHazards = state.entities);
    itemStore.subscribe(state => libraryItems = state.entities);

    let eventsToCreate: InsertEvent[] = [];

    let showEventDetails = false;
    let manualEvents: InsertEvent[] = [];

    let name = '';
    let description = '';

    // Load initial data if provided
    export function setInitialData(i : WIPInsertLog | null) {
        console.log("setting initial data", i);
        if (!i) return;
        name = i.name;
        description = i.description;
        enemies = i.enemies;
        treasures = i.treasures;
        manualEvents = i.current_manual_events;
    }
    setInitialData(initialData);

    let logs: Log[] = [];

    // Helper to generate all events based on current form state
    function generateEvents(characterIds: number[]): InsertEvent[] {
        return generateEventsFromData(characterIds, characters, enemies, treasures);
    }

    function removeEnemy(index: number) {
        enemies = enemies.filter((_, i) => i !== index);
    }

    function addTreasure() {
        treasures = [...treasures, {
            type: 'currency',
            amount: 0
        }];
    }

    function removeTreasure(index: number) {
        treasures = treasures.filter((_, i) => i !== index);
    }

    async function createLog(event: SubmitEvent) {
        event.preventDefault();
        const form = event.target as HTMLFormElement;
        const formData = new FormData(form);
        
        const newLog : InsertLog = {
            name: name,
            description: description,
            events: eventsToCreate
        };

        if (updateOnlyCallback) {
            // Skipping post if we have an alternative callback
            const characterIds = selectedCharacterIds.length > 0 
                ? selectedCharacterIds.map(id => id)
                : characters.map(c => c.id);
            let wipLog : WIPInsertLog = {
                name: newLog.name,
                description: newLog.description,
                characterIds: characterIds,
                current_manual_events: newLog.events,
                extra_experience: 0, // TODO
                enemies: enemies,
                treasures: treasures
            };
            updateOnlyCallback(wipLog);

        } else {
            try {
            const response = await fetch(`${API_URL}/campaign/${selectedCampaignId}/logs`, {
                method: 'POST',
                credentials: 'include',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(newLog),
            });

            if (!response.ok) throw new Error('Failed to create log');
            
            await fetchLogs();
            form.reset();
            enemies = [];
            treasures = [];

            show = false;
        } catch (e) {
            error = e instanceof Error ? e.message : 'Failed to create log';
        }

        }
    }

    // On every update to the form, generate the events to be created
    $: if (enemies.length > 0 || treasures.length > 0) {
        // Default to all characters if none selected
        const characterIds = selectedCharacterIds.length > 0 
            ? selectedCharacterIds.map(id => id)
            : characters.map(c => c.id);

        eventsToCreate = generateEvents(characterIds);
    }
</script>

{#if show}
    <div class="modal" on:click={() => show = false}>
        <div class="modal-content" on:click|stopPropagation>
            <form on:submit={createLog} class="log-form">
                <h2>Create Log</h2>
                <div class="form-header">
                    <div class="form-card">
                        <h3>Log Details</h3>
                        <label for="name">Log Name</label>
                        <input type="text" id="name" name="name" bind:value={name} required />
                            <label for="description">Description</label>
                            <textarea
                                name="description"
                                id="description"
                                bind:value={description}
                                required
                                placeholder="Describe what happened..."
                            ></textarea>
                    </div>    
                    <div class="form-card">  
                        <h3>Characters Details</h3>
                        <div class="form-group">
                            <label>Characters Involved (defaults to all if none selected)</label>
                            {#each characters as character}
                                <div class="character-checkbox">
                                    <input 
                                        type="checkbox" 
                                        id="character-{character.id}" 
                                        value={character.id} 
                                        bind:group={selectedCharacterIds}
                                    />
                                    <label for="character-{character.id}">{character.name}</label>
                                </div>
                            {/each}
                        </div>
                    </div>
        
                </div>
        
                <div class="enemies-section">
                    <h3>Enemies/Hazards</h3>
                    {#each enemies as enemy, i}
                        {JSON.stringify(enemy)}
                        <div class="enemy-entry">
                            <select bind:value={enemy.type}>
                                <option value="enemy">Enemy</option>
                                <option value="hazard">Hazard</option>
                            </select>
                            {#if enemy.type === 'enemy'}
                            <span class="item-name">{libraryEnemies.get(enemy.id)?.name}</span>
                                <LibrarySelector
                                    entityType="creature"
                                    onSelect={(id) => {
                                        enemies[i].id = id;
                                        enemies = enemies;  // trigger reactivity
                                    }}
                                    placeholder="Search for enemies..."
                                />
                            {:else}
                            <span class="item-name">{libraryHazards.get(enemy.id)?.name}</span>
        
                                <LibrarySelector
                                    entityType="hazard"
                                    onSelect={(id) => {
                                        enemies[i].id = id;
                                        enemies = enemies;  // trigger reactivity
                                    }}
                                    placeholder="Search for hazards..."
                                />
                            {/if}
                            <button type="button" on:click={() => removeEnemy(i)}>Remove</button>
                        </div>
                    {/each}
                    <button type="button" on:click={() => enemies = [...enemies, { id: 0, count: 1, type: 'enemy' }]}>
                        Add Enemy/Hazard
                    </button>
                </div>
        
                <div class="treasure-section">
                    <h3>Treasure</h3>
                    {#each treasures as treasure, i}
                        <div class="treasure-entry">
                            <select bind:value={treasure.type}>
                                <option value="currency">Currency</option>
                                <option value="item">Item</option>
                            </select>
                            {#if treasure.type === 'currency'}
                                <input 
                                    type="number" 
                                    placeholder="Amount"
                                    bind:value={treasure.amount}
                                    min="0"
                                    required
                                />
                            {:else}
                                <span class="item-name">{libraryItems.get(treasures[i].itemId)?.name}</span>
                                <LibrarySelector
                                    entityType="item"
                                    onSelect={(id) => {
                                        treasures[i].itemId = id;
                                        treasures = treasures;  // trigger reactivity
                                    }}
                                    placeholder="Search for items..."
                                />
                            {/if}
                            <button type="button" on:click={() => removeTreasure(i)}>Remove</button>
                        </div>
                    {/each}
                    <button type="button" on:click={() => treasures = [...treasures, { type: 'currency', amount: 0 }]}>
                        Add Treasure
                    </button>
                </div>
        
                <div class="advanced-section">
                    <div class="section-header" on:click={() => showEventDetails = !showEventDetails}>
                        <h3>
                            {showEventDetails ? 'Hide' : 'Show'} Event Details
                            <span class="toggle-icon">{showEventDetails ? '▼' : '▶'}</span>
                        </h3>
                    </div>
                    
                    {#if showEventDetails}
                        <div class="manual-events">
                            <h3>Event Preview & Manual Editor</h3>
                            
                            <!-- Event Creator -->
                            <EventCreator 
                                characters={characters}
                                onEventCreate={(event) => {
                                    eventsToCreate = [...eventsToCreate, event];
                                }}
                            />
        
                            <!-- Event List/Editor -->
                             <!-- TODO: Can use EventManager instead -->
                            <div class="events-preview">
                                <h4>Events to be Created ({eventsToCreate.length})
                                
                                {#each eventsToCreate as event, index}
                                    <div class="event-preview">
                                        <div class="event-info">
                                            <span class="event-type">{event.event_type}</span>
                                            <span class="event-character">
                                                {characters.find(c => c.id === event.character)?.name}
                                            </span>
                                            <span class="event-data">
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
                                                {/if}
                                            </span>
                                        </div>
                                        {#if eventsToCreate.includes(event)}
                                            <button 
                                                class="remove-event" 
                                                on:click={() => {
                                                    eventsToCreate = eventsToCreate.filter((_, i) => i !== index);
                                                }}
                                            >
                                                Remove
                                            </button>
                                        {/if}
                                    </div>
                                {/each}
                            </div>
                        </div>
                    {/if}
                </div>
        
                <button type="submit">{updateOnlyCallback ? "Update" : "Create"} Log Entry</button>
            </form>
        </div>
    </div>
{/if}

<style>
    .logs-page {
        padding: 2rem;
        max-width: 1200px;
        margin: 0 auto;
    }

    .log-form {
        display: flex;
        flex-direction: column;
        background: #f8f8f8;
        padding: 1.5rem;
        border-radius: 8px;
        margin-bottom: 2rem;
        gap: 1rem;
    }

    .template-description {
        display: block;
        color: #666;
        margin-top: 0.25rem;
    }

    .logs-section {
        margin-top: 2rem;
        background: white;
        border-radius: 8px;
        padding: 1.5rem;
        box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
    }

    .logs-controls {
        margin-bottom: 1.5rem;
        padding: 1rem;
        background: #f9fafb;
        border-radius: 4px;
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    .logs-list {
        max-height: 600px;
        overflow-y: auto;
        border: 1px solid #e5e7eb;
        border-radius: 4px;
    }

    .log-row {
        display: flex;
        align-items: center;
        padding: 0.5rem 1rem;
        border-bottom: 1px solid #e5e7eb;
        background: white;
    }

    .log-row:hover {
        background: #f9fafb;
    }

    .log-summary {
        display: flex;
        justify-content: space-between;
        align-items: center;
        flex: 1;
        cursor: pointer;
        margin-left: 1rem;
    }

    .log-title {
        display: flex;
        align-items: center;
        gap: 0.5rem;
    }

    .log-events-count {
        color: #6b7280;
        font-size: 0.875rem;
    }

    .log-meta {
        display: flex;
        align-items: center;
        gap: 1rem;
    }

    .log-actions {
        display: flex;
        gap: 0.5rem;
    }

    .log-actions button {
        padding: 0.25rem 0.5rem;
        font-size: 0.875rem;
    }

    .log-details {
        padding: 1rem;
        background: #f9fafb;
        border-bottom: 1px solid #e5e7eb;
    }

    .delete-selected {
        background: #ef4444;
        color: white;
        border: none;
        padding: 0.5rem 1rem;
        border-radius: 4px;
        cursor: pointer;
    }

    .delete-selected:hover {
        background: #dc2626;
    }

    .modal {
        position: fixed;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        background: rgba(0,0,0,0.5);
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .modal-content {
        background: white;
        padding: 2rem;
        border-radius: 8px;
        max-width: 800px;
        width: 90%;
        max-height: 90vh;
        overflow-y: auto;
    }

    .enemy-entry, .treasure-entry {
        display: flex;
        gap: 0.5rem;
        margin-bottom: 0.5rem;
        align-items: center;
    }

    .enemies-section, .treasure-section {
        margin: 1rem 0;
        padding: 1rem;
        background: #f0f0f0;
        border-radius: 4px;
    }

    .advanced-section {
        margin-top: 1rem;
        padding-top: 1rem;
        border-top: 1px solid #ddd;
        
    }

    .manual-events {
        margin-top: 1rem;
        padding: 1rem;
        background: #f8f8f8;
        border-radius: 4px;
    }

    .events-preview {
        margin-top: 1rem;
        border-top: 1px solid #ddd;
        padding-top: 1rem;
    }

    .event-preview {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 0.5rem;
        background: white;
        border-radius: 4px;
        margin-bottom: 0.5rem;
    }

    .event-info {
        display: flex;
        gap: 1rem;
        align-items: center;
    }

    .event-type {
        font-weight: 500;
        min-width: 120px;
    }

    .event-character {
        color: #666;
        min-width: 100px;
    }

    .remove-event {
        background: #ef4444;
        color: white;
        border: none;
        padding: 0.25rem 0.5rem;
        border-radius: 4px;
        cursor: pointer;
    }

    .remove-event:hover {
        background: #dc2626;
    }

    .modal-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    .item-name {
        font-weight: 500;
        white-space: nowrap;
    }

    .form-header {
        display: flex;
        justify-content: space-between;
        gap: 1rem;
        align-items: center;
        background: #f9fafb;

        border-radius: 4px;
        align-items: stretch;
        height: 100%;
    }

    .form-card {
        display: flex;
        justify-content: flex-start;
        align-items: stretch; /* Changed from flex-start to stretch to fill height */
        flex-direction: column;
        margin-bottom: 0.5rem; /* Fixed the comment syntax */
        padding: 1rem;
        background: #f0f0f0;
        border-radius: 4px;
        width: 100%;
    }

    .form-card h3 {
        margin: 0;
        min-width: 200px;
    }

    .form-card input {
        width: 100%;
    }
    
    .form-card textarea {
        width: 100%;
        height: 100%;
        resize: none;
    }

    .form-group {
        width: 100%;
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    .character-checkbox {
        display: flex;
        flex-direction: row;        
        white-space: nowrap;
        gap: 0.5rem;
        border-radius: 4px;
    }

    .character-checkbox input[type="checkbox"] {
       width: auto;
       cursor: pointer;
    }

    .log-form-container {
        height: 100%;
    }

    .create-log-button {
        margin-top: 1rem;
    }
    </style> 
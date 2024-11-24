<script lang="ts">
    import { page } from '$app/stores';
    import { onMount } from 'svelte';
    import { LOG_TEMPLATES } from '$lib/types/logs';
    import type { Log, Character, InsertLog, InsertEvent } from '$lib/types/types';
    import type { LibraryEntity } from '$lib/types/library';
    import { formatCurrency } from '$lib/types/library';
    import EventManager from '$lib/components/EventManager.svelte';
    import LibrarySelector from '$lib/components/LibrarySelector.svelte';
    import EventCreator from '$lib/components/EventCreator.svelte';
    import LibraryEntityName from '$lib/components/LibraryEntityName.svelte';
    import { fade } from 'svelte/transition';
    import { getExperienceFromLevel } from '$lib/utils/encounter';
    import { API_URL } from '$lib/config';

    const campaignId = parseInt($page.params.id);
    let logs: Log[] = [];
    let campaignCharacters: Character[] = [];
    let loading = true;
    let error: string | null = null;
    let selectedTemplateId: string = 'WonBattle';
    let selectedCharacterIds: string[] = [];
    let showingEventsForLog: Log | null = null;

    let eventsToCreate: InsertEvent[] = [];

    interface Enemy {
        id: number;
        count: number;
        type: 'enemy' | 'hazard';
    }

    interface Treasure {
        type: 'currency' | 'item';
        amount?: number;
        itemId?: number;
    }

    let enemies: Enemy[] = [];
    let treasures: Treasure[] = [];
    let showEventDetails = false;
    let manualEvents: InsertEvent[] = [];

    let editingLog: Log | null = null;

    let libraryEnemies: Map<number, LibraryEntity> = new Map();
    let libraryHazards: Map<number, LibraryEntity> = new Map();
    let libraryItems: Map<number, LibraryEntity> = new Map();

    let logsListOpen = true;
    let logNewOpen = false;
    let logOpenStates: { [key: number]: boolean } = {};
    let logFilter = '';
    let logSort: 'date' | 'name' = 'date';
    let sortDirection: 'asc' | 'desc' = 'desc';
    let selectedLogs: number[] = [];

    $: filteredAndSortedLogs = logs
        .filter(log => log.name.toLowerCase().includes(logFilter.toLowerCase()))
        .sort((a, b) => {
            const direction = sortDirection === 'asc' ? 1 : -1;
            switch (logSort) {
                case 'date':
                    return direction * (new Date(a.timestamp).getTime() - new Date(b.timestamp).getTime());
                case 'name':
                    return direction * a.name.localeCompare(b.name);
                default:
                    return 0;
            }
        });

    async function loadLibraryData() {
        try {
            // Load enemies
            const enemiesResponse = await fetch(`${API_URL}/library/creatures`);
            if (!enemiesResponse.ok) throw new Error('Failed to fetch creatures');
            const enemies: LibraryEntity[] = await enemiesResponse.json();
            libraryEnemies = new Map(enemies.map(e => [e.id, e]));

            // Load items
            const itemsResponse = await fetch(`${API_URL}/library/items`);
            if (!itemsResponse.ok) throw new Error('Failed to fetch items');
            const items: LibraryEntity[] = await itemsResponse.json();
            libraryItems = new Map(items.map(i => [i.id, i]));

            // Load hazards
            const hazardsResponse = await fetch(`${API_URL}/library/hazards`);
            if (!hazardsResponse.ok) throw new Error('Failed to fetch hazards');
            const hazards: LibraryEntity[] = await hazardsResponse.json();
            libraryHazards = new Map(hazards.map(h => [h.id, h]));
        } catch (e) {
            error = e instanceof Error ? e.message : 'Failed to load library data';
        }
    }

    async function fetchCampaignCharacters() {
        try {
            const response = await fetch(`${API_URL}/campaign/${campaignId}/characters`, {
                credentials: 'include',
            });
            if (!response.ok) throw new Error('Failed to fetch campaign characters');
            campaignCharacters = await response.json();
        } catch (e) {
            error = e instanceof Error ? e.message : 'Failed to fetch campaign characters';
        }
    }

    onMount(async () => {
        try {
            await Promise.all([
                fetchLogs(),
                loadLibraryData(),
                fetchCampaignCharacters()
            ]);
        } catch (e) {
            error = e instanceof Error ? e.message : 'An error occurred';
        } finally {
            loading = false;
        }
    });

    async function fetchLogs() {
        try {
            const logsResponse = await fetch(`${API_URL}/campaign/${campaignId}/logs`, {
                credentials: 'include',
            });
            if (!logsResponse.ok) throw new Error('Failed to fetch logs');
            logs = await logsResponse.json();
        } catch (e) {
            error = e instanceof Error ? e.message : 'Failed to fetch logs';
        }
    }

    // Helper to generate all events based on current form state
    function generateEvents(characterIds: number[]): InsertEvent[] {
        const events: InsertEvent[] = [];
        
        // Generate defeat and experience events for each enemy/hazard
        for (const enemy of enemies) {
            for (const characterId of characterIds) {
                events.push({
                    character: characterId,
                    event_type: enemy.type === 'enemy' ? 'EnemyDefeated' : 'HazardDefeated',
                    description: `Defeated ${enemy.count} ${enemy.type}`,
                    data: {
                        id: enemy.id,
                        count: enemy.count
                    }
                });

                // Add experience event
                events.push({
                    character: characterId,
                    event_type: 'ExperienceGain',
                    description: `Gained experience from ${enemy.type}`,
                    data: {
                        experience: getExperienceFromLevel(enemy.level || 0, campaignCharacters.find(c => c.id === characterId)?.level || 0)
                    }
                });
            }
        }

        // Generate treasure events
        for (const treasure of treasures) {
            for (const characterId of characterIds) {
                events.push({
                    character: characterId,
                    event_type: treasure.type === 'currency' ? 'CurrencyGain' : 'ItemGain',
                    description: treasure.type === 'currency' 
                        ? `Gained ${treasure.amount} currency`
                        : `Gained item`,
                    data: treasure.type === 'currency' 
                        ? { currency: { gold: treasure.amount } }
                        : { id: treasure.itemId }
                });
            }
        }

        return events;
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
        
        const newLog = {
            name: formData.get('name') as string,
            description: formData.get('description') as string,
            events: eventsToCreate
        };

        try {
            const response = await fetch(`${API_URL}/campaign/${campaignId}/logs`, {
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
        } catch (e) {
            error = e instanceof Error ? e.message : 'Failed to create log';
        }
    }

    async function viewLogEvents(log: Log) {
        showingEventsForLog = log;
    }

    async function updateLog(log: Log) {
        try {
            const response = await fetch(`${API_URL}/campaign/${campaignId}/logs/${log.id}`, {
                method: 'PATCH',
                credentials: 'include',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    name: log.name,
                    description: log.description
                }),
            });

            if (!response.ok) throw new Error('Failed to update log');
            
            await fetchLogs();
            editingLog = null;
        } catch (e) {
            error = e instanceof Error ? e.message : 'Failed to update log';
        }
    }

    async function deleteLog(logId: number) {
        if (!confirm('Are you sure you want to delete this log? This will also delete all associated events.')) {
            return;
        }

        try {
            const response = await fetch(`${API_URL}/campaign/${campaignId}/logs/${logId}`, {
                method: 'DELETE',
                credentials: 'include',
            });

            if (!response.ok) throw new Error('Failed to delete log');
            
            await fetchLogs();
            if (showingEventsForLog?.id === logId) {
                showingEventsForLog = null;
            }
        } catch (e) {
            error = e instanceof Error ? e.message : 'Failed to delete log';
        }
    }

    async function deleteSelectedLogs() {
        if (!confirm(`Are you sure you want to delete ${selectedLogs.length} logs? This will also delete all associated events.`)) {
            return;
        }

        try {
            await Promise.all(selectedLogs.map(logId => 
                fetch(`${API_URL}/campaign/${campaignId}/logs/${logId}`, {
                    method: 'DELETE',
                    credentials: 'include',
                })
            ));
            
            await fetchLogs();
            selectedLogs = [];
        } catch (e) {
            error = e instanceof Error ? e.message : 'Failed to delete logs';
        }
    }

    // On every update to the form, generate the events to be created
    $: if (enemies.length > 0 || treasures.length > 0) {
        // Default to all characters if none selected
        const characterIds = selectedCharacterIds.length > 0 
            ? selectedCharacterIds.map(id => parseInt(id))
            : campaignCharacters.map(c => c.id);

        eventsToCreate = generateEvents(characterIds);
    }
</script>

<div class="logs-page">
    <h1>Campaign Logs</h1>

    {#if error}
        <div class="error">{error}</div>
    {/if}

    {#if loading}
        <div class="loading">Loading logs...</div>
    {:else}
        <div class="logs-section">
            <div class="section-header" on:click={() => logsListOpen = !logsListOpen}>
                <h2>
                    Campaign Logs ({logs.length})
                    <span class="toggle-icon">{logsListOpen ? '▼' : '▶'}</span>
                </h2>
            </div>

            {#if logsListOpen}
                <div class="logs-controls" transition:fade>
                    <div class="filter-sort">
                        <input
                            type="text"
                            placeholder="Filter logs..."
                            bind:value={logFilter}
                            class="filter-input"
                        />
                        <div class="sort-controls">
                            <select bind:value={logSort}>
                                <option value="date">Sort by Date</option>
                                <option value="name">Sort by Name</option>
                            </select>
                            <button 
                                class="sort-direction"
                                on:click={() => sortDirection = sortDirection === 'asc' ? 'desc' : 'asc'}
                            >
                                {sortDirection === 'asc' ? '↑' : '↓'}
                            </button>
                        </div>
                    </div>
                    {#if selectedLogs.length > 0}
                        <button 
                            class="delete-selected"
                            on:click={deleteSelectedLogs}
                        >
                            Delete Selected ({selectedLogs.length})
                        </button>
                    {/if}
                </div>

                <div class="logs-list" transition:fade>
                    {#each filteredAndSortedLogs as log (log.id)}
                        <div class="log-row">
                            <input
                                type="checkbox"
                                value={log.id}
                                bind:group={selectedLogs}
                                on:click|stopPropagation
                            />
                            <div class="log-summary" on:click={() => viewLogEvents(log)}>
                                <div class="log-title">
                                    <span class="log-name">{log.name}</span>
                                    <span class="log-events-count">({log.events.length} events)</span>
                                </div>
                                <div class="log-meta">
                                    <span class="timestamp">{new Date(log.timestamp).toLocaleString()}</span>
                                    
                                </div>
                            </div>
                        </div>
                    {/each}
                </div>
                <button class="create-log-button" type="button" transition:fade on:click={() => {logNewOpen = !logNewOpen; logsListOpen = false;}}>Create Log</button>
                {/if}
        </div>
    {/if}



{#if logNewOpen}
                        <div class="log-form-container" transition:fade>
    <form on:submit={createLog} class="log-form">
        <h2>Create Log</h2>
        <div class="form-header">
            <div class="form-card">
                <h3>Log Details</h3>
                <label for="name">Log Name</label>
                <input type="text" id="name" name="name" required />
                    <label for="description">Description</label>
                    <textarea
                        name="description"
                        id="description"
                        required
                        placeholder="Describe what happened..."
                    ></textarea>
            </div>    
            <div class="form-card">  
                <h3>Characters Details</h3>
                <div class="form-group">
                    <label>Characters Involved (defaults to all if none selected)</label>
                    {#each campaignCharacters as character}
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
                        characters={campaignCharacters}
                        onEventCreate={(event) => {
                            eventsToCreate = [...eventsToCreate, event];
                        }}
                    />

                    <!-- Event List/Editor -->
                    <div class="events-preview">
                        <h4>Events to be Created ({eventsToCreate.length})
                        
                        {#each eventsToCreate as event, index}
                            <div class="event-preview">
                                <div class="event-info">
                                    <span class="event-type">{event.event_type}</span>
                                    <span class="event-character">
                                        {campaignCharacters.find(c => c.id === event.character)?.name}
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

        <button type="submit" on:click={() => {logNewOpen = false; logsListOpen = true;}}>Create Log Entry</button>
    </form>
</div>
{/if}

</div>

{#if showingEventsForLog}
    <div class="modal" on:click={() => showingEventsForLog = null}>
        <div class="modal-content" on:click|stopPropagation>
            <div class="modal-header">
                <h2>Events for: {showingEventsForLog.name}</h2>
                <button class="close-button" on:click={() => showingEventsForLog = null}>Close</button>
            </div>
            <p>{showingEventsForLog.description}</p>
            
            <!-- Add EventCreator above the EventManager -->
            <div class="modal-event-creator">
                <EventCreator 
                    characters={campaignCharacters}
                    onEventCreate={async (event) => {
                        try {
                            const response = await fetch(`${API_URL}/campaign/${campaignId}/events`, {
                                method: 'POST',
                                credentials: 'include',
                                headers: { 'Content-Type': 'application/json' },
                                body: JSON.stringify({
                                    ...event,
                                    event_group: showingEventsForLog.id
                                }),
                            });
                            if (!response.ok) throw new Error('Failed to create event');
                            await fetchLogs();
                        } catch (e) {
                            error = e instanceof Error ? e.message : 'Failed to create event';
                        }
                    }}
                />
            </div>

            <EventManager 
                events={showingEventsForLog.events}
                characters={campaignCharacters}
                campaignId={campaignId}
                groupId={showingEventsForLog.id.toString()}
                onEventsUpdate={fetchLogs}
            />
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
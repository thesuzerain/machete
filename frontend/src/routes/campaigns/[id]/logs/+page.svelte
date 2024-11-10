<script lang="ts">
    import { page } from '$app/stores';
    import { onMount } from 'svelte';
    import { LOG_TEMPLATES } from '$lib/types/logs';
    import type { Log, Character, InsertLog, InsertEvent } from '$lib/types/types';
    import EventManager from '$lib/components/EventManager.svelte';

    const campaignId = parseInt($page.params.id);
    let logs: Log[] = [];
    let campaignCharacters: Character[] = [];
    let loading = true;
    let error: string | null = null;
    let selectedTemplateId: string = 'WonBattle';
    let selectedCharacterIds: string[] = [];
    let showingEventsForLog: Log | null = null;

    interface Enemy {
        name: string;
        count: number;
        experience: number;
        type: 'enemy' | 'hazard';
    }

    interface Treasure {
        type: 'currency' | 'item';
        amount?: number;
        itemName?: string;
    }

    let enemies: Enemy[] = [];
    let treasures: Treasure[] = [];
    let showEventDetails = false;
    let manualEvents: InsertEvent[] = [];

    let editingLog: Log | null = null;

    onMount(async () => {
        try {
            // Fetch characters
            const charactersResponse = await fetch(`/api/campaign/${campaignId}/characters`);
            if (!charactersResponse.ok) throw new Error('Failed to fetch characters');
            campaignCharacters = await charactersResponse.json();

            // Fetch logs
            await fetchLogs();
        } catch (e) {
            error = e instanceof Error ? e.message : 'An error occurred';
        } finally {
            loading = false;
        }
    });

    async function fetchLogs() {
        try {
            const logsResponse = await fetch(`/api/campaign/${campaignId}/logs`);
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
                // Add defeat event
                events.push({
                    character: characterId,
                    event_type: enemy.type === 'enemy' ? 'EnemyDefeated' : 'HazardDefeated',
                    description: `Defeated ${enemy.count} ${enemy.name}`,
                    data: {
                        name: enemy.name,
                        count: enemy.count
                    }
                });

                // Add experience event
                events.push({
                    character: characterId,
                    event_type: 'ExperienceGain',
                    description: `Gained ${enemy.experience} experience from ${enemy.name}`,
                    data: {
                        experience: enemy.experience
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
                        : `Gained item: ${treasure.itemName}`,
                    data: treasure.type === 'currency' 
                        ? { currency: treasure.amount }
                        : { name: treasure.itemName }
                });
            }
        }

        return showEventDetails ? manualEvents : events;
    }

    function addEnemy() {
        enemies = [...enemies, {
            name: '',
            count: 1,
            experience: 0,
            type: 'enemy'
        }];
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
        
        // Default to all characters if none selected
        const characterIds = selectedCharacterIds.length > 0 
            ? selectedCharacterIds.map(id => parseInt(id))
            : campaignCharacters.map(c => c.id);

        const events = generateEvents(characterIds);

        const newLog = {
            name: formData.get('name') as string,
            description: formData.get('description') as string,
            events: events
        };

        try {
            console.log("Submitting log:", JSON.stringify(newLog))
            const response = await fetch(`/api/campaign/${campaignId}/logs`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(newLog),
            });

            if (!response.ok) throw new Error('Failed to create log');
            
            await fetchLogs();
            form.reset();
            enemies = [];
            treasures = [];
            manualEvents = [];
        } catch (e) {
            error = e instanceof Error ? e.message : 'Failed to create log';
        }
    }

    async function viewLogEvents(log: Log) {
        showingEventsForLog = log;
    }

    async function updateLog(log: Log) {
        try {
            const response = await fetch(`/api/campaign/${campaignId}/logs/${log.id}`, {
                method: 'PATCH',
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
            const response = await fetch(`/api/campaign/${campaignId}/logs/${logId}`, {
                method: 'DELETE',
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
</script>

<div class="logs-page">
    <h1>Campaign Logs</h1>

    {#if error}
        <div class="error">{error}</div>
    {/if}

    <form on:submit={createLog} class="log-form">
        <h2>Create New Log Entry</h2>
        
        <div class="form-group">
            <label for="name">Log Name</label>
            <input type="text" id="name" name="name" required />
        </div>

        <div class="form-group">
            <label for="description">Description</label>
            <textarea
                name="description"
                id="description"
                required
                placeholder="Describe what happened..."
            ></textarea>
        </div>

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

        <div class="enemies-section">
            <h3>Enemies/Hazards</h3>
            {#each enemies as enemy, i}
                <div class="enemy-entry">
                    <select bind:value={enemy.type}>
                        <option value="enemy">Enemy</option>
                        <option value="hazard">Hazard</option>
                    </select>
                    <input 
                        type="text" 
                        placeholder="Name"
                        bind:value={enemy.name}
                        required
                    />
                    <input 
                        type="number" 
                        placeholder="Count"
                        bind:value={enemy.count}
                        min="1"
                        required
                    />
                    <input 
                        type="number" 
                        placeholder="XP each"
                        bind:value={enemy.experience}
                        min="0"
                        required
                    />
                    <button type="button" on:click={() => removeEnemy(i)}>Remove</button>
                </div>
            {/each}
            <button type="button" on:click={addEnemy}>Add Enemy/Hazard</button>
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
                        <input 
                            type="text" 
                            placeholder="Item Name"
                            bind:value={treasure.itemName}
                            required
                        />
                    {/if}
                    <button type="button" on:click={() => removeTreasure(i)}>Remove</button>
                </div>
            {/each}
            <button type="button" on:click={addTreasure}>Add Treasure</button>
        </div>

        <div class="advanced-section">
            <button type="button" on:click={() => showEventDetails = !showEventDetails}>
                {showEventDetails ? 'Hide' : 'Show'} Event Details
            </button>
            
            {#if showEventDetails}
                <div class="manual-events">
                    <h3>Manual Event Editor</h3>
                    <!-- Add UI for manually editing events here -->
                    <!-- This could be a more advanced editor for power users -->
                </div>
            {/if}
        </div>

        <button type="submit">Create Log Entry</button>
    </form>

    {#if loading}
        <div class="loading">Loading logs...</div>
    {:else}
        <div class="logs-list">
            {#each logs as log}
                <div class="log-entry">
                    {#if editingLog?.id === log.id}
                        <div class="edit-form">
                            <input 
                                type="text" 
                                bind:value={log.name}
                                placeholder="Log name"
                            />
                            <textarea 
                                bind:value={log.description}
                                placeholder="Log description"
                            ></textarea>
                            <div class="edit-actions">
                                <button 
                                    class="save-button" 
                                    on:click={() => updateLog(log)}
                                >
                                    Save
                                </button>
                                <button 
                                    class="cancel-button" 
                                    on:click={() => editingLog = null}
                                >
                                    Cancel
                                </button>
                            </div>
                        </div>
                    {:else}
                        <div class="log-header">
                            <h3>{log.name}</h3>
                            <div class="log-actions">
                                <span class="timestamp">
                                    {new Date(log.timestamp).toLocaleString()}
                                </span>
                                <button 
                                    class="edit-button" 
                                    on:click={() => editingLog = log}
                                >
                                    Edit
                                </button>
                                <button 
                                    class="delete-button" 
                                    on:click={() => deleteLog(log.id)}
                                >
                                    Delete
                                </button>
                            </div>
                        </div>
                        <p class="description">{log.description}</p>
                        <button 
                            class="view-events" 
                            on:click={() => viewLogEvents(log)}
                        >
                            View Events ({log.events.length})
                        </button>
                    {/if}
                </div>
            {/each}
        </div>
    {/if}
</div>

{#if showingEventsForLog}
    <div class="modal">
        <div class="modal-content">
            <h2>Events for: {showingEventsForLog.name}</h2>
            <p>{showingEventsForLog.description}</p>
            
            <EventManager 
                events={showingEventsForLog.events}
                characters={campaignCharacters}
                campaignId={campaignId}
                groupId={showingEventsForLog.id.toString()}
                onEventsUpdate={fetchLogs}
            />
            
            <button class="close-button" on:click={() => showingEventsForLog = null}>Close</button>
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
        background: #f8f8f8;
        padding: 1.5rem;
        border-radius: 8px;
        margin-bottom: 2rem;
    }

    .template-description {
        display: block;
        color: #666;
        margin-top: 0.25rem;
    }

    .logs-list {
        display: flex;
        flex-direction: column;
        gap: 1rem;
    }

    .log-entry {
        background: white;
        padding: 1.5rem;
        border-radius: 8px;
        box-shadow: 0 2px 4px rgba(0,0,0,0.1);
    }

    .log-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 0.5rem;
    }

    .timestamp {
        color: #666;
        font-size: 0.9rem;
    }

    .view-events {
        margin-top: 1rem;
        background: #3b82f6;
        color: white;
        border: none;
        padding: 0.5rem 1rem;
        border-radius: 4px;
        cursor: pointer;
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

    .log-actions {
        display: flex;
        gap: 0.5rem;
        align-items: center;
    }

    .edit-form {
        display: flex;
        flex-direction: column;
        gap: 1rem;
        padding: 1rem;
    }

    .edit-form input,
    .edit-form textarea {
        width: 100%;
        padding: 0.5rem;
        border: 1px solid #ddd;
        border-radius: 4px;
    }

    .edit-form textarea {
        min-height: 100px;
        resize: vertical;
    }

    .edit-actions {
        display: flex;
        gap: 0.5rem;
        justify-content: flex-end;
    }

    .save-button {
        background-color: #22c55e;
        color: white;
        border: none;
        padding: 0.5rem 1rem;
        border-radius: 4px;
        cursor: pointer;
    }

    .cancel-button {
        background-color: #6b7280;
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

    .delete-button {
        background-color: #ef4444;
        color: white;
        border: none;
        padding: 0.25rem 0.5rem;
        border-radius: 4px;
        cursor: pointer;
    }

    .edit-button:hover { background-color: #2563eb; }
    .delete-button:hover { background-color: #dc2626; }
    .save-button:hover { background-color: #16a34a; }
    .cancel-button:hover { background-color: #4b5563; }
</style> 
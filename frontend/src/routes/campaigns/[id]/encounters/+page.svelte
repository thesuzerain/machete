<script lang="ts">
    import { page } from '$app/stores';
    import { onMount } from 'svelte';
    import type { Character } from '$lib/types/types';
    import LibrarySelector from '$lib/components/LibrarySelector.svelte';

    interface Encounter {
        id: number;
        status: 'Prepared' | 'Archived' | 'Success' | 'Failure';
        name: string;
        description: string;
        enemies: number[];
        hazards: number[];
        treasure_items: number[];
        treasure_currency: number;
    }

    interface LibraryEntity {
        id: number;
        name: string;
        level?: number;  // Optional for creatures/hazards
        value?: number;  // Optional for items (cost)
    }

    let libraryEnemies: Map<number, LibraryEntity> = new Map();
    let libraryHazards: Map<number, LibraryEntity> = new Map();
    let libraryItems: Map<number, LibraryEntity> = new Map();

    const campaignId = parseInt($page.params.id);
    let encounters: Encounter[] = [];
    let campaignCharacters: Character[] = [];
    let loading = true;
    let error: string | null = null;

    // Form states
    let editingEncounter: Encounter | null = null;
    let completingEncounter: Encounter | null = null;
    let selectedCharacterIds: number[] = [];

    // New encounter form state
    let newEncounter: Omit<Encounter, 'id' | 'status'> = {
        name: '',
        description: '',
        enemies: [],
        hazards: [],
        treasure_items: [],
        treasure_currency: 0
    };

    async function loadLibraryData() {
        try {
            // Load enemies
            const enemiesResponse = await fetch('/api/library/creatures');
            if (!enemiesResponse.ok) throw new Error('Failed to fetch creatures');
            const enemies: LibraryEntity[] = await enemiesResponse.json();
            libraryEnemies = new Map(enemies.map(e => [e.id, e]));

            // Load hazards
            const hazardsResponse = await fetch('/api/library/hazards');
            if (!hazardsResponse.ok) throw new Error('Failed to fetch hazards');
            const hazards: LibraryEntity[] = await hazardsResponse.json();
            libraryHazards = new Map(hazards.map(h => [h.id, h]));

            // Load items
            const itemsResponse = await fetch('/api/library/items');
            if (!itemsResponse.ok) throw new Error('Failed to fetch items');
            const items: LibraryEntity[] = await itemsResponse.json();
            libraryItems = new Map(items.map(i => [i.id, i]));
        } catch (e) {
            error = e instanceof Error ? e.message : 'Failed to load library data';
        }
    }

    onMount(async () => {
        try {
            // Load all data in parallel
            const [charactersResponse, encountersResponse] = await Promise.all([
                fetch(`/api/campaign/${campaignId}/characters`),
                fetch(`/api/campaign/${campaignId}/encounters`)
            ]);

            // Check responses and load data
            if (!charactersResponse.ok) throw new Error('Failed to fetch characters');
            if (!encountersResponse.ok) throw new Error('Failed to fetch encounters');

            campaignCharacters = await charactersResponse.json();
            encounters = await encountersResponse.json();

            // Load library data
            await loadLibraryData();
        } catch (e) {
            error = e instanceof Error ? e.message : 'An error occurred';
        } finally {
            loading = false;
        }
    });

    async function createEncounter() {
        try {
            const response = await fetch(`/api/campaign/${campaignId}/encounters`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(newEncounter),
            });

            if (!response.ok) throw new Error('Failed to create encounter');
            
            await fetchEncounters();
            // Reset form
            newEncounter = {
                name: '',
                description: '',
                enemies: [],
                hazards: [],
                treasure_items: [],
                treasure_currency: 0
            };
        } catch (e) {
            error = e instanceof Error ? e.message : 'Failed to create encounter';
        }
    }

    async function updateEncounter(encounter: Encounter) {
        try {
            const response = await fetch(`/api/campaign/${campaignId}/encounters/${encounter.id}`, {
                method: 'PATCH',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(encounter),
            });

            if (!response.ok) throw new Error('Failed to update encounter');
            
            await fetchEncounters();
            editingEncounter = null;
        } catch (e) {
            error = e instanceof Error ? e.message : 'Failed to update encounter';
        }
    }

    async function completeEncounter(encounter: Encounter) {
        if (!selectedCharacterIds.length) {
            error = "Please select at least one character";
            return;
        }

        try {
            // Create a log with all the events
            const logResponse = await fetch(`/api/campaign/${campaignId}/logs`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    name: `Completed: ${encounter.name}`,
                    description: encounter.description,
                    events: [
                        // Enemy defeated events
                        ...encounter.enemies.flatMap(enemyId => 
                            selectedCharacterIds.map(charId => ({
                                character_id: charId,
                                event_type: 'EnemyDefeated',
                                description: `Defeated ${libraryEnemies.get(enemyId)?.name}`,
                                data: {
                                    name: libraryEnemies.get(enemyId)?.name,
                                    count: 1,
                                    experience: libraryEnemies.get(enemyId)?.experience
                                }
                            }))
                        ),
                        // Hazard defeated events
                        ...encounter.hazards.flatMap(hazardId => 
                            selectedCharacterIds.map(charId => ({
                                character_id: charId,
                                event_type: 'HazardDefeated',
                                description: `Overcame ${libraryHazards.get(hazardId)?.name}`,
                                data: {
                                    name: libraryHazards.get(hazardId)?.name,
                                    count: 1,
                                    experience: libraryHazards.get(hazardId)?.experience
                                }
                            }))
                        ),
                        // Treasure events
                        ...selectedCharacterIds.map(charId => ({
                            character_id: charId,
                            event_type: 'CurrencyGain',
                            description: `Gained ${encounter.treasure_currency} currency from ${encounter.name}`,
                            data: {
                                currency: encounter.treasure_currency
                            }
                        })),
                        // Item gain events
                        ...encounter.treasure_items.flatMap(itemId => 
                            selectedCharacterIds.map(charId => ({
                                character_id: charId,
                                event_type: 'ItemGain',
                                description: `Found ${libraryItems.get(itemId)?.name}`,
                                data: {
                                    name: libraryItems.get(itemId)?.name
                                }
                            }))
                        )
                    ]
                })
            });

            if (!logResponse.ok) throw new Error('Failed to create completion log');

            // Update encounter status
            const statusResponse = await fetch(`/api/campaign/${campaignId}/encounters/${encounter.id}`, {
                method: 'PATCH',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    ...encounter,
                    status: 'Success'
                }),
            });

            if (!statusResponse.ok) throw new Error('Failed to update encounter status');
            
            await fetchEncounters();
            completingEncounter = null;
            selectedCharacterIds = [];
        } catch (e) {
            error = e instanceof Error ? e.message : 'Failed to complete encounter';
        }
    }

    function getEnemyDetails(id: number) {
        return libraryEnemies.get(id);
    }

    function getHazardDetails(id: number) {
        return libraryHazards.get(id);
    }

    function getItemDetails(id: number) {
        return libraryItems.get(id);
    }
</script>

<div class="encounters-page">
    <h1>Campaign Encounters</h1>

    {#if error}
        <div class="error">{error}</div>
    {/if}

    <div class="encounter-form">
        <h2>Create New Encounter</h2>
        <form on:submit|preventDefault={createEncounter}>
            <div class="form-group">
                <label for="name">Name</label>
                <input 
                    type="text" 
                    id="name" 
                    bind:value={newEncounter.name}
                    required
                />
            </div>

            <div class="form-group">
                <label for="description">Description</label>
                <textarea
                    id="description"
                    bind:value={newEncounter.description}
                    required
                ></textarea>
            </div>

            <div class="section">
                <h3>Enemies</h3>
                <div class="list-items">
                    {#each newEncounter.enemies as enemyId}
                        {#if getEnemyDetails(enemyId)}
                            <div class="list-item">
                                <span>{getEnemyDetails(enemyId)?.name}</span>
                                <button 
                                    type="button" 
                                    on:click={() => {
                                        newEncounter.enemies = newEncounter.enemies.filter(id => id !== enemyId);
                                    }}
                                >
                                    Remove
                                </button>
                            </div>
                        {/if}
                    {/each}
                </div>
                <LibrarySelector
                    entityType="creature"
                    onSelect={(id) => {
                        newEncounter.enemies = [...newEncounter.enemies, id];
                    }}
                    placeholder="Search for enemies..."
                />
            </div>

            <div class="section">
                <h3>Hazards</h3>
                <div class="list-items">
                    {#each newEncounter.hazards as hazardId}
                        {#if getHazardDetails(hazardId)}
                            <div class="list-item">
                                <span>{getHazardDetails(hazardId)?.name}</span>
                                <button 
                                    type="button" 
                                    on:click={() => {
                                        newEncounter.hazards = newEncounter.hazards.filter(id => id !== hazardId);
                                    }}
                                >
                                    Remove
                                </button>
                            </div>
                        {/if}
                    {/each}
                </div>
                <LibrarySelector
                    entityType="hazard"
                    onSelect={(id) => {
                        newEncounter.hazards = [...newEncounter.hazards, id];
                    }}
                    placeholder="Search for hazards..."
                />
            </div>

            <div class="section">
                <h3>Treasure</h3>
                <div class="form-group">
                    <label for="currency">Currency</label>
                    <input 
                        type="number"
                        id="currency"
                        bind:value={newEncounter.treasure_currency}
                        min="0"
                    />
                </div>

                <h4>Items</h4>
                <div class="list-items">
                    {#each newEncounter.treasure_items as itemId}
                        {#if getItemDetails(itemId)}
                            <div class="list-item">
                                <span>{getItemDetails(itemId)?.name}</span>
                                <button 
                                    type="button" 
                                    on:click={() => {
                                        newEncounter.treasure_items = newEncounter.treasure_items.filter(id => id !== itemId);
                                    }}
                                >
                                    Remove
                                </button>
                            </div>
                        {/if}
                    {/each}
                </div>
                <LibrarySelector
                    entityType="item"
                    onSelect={(id) => {
                        newEncounter.treasure_items = [...newEncounter.treasure_items, id];
                    }}
                    placeholder="Search for items..."
                />
            </div>

            <button type="submit">Create Encounter</button>
        </form>
    </div>

    {#if loading}
        <div class="loading">Loading encounters...</div>
    {:else}
        <div class="encounters-list">
            {#each encounters as encounter}
                <div class="encounter-card">
                    <div class="encounter-header">
                        <h3>{encounter.name}</h3>
                        <span class="status {encounter.status.toLowerCase()}">{encounter.status}</span>
                    </div>
                    <p>{encounter.description}</p>
                    
                    <div class="details">
                        <div class="detail-section">
                            <h4>Enemies ({encounter.enemies.length})</h4>
                            <ul>
                                {#each encounter.enemies as enemyId}
                                    {#if getEnemyDetails(enemyId)}
                                        <li>{getEnemyDetails(enemyId)?.name} (XP: {getEnemyDetails(enemyId)?.experience})</li>
                                    {/if}
                                {/each}
                            </ul>
                        </div>

                        <div class="detail-section">
                            <h4>Hazards ({encounter.hazards.length})</h4>
                            <ul>
                                {#each encounter.hazards as hazardId}
                                    {#if getHazardDetails(hazardId)}
                                        <li>{getHazardDetails(hazardId)?.name} (XP: {getHazardDetails(hazardId)?.experience})</li>
                                    {/if}
                                {/each}
                            </ul>
                        </div>

                        <div class="detail-section">
                            <h4>Treasure</h4>
                            <p>Currency: {encounter.treasure_currency}</p>
                            <ul>
                                {#each encounter.treasure_items as itemId}
                                    {#if getItemDetails(itemId)}
                                        <li>{getItemDetails(itemId)?.name}</li>
                                    {/if}
                                {/each}
                            </ul>
                        </div>
                    </div>

                    {#if encounter.status === 'Prepared'}
                        <div class="actions">
                            <button 
                                class="edit-button"
                                on:click={() => editingEncounter = encounter}
                            >
                                Edit
                            </button>
                            <button 
                                class="complete-button"
                                on:click={() => completingEncounter = encounter}
                            >
                                Complete
                            </button>
                        </div>
                    {/if}
                </div>
            {/each}
        </div>
    {/if}
</div>

{#if completingEncounter}
    <div class="modal">
        <div class="modal-content">
            <h2>Complete Encounter: {completingEncounter.name}</h2>
            <p>{completingEncounter.description}</p>

            <div class="form-group">
                <h3>Select Participating Characters</h3>
                {#each campaignCharacters as character}
                    <label class="checkbox-label">
                        <input 
                            type="checkbox"
                            value={character.id}
                            bind:group={selectedCharacterIds}
                        />
                        {character.name}
                    </label>
                {/each}
            </div>

            <div class="modal-actions">
                <button 
                    class="complete-button"
                    on:click={() => completeEncounter(completingEncounter)}
                    disabled={selectedCharacterIds.length === 0}
                >
                    Complete Encounter
                </button>
                <button 
                    class="cancel-button"
                    on:click={() => {
                        completingEncounter = null;
                        selectedCharacterIds = [];
                    }}
                >
                    Cancel
                </button>
            </div>
        </div>
    </div>
{/if}

<style>
    .encounters-page {
        padding: 2rem;
        max-width: 1200px;
        margin: 0 auto;
    }

    .encounter-form {
        background: #f8f8f8;
        padding: 1.5rem;
        border-radius: 8px;
        margin-bottom: 2rem;
    }

    .section {
        margin: 1.5rem 0;
        padding: 1rem;
        background: #fff;
        border-radius: 4px;
    }

    .list-item {
        display: flex;
        gap: 1rem;
        margin-bottom: 0.5rem;
        align-items: center;
    }

    .encounters-list {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
        gap: 1.5rem;
    }

    .encounter-card {
        background: white;
        padding: 1.5rem;
        border-radius: 8px;
        box-shadow: 0 2px 4px rgba(0,0,0,0.1);
    }

    .encounter-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 1rem;
    }

    .status {
        padding: 0.25rem 0.5rem;
        border-radius: 4px;
        font-size: 0.875rem;
    }

    .status.prepared { background: #dbeafe; color: #1e40af; }
    .status.success { background: #dcfce7; color: #166534; }
    .status.failure { background: #fee2e2; color: #991b1b; }
    .status.archived { background: #f3f4f6; color: #1f2937; }

    .details {
        margin: 1rem 0;
    }

    .detail-section {
        margin: 0.5rem 0;
    }

    .detail-section h4 {
        margin-bottom: 0.5rem;
    }

    .detail-section ul {
        list-style: none;
        padding-left: 1rem;
    }

    .actions {
        display: flex;
        gap: 0.5rem;
        margin-top: 1rem;
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
        max-width: 500px;
        width: 90%;
    }

    .checkbox-label {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        margin: 0.5rem 0;
    }

    .modal-actions {
        display: flex;
        gap: 1rem;
        margin-top: 1.5rem;
        justify-content: flex-end;
    }

    .complete-button {
        background: #22c55e;
        color: white;
        border: none;
        padding: 0.5rem 1rem;
        border-radius: 4px;
        cursor: pointer;
    }

    .complete-button:disabled {
        background: #9ca3af;
        cursor: not-allowed;
    }

    .edit-button {
        background: #3b82f6;
        color: white;
        border: none;
        padding: 0.5rem 1rem;
        border-radius: 4px;
        cursor: pointer;
    }

    .cancel-button {
        background: #6b7280;
        color: white;
        border: none;
        padding: 0.5rem 1rem;
        border-radius: 4px;
        cursor: pointer;
    }

    .selected-creatures {
        margin-bottom: 1rem;
    }

    .selected-creature {
        display: flex;
        align-items: center;
        gap: 1rem;
        padding: 0.5rem;
        background: #f8f8f8;
        border-radius: 4px;
        margin-bottom: 0.5rem;
    }

    .selected-creature .xp {
        color: #666;
        font-size: 0.875rem;
    }

    .remove-button {
        margin-left: auto;
        background: #ef4444;
        color: white;
        border: none;
        padding: 0.25rem 0.5rem;
        border-radius: 4px;
        cursor: pointer;
    }

    .remove-button:hover {
        background: #dc2626;
    }

    .list-items {
        margin-bottom: 1rem;
    }

    .list-item {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 0.5rem;
        background: #f8f8f8;
        border-radius: 4px;
        margin-bottom: 0.5rem;
    }

    .list-item button {
        background: #ef4444;
        color: white;
        border: none;
        padding: 0.25rem 0.5rem;
        border-radius: 4px;
        cursor: pointer;
    }

    .list-item button:hover {
        background: #dc2626;
    }
</style> 
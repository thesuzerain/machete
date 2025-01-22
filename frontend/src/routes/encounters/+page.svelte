<script lang="ts">
    import { requireAuth } from '$lib/guards/auth';
    import { page } from '$app/stores';
    import { onMount } from 'svelte';
    import type { Character } from '$lib/types/types';
    import LibrarySelector from '$lib/components/LibrarySelector.svelte';
    import EncounterCreator from "$lib/components/EncounterCreator.svelte";
    import { id } from 'date-fns/locale';
    import { 
        getExperienceFromLevel, 
        getSeverityFromRawExperience, 
        getRewardForLevelSeverity,
        EncounterDifficulty, 
        getSeverityFromFinalExperience

    } from '$lib/utils/encounter';
    import type { Encounter, CreateEncounter, EncounterStatus } from '$lib/types/encounters';
    import { getFullUrl } from '$lib/types/library';
    import { fade } from 'svelte/transition';
    import { FontAwesomeIcon } from '@fortawesome/svelte-fontawesome';
    import {
    type IconLookup,
    type IconDefinition,
    findIconDefinition,
    library
  } from '@fortawesome/fontawesome-svg-core'
  import {
    faLink,
  } from '@fortawesome/free-solid-svg-icons'
  import { API_URL } from '$lib/config';
  import { encounterStore } from '$lib/stores/encounters';
  import { campaignStore } from '$lib/stores/campaigns';
  import { creatureStore, hazardStore, itemStore } from '$lib/stores/libraryStore';
  import { characterStore } from '$lib/stores/characters';

library.add(faLink)

    let loading = $state(true);
    let error: string | null = $state(null);
    

    // Variables for encounter display
    let encountersListOpen = $state(true);
    let encounterOpenStates: { [key: number]: boolean } = $state({});
    let encounterFilter = $state('');
    let encounterSort: 'name' | 'level' | 'xp' = $state('name');
    let sortDirection: 'asc' | 'desc' = $state('asc');

    // Form values for editing/completing encounters
    let editingEncounter: Encounter | null = $state(null);
    let completingEncounter: Encounter | null = $state(null);
    let selectedCharacterIds: number[] = $state([]);

    // Add a new state variable for selected campaign
    let selectedCompletionCampaign: number | null = $state(null);
    let campaignCharacters = $derived((selectedCompletionCampaign ? $characterStore.get(selectedCompletionCampaign) : []) || []);


    // Subscribe to the stores
    let encounters = $derived($encounterStore);
    let campaigns = $derived( $campaignStore);
    let libraryEnemies = $derived($creatureStore);
    let libraryHazards = $derived($hazardStore);
    let libraryItems = $derived($itemStore);

    // Fetch campaigns data
    async function fetchCampaigns() {
        await campaignStore.fetchCampaigns();
    }

    async function loadLibraryData() {
        try {
            // TODO: This pattern is repeated in multiple places, consider refactoring
            // Load any enemies that are in current encounters
            const enemyIds = new Set(
                encounters.flatMap(e => e.enemies)
            );
            
            if (enemyIds.size > 0) {
                await creatureStore.fetchEntities({
                    ids: Array.from(enemyIds).map((x) => x.id).join(',')
                })
            }

            // Load any hazards that are in current encounters
            const hazardIds = new Set(
                encounters.flatMap(e => e.hazards)
            );
            if (hazardIds.size > 0) {
                await hazardStore.fetchEntities({
                    ids: Array.from(hazardIds).join(',')
                });
            }

            // Load any items that are in current encounters
            const itemIds = new Set(
                encounters.flatMap(e => e.treasure_items)
            );
            if (itemIds.size > 0) {
                await itemStore.fetchEntities({
                    ids: Array.from(itemIds).join(',')
                }); 
            }
        } catch (e) {
            console.error(e);
            error = e instanceof Error ? e.message : 'Failed to load library data';
        }
    }

    onMount(async () => {
        requireAuth();

        try {
            await Promise.all([
                fetchEncounters(),
                fetchCampaigns(),
            ]);
            await loadLibraryData();
        } catch (e) {
            error = e instanceof Error ? e.message : 'An error occurred';
        } finally {
            loading = false;
        }
    });

    function getEnemyDetails(id: number) {
        return libraryEnemies.entities.get(id);
    }

    function getHazardDetails(id: number) {
        return libraryHazards.entities.get(id);
    }

    function getItemDetails(id: number) {
        return libraryItems.entities.get(id);
    }


    async function fetchEncounters() {
        await encounterStore.fetchEncounters();
    }

    async function completeEncounter(encounter: Encounter | null) {
        if (!encounter || !selectedCompletionCampaign) return;

        if (!selectedCharacterIds.length) {
            error = "Please select at least one character";
            return;
        }

        try {
            // Create a log with all the events
            const logResponse = await fetch(`${API_URL}/campaign/${selectedCompletionCampaign}/logs`, {
                method: 'POST',
                credentials: 'include',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    name: `Completed: ${encounter.name}`,
                    description: encounter.description,
                    events: [
                        // Enemy defeated events
                        ...encounter.enemies.flatMap(encounterEnemy => 
                            selectedCharacterIds.map(charId => Array(encounter.enemies.length).fill({
                                character: charId,
                                event_type: 'EnemyDefeated',
                                description: `Defeated ${libraryEnemies.entities.get(encounterEnemy.id)?.name}`,
                                data: {
                                    id: encounterEnemy.id,
                                    level_adjustment: encounterEnemy.level_adjustment
                                }
                            })).flat()
                        ),
                        // Enemy experience events
                        ...encounter.enemies.flatMap(encounterEnemy => 
                            selectedCharacterIds.map(charId => Array(encounter.enemies.length).fill({
                                character: charId,
                                event_type: 'ExperienceGain',
                                description: `Gained ${getExperienceFromLevel(encounter.party_level, libraryEnemies.entities.get(encounterEnemy.id)?.level || 0)} experience from defeating ${libraryEnemies.entities.get(encounterEnemy.id)?.name}`,
                                data: {
                                    experience: getExperienceFromLevel(encounter.party_level, (libraryEnemies.entities.get(encounterEnemy.id)?.level || 0) + encounterEnemy.level_adjustment)
                                }
                            })).flat()
                        ),
                        // Hazard defeated events
                        ...encounter.hazards.flatMap(hazardId => 
                            selectedCharacterIds.map(charId => Array(encounter.hazards.length).fill({
                                character: charId,
                                event_type: 'HazardDefeated',
                                description: `Overcame ${libraryHazards.entities.get(hazardId)?.name}`,
                                data: {
                                    id: hazardId,
                                }
                            })).flat()
                        ),
                        // Hazard experience events
                        ...encounter.hazards.flatMap(hazardId => 
                            selectedCharacterIds.map(charId => Array(encounter.hazards.length).fill({
                                character: charId,
                                event_type: 'ExperienceGain',
                                description: `Gained ${getExperienceFromLevel(encounter.party_level, libraryHazards.entities.get(hazardId)?.level || 0)} experience from overcoming ${libraryHazards.entities.get(hazardId)?.name}`,
                                data: {
                                    experience: getExperienceFromLevel(encounter.party_level, libraryHazards.entities.get(hazardId)?.level || 0)
                                }
                            })).flat()
                        ),
                        // Treasure events
                        ...selectedCharacterIds.map(charId => ({
                            character: charId,
                            event_type: 'CurrencyGain',
                            description: `Gained ${encounter.treasure_currency} currency from ${encounter.name}`,
                            data: {
                                currency: encounter.treasure_currency
                            }
                        })),
                        // Item gain events
                        ...encounter.treasure_items.flatMap(itemId => 
                            selectedCharacterIds.map(charId => ({
                                character: charId,
                                event_type: 'ItemGain',
                                description: `Found ${libraryItems.entities.get(itemId)?.name}`,
                                data: {
                                    id: itemId
                                }
                            }))
                        )
                    ]
                })
            });

            if (!logResponse.ok) throw new Error('Failed to create completion log');

            // Update encounter status
            const statusResponse = await fetch(`${API_URL}/encounters/${encounter.id}`, {
                method: 'PUT',
                credentials: 'include',
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


    // Add this reactive statement to sort and filter encounters
    let filteredAndSortedEncounters = $derived(encounters
        .filter(enc => enc.name.toLowerCase().includes(encounterFilter.toLowerCase()))
        .sort((a, b) => {
            const direction = sortDirection === 'asc' ? 1 : -1;
            switch (encounterSort) {
                case 'name':
                    return direction * a.name.localeCompare(b.name);
                // TODO: Other sorts require not draftEncounter but encounter-specific data
                case 'level':
                    return direction * (a.party_level - b.party_level);
                case 'xp':
                    return direction * (a.total_experience - b.total_experience);
                default:
                    return 0;
            }
        })); 

    // TODO: modularize, along with css classes
    function getClassForDifficulty(difficulty: EncounterDifficulty): string {
        switch (difficulty) {
            case 'Trivial':
                return 'difficulty-trivial';
            case 'Low':
                return 'difficulty-low';
            case 'Moderate':
                return 'difficulty-moderate';
            case 'Severe':
                return 'difficulty-severe';
            case 'Extreme':
                return 'difficulty-extreme';
            default:
                return '';
        }
    }

    function getAdjustmentName(adjustment: number): string {
        if (adjustment === 0) return 'Normal';
        return adjustment > 0 ? 'Elite' : 'Weak';
    }

</script>

<div class="encounters-page">
    <h1>Campaign Encounters</h1>
    {#if error}
        <div class="error">{error}</div>
    {/if}

    <EncounterCreator bind:editingEncounter />

    {#if loading}
        <div class="loading">Loading encounters...</div>
    {:else}
        <div class="encounters-section">
            <div class="section-header" on:click={() => encountersListOpen = !encountersListOpen}>
                <h2>
                    Existing Encounters ({encounters.length})
                    <span class="toggle-icon">{encountersListOpen ? '▼' : '▶'}</span>
                </h2>
            </div>

            {#if encountersListOpen}
                <div class="encounters-controls" transition:fade>
                    <div class="filter-sort">
                        <input
                            type="text"
                            placeholder="Filter encounters..."
                            bind:value={encounterFilter}
                            class="filter-input"
                        />
                        <div class="sort-controls">
                            <select bind:value={encounterSort}>
                                <option value="name">Sort by Name</option>
                                <option value="level">Sort by Level</option>
                                <option value="xp">Sort by XP</option>
                            </select>
                            <button 
                                class="sort-direction"
                                on:click={() => sortDirection = sortDirection === 'asc' ? 'desc' : 'asc'}
                            >
                                {sortDirection === 'asc' ? '↑' : '↓'}
                            </button>
                        </div>
                    </div>
                </div>

                <div class="encounters-list" transition:fade>
                    {#each filteredAndSortedEncounters as encounter (encounter.id)}
                        <div class="encounter-card">
                            <div 
                                class="encounter-header"
                                on:click={() => encounterOpenStates[encounter.id] = !encounterOpenStates[encounter.id]}
                            >
                                <div class="encounter-summary">
                                    <h3>{encounter.name}</h3>
                                    <div class="encounter-meta">
                                        <span class="status {encounter.status.toLowerCase()}">{encounter.status}</span>
                                        <span class="xp">XP: {encounter.total_experience} (<span class="{getClassForDifficulty(getSeverityFromFinalExperience(encounter.total_experience, encounter.extra_experience))}">{getSeverityFromFinalExperience(encounter.total_experience, encounter.extra_experience).toWellFormed()}</span>)</span>
                                        <span class="party">Level {encounter.party_level} ({encounter.party_size} players)</span>
                                    </div>
                                </div>
                                <span class="toggle-icon">{encounterOpenStates[encounter.id] ? '▼' : '▶'}</span>
                            </div>
                            
                            {#if encounterOpenStates[encounter.id]}
                                <div class="encounter-details" transition:fade>
                                    <p>{encounter.description}</p>
                                    
                                    <div class="details">
                                        <div class="detail-section">
                                            <h4>Enemies ({encounter.enemies.length})</h4>
                                            <ul>
                                                {#each encounter.enemies as encounterEnemy : EncounterEnemy}
                                                    {#if getEnemyDetails(encounterEnemy.id)}
                                                        <li>{getEnemyDetails(encounterEnemy.id)?.name} 
                                                            {#if encounterEnemy.level_adjustment !== 0}
                                                                ({getAdjustmentName(encounterEnemy.level_adjustment)})
                                                            {/if}
                                                            (Level {(getEnemyDetails(encounterEnemy.id)?.level || 0) + encounterEnemy.level_adjustment})
                                                            (XP: {getExperienceFromLevel(encounter.party_level, getEnemyDetails(encounterEnemy.id)?.level || 0)})</li>
                                                    {/if}
                                                {/each}
                                            </ul>
                                        </div>

                                        <div class="detail-section">
                                            <h4>Hazards ({encounter.hazards.length})</h4>
                                            <ul>
                                                {#each encounter.hazards as hazardId}
                                                    {#if getHazardDetails(hazardId)}
                                                        <li>{getHazardDetails(hazardId)?.name} (XP: {getExperienceFromLevel(encounter.party_level, getHazardDetails(hazardId)?.level || 0)})</li>
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
                            {/if}
                        </div>
                    {/each}
                </div>
            {/if}
        </div>
    {/if}
    
</div>

{#if completingEncounter}
    <div class="modal">
        <div class="modal-content">
            <h2>Complete Encounter: {completingEncounter.name}</h2>
            <p>{completingEncounter.description}</p>

            <div class="form-group">
                <h3>Select Campaign</h3>
                <select 
                    bind:value={selectedCompletionCampaign}
                    required
                >
                    <option value="">Select a campaign...</option>
                    {#each campaigns as [cid, campaign]}
                        <option value={cid}>{campaign.name}</option>
                    {/each}
                </select>
            </div>

            {#if selectedCompletionCampaign}
                <div class="form-group">
                    <div class="form-group-line"><h3>Select Participating Characters</h3> 
                    <button class="edit-button" on:click={() => selectedCharacterIds = campaignCharacters.map(c => c.id)}>Select All</button>
                    </div>
                    {#if campaignCharacters.length === 0}
                        <p>No characters found in this campaign.</p>
                    {:else}
                        {#each campaignCharacters as character}
                            <label class="checkbox-label">
                                <input 
                                    type="checkbox"
                                    value={character.id}
                                    bind:group={selectedCharacterIds}
                                />
                                {character.name} (Level {character.level})
                            </label>
                        {/each}
                    {/if}
                </div>
            {/if}

            <div class="modal-actions">
                <button 
                    class="complete-button"
                    on:click={() => completeEncounter(completingEncounter)}
                    disabled={!selectedCompletionCampaign || selectedCharacterIds.length === 0}
                >
                    Complete Encounter
                </button>
                <button 
                    class="cancel-button"
                    on:click={() => {
                        completingEncounter = null;
                        selectedCharacterIds = [];
                        selectedCompletionCampaign = null;
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
        display: grid;
        grid-template-columns: minmax(200px, 1fr) auto auto auto auto;
        gap: 1rem;
        padding: 0.5rem 1rem;
        background: #f8f8f8;
        border-radius: 4px;
        margin-bottom: 0.5rem;
        align-items: center;
    }

    .entity-name {
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }

    .entity-xp, .entity-level {
        white-space: nowrap;
        color: #666;
    }

    .list-item button {
        white-space: nowrap;
        background: #ef4444;
        color: white;
        border: none;
        padding: 0.25rem 0.5rem;
        border-radius: 4px;
        cursor: pointer;
    }

    .encounters-list {
        display: flex;
        flex-direction: column;
        gap: 1rem;
        max-width: 100%;
    }

    .encounter-card {
        margin-bottom: 0.5rem;
        border: 1px solid #e5e7eb;
        border-radius: 4px;
        background: white;
        width: 100%;
    }

    .encounter-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 1rem 1.5rem;
        cursor: pointer;
        user-select: none;
        background: #f9fafb;
        border-radius: 4px;
    }

    .encounter-summary {
        display: flex;
        align-items: center;
        gap: 2rem;
        flex: 1;
    }

    .encounter-summary h3 {
        margin: 0;
        min-width: 200px;
    }

    .encounter-meta {
        display: flex;
        gap: 2rem;
        align-items: center;
        font-size: 0.875rem;
        color: #666;
    }

    .encounter-details {
        padding: 1.5rem;
        border-top: 1px solid #e5e7eb;
        background: white;
    }

    .status {
        padding: 0.25rem 0.75rem;
        border-radius: 999px;
        font-size: 0.75rem;
        font-weight: 500;
        text-transform: uppercase;
        letter-spacing: 0.05em;
    }

    .status.prepared { 
        background: #dbeafe; 
        color: #1e40af; 
    }

    .status.success { 
        background: #dcfce7; 
        color: #166534; 
    }

    .status.failure { 
        background: #fee2e2; 
        color: #991b1b; 
    }

    .status.archived { 
        background: #f3f4f6; 
        color: #1f2937; 
    }

    .encounters-section {
        margin-top: 2rem;
        background: white;
        border-radius: 8px;
        padding: 1.5rem;
        box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
    }

    .encounters-controls {
        margin-bottom: 1.5rem;
        padding: 1rem;
        background: #f9fafb;
        border-radius: 4px;
    }

    .filter-sort {
        display: flex;
        gap: 1rem;
        align-items: center;
    }

    .filter-input {
        flex: 1;
        padding: 0.5rem 1rem;
        border: 1px solid #e5e7eb;
        border-radius: 4px;
        font-size: 0.875rem;
    }

    .sort-controls {
        display: flex;
        gap: 0.5rem;
        align-items: center;
    }

    .sort-controls select {
        padding: 0.5rem 1rem;
        border: 1px solid #e5e7eb;
        border-radius: 4px;
        font-size: 0.875rem;
        min-width: 150px;
    }

    .sort-direction {
        padding: 0.5rem 0.75rem;
        border: 1px solid #e5e7eb;
        border-radius: 4px;
        background: white;
        cursor: pointer;
        font-size: 1rem;
    }

    .detail-section {
        margin-bottom: 1.5rem;
    }

    .detail-section:last-child {
        margin-bottom: 0;
    }

    .detail-section h4 {
        margin-bottom: 0.5rem;
        color: #374151;
    }

    .detail-section ul {
        list-style: none;
        padding: 0;
        margin: 0;
    }

    .detail-section li {
        padding: 0.25rem 0;
        color: #6b7280;
    }

    .actions {
        display: flex;
        gap: 0.75rem;
        margin-top: 1.5rem;
        padding-top: 1rem;
        border-top: 1px solid #e5e7eb;
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

    .draft-indicator {
        position: fixed;
        bottom: 1rem;
        right: 1rem;
        background: white;
        padding: 0.75rem 1rem;
        border-radius: 0.5rem;
        box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        display: flex;
        align-items: center;
        gap: 0.5rem;
        color: #6b7280;
        font-size: 0.875rem;
    }

    .draft-badge {
        background: #3b82f6;
        color: white;
        padding: 0.25rem 0.5rem;
        border-radius: 0.375rem;
        font-size: 0.75rem;
        font-weight: 500;
    }

    .form-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 1.5rem;
    }

    .section-header {
        cursor: pointer;
        user-select: none;
    }

    .section-header h3 {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin: 0;
        padding: 0.5rem 0;
    }

    .toggle-icon {
        font-size: 0.8em;
        color: #666;
    }

    .section-content {
        padding-top: 1rem;
    }

    .collapsible {
        transition: all 0.3s ease;
    }

    .library-selector-container {
        margin-top: 1rem;
        display: flex;
        gap: 0.5rem;
    }

    .browse-library-button {
        display: inline-flex;
        align-items: center;
        gap: 0.5rem;
        background: #3b82f6;
        color: white;
        text-decoration: none;
        padding: 0.1rem 1rem;
        border-radius: 0.375rem;
        transition: background-color 0.2s;
        white-space: nowrap;
    }

    .browse-library-button:hover {
        background: #2563eb;
    }

    .browse-library-button::before {
        content: "📚";  /* Optional: adds a library emoji */
    }

    .encounter-form-container {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 1rem;
    }

    .party-config-row {
        display: grid;
        grid-template-columns: 1fr 1fr;
    }

    .name-input {
        width: 100%;
        font-size: 1.2rem;
        font-family: inherit;
    }

    .description-input {
        width: 100%;
        font-size: 1rem;
        /* lock size */
        resize: none;
        font-family: inherit;
    }

    .modal select {
        width: 100%;
        padding: 0.5rem;
        margin-bottom: 1rem;
        border: 1px solid #e5e7eb;
        border-radius: 4px;
        font-size: 1rem;
    }

    .form-group-line {
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    .form-group h3 {
        margin-bottom: 0.75rem;
        color: #374151;
    }

    .form-group p {
        color: #6b7280;
        font-style: italic;
    }

    .difficulty-trivial {
        color: #10b981;
    }

    .difficulty-low {
        color: #f59e0b;
    }

    .difficulty-moderate {
        color: #f59e0b;
    }

    .difficulty-severe {
        color: #ef4444;
    }

    .difficulty-extreme {
        color: #ef4444;
    }
</style> 
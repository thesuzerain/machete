<script lang="ts">
    import { requireAuth } from '$lib/guards/auth';
    import { page } from '$app/stores';
    import { onMount } from 'svelte';
    import type { Character } from '$lib/types/types';
    import LibrarySelector from '$lib/components/LibrarySelector.svelte';
    import { id } from 'date-fns/locale';
    import { 
        getExperienceFromLevel, 
        getSeverityFromRawExperience, 
        getRewardForLevelSeverity,
        EncounterDifficulty, 
        getAdjustedExperienceFromPartySize

    } from '$lib/utils/encounter';
    import type { Encounter, CreateOrReplaceEncounter, EncounterStatus, CreateEncounterFinalized, EncounterEnemy, CreateOrReplaceEncounterExtended } from '$lib/types/encounters';
    import { getFullUrl, getFullUrlWithAdjustment, type LibraryCreature, type LibraryHazard, type LibraryItem } from '$lib/types/library';
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
  import { campaignStore, selectedCampaignStore } from '$lib/stores/campaigns';
  import EncounterCreatorNlp from './EncounterCreatorNlp.svelte';
  import { writable, type Writable } from 'svelte/store';
  import { creatureStore, hazardStore, itemStore } from '$lib/stores/libraryStore';
  import { characterStore } from '$lib/stores/characters';
  import { campaignSessionStore } from '$lib/stores/campaignSessions';
  import { goto } from '$app/navigation';

    interface Props {
        editingEncounter: Encounter | null;
        chosenSessionId: number | null;
    }
    let { 
        editingEncounter = $bindable(),
        chosenSessionId = $bindable()
     } : Props = $props();

     library.add(faLink)

    let loading = $state(true);
    let error: string | null = $state(null);

    let encounterCreatorNlp : EncounterCreatorNlp;
    
    // Add new state for auto-saving
    let saveTimeout: NodeJS.Timeout;
    const AUTOSAVE_DELAY = 2000; // 2 seconds

    // Modify the newEncounter state to track the current draft or editing encounter
    let wipEncounter: CreateOrReplaceEncounter = $state({
        name: '',
        description: '',
        enemies: [],
        enemy_level_adjustments: [],
        hazards: [],
        treasure_items: [],
        treasure_currency: 0,
        extra_experience: 0,
        party_level: 1,
        party_size: 4,
        status: 'Draft',
    });

    function setWipEncounterAs(encounter : Encounter) {
        wipEncounter = {
            name: encounter.name,
            description: encounter.description,
            enemies: encounter.enemies.map(e => ({ id: e.id, level_adjustment: e.level_adjustment })),
            hazards: encounter.hazards,
            treasure_items: encounter.treasure_items,
            treasure_currency: encounter.treasure_currency,
            extra_experience: encounter.extra_experience,
            party_level: encounter.party_level,
            party_size: encounter.party_size,
            status: encounter.status,
        };
    }
    if (editingEncounter) {
        setWipEncounterAs(editingEncounter);
    }

    // TODO: Effect is a smell, refactor when possible. (But needs to update when editingEncounter updates from inside here or elsewhere)
    $effect(() => {
        if (editingEncounter) {
            setWipEncounterAs(editingEncounter);
        }
    });

    // Add these variables near the top of the script section, with the other state variables
    let enemiesSectionOpen = $state(true);
    let hazardsSectionOpen = $state(true);
    let treasureSectionOpen = $state(true);

    // Subscribe to the stores
    let encounters = $derived($encounterStore);
    let libraryEnemies = $derived($creatureStore);
    let libraryHazards = $derived($hazardStore);
    let libraryItems = $derived($itemStore);
    let selectedCampaignId = $derived($selectedCampaignStore);
    let campaignSessions = $derived.by(() => {
        if (selectedCampaignId) {
            return $campaignSessionStore.get(selectedCampaignId) || null;
        } else {
            return null;
        }
    });

    let chosenSessionIndex = $derived.by(() => {
        if (campaignSessions && chosenSessionId) {
            return campaignSessions.findIndex(s => s.id === chosenSessionId);
        } else {
            return -1;
        }
    });

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
                    .concat(wipEncounter.enemies)
            );
            
            if (enemyIds.size > 0) {
                await creatureStore.fetchEntities({
                    ids: Array.from(enemyIds).map((x) => x.id).join(',')
                })
            }

            // Load any hazards that are in current encounters
            const hazardIds = new Set(
                encounters.flatMap(e => e.hazards)
                    .concat(wipEncounter.hazards)
            );
            if (hazardIds.size > 0) {
                await hazardStore.fetchEntities({
                    ids: Array.from(hazardIds).join(',')
                });
            }

            // Load any items that are in current encounters
            const itemIds = new Set(
                encounters.flatMap(e => e.treasure_items)
                    .concat(wipEncounter.treasure_items)
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

    export function loadEncounterCopyToDraft(encounter: Encounter) {
        editingEncounter = null;
        setWipEncounterAs(encounter);
    }

    onMount(async () => {
        try {
            // First check for any in-progress draft encounter, if we are not editing an existing one
            const inProgress = await encounterStore.getDraft();
            if (inProgress && !editingEncounter) {
                wipEncounter = inProgress;
            }

            // Setting sessions
            if (selectedCampaignId) {
                await campaignSessionStore.fetchCampaignSessions(selectedCampaignId);
            }

            // Then load other encounters and campaigns
            await Promise.all([
                fetchEncounters(),
                fetchCampaigns(),
                loadLibraryData()
            ]);
        } catch (e) {
            error = e instanceof Error ? e.message : 'An error occurred';
        } finally {
            loading = false;
        }
    });

    // Auto-save function
    async function autoSave() {
        if (saveTimeout) clearTimeout(saveTimeout);

        // Do not autosave if we are editing an existing encounter
        if (editingEncounter) return;
        
        saveTimeout = setTimeout(async () => {
            try {
                console.log("Auto-saving...");
                await encounterStore.updateDraft({
                    ...wipEncounter,
                });
            } catch (e) {
                error = e instanceof Error ? e.message : 'Failed to save draft';
            }
        }, AUTOSAVE_DELAY);
    }

    // Modify the createEncounter function
    async function createEncounter(event: SubmitEvent) {
        event.preventDefault();

        try {      
            let extended : CreateOrReplaceEncounterExtended = {
                ...wipEncounter,
                total_experience: totalEarnedXP,
                total_treasure_value: totalTreasure,
            };

            if (editingEncounter) {                
                await encounterStore.updateEncounter(editingEncounter.id, extended);
            } else {
                let finalized : CreateEncounterFinalized = {
                    ...extended,
                    session_id: chosenSessionId,
                };

                // Creating for first time
                // TODO: Some kind of way to choose/change this?
                finalized.status = 'Prepared';

                await encounterStore.addEncounter(finalized);
            
                // Reset draft
                wipEncounter = {
                    name: '',
                    description: '',
                    enemies: [],
                    hazards: [],
                    treasure_items: [],
                    extra_experience: 0,
                    treasure_currency: 0,
                    party_level: 1,
                    party_size: 4,
                    status: 'Draft',
                };
                
                // Clear NLP box
                encounterCreatorNlp.clear();
                
                // Clear any existing draft
                await fetch(`${API_URL}/encounters/draft`, {
                    method: 'DELETE',
                    credentials: 'include',
                });
            }

            // This creates a new draft on backend
            await fetchEncounters();

            if (!editingEncounter) {
                // If a campaign is selected, get player/party count defaults
                pickDefaultCampaignForParty(selectedCampaignId);
            }
        } catch (e) {
            error = e instanceof Error ? e.message : 'Failed to create encounter';
        }
    }


    function getEnemyDetails(id: number) : LibraryCreature | null {
        return libraryEnemies.entities.get(id) || null;
    }

    function getHazardDetails(id: number) : LibraryHazard | null {
        return libraryHazards.entities.get(id) || null;
    }

    function getItemDetails(id: number) : LibraryItem | null {
        return libraryItems.entities.get(id) || null;
    }

    let totalTreasure : number = $derived.by(() => {
        let total = wipEncounter.treasure_currency || 0;
        wipEncounter.treasure_items.forEach(itemId => {
            const item = getItemDetails(itemId);
            if (item && item.price) {
                total += item.price;
            }
        });
        return total;
    })

    async function fetchEncounters() {
        await encounterStore.fetchEncounters();
    }

    let subtotalXPEnemies : number = $derived(
        wipEncounter.enemies.reduce((total, encounterEnemy) => {
            const enemy = getEnemyDetails(encounterEnemy.id);
            if (enemy?.level) {
                return total + getExperienceFromLevel(wipEncounter.party_level, enemy.level + encounterEnemy.level_adjustment);
            }
            return total;
        }, 0)
    );

    let subtotalXPHazards : number = $derived(
        wipEncounter.hazards.reduce((total, hazardId) => {
            const hazard = getHazardDetails(hazardId);
            if (hazard?.level) {
                return total + getExperienceFromLevel(wipEncounter.party_level, hazard.level);
            }
            return total;
        }, 0)
    );

    async function pickDefaultCampaignForParty(newCampaignId: number | null) {
        if (newCampaignId) {
            await characterStore.fetchCharacters(newCampaignId); // Fetch characters for the selected campaign
            const characters = $characterStore.get(newCampaignId);
            if (characters) {                
                // Set PartyConfig to the player count and level of the campaign
                wipEncounter.party_size = characters.length;
                wipEncounter.party_level = characters.reduce((max, char) => Math.max(max, char.level), 1);
            }   
        }
    }
    selectedCampaignStore.subscribe(pickDefaultCampaignForParty);

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

    function toggleEnemyAdjustment(enemyId : number) {
        let cycleOrder = new Map([
            [0, 1],
            [1, -1],
            [-1, 0]
        ]);
        let encounterEnemy = wipEncounter.enemies[enemyId];
        encounterEnemy.level_adjustment = cycleOrder.get(encounterEnemy.level_adjustment) || 0;
    }

    function getAdjustmentName(levelAdjustment: number) {
        switch (levelAdjustment) {
            case 1:
                return 'Elite';
            case -1:
                return 'Weak';
            default:
                return 'Normal';
        }
    }

    function getAdjustmentClass(levelAdjustment: number) {
        switch (levelAdjustment) {
            case 1:
                return 'elite-button';
            case -1:
                return 'weak-button';
            default:
                return 'normal-button';
        }
    }


    // Add reactive statement for difficulty calculation
    let encounterDifficulty : EncounterDifficulty = $derived(getSeverityFromRawExperience(subtotalXPEnemies + subtotalXPHazards, wipEncounter.party_size));

    // Sum up the total XP for the encounter, adjusted by party
    // Does not include extra experience
    let subtotalXPPartyAdjusted : number = $derived(
        getAdjustedExperienceFromPartySize(subtotalXPEnemies + subtotalXPHazards, wipEncounter.party_size)
    );
    let adjustedXPAmount : number = $derived(subtotalXPPartyAdjusted - (subtotalXPEnemies + subtotalXPHazards));
    let totalEarnedXP : number = $derived(subtotalXPPartyAdjusted + wipEncounter.extra_experience);

    // Add reactive statements for auto-saving
    // TODO: Are these code smell usages of effect? I don't know enough Svelte yet to say so.
    $effect(() => {
        autoSave();
        if (wipEncounter.name && 
            wipEncounter.description && 
            wipEncounter.enemies &&
            wipEncounter.hazards && 
            wipEncounter.treasure_items && 
            wipEncounter.extra_experience) {
            autoSave();
        }
    });
</script>

<div class="encounter-form">
    <h2>Create New Encounter</h2>
    <form on:submit={createEncounter} class="encounter-form">
        <div class="encounter-form-container">

            <div class="party-config section">
                <h3>Encounter Configuration</h3>

                <div class="form-group">
                    <label for="name">Name</label>
                    <input 
                        type="text" 
                        id="name" 
                        class="name-input"
                        bind:value={wipEncounter.name}
                        required
                    />
                </div>

                <label for="description">Description</label>
                <div class="form-group">
                    <textarea
                        id="description"
                        class="description-input"
                        bind:value={wipEncounter.description}
                    ></textarea>
                </div>
            </div>
            
                <div class="party-config section">

                    <h3>Party Configuration</h3>
                    <div class="party-config-row">
                        <div class="party-config-input">
                        <label for="playerCount">Number of Players</label>
                            <input 
                                type="number" 
                                id="playerCount"
                                bind:value={wipEncounter.party_size}
                                min="1"
                                max="6"
                            /></div>
                            <div class="party-config-input">
                            <label for="partyLevel">Party Level</label>
                            <input 
                                type="number" 
                                id="partyLevel"
                                bind:value={wipEncounter.party_level}
                                min="1"
                                max="20"
                            /></div>

                    </div>
                    <div class="difficulty-indicator {encounterDifficulty.toLowerCase()}">
                        <div class="xp-total">Total earned XP: <b>{totalEarnedXP}</b> ({subtotalXPEnemies} + {subtotalXPHazards} + {adjustedXPAmount} + {wipEncounter.extra_experience})</div>
                        This is a <b class="{getClassForDifficulty(encounterDifficulty)}">{encounterDifficulty.toLowerCase()}</b> difficulty encounter for <b>{wipEncounter.party_size}</b> level <b>{wipEncounter.party_level}</b> players
                        </div>
                    </div>
            
        </div>

        <EncounterCreatorNlp
            bind:this={encounterCreatorNlp}
            bind:enemies={wipEncounter.enemies}
            bind:hazards={wipEncounter.hazards}
            bind:treasures={wipEncounter.treasure_items}
            ></EncounterCreatorNlp>


        <div class="section collapsible">
            <div class="section-header" on:click={() => enemiesSectionOpen = !enemiesSectionOpen}>
                <h3>
                    Enemies ({wipEncounter.enemies.length}) - {subtotalXPEnemies} XP
                    <span class="toggle-icon">{enemiesSectionOpen ? '▼' : '▶'}</span>
                </h3>
            </div>
            
            {#if enemiesSectionOpen}
                <div class="section-content" transition:fade>
                    
                    <div class="list-items">
                        {#each wipEncounter.enemies as encounterEnemy, i}
                            {#if getEnemyDetails(encounterEnemy.id)}
                                <div class="list-item">
                                    <div class="entity-adjustment">
                                        <button 
                                        type="button"
                                            class="adjustment-button {getAdjustmentClass(encounterEnemy.level_adjustment)}"
                                        on:click={() => {toggleEnemyAdjustment(i)}}
                                        >{getAdjustmentName(encounterEnemy.level_adjustment)}</button>
                                    </div>
                                    <div class="entity-name">{getEnemyDetails(encounterEnemy.id)?.name}</div>
                                    <div class="entity-link">
                                        <a href={getFullUrlWithAdjustment(getEnemyDetails(encounterEnemy.id)?.url || '', encounterEnemy.level_adjustment)} target="_blank" rel="noopener noreferrer">
                                            <FontAwesomeIcon icon={['fas', 'link']} />
                                        </a>
                                    </div>
                                    <div class="entity-xp">XP: {getExperienceFromLevel(wipEncounter.party_level, (getEnemyDetails(encounterEnemy.id)?.level || 0) + encounterEnemy.level_adjustment)}</div>
                                    <div class="entity-level">Level {(getEnemyDetails(encounterEnemy.id)?.level || 0) + encounterEnemy.level_adjustment}</div>
                                    <button 
                                        type="button" 
                                        class="remove-button"
                                        on:click={() => {
                                            wipEncounter.enemies = wipEncounter.enemies.filter((_, index) => index !== i);
                                        }}
                                    >
                                        Remove
                                    </button>
                                </div>
                            {/if}
                        {/each}
                    </div>
                    <div class="library-selector-container">
                    <LibrarySelector
                        entityType="creature"
                        onSelect={(id) => {
                            let newEnemy : EncounterEnemy = {
                                id: id,
                                level_adjustment: 0
                            };
                            wipEncounter.enemies = [...wipEncounter.enemies, newEnemy];
                        }}
                        placeholder="Search for enemies..."
                        initialIds={wipEncounter.enemies.map(e => e.id)}
                    />
                    <a 
                        href="/library?encounter=true&tab=creature"
                        class="browse-library-button"
                    >
                        Browse Library
                    </a>
                    </div>
                </div>
            {/if}
        </div>

        <div class="section collapsible">
            <div class="section-header" on:click={() => hazardsSectionOpen = !hazardsSectionOpen}>
                <h3>
                    Hazards ({wipEncounter.hazards.length}) - {subtotalXPHazards} XP
                    <span class="toggle-icon">{hazardsSectionOpen ? '▼' : '▶'}</span>
                </h3>
            </div>
            
            {#if hazardsSectionOpen}
                <div class="section-content" transition:fade>
                    <div class="list-items">
                        {#each wipEncounter.hazards as hazardId}
                            {#if getHazardDetails(hazardId)}
                                <div class="list-item">
                                    <div class="entity-name">{getHazardDetails(hazardId)?.name}</div>
                                    <div class="entity-link">
                                        <a href={getFullUrl(getHazardDetails(hazardId)?.url || '')} target="_blank" rel="noopener noreferrer">
                                            <FontAwesomeIcon icon={['fas', 'link']} />
                                        </a>
                                    </div>
                                    <div class="entity-xp">XP: {getExperienceFromLevel(wipEncounter.party_level, getHazardDetails(hazardId)?.level || 0)}</div>
                                    <div class="entity-level">Level {getHazardDetails(hazardId)?.level}</div>
                                    <button 
                                        type="button" 
                                        on:click={() => {
                                            wipEncounter.hazards = wipEncounter.hazards.filter(id => id !== hazardId);
                                        }}
                                    >
                                        Remove
                                    </button>
                                </div>
                            {/if}
                        {/each}
                    </div>
                    <div class="library-selector-container">
                    <LibrarySelector
                        entityType="hazard"
                        onSelect={(id) => {
                            wipEncounter.hazards = [...wipEncounter.hazards, id];
                        }}
                        placeholder="Search for hazards..."
                        initialIds={wipEncounter.hazards}
                    />
                    <a 
                        href="/library?encounter=true&tab=hazard"
                        class="browse-library-button"
                    >
                        Browse Library
                    </a>
                    </div>
                </div>
            {/if}
        </div>

        <div class="section collapsible">
            <div class="section-header" on:click={() => treasureSectionOpen = !treasureSectionOpen}>
                <h3>
                    Treasure - {totalTreasure} gold
                    <span class="toggle-icon">{treasureSectionOpen ? '▼' : '▶'}</span>
                </h3>
            </div>
            
            {#if treasureSectionOpen}
                <div class="section-content" transition:fade>
                    <div class="form-group-treasure-row">
                        <label for="currency">Currency</label>
                        <input 
                            type="number"
                            id="currency"
                            class="currency-input"
                            bind:value={wipEncounter.treasure_currency}
                            min="0"
                        />
                    </div>
                    <div class="form-group-treasure-row">
                        <label for="currency">Experience</label>
                        <input 
                            type="number"
                            id="currency"
                            class="currency-input"
                            bind:value={wipEncounter.extra_experience}
                        />
                    </div>


                    <h4>Items</h4>
                    <div class="list-items">
                        {#each wipEncounter.treasure_items as itemId}
                            {#if getItemDetails(itemId)}
                                <div class="list-item">
                                    <span>{getItemDetails(itemId)?.name}</span> 
                                    <div class="entity-link">
                                        <a href={getFullUrl(getItemDetails(itemId)?.url || '')} target="_blank" rel="noopener noreferrer">
                                            <FontAwesomeIcon icon={['fas', 'link']} />
                                        </a>
                                    </div>
                                    <button 
                                        type="button" 
                                        on:click={() => {
                                            wipEncounter.treasure_items = wipEncounter.treasure_items.filter(id => id !== itemId);
                                        }}
                                    >
                                        Remove
                                    </button>
                                </div>
                            {/if}
                        {/each}
                    </div>
                    <div class="library-selector-container">
                    <LibrarySelector
                        entityType="item"
                        onSelect={(id) => {
                            wipEncounter.treasure_items = [...wipEncounter.treasure_items, id];
                        }}
                        placeholder="Search for items..."
                        initialIds={wipEncounter.treasure_items}
                    />
                    <a 
                        href="/library?encounter=true&tab=item"
                        class="browse-library-button"
                    >
                        Browse Library
                    </a>
                    </div>
                </div>
            {/if}
        </div>
        <div class="section">

        {#if campaignSessions && !editingEncounter}
        <div class="session-selector">
            <label for="session">Add to existing session:</label> 
            <select bind:value={chosenSessionId}>
                <option value={null}>Select a session...</option>

                {#each campaignSessions as session, ind}
                    <option value={session.id}>Session {ind}: {session.name}</option>
                {/each}
            </select>
        </div>
        {/if}

        <button type="submit" class="create-button">
            {#if chosenSessionId && campaignSessions && !editingEncounter}
                Create Encounter into Session {chosenSessionIndex}
            {:else if !editingEncounter}
                Create Encounter
            {:else}
                Update Encounter
            {/if}
            </button>
       </div>

    
        </form>
</div>

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
        grid-template-columns: auto minmax(200px, 1fr)  auto auto auto auto;
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

    .list-items {
        margin-bottom: 1rem;
    }

    .remove-button {
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
        content: "📚";  /* TODO Optional: adds a library emoji */
    }

    .encounter-form-container {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 1rem;
    }

    .party-config-row {
        display: grid;
        grid-template-columns: 1fr 1fr;
        padding-top: 0.5rem;
    }

    .party-config-input input {
        padding: 0.5rem;
        border: 1px solid #e5e7eb;
        border-radius: 4px;
        font-size: 1rem;
    }

    .campaign-defaults-setter {
        padding-top: 0.5rem;
        display: flex;
        gap: 1rem;
    }
    
    .session-selector-row {
        display: flex;
        gap: 1rem;
        align-items: center;
    }

    .session-selector {
        display: flex;
        gap: 1rem;
        margin-bottom: 2rem;
    }

    .session-selector select {
        flex: 1;
        padding: 0.5rem;
        font-size: 1rem;
        border: 1px solid #e2e8f0;
        border-radius: 0.375rem;
    }

    .create-button {
        width: 100%;
        font-size: 1.2rem;


        padding: 0.5rem 1rem;
        border-radius: 0.375rem;
        background: #3b82f6;
        color: white;
        border: none;
        cursor: pointer;
        font-weight: 500;
        transition: background-color 0.2s;


    }   

    .difficulty-indicator {
        padding: 1rem;
    }

    .name-input {
        width: 100%;
        font-size: 1.2rem;
        font-family: inherit;
        border: 1px solid #e5e7eb;
        border-radius: 4px;
    }

    .description-input {
        width: 100%;
        font-size: 1rem;
        /* lock size */
        resize: none;
        font-family: inherit;
        border: 1px solid #e5e7eb;
        border-radius: 4px;
    }

    .currency-input {
        padding: 0.5rem;
        border: 1px solid #e5e7eb;
        border-radius: 4px;
        font-size: 1rem;
    }

    .form-group-treasure-row {
        display: grid;
        grid-template-columns: 0.1fr 0.4fr 0.5fr;
        gap: 1rem;
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

    /* TODO: Modularize these classes as you modularize the difficulty ones */
    .adjustment-button {
        color: white;
        border: none;
        padding: 0.25rem 0.5rem;
        border-radius: 4px;
        cursor: pointer;
    }

    
    .elite-button {
        background: #ef4444;
    }
    .elite-button:hover {
        background: #dc2626;
    }
    .weak-button {
        background: #10b981;
    }
    .weak-button:hover {
        background: #059669;
    }

    .normal-button {
        background: #999999;
    }
    .normal-button:hover {
        background: #059669;
    }

    
</style> 
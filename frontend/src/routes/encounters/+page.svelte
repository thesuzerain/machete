<script lang="ts">
    import { page } from '$app/stores';
    import { onMount } from 'svelte';
    import type { Character } from '$lib/types/types';
    import LibrarySelector from '$lib/components/LibrarySelector.svelte';
    import { id } from 'date-fns/locale';
    import { 
        getExperienceFromLevel, 
        getSeverityFromExperience, 
        getRewardForLevelSeverity,
        EncounterDifficulty 
    } from '$lib/utils/encounter';
    import type { Encounter, CreateEncounter, EncounterStatus } from '$lib/types/encounters';
    import type { Currency } from '$lib/types/library';
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

library.add(faLink)


    interface LibraryEntity {
        id: number;
        name: string;
        level?: number;  // Optional for creatures/hazards
        price?: Currency;  // Optional for items (cost)
    }

    interface PartyConfig {
        playerCount: number;
        partyLevel: number;
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

    
    // Add new state for auto-saving
    let saveTimeout: NodeJS.Timeout;
    const AUTOSAVE_DELAY = 2000; // 2 seconds

    // Modify the newEncounter state to track the current draft
    let draftEncounter: CreateEncounter = {
        name: '',
        description: '',
        enemies: [],
        hazards: [],
        treasure_items: [],
        treasure_currency: { gold: 0 },
        party_level: 1,
        party_size: 4
    };

    let campaignPartyConfig: PartyConfig = {
        party_level: 1,
        party_size: 4
    };

    // Add these variables near the top of the script section, with the other state variables
    let enemiesSectionOpen = true;
    let hazardsSectionOpen = true;
    let treasureSectionOpen = true;

    // Add these to your script section near the top with other state variables
    let encountersListOpen = true;
    let encounterOpenStates: { [key: number]: boolean } = {};
    let encounterFilter = '';
    let encounterSort: 'name' | 'level' | 'xp' = 'name';
    let sortDirection: 'asc' | 'desc' = 'asc';

    // Add a new state variable for selected campaign
    let selectedCampaign: string | null = null;
    let campaigns: { id: string, name: string, playerCount: number, partyLevel: number }[] = [];

    // Add this state variable near the top with other state variables
    let selectedCompletionCampaign: string | null = null;

    // Fetch campaigns data
    async function fetchCampaigns() {
        try {
            const response = await fetch(`${API_URL}/campaign`, {
                credentials: 'include',
            });
            if (!response.ok) {
                console.error('Failed to fetch campaigns', response);
                throw new Error('Failed to fetch campaigns');
            }
            campaigns = await response.json();
            if (campaigns.length > 0) {
                selectedCampaign = campaigns[0].id; // Default to the first available campaign
                await fetchCampaignCharacters(selectedCampaign); // Fetch characters for the default campaign
            }
        } catch (e) {
            console.error(e);
            error = e instanceof Error ? e.message : 'Failed to fetch campaigns';
        }
    }

    // Fetch characters for the selected campaign
    async function fetchCampaignCharacters(campaignId: string) {
        try {
            const response = await fetch(`${API_URL}/campaign/${campaignId}/characters`, {
                credentials: 'include',
            });
            if (!response.ok) throw new Error('Failed to fetch campaign characters');
            campaignCharacters = await response.json();

            // Set PartyConfig to the player count and level of the campaign
            draftEncounter.party_size = campaignCharacters.length;
            draftEncounter.party_level = campaignCharacters.reduce((max, char) => Math.max(max, char.level), 0);
            campaignPartyConfig = {
                party_size: campaignCharacters.length,
                party_level: draftEncounter.party_level
            };
        } catch (e) {
            console.error(e);
            error = e instanceof Error ? e.message : 'Failed to fetch campaign characters';
        }
    }

    // Watch for changes in selectedCampaign
    $: if (selectedCampaign) {
        const campaign = campaigns.find(c => c.id === selectedCampaign);
        if (campaign) {
            fetchCampaignCharacters(selectedCampaign); // Fetch characters whenever a new campaign is selected
        }
    }

    // Reset selectedCampaign to 'None' if player count or level is manually changed
    $: if (draftEncounter.party_size !== draftEncounter.party_size ||
           draftEncounter.party_level !== campaignPartyConfig.party_level) {
        selectedCampaign = null;
    }

    async function loadLibraryData() {
        try {
            // Load any enemies that are in current encounters
            const enemyIds = new Set(
                encounters.flatMap(e => e.enemies)
                    .concat(draftEncounter.enemies)
            );
            
            if (enemyIds.size > 0) {
                const enemiesResponse = await fetch(`${API_URL}/library/creatures?ids=${Array.from(enemyIds).join(',')}`, {
                    credentials: 'include',
                });
                if (!enemiesResponse.ok) throw new Error('Failed to fetch creatures');
                const enemies: LibraryEntity[] = await enemiesResponse.json();
                libraryEnemies = new Map(enemies.map(e => [e.id, e]));
            }

            // Load any hazards that are in current encounters
            const hazardIds = new Set(
                encounters.flatMap(e => e.hazards)
                    .concat(draftEncounter.hazards)
            );
            if (hazardIds.size > 0) {
                const hazardsResponse = await fetch(`${API_URL}/library/hazards?ids=${Array.from(hazardIds).join(',')}`, {
                    credentials: 'include',
                });
                if (!hazardsResponse.ok) throw new Error('Failed to fetch hazards');
                const hazards: LibraryEntity[] = await hazardsResponse.json();
                libraryHazards = new Map(hazards.map(h => [h.id, h]));
            }

            // Load any items that are in current encounters
            const itemIds = new Set(
                encounters.flatMap(e => e.treasure_items)
                    .concat(draftEncounter.treasure_items)
            );
            if (itemIds.size > 0) {
                const itemsResponse = await fetch(`${API_URL}/library/items?ids=${Array.from(itemIds).join(',')}`, {
                    credentials: 'include',
                });
                if (!itemsResponse.ok) throw new Error('Failed to fetch items');
                const items: LibraryEntity[] = await itemsResponse.json();
                libraryItems = new Map(items.map(i => [i.id, i]));
            }
        } catch (e) {
            console.error(e);
            error = e instanceof Error ? e.message : 'Failed to load library data';
        }
    }

    onMount(async () => {
        try {
            // First check for any in-progress encounter
            const inProgressResponse = await fetch(`${API_URL}/encounters/draft`, {
                credentials: 'include',
            });
            if (inProgressResponse.ok) {
                const inProgress: CreateEncounter = await inProgressResponse.json();
                if (inProgress) {
                    draftEncounter = inProgress;
                }
            }

            // Then load other encounters
            await fetchEncounters();
            
            // After we have all encounters (including draft), load their entity details
            await loadLibraryData();
            await fetchCampaigns();
        } catch (e) {
            error = e instanceof Error ? e.message : 'An error occurred';
        } finally {
            loading = false;
        }
    });

    // Auto-save function
    async function autoSave() {
        if (saveTimeout) clearTimeout(saveTimeout);
        
        saveTimeout = setTimeout(async () => {
            try {
                const response = await fetch(`${API_URL}/encounters/draft`, {
                    method: 'POST',
                    credentials: 'include',
                    headers: {
                        'Content-Type': 'application/json',
                    },
                    body: JSON.stringify({
                        ...draftEncounter,
                        status: 'InProgress'
                    }),
                });

                if (!response.ok) throw new Error('Failed to save draft');
                
            } catch (e) {
                error = e instanceof Error ? e.message : 'Failed to save draft';
            }
        }, AUTOSAVE_DELAY);
    }

    // Modify the createEncounter function
    async function createEncounter(event: SubmitEvent) {
        event.preventDefault();
        
        try {
            const response = await fetch(`${API_URL}/encounters`, {
                method: 'POST',
                credentials: 'include',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify([{
                    ...draftEncounter,
                }]),
            });

            if (!response.ok) throw new Error('Failed to create encounter');
            
            // Reset draft
            draftEncounter = {
                name: '',
                description: '',
                enemies: [],
                hazards: [],
                treasure_items: [],
                treasure_currency: { gold: 0 }
            };
            
            // Clear any existing draft
            await fetch(`${API_URL}/encounters/draft`, {
                method: 'DELETE',
                credentials: 'include',
            });

            await fetchEncounters();
        } catch (e) {
            error = e instanceof Error ? e.message : 'Failed to create encounter';
        }
    }

    async function updateEncounter(encounter: Encounter) {
        try {
            const response = await fetch(`${API_URL}/encounters/${encounter.id}`, {
                method: 'PATCH',
                credentials: 'include',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({encounter}),
            });

            if (!response.ok) throw new Error('Failed to update encounter');
            
            await fetchEncounters();
            editingEncounter = null;
        } catch (e) {
            error = e instanceof Error ? e.message : 'Failed to update encounter';
        }
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
                        ...encounter.enemies.flatMap(enemyId => 
                            selectedCharacterIds.map(charId => Array(encounter.enemies.length).fill({
                                character: charId,
                                event_type: 'EnemyDefeated',
                                description: `Defeated ${libraryEnemies.get(enemyId)?.name}`,
                                data: {
                                    id: enemyId,
                                }
                            })).flat()
                        ),
                        // Enemy experience events
                        ...encounter.enemies.flatMap(enemyId => 
                            selectedCharacterIds.map(charId => Array(encounter.enemies.length).fill({
                                character: charId,
                                event_type: 'ExperienceGain',
                                description: `Gained ${getExperienceFromLevel(draftEncounter.party_level, libraryEnemies.get(enemyId)?.level || 0)} experience from defeating ${libraryEnemies.get(enemyId)?.name}`,
                                data: {
                                    experience: getExperienceFromLevel(draftEncounter.party_level, libraryEnemies.get(enemyId)?.level || 0)
                                }
                            })).flat()
                        ),
                        // Hazard defeated events
                        ...encounter.hazards.flatMap(hazardId => 
                            selectedCharacterIds.map(charId => Array(encounter.hazards.length).fill({
                                character: charId,
                                event_type: 'HazardDefeated',
                                description: `Overcame ${libraryHazards.get(hazardId)?.name}`,
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
                                description: `Gained ${getExperienceFromLevel(draftEncounter.party_level, libraryHazards.get(hazardId)?.level || 0)} experience from overcoming ${libraryHazards.get(hazardId)?.name}`,
                                data: {
                                    experience: getExperienceFromLevel(draftEncounter.party_level, libraryHazards.get(hazardId)?.level || 0)
                                }
                            })).flat()
                        ),
                        // Treasure events
                        ...selectedCharacterIds.map(charId => ({
                            character: charId,
                            event_type: 'CurrencyGain',
                            description: `Gained ${encounter.treasure_currency.gold} currency from ${encounter.name}`,
                            data: {
                                currency: encounter.treasure_currency
                            }
                        })),
                        // Item gain events
                        ...encounter.treasure_items.flatMap(itemId => 
                            selectedCharacterIds.map(charId => ({
                                character: charId,
                                event_type: 'ItemGain',
                                description: `Found ${libraryItems.get(itemId)?.name}`,
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
                method: 'PATCH',
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

    function getEnemyDetails(id: number) {
        return libraryEnemies.get(id);
    }

    function getHazardDetails(id: number) {
        return libraryHazards.get(id);
    }

    function getItemDetails(id: number) {
        return libraryItems.get(id);
    }

    function getTotalTreasure() {
        let total = draftEncounter.treasure_currency.gold || 0;
        draftEncounter.treasure_items.forEach(itemId => {
            const item = getItemDetails(itemId);
            if (item && item.price?.gold) {
                total += item.price.gold;
            }
        });
        return total;
    }

    async function fetchEncounters() {
        try {
            console.log('Fetching encounters...');
            const response = await fetch(`${API_URL}/encounters`, {
                credentials: 'include',
            });
            console.log('Response status:', response.status);
            console.log('Response headers:', response.headers);
            if (!response.ok) throw new Error('Failed to fetch encounters');
            encounters = await response.json();
            console.log('Encounters loaded:', encounters);
        } catch (e) {
            console.error('Error fetching encounters:', e);
            error = e instanceof Error ? e.message : 'Failed to fetch encounters';
        }
    }

    function getEnemiesXP(): number {
        return draftEncounter.enemies.reduce((total, enemyId) => {
            const enemy = getEnemyDetails(enemyId);
            if (enemy?.level) {
                return total + getExperienceFromLevel(draftEncounter.party_level, enemy.level);
            }
            return total;
        }, 0);
    }

    function getHazardsXP(): number {
        return draftEncounter.hazards.reduce((total, hazardId) => {
            const hazard = getHazardDetails(hazardId);
            if (hazard?.level) {
                return total + getExperienceFromLevel(draftEncounter.party_level, hazard.level);
            }
            return total;
        }, 0);
    }

    // Add reactive statement for difficulty calculation
    $: encounterDifficulty = getSeverityFromExperience(getEnemiesXP() + getHazardsXP(), draftEncounter.party_level);
    $: if (libraryEnemies || libraryHazards || libraryItems || draftEncounter.enemies || draftEncounter.hazards || draftEncounter.treasure_items) {
        encounterDifficulty = getSeverityFromExperience(getEnemiesXP() + getHazardsXP(), draftEncounter.party_level);
    }

    // Add reactive statements for auto-saving
    $: {
        if (draftEncounter.name || 
            draftEncounter.description || 
            draftEncounter.enemies.length || 
            draftEncounter.hazards.length || 
            draftEncounter.treasure_items.length || 
            draftEncounter.treasure_currency.gold) {
            autoSave();
        }
    }

    // Add these helper functions
    function getEncounterXP(encounter: Encounter): number {
        let total = 0;
        encounter.enemies.forEach(enemyId => {
            const enemy = getEnemyDetails(enemyId);
            if (enemy?.level) {
                total += getExperienceFromLevel(draftEncounter.party_level, enemy.level);
            }
        });
        encounter.hazards.forEach(hazardId => {
            const hazard = getHazardDetails(hazardId);
            if (hazard?.level) {
                total += getExperienceFromLevel(draftEncounter.party_level, hazard.level);
            }
        });
        return total;
    }

    // Add this reactive statement to sort and filter encounters
    $: filteredAndSortedEncounters = encounters
        .filter(enc => enc.name.toLowerCase().includes(encounterFilter.toLowerCase()))
        .sort((a, b) => {
            const direction = sortDirection === 'asc' ? 1 : -1;
            switch (encounterSort) {
                case 'name':
                    return direction * a.name.localeCompare(b.name);
                case 'level':
                    return direction * (draftEncounter.party_level - draftEncounter.party_level);
                case 'xp':
                    return direction * (getEncounterXP(a) - getEncounterXP(b));
                default:
                    return 0;
            }
        });

    // Add this reactive statement to reset character selection when campaign changes
    $: if (selectedCompletionCampaign) {
        selectedCharacterIds = [];
        fetchCampaignCharacters(selectedCompletionCampaign);
    }
</script>

<div class="encounters-page">
    <h1>Campaign Encounters</h1>

    {#if error}
        <div class="error">{error}</div>
    {/if}

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
                        bind:value={draftEncounter.name}
                        required
                    />
                </div>
    
                <label for="description">Description</label>
                <div class="form-group">
                    <textarea
                        id="description"
                        class="description-input"
                        bind:value={draftEncounter.description}
                    ></textarea>
                </div>
            </div>
            
                <div class="party-config section">

                    <h3>Party Configuration</h3>
                    <div class="party-config-row">
                        <div>
                         <label for="playerCount">Number of Players</label>
                            <input 
                                type="number" 
                                id="playerCount"
                                bind:value={draftEncounter.party_size}
                                min="1"
                                max="6"
                            /></div>
                            <div>
                            <label for="partyLevel">Party Level</label>
                            <input 
                                type="number" 
                                id="partyLevel"
                                bind:value={draftEncounter.party_level}
                                min="1"
                                max="20"
                            /></div>

                    </div>
                    <div class="campaign-defaults-setter">
                        <label for="campaign">Select Campaign</label>
                        <select id="campaign" bind:value={selectedCampaign}>
                            <option value={null}>None</option>
                            {#each campaigns as campaign}
                                <option value={campaign.id}>{campaign.name}</option>
                            {/each}
                        </select>
                    </div>
                    <div class="difficulty-indicator {encounterDifficulty.toLowerCase()}">
                        <div class="xp-total">Total XP: {getEnemiesXP() + getHazardsXP()}</div>
                        This is a <b>{encounterDifficulty.toLowerCase()}</b> encounter for <b>{draftEncounter.party_size}</b> level <b>{draftEncounter.party_level}</b> players
                        </div>
                    </div>
            
            </div>

            <div class="form-group">


            <div class="section collapsible">
                <div class="section-header" on:click={() => enemiesSectionOpen = !enemiesSectionOpen}>
                    <h3>
                        Enemies ({draftEncounter.enemies.length}) - {getEnemiesXP()} XP
                        <span class="toggle-icon">{enemiesSectionOpen ? '▼' : '▶'}</span>
                    </h3>
                </div>
                
                {#if enemiesSectionOpen}
                    <div class="section-content" transition:fade>
                        
                        <div class="list-items">
                            {#each draftEncounter.enemies as enemyId : number}
                                {#if getEnemyDetails(enemyId)}
                                    <div class="list-item">
                                        <div class="entity-name">{getEnemyDetails(enemyId)?.name}</div>
                                        <div class="entity-link">
                                            <a href={getFullUrl(getEnemyDetails(enemyId)?.url)} target="_blank" rel="noopener noreferrer">
                                                <FontAwesomeIcon icon={['fas', 'link']} />
                                            </a>
                                        </div>
                                        <div class="entity-xp">XP: {getExperienceFromLevel(draftEncounter.party_level, getEnemyDetails(enemyId)?.level || 0)}</div>
                                        <div class="entity-level">Level {getEnemyDetails(enemyId)?.level}</div>
                                        <button 
                                            type="button" 
                                            on:click={() => {
                                                draftEncounter.enemies = draftEncounter.enemies.filter(id => id !== enemyId);
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
                                draftEncounter.enemies = [...draftEncounter.enemies, id];
                            }}
                            placeholder="Search for enemies..."
                            initialIds={draftEncounter.enemies}
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
                        Hazards ({draftEncounter.hazards.length}) - {getHazardsXP()} XP
                        <span class="toggle-icon">{hazardsSectionOpen ? '▼' : '▶'}</span>
                    </h3>
                </div>
                
                {#if hazardsSectionOpen}
                    <div class="section-content" transition:fade>
                        <div class="list-items">
                            {#each draftEncounter.hazards as hazardId}
                                {#if getHazardDetails(hazardId)}
                                    <div class="list-item">
                                        <div class="entity-name">{getHazardDetails(hazardId)?.name}</div>
                                        <div class="entity-link">
                                            <a href={getFullUrl(getHazardDetails(hazardId)?.url)} target="_blank" rel="noopener noreferrer">
                                                <FontAwesomeIcon icon={['fas', 'link']} />
                                            </a>
                                        </div>
                                        <div class="entity-xp">XP: {getExperienceFromLevel(draftEncounter.party_level, getHazardDetails(hazardId)?.level || 0)}</div>
                                        <div class="entity-level">Level {getHazardDetails(hazardId)?.level}</div>
                                        <button 
                                            type="button" 
                                            on:click={() => {
                                                draftEncounter.hazards = draftEncounter.hazards.filter(id => id !== hazardId);
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
                                draftEncounter.hazards = [...draftEncounter.hazards, id];
                            }}
                            placeholder="Search for hazards..."
                            initialIds={draftEncounter.hazards}
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
                        Treasure - {getTotalTreasure()} gold
                        <span class="toggle-icon">{treasureSectionOpen ? '▼' : '▶'}</span>
                    </h3>
                </div>
                
                {#if treasureSectionOpen}
                    <div class="section-content" transition:fade>
                        <div class="form-group">
                            <label for="currency">Currency</label>
                            <input 
                                type="number"
                                id="currency"
                                bind:value={draftEncounter.treasure_currency.gold}
                                min="0"
                            />
                        </div>

                        <h4>Items</h4>
                        <div class="list-items">
                            {#each draftEncounter.treasure_items as itemId}
                                {#if getItemDetails(itemId)}
                                    <div class="list-item">
                                        <span>{getItemDetails(itemId)?.name}</span> 
                                        <div class="entity-link">
                                            <a href={getFullUrl(getItemDetails(itemId)?.url)} target="_blank" rel="noopener noreferrer">
                                                <FontAwesomeIcon icon={['fas', 'link']} />
                                            </a>
                                        </div>
                                        <button 
                                            type="button" 
                                            on:click={() => {
                                                draftEncounter.treasure_items = draftEncounter.treasure_items.filter(id => id !== itemId);
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
                                draftEncounter.treasure_items = [...draftEncounter.treasure_items, id];
                            }}
                            placeholder="Search for items..."
                            initialIds={draftEncounter.treasure_items}
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

            <button type="submit">Create Encounter</button>
        </form>
    </div>

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
                                        <span class="xp">XP: {getEncounterXP(encounter)}</span>
                                        <span class="party">Level {draftEncounter.party_level} ({draftEncounter.party_size} players)</span>
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
                                                {#each encounter.enemies as enemyId}
                                                    {#if getEnemyDetails(enemyId)}
                                                        <li>{getEnemyDetails(enemyId)?.name} (XP: {getExperienceFromLevel(draftEncounter.party_level, getEnemyDetails(enemyId)?.level || 0)})</li>
                                                    {/if}
                                                {/each}
                                            </ul>
                                        </div>

                                        <div class="detail-section">
                                            <h4>Hazards ({encounter.hazards.length})</h4>
                                            <ul>
                                                {#each encounter.hazards as hazardId}
                                                    {#if getHazardDetails(hazardId)}
                                                        <li>{getHazardDetails(hazardId)?.name} (XP: {getExperienceFromLevel(draftEncounter.party_level, getHazardDetails(hazardId)?.level || 0)})</li>
                                                    {/if}
                                                {/each}
                                            </ul>
                                        </div>

                                        <div class="detail-section">
                                            <h4>Treasure</h4>
                                            <p>Currency: {encounter.treasure_currency.gold}</p>
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
                    {#each campaigns as campaign}
                        <option value={campaign.id}>{campaign.name}</option>
                    {/each}
                </select>
            </div>

            {#if selectedCompletionCampaign}
                <div class="form-group">
                    <h3>Select Participating Characters</h3>
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

{#if draftEncounter.name || draftEncounter.description}
    <div class="draft-indicator" transition:fade>
        <span class="draft-badge">Draft</span>
        Last saved automatically
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

    .form-group h3 {
        margin-bottom: 0.75rem;
        color: #374151;
    }

    .form-group p {
        color: #6b7280;
        font-style: italic;
    }
</style> 
<script lang="ts">
    import { fade } from 'svelte/transition';
    import type { Character, Log, WIPInsertLog, WIPLogEnemy, WIPLogTreasure } from '$lib/types/types';
    import { API_URL } from '$lib/config';
    import { characterStore } from '$lib/stores/characters';
    import { creatureStore, hazardStore, itemStore } from '$lib/stores/libraryStore';
    import LibraryEntityName from '$lib/components/LibraryEntityName.svelte';
    import { onMount } from 'svelte';
    import { id } from 'date-fns/locale';
    import LibrarySelector from './LibrarySelector.svelte';
    import LogCreationModal from './LogCreationModal.svelte';
    import { generateEventsFromData } from '$lib/utils/logs';
    import { arraysEqual } from '$lib/utils';
  import CampaignImportTabAlignmentOption from './CampaignImportTabAlignmentOption.svelte';

    export let selectedCampaignId: number;
    export let characters: Character[];
    export let error: string | null;
    export let fetchLogs: () => Promise<void>;

    // Add library data
    $: enemies = $creatureStore;
    $: items = $itemStore;
    $: hazards = $hazardStore;


    // Add mappings for enemies and items
    let enemyMappings: Record<string, Array<number>> = {};
    let itemMappings: Record<string, Array<number>> = {};
    let trapMappings: Record<string, Array<number>> = {};

    // Mappings of whether these are real things or new things
    // TODO: Also include custom things: allows us to differntiate between 'real', 'custom' and 'fake' things
    let enemyIncludes: Record<string, boolean> = {};
    let itemIncludes: Record<string, boolean> = {};
    let trapIncludes: Record<string, boolean> = {};

    // WIP logs- converted from importData format
    let participatingCharacterIds: number[] = [];
    let wipLogs : WIPInsertLog[] = [];

    // Initialize stores if needed
    onMount(async () => {
        await Promise.all([
            creatureStore.fetchEntities({}),
            itemStore.fetchEntities({})
        ]);
    });

    type ImportCharacter = {
        name: string;
        description: string;
    };

    type ImportLog = {
        name: string;
        session: number;
        enemies: string[];
        traps: string[];
        experience_gained: number;
        rewards: Array<number | string>;
    };

    type ImportData = {
        characters: ImportCharacter[];
        logs: ImportLog[];
    };

    let jsonInput = '';
    let importData: ImportData | null = null;
    let step: 'input' | 'characters' | 'entities' | 'events' = 'input';
    
    // Track both existing and to-be-created characters
    let characterMappings: Record<string, {
        targetId: number | string;  // string for new characters
        isNew: boolean;
    }> = {};

    // Track log mappings
    let logMappings: Record<string, number | null> = {};

    let editingLogName: string | null = null;
    let logCreationModal : LogCreationModal;

    function mergeLogs(sourceLogName: string, targetLogName: string) {
        if (!importData) return;

        const sourceLog = wipLogs.find(l => l.name === sourceLogName);
        const targetLog = wipLogs.find(l => l.name === targetLogName);
        
        if (!sourceLog || !targetLog) return;

        // Merge enemies
        targetLog.enemies = [...new Set([...targetLog.enemies, ...sourceLog.enemies])];
                
        // Add experience
        targetLog.extra_experience += sourceLog.extra_experience;
        
        // Merge rewards
        targetLog.treasures = [...targetLog.treasures, ...sourceLog.treasures];

        // Remove the source log
        importData.logs = importData.logs.filter(l => l.name !== sourceLogName);
    }

    async function tryParseJson() {
        try {
            importData = JSON.parse(jsonInput);
            if (!importData) throw new Error('Invalid JSON format');

            // Initialize character mappings
            const uniqueCharNames = new Set<string>();
            importData.characters.forEach(char => uniqueCharNames.add(char.name));
            
            // Create initial mappings for each unique character
            uniqueCharNames.forEach(name => {
                characterMappings[name] = {
                    targetId: `new_${name}`,  // Default to creating new
                    isNew: true
                };

                // Check if a character with the same exact name already exists in the campaign, default to mapping to that
                const existingChar = characters.find(c => c.name === name);
                if (existingChar) {
                    characterMappings[name].targetId = existingChar.id;
                    characterMappings[name].isNew = false;
                }
            });

            // Initialize log mappings
            importData.logs.forEach(log => {
                logMappings[log.name] = null;  // Default to creating new
            });

            // Initialize enemy mappings
            const uniqueEnemyNames = new Set(
                importData.logs.flatMap(log => log.enemies)
            );

            // TODO: Should do a better actual database check. Should start by  checking cached, then maybe with a fuzzy search, and/or a bulk search?
            const creatureMatches = await creatureStore.searchBestEntities(Array.from(uniqueEnemyNames), 0.4, {}) || new Map<string, { id: number, name: string }[]>();
            for (const name of uniqueEnemyNames) {
                let match = creatureMatches.get(name) || [];
                if (match.length > 0) {
                    // TODO: Revisit logic when addressing diverging mappings
                    enemyMappings[name] = [match[0].id];
                    enemyIncludes[name] = true;
                } else {
                    enemyMappings[name] = [0];
                    enemyIncludes[name] = false;
                }
            }

            // Initialize trap mappings
            const uniqueTrapNames = new Set(
                importData.logs.flatMap(log => log.traps)
            );

            const hazardMatches = await hazardStore.searchBestEntities(Array.from(uniqueTrapNames), 0.4, {}) || new Map<string, { id: number, name: string }[]>();
            for (const name of uniqueTrapNames) {
                let match = hazardMatches.get(name) || [];
                if (match.length > 0) {
                    // TODO: Revisit logic when addressing diverging mappings
                    trapMappings[name] = [match[0].id];
                    trapIncludes[name] = true;
                } else {
                    trapMappings[name] = [0];
                    trapIncludes[name] = false;
                }
            }

            // Initialize item mappings
            const uniqueItemNames = new Set(
                importData.logs.flatMap(log => 
                    log.rewards.filter((r): r is string => typeof r === 'string')
                )
            );

            const itemMatches = await itemStore.searchBestEntities(Array.from(uniqueItemNames), 0.4, {}) || new Map<string, { id: number, name: string }[]>();
            for (const name of uniqueItemNames) {
                let match = itemMatches.get(name) || [];
                if (match.length > 0) {
                    // TODO: Revisit logic when addressing diverging mappings
                    itemMappings[name] = [match[0].id];
                    itemIncludes[name] = true;
                } else {
                    itemMappings[name] = [0];
                    itemIncludes[name] = false;
                }
            }
           
            step = 'characters';
        } catch (e) {
            error = 'Invalid JSON format: ' + e;
        }
    }

    async function handleCharacterConfirmation() {
        try {
            // Group characters by their target mapping
            const charactersByTarget = new Map<string | number, ImportCharacter[]>();
            
            // TODO: Ensure no infinite loops of one mapping to another
            // TODO: Guessing default option for identical names to existing ones
            // TODO: Add a skip option for characters that don't map to anything and shouldn't be considered
            for (const char of importData?.characters || []) {
                const mapping = characterMappings[char.name];
                if (!mapping) continue;

                const targetId = mapping.targetId;
                if (!charactersByTarget.has(targetId)) {
                    charactersByTarget.set(targetId, []);
                }
                charactersByTarget.get(targetId)?.push(char);
            }

            // Create new characters where needed
            const submittableChars = [...charactersByTarget]
                .filter(([targetId]) => typeof targetId === 'string' && targetId.startsWith('new_'))
                .map(([_, chars]) => {
                    // Create new character using first character in group
                    const char = chars[0];
                    return {
                        name: char.name,
                        description: char.description,
                        class: 1, // Default class ID TODO: Add class selection
                        level: 1,  // Default level TODO: Add level selection,
                        class_name: "", // Default class name TODO: Add class selection (maybe don't even need this- should Omit)
                        experience: 0,  // Default experience TODO: Add experience selection. 
                        campaign_id: selectedCampaignId // TODO: remove this. probably not needed as a field for inserting characters in general
                    };
                });
            

            await characterStore.addCharacters(selectedCampaignId, submittableChars);
            
            step = 'entities';
        } catch (e) {
            error = e instanceof Error ? e.message : 'Failed to create characters';
        }
    }

    async function handleEntityConfirmation() {
        if (!importData) return;

        // Validate that all entities are mapped
        const unmappedEnemies = Object.entries(enemyMappings)
            .filter(([_, id]) => id[0] === 0)
            .filter(([name]) => enemyIncludes[name])
            .map(([name]) => name);
        
        const ummappedTraps = Object.entries(trapMappings)
            .filter(([_, id]) => id[0] === 0)
            .filter(([name]) => trapIncludes[name])
            .map(([name]) => name);

        const unmappedItems = Object.entries(itemMappings)
            .filter(([_, id]) => id[0] === 0)
            .filter(([name]) => itemIncludes[name])
            .map(([name]) => name);

        if (unmappedEnemies.length > 0 || unmappedItems.length > 0 || ummappedTraps.length > 0) {
            error = 'Please map all enemies, traps, and items before continuing';
            return;
        }

        // For a fast import, assume supplied characters are for every log
        participatingCharacterIds = characters.filter(c => characterMappings[c.name])
            .map(c => c.id);

            // Now that mappings are confirmed, we can create logs.
        // Only include ones we have mappings for

        for (const log of importData.logs) {
            let calculated_enemies : WIPLogEnemy[] = log.enemies.filter(e => enemyIncludes[e]).map(e => {
                // TODO: More than just [0]- should be a list of all possible enemies. (Mappings may diverge- "Animals" may mean "Wolf" and "Bear" together)
                let level = enemies.entities.get(enemyMappings[e][0])?.level || 1;
                let enemy : WIPLogEnemy = {
                    id: enemyMappings[e][0],
                    count: 1, // TODO: Support importing of multiple enemies (eg: two wolves)
                    level: level,  // TODO: strong/weak
                    type: 'enemy'
                };
                return enemy;
            });

            let calculated_traps = log.traps.filter(t => trapIncludes[t]).map(t => {
                // TODO: More than just [0]- should be a list of all possible traps. (Mappings may diverge- "Animals" may mean "Wolf" and "Bear" together)
                let level = hazards.entities.get(trapMappings[t][0])?.level || 1;
                let trap : WIPLogEnemy = {
                    id: trapMappings[t][0],
                    level: level,  // TODO: strong/weak
                    count: 1, // TODO: Support importing of multiple traps
                    type: 'hazard'
                };
                return trap;
            });

            let calculated_items = log.rewards.filter(r => typeof r === 'string' && itemIncludes[r]).map(r => {
                // TODO: More than just [0]- should be a list of all possible items. (Mappings may diverge- "Animals" may mean "Wolf" and "Bear" together)
                let item : WIPLogTreasure = {
                    itemId: itemMappings[r][0],
                    amount: 1, // TODO: Support importing of multiple items
                    type: 'item'
                };
                return item;
            });

            // TODO: Gold

            const wipLog: WIPInsertLog = {
                name: log.name,
                description: '', // TODO: Add description
                // session: log.session, // TODO: Sessions
                characterIds: participatingCharacterIds,
                enemies: calculated_enemies.concat(calculated_traps),
                extra_experience: 0, // TODO: Should this be calculated? can't just be experience_gained
                treasures: calculated_items,
                current_manual_events: generateEventsFromData(participatingCharacterIds, characters, calculated_enemies.concat(calculated_traps), calculated_items),
            };

            wipLogs.push(wipLog);
        }


        step = 'events';
    }

    async function handleEventConfirmation() {
        if (!importData) return;

        try {
            for (const log of wipLogs) {
                const targetLogId = logMappings[log.name];
                
                if (targetLogId) {
                    // Add events to existing log
                    await fetch(`${API_URL}/campaign/${selectedCampaignId}/events`, {
                        method: 'POST',
                        credentials: 'include',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify({
                            event_group: targetLogId,
                            events: log.current_manual_events
                        })
                    });
                } else {
                    // Create new log
                    const logResponse = await fetch(`${API_URL}/campaign/${selectedCampaignId}/logs`, {
                        method: 'POST',
                        credentials: 'include',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify({
                            name: log.name,
                            // TODO: Re-add, and also, logs should be able to have sessions
                            // description: `Session ${log.session}`,
                            events: log.current_manual_events,
                        })
                    });

                    if (!logResponse.ok) throw new Error('Failed to create log');
                }
            }

            await fetchLogs();
            step = 'input';
            jsonInput = '';
            importData = null;

            characterMappings = {};
            logMappings = {};
            enemyMappings = {};
            trapMappings = {};
            itemMappings = {};
        } catch (e) {
            error = e instanceof Error ? e.message : 'Failed to create events';
        }
    }

    async function removeAllUnconfirmedMappings() {
        for (const name in enemyMappings) {
            // TODO: More than just 0 for diverging mappings.
            if (enemyMappings[name][0] === 0) {
                enemyIncludes[name] = false;
            }
        }

        for (const name in trapMappings) {
            // TODO: Confirm this works when finishing import.
            if (trapMappings[name][0] === 0) {
                trapIncludes[name] = false;
            }
        }

        for (const name in itemMappings) {
            // TODO: Confirm this works when finishing import.
            if (itemMappings[name][0] === 0) {
                itemIncludes[name] = false;
            }
        }
    }
</script>

<div class="import-section" transition:fade>
   
    <div class="import-header">
        <h2>Import Campaign Data</h2>
    </div>

    {#if step === 'input'}
        <div class="import-form">
            <p class="help-text">
                Paste your JSON data below. The format should be:
            </p>
            <pre><code>
{`
{
    "characters": [{
        "name": "string",
        "description": "string"
    }],
    "logs": [{
        "name": "string",
        "session": number,
        "characters": ["string"],  // Character names that participated
        "enemies": ["string"],
        "traps": ["string"],
        "experience_gained": number,
        "rewards": [number or "string"]
    }]
}
`}
            </code></pre>
            <textarea
                bind:value={jsonInput}
                placeholder="Paste your JSON here..."
                rows="10"
            ></textarea>
            <button class="import-btn" on:click={tryParseJson}>
                Import
            </button>
        </div>
    {:else if step === 'characters'}
        <div class="character-confirmation">
            <h3>Confirm Characters</h3>
            <p>Map characters to existing ones or other imported characters:</p>
            
            {#each importData?.characters || [] as char}
                <div class="character-mapping">
                    <div class="character-info">
                        <strong>{char.name}</strong>
                        <p>{char.description}</p>
                    </div>
                    <select 
                        value={characterMappings[char.name].targetId}
                        on:change={() => {
                            // isNew if the targetId is a string and starts with 'new_'
                            let target = characterMappings[char.name].targetId;
                            if (typeof target === 'string') {
                                characterMappings[char.name].isNew = target.startsWith('new_');
                            } else {
                                characterMappings[char.name].isNew = false;
                            }
                        }}
                    >
                        <option value={`new_${char.name}`}>Create New Character</option>
                        <optgroup label="Existing Characters">
                            {#each characters as existingChar}
                                <option value={existingChar.id}>{existingChar.name}</option>
                            {/each}
                        </optgroup>
                        <optgroup label="Importing">
                            {#each importData?.characters || [] as importChar}
                                {#if importChar.name !== char.name}
                                    <option value={`new_${importChar.name}`}>
                                        {importChar.name} (Importing)
                                    </option>
                                {/if}
                            {/each}
                        </optgroup>
                    </select>
                </div>
            {/each}

            <button class="confirm-btn" on:click={handleCharacterConfirmation}>
                Confirm Characters
            </button>
        </div>
    {:else if step === 'entities'}
        <div class="entity-confirmation">
            <h3>Map Enemies, Traps and Items</h3>
            <button class="confirm-btn" on:click={removeAllUnconfirmedMappings}>
                Remove all unconfirmed mappings
            </button>
            {#if Object.keys(enemyMappings).length > 0}
                <div class="entity-section">
                    <h4>Enemies</h4>
                    {#each Object.entries(enemyMappings) as [name, id]}
                        <CampaignImportTabAlignmentOption 
                        name={name}
                        mappings={enemyMappings}
                        includes={enemyIncludes}
                        alignmentType="enemy"
                        />
                    {/each}
                </div>
            {/if}

            {#if Object.keys(trapMappings).length > 0}
                <div class="entity-section">
                    <h4>Traps</h4>
                    {#each Object.entries(trapMappings) as [name, id]}
                        <CampaignImportTabAlignmentOption 
                        name={name}
                        mappings={trapMappings}
                        includes={trapIncludes}
                        alignmentType="hazard"
                        />
                    {/each}
                </div>
            {/if}

            {#if Object.keys(itemMappings).length > 0}
                <div class="entity-section">
                    <h4>Items</h4>
                    {#each Object.entries(itemMappings) as [name, id]}
                        <CampaignImportTabAlignmentOption 
                            name={name}
                            mappings={itemMappings}
                            includes={itemIncludes}
                            alignmentType="item"
                            />
                    {/each}
                </div>
            {/if}

            <button class="confirm-btn" on:click={handleEntityConfirmation}>
                Confirm Mappings
            </button>
        </div>
    {:else if step === 'events'}
        <div class="events-confirmation">
            <h3>Review Logs</h3>
            <p>Review logs and merge if needed. Logs with no data will not be created.</p>

            {#each wipLogs || [] as log}
                <div class={log.current_manual_events.length > 0 ? 'log-preview' : 'log-preview-skipped'}>
                    <div class="log-header">
                        <h4>{log.name} (Session 0)</h4>
                        <div class="log-actions">
                            <select>
                                <option value="">Select log to merge with...</option>
                                {#each importData?.logs || [] as otherLog}
                                    {#if otherLog.name !== log.name}
                                        <option value={otherLog.name}>{otherLog.name}</option>
                                    {/if}
                                {/each}
                            </select>
                            <button 
                                class="merge-btn" 
                                on:click={(e) => {
                                    const select = e.currentTarget.previousElementSibling as HTMLSelectElement;
                                    if (select.value) {
                                        mergeLogs(log.name, select.value);
                                        select.value = '';
                                    }
                                }}
                            >
                                Merge
                            </button>
                            <button 
                                class="edit-btn"
                                on:click={() => {
                                    editingLogName = log.name
                                    const foundLog = wipLogs.find(l => l.name === log.name);
                                    logCreationModal.setInitialData(foundLog || null);
                                    }}
                            >
                                Edit
                            </button>
                        </div>
                    </div>

                    {#if log.current_manual_events.length <= 0}
                            <p>(No events, will be skipped)</p>
                    {/if}
                    
                     {#if !arraysEqual(log.characterIds, participatingCharacterIds)}
                        <div class="event-group">
                            <strong>Participating Characters:</strong>
                            <ul>
                                {#each log.characterIds as charId}
                                    <li>
                                        {characters.find(c => c.id === charId)?.name || 'Unknown'}
                                    </li>
                                {/each}
                            </ul>
                        </div>
                    {/if}

                    {#if log.enemies.length > 0}
                        <div class="event-group">
                            <strong>Enemies / Hazards:</strong>
                            <ul>
                                {#each log.enemies as enemy}
                                    <li>
                                        <LibraryEntityName 
                                            entityType='creature' entityId={enemy.id}
                                        />
                                    </li>
                                {/each}
                            </ul>
                        </div>
                    {/if}

                    <!-- <div class="event-group">
                        <strong>Experience:</strong> {log.experience_gained}
                    </div>-->

                    {#if log.treasures.length > 0}
                        <div class="event-group">
                            <strong>Rewards:</strong>
                            <ul>
                                {#each log.treasures as reward}
                                <!-- TODO: Silver, etc -->
                                    <li>
                                        {#if reward.type === 'currency'}
                                            {reward.amount} gold
                                        {:else if reward.itemId}
                                            <LibraryEntityName 
                                                entityType='item' entityId={reward.itemId} 
                                            />
                                        {/if}
                                    </li>
                                {/each}
                            </ul>
                        </div>
                    {/if} 
                </div>
            {/each}

            <button class="confirm-btn" on:click={handleEventConfirmation}>
                Confirm Events
            </button>
        </div>
    {/if}
</div>
        <!-- TODO: wipLogs.find should use an id for duplicate reasons  -->
    <LogCreationModal
        show={editingLogName !== null}
        bind:this={logCreationModal}
        fetchLogs={fetchLogs}
        {selectedCampaignId}
        {characters}
        initialData={wipLogs.find(l => l.name === editingLogName) || null}
        on:close={() => editingLogName = null}
        updateOnlyCallback={(log) => {
            const index = wipLogs.findIndex(l => l.name === editingLogName);
            wipLogs[index] = log;
            editingLogName = null;
        }}
/>

<style>
    .import-section {
        background: white;
        border-radius: 0.5rem;
        box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
        padding: 1.5rem;
    }

    .import-header {
        margin-bottom: 1.5rem;
    }

    .help-text {
        margin-bottom: 1rem;
        color: #666;
    }

    textarea {
        width: 100%;
        padding: 0.5rem;
        border: 1px solid #ddd;
        border-radius: 0.25rem;
        margin-bottom: 1rem;
    }

    .character-mapping {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 1rem;
        border: 1px solid #ddd;
        border-radius: 0.25rem;
        margin-bottom: 1rem;
    }

    .character-info {
        flex: 1;
        margin-right: 1rem;
    }

    .character-info p {
        margin: 0.5rem 0 0 0;
        color: #666;
    }

    .log-preview {
        background: #f5f5f5;
        padding: 1rem;
        border-radius: 0.25rem;
        margin-bottom: 1rem;
    }

    .log-preview-skipped {
        background: #bcbcbc;
        padding: 1rem;
        border-radius: 0.25rem;
        margin-bottom: 1rem;
    }

    .event-group {
        margin: 0.5rem 0;
    }

    .event-group ul {
        margin: 0.5rem 0;
        padding-left: 1.5rem;
    }

    .import-btn, .confirm-btn {
        background: #3b82f6;
        color: white;
        padding: 0.5rem 1rem;
        border-radius: 0.25rem;
        border: none;
        cursor: pointer;
    }

    .import-btn:hover, .confirm-btn:hover {
        background: #2563eb;
    }

    .character-mapping {
        display: flex;
        gap: 1rem;
        margin-bottom: 2rem;
    }

    .character-mapping select {
        flex: 1;
        padding: 0.5rem;
        font-size: 1rem;
        border: 1px solid #e2e8f0;
        border-radius: 0.475rem;
    }


    .log-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 1rem;
    }

    .log-header select {
        min-width: 200px;
    }

    optgroup {
        font-weight: 600;
    }

    .entity-section {
        margin: 1rem 0;
        padding: 1rem;
        background: #f8f8f8;
        border-radius: 0.25rem;
    }



    .log-actions {
        display: flex;
        gap: 0.5rem;
        align-items: center;
    }

    .merge-btn, .edit-btn {
        padding: 0.25rem 0.5rem;
        border-radius: 0.25rem;
        border: none;
        cursor: pointer;
        font-size: 0.475rem;
    }

    .merge-btn {
        background: #4f46e5;
        color: white;
    }

    .merge-btn:hover {
        background: #4338ca;
    }

    .edit-btn {
        background: #10b981;
        color: white;
    }

    .edit-btn:hover {
        background: #059669;
    }
</style> 

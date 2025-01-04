<script lang="ts">
    import { fade } from 'svelte/transition';
    import type { Character, Log } from '$lib/types/types';
    import { API_URL } from '$lib/config';
    import { characterStore } from '$lib/stores/characters';
    import { creatureStore, itemStore } from '$lib/stores/libraryStore';
    import LibraryEntityName from '$lib/components/LibraryEntityName.svelte';
    import { onMount } from 'svelte';
    import { id } from 'date-fns/locale';
    import LibrarySelector from './LibrarySelector.svelte';

    export let selectedCampaignId: number;
    export let characters: Character[];
    export let campaignLogs: Log[];
    export let error: string | null;
    export let fetchLogs: () => Promise<void>;

    // Add library data
    $: enemies = $creatureStore;
    $: items = $itemStore;


    // Add mappings for enemies and items
    let enemyMappings: Record<string, number> = {};
    let itemMappings: Record<string, number> = {};

    // Mappings of whether these are real things or new things
    // TODO: Also include custom things: allows us to differntiate between 'real', 'custom' and 'fake' things
    let enemyIncludes: Record<string, boolean> = {};
    let itemIncludes: Record<string, boolean> = {};

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

    async function tryParseJson() {
        try {
            importData = JSON.parse(jsonInput);

            console.log(importData);
            console.log("here1");

            // Initialize character mappings
            const uniqueCharNames = new Set<string>();
            importData.characters.forEach(char => uniqueCharNames.add(char.name));
            
            console.log("here2");
            // Create initial mappings for each unique character
            uniqueCharNames.forEach(name => {
                characterMappings[name] = {
                    targetId: `new_${name}`,  // Default to creating new
                    isNew: true
                };
            });
            console.log("here3");

            // Initialize log mappings
            importData.logs.forEach(log => {
                logMappings[log.name] = null;  // Default to creating new
            });

            // Initialize enemy mappings - try to match names exactly first
            const uniqueEnemyNames = new Set(
                importData.logs.flatMap(log => log.enemies)
            );
            console.log("here4");

            // TODO: Should do a better actual database check. Should start by  checking cached, then maybe with a fuzzy search, and/or a bulk search?
            for (const name of uniqueEnemyNames) {
                // Search for the enemy in the library
                // TODO: As said above, should do one fetch instead of n times
                await creatureStore.fetchEntities({ name: name });

                // Check if an enemy was found matching the name exactly
                const match = Array.from(enemies.entities.values()).find(e => e.name === name); 
                enemyMappings[name] = match?.id || 0; // 0 means "needs mapping"
                enemyIncludes[name] = true;
            }
            console.log("here5");

            // Initialize item mappings - try to match names exactly first
            const uniqueItemNames = new Set(
                importData.logs.flatMap(log => 
                    log.rewards.filter((r): r is string => typeof r === 'string')
                )
            );
            console.log("here6");

            for (const name of uniqueItemNames) {
                // TODO: As said above, should do one fetch instead of n times
                await itemStore.fetchEntities({ name: name });

                const match = Array.from(items.entities.values()).find(i => i.name === name);
                itemMappings[name] = match?.id || 0; // 0 means "needs mapping"
                itemIncludes[name] = true;
            }
            console.log("her72");

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
            
            console.log(characterMappings);

            for (const char of importData?.characters || []) {
                const mapping = characterMappings[char.name];
                if (!mapping) continue;

                const targetId = mapping.targetId;
                if (!charactersByTarget.has(targetId)) {
                    charactersByTarget.set(targetId, []);
                }
                charactersByTarget.get(targetId)?.push(char);
            }

            console.log(charactersByTarget);

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
            
            console.log(submittableChars);

            await characterStore.addCharacters(selectedCampaignId, submittableChars);
            
            console.log("Done, moving to entities");
            step = 'entities';
        } catch (e) {
            error = e instanceof Error ? e.message : 'Failed to create characters';
        }
    }

    async function handleEntityConfirmation() {
        // Validate that all entities are mapped
        const unmappedEnemies = Object.entries(enemyMappings)
            .filter(([_, id]) => id === 0)
            .filter(([name]) => enemyIncludes[name])
            .map(([name]) => name);
        
        const unmappedItems = Object.entries(itemMappings)
            .filter(([_, id]) => id === 0)
            .filter(([name]) => itemIncludes[name])
            .map(([name]) => name);

        if (unmappedEnemies.length > 0 || unmappedItems.length > 0) {
            error = 'Please map all enemies and items before continuing';
            return;
        }

        step = 'events';
    }

    async function handleEventConfirmation() {
        if (!importData) return;
        

        try {
            for (const log of importData.logs) {
                const targetLogId = logMappings[log.name];
                
                console.log(log);

                // For a fast import, assume supplied characters are for every log
                const participatingCharacterIds = characters.filter(c => characterMappings[c.name])
                    .map(c => c.id);

                const events = [];
                
                // Add enemy events
                for (const enemy of log.enemies) {
                    events.push({
                        characters: participatingCharacterIds,
                        event_type: 'EnemyDefeated',
                        description: `Defeated ${enemy}`,
                        data: { id: enemyMappings[enemy] }
                    });
                }

                // Add trap events
                for (const trap of log.traps) {
                    events.push({
                        characters: participatingCharacterIds,
                        event_type: 'HazardDefeated',
                        description: `Overcame ${trap}`,
                        data: { name: trap }
                    });
                }

                // Add experience event
                events.push({
                    characters: participatingCharacterIds,
                    event_type: 'ExperienceGain',
                    description: `Gained ${log.experience_gained} experience`,
                    data: { experience: log.experience_gained }
                });

                // Add reward events
                for (const reward of log.rewards) {
                    events.push({
                        characters: participatingCharacterIds,
                        event_type: typeof reward === 'number' ? 'CurrencyGain' : 'ItemGain',
                        description: typeof reward === 'number' 
                            ? `Gained ${reward} gold` 
                            : `Gained item: ${reward}`,
                        data: typeof reward === 'number' 
                            ? { currency: { gold: reward } }
                            : { id: itemMappings[reward] }
                    });
                }

                if (targetLogId) {
                    // Add events to existing log
                    await fetch(`${API_URL}/campaign/${selectedCampaignId}/events`, {
                        method: 'POST',
                        credentials: 'include',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify({
                            event_group: targetLogId,
                            events
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
                            description: `Session ${log.session}`,
                            events
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
            itemMappings = {};
        } catch (e) {
            error = e instanceof Error ? e.message : 'Failed to create events';
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
                        bind:value={characterMappings[char.name].targetId}
                        on:change={() => {
                            characterMappings[char.name].isNew = 
                                typeof characterMappings[char.name].targetId === 'string';
                        }}
                    >
                        <option value={`new_${char.name}`}>Create New Character</option>
                        <optgroup label="Existing Characters">
                            {#each characters as existingChar}
                                <option value={existingChar.id}>{existingChar.name}</option>
                            {/each}
                        </optgroup>
                        <optgroup label="Importing">
                            {#each importData.characters as importChar}
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
            <h3>Map Enemies and Items</h3>
            
            {#if Object.keys(enemyMappings).length > 0}
                <div class="entity-section">
                    <h4>Enemies</h4>
                    {#each Object.entries(enemyMappings) as [name, id]}
                        <div class="entity-mapping">
                            <strong>{name}</strong>
                            <div class="entity-mapping-options">
                                {#if enemyIncludes[name]}
                                <LibrarySelector
                                    entityType="creature"
                                    onSelect={id => enemyMappings[name] = id}
                                    showSelected={enemyMappings[name]}
                                    placeholder="Search for a enemy..."
                                    initialIds={[]}
                                />
                                {/if}
                                <button on:click={() => enemyIncludes[name] = !enemyIncludes[name]} class="confirm-btn">
                                    {enemyIncludes[name] ? 'Included' : 'Not included'}
                                </button>

                            </div>
                        </div>
                    {/each}
                </div>
            {/if}

            {#if Object.keys(itemMappings).length > 0}
                <div class="entity-section">
                    <h4>Items</h4>
                    {#each Object.entries(itemMappings) as [name, id]}
                        <div class="entity-mapping">
                            <strong>{name}</strong>
                            <div class="entity-mapping-options">
                                {#if itemIncludes[name]}
                                <LibrarySelector
                                    entityType="item"
                                    onSelect={id => itemMappings[name] = id}
                                    showSelected={itemMappings[name]}
                                    placeholder="Search for a item..."
                                    initialIds={[]}
                                />
                            {/if}
                            <button on:click={() => itemIncludes[name] = !itemIncludes[name]} class="confirm-btn">
                                {itemIncludes[name] ? 'Included' : 'Not included'}
                            </button>
                            </div>
                        </div>
                    {/each}
                </div>
            {/if}

            <button class="confirm-btn" on:click={handleEntityConfirmation}>
                Confirm Mappings
            </button>
        </div>
    {:else if step === 'events'}
        <div class="events-confirmation">
            <h3>Confirm Logs</h3>
            <p>Review and map logs to existing ones (mapping will merge events):</p>

            {#each importData?.logs || [] as log}
                <div class="log-preview">
                    <div class="log-header">
                        <h4>{log.name} (Session {log.session})</h4>
                        <select bind:value={logMappings[log.name]}>
                            <option value={null}>Create New Log</option>
                            {#each campaignLogs as existingLog}
                                <option value={existingLog.id}>{existingLog.name}</option>
                            {/each}
                        </select>
                    </div>
                    
                    <div class="event-group">
                        <strong>Participating Characters:</strong>
                        <ul>
                            {#each log.characters as charName}
                                <li>
                                    {charName} → 
                                    {#if characterMappings[charName]?.isNew}
                                        New Character
                                    {:else}
                                        {characters.find(c => c.id === characterMappings[charName]?.targetId)?.name || 'Unknown'}
                                    {/if}
                                </li>
                            {/each}
                        </ul>
                    </div>

                    {#if log.enemies.length > 0}
                        <div class="event-group">
                            <strong>Enemies:</strong>
                            <ul>
                                {#each log.enemies as enemy}
                                    <li>
                                        {enemy} → 
                                        <LibraryEntityName 
                                            entity={enemies.find(e => e.id === enemyMappings[enemy])} 
                                        />
                                    </li>
                                {/each}
                            </ul>
                        </div>
                    {/if}

                    {#if log.traps.length > 0}
                        <div class="event-group">
                            <strong>Traps:</strong>
                            <ul>
                                {#each log.traps as trap}
                                    <li>{trap}</li>
                                {/each}
                            </ul>
                        </div>
                    {/if}

                    <div class="event-group">
                        <strong>Experience:</strong> {log.experience_gained}
                    </div>

                    {#if log.rewards.length > 0}
                        <div class="event-group">
                            <strong>Rewards:</strong>
                            <ul>
                                {#each log.rewards as reward}
                                    <li>
                                        {#if typeof reward === 'number'}
                                            {reward} gold
                                        {:else}
                                            {reward} → 
                                            <LibraryEntityName 
                                                entity={items.find(i => i.id === itemMappings[reward])} 
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

    .entity-mapping {
        display: grid;
        grid-template-columns: 0.5fr 0.5fr;

        padding: 0.5rem;
        margin: 0.5rem 0;
        background: white;
        border-radius: 0.25rem;
    }

    .entity-mapping-options {
        display: flex;
        justify-content: right;
        gap: 1rem;
    }

    .entity-mapping button {
        background: #3b82f6;
        color: white;
        padding: 0.5rem 1rem;
        border-radius: 0.25rem;
        border: none;
        cursor: pointer;
    }

    .entity-mapping button:hover {
        background: #2563eb;
    }

    .entity-mapping strong {
        flex: 1;
    }

    .entity-mapping-options :global(.selected-input) { 
        background-color: #f0f9eb;
    }

    .entity-mapping-options :global(.unselected-input) { 
        background-color: #f4b4b4;
    }
</style> 
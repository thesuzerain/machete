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

    export let name: string;
    export let alignmentType: 'enemy' | 'hazard' | 'item';
    export let mappings: Record<string, Array<number>>;
    export let includes: Record<string, boolean>;

    let open = false;

    let entityTypeChosen : 'creature' | 'item' | 'hazard' = {
        enemy: 'creature' as 'creature',
        item: 'item' as 'item',
        hazard: 'hazard' as 'hazard'
    }[alignmentType];

    let placeholderString = {
        enemy: 'Search for an enemy...',
        item: 'Search for an item...',
        hazard: 'Search for a hazard...'
    }[alignmentType];

    function handleRowClick() {
        open = !open;
    }

</script>
<div class="entity-mapping-container" on:click={(e) => { e.stopPropagation(); handleRowClick(); }}> 
    
<div class="entity-mapping">
    {#if open}
        -
    {:else}
        +
    {/if}
    <strong>{name}</strong>
    <div class="entity-mapping-options">
        {#if includes[name]}
            {#if mappings[name] && mappings[name].length > 1}
                {#each mappings[name] as id}
                    <LibraryEntityName entityType={entityTypeChosen} entityId={id} />
                {/each}
            {:else}
                {#if open}
                    <LibraryEntityName entityType={entityTypeChosen} entityId={mappings[name][0]} />
                {:else}
                    <LibrarySelector
                    entityType={entityTypeChosen}
                    onSelect={id => mappings[name][0] = id}
                    showSelected={mappings[name][0]}
                    placeholder={placeholderString}
                    initialIds={[]}
                />
                {/if}

            {/if}
        {/if}
        <button on:click={() => includes[name] = !includes[name]} class="confirm-btn">
            {includes[name] ? 'Included' : 'Not included'}
        </button>

    </div>
</div>
{#if open}
{#if includes[name]}
        {#if mappings[name] && mappings[name].length > 0}
            {#each mappings[name] as id, i}
                <LibrarySelector
                    entityType={entityTypeChosen}
                    onSelect={id => mappings[name][i] = id}
                    showSelected={mappings[name][i]}
                    placeholder={placeholderString}
                    initialIds={[]}
                />
            {/each}
        {/if}
        <button on:click={() => mappings[name].push(0)} class="confirm-btn">Add another</button>

    {/if}
{/if}
</div>

<style>
.entity-mapping-container {
    padding: 0.5rem;
    margin: 0.5rem 0;
    background: white;
    border-radius: 0.25rem;
}

.entity-mapping {
    display: grid;
    grid-template-columns: 0.02fr 0.48fr 0.5fr;
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
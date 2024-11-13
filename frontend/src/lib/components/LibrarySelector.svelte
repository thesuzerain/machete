<script lang="ts">
    import { onMount } from 'svelte';

    interface LibraryEntity {
        id: number;
        name: string;
        level?: number;  // Optional for creatures/hazards
        price?: Currency;  // Optional for items (cost)
    }

    // TODO: Abstract, this shouldn't be here.
    interface Currency {
        gold?: number;
        silver?: number;
        copper?: number;
    }

    export let entityType: 'creature' | 'hazard' | 'item';
    export let onSelect: (entityId: number) => void;
    export let placeholder = "Search...";

    let entities: LibraryEntity[] = [];
    let searchTerm = '';
    let loading = true;
    let error: string | null = null;
    let showDropdown = false;

    onMount(async () => {
        try {
            // Map entity type to endpoint
            const endpoint = entityType === 'item' ? 'items' : `${entityType}s`;
            const response = await fetch(`/api/library/${endpoint}`);
            if (!response.ok) throw new Error(`Failed to fetch ${entityType}s`);
            entities = await response.json();
            console.log("entities",entities)
        } catch (e) {
            error = e instanceof Error ? e.message : `Failed to load ${entityType}s`;
        } finally {
            loading = false;
        }
    });

    $: filteredEntities = entities.filter(entity => 
        entity.name.toLowerCase().includes(searchTerm.toLowerCase())
    );

    function handleSelect(entity: LibraryEntity) {
        console.log("selected:", entity);
        onSelect(entity.id);
        searchTerm = '';
        showDropdown = false;
    }
</script>

<div class="entity-selector">
    <input
        type="text"
        {placeholder}
        bind:value={searchTerm}
        on:focus={() => showDropdown = true}
    />
    
    {#if showDropdown && searchTerm.length > 0}
        <div class="dropdown">
            {#if loading}
                <div class="dropdown-item loading">Loading...</div>
            {:else if error}
                <div class="dropdown-item error">{error}</div>
            {:else if filteredEntities.length === 0}
                <div class="dropdown-item no-results">No matches found</div>
            {:else}
                {#each filteredEntities as entity}
                    <button
                        class="dropdown-item"
                        on:click={() => handleSelect(entity)}
                    >
                        <span class="name">{entity.name}</span>
                        {#if entity.level !== undefined}
                            <span class="detail">Lv: {entity.level}</span>
                        {/if}
                        {#if entity.value !== undefined}
                            <span class="detail">{entity.value} gp</span>
                        {/if}
                    </button>
                {/each}
            {/if}
        </div>
    {/if}
</div>

<style>
    .entity-selector {
        position: relative;
        width: 100%;
    }

    input {
        width: 100%;
        padding: 0.5rem;
        border: 1px solid #ddd;
        border-radius: 4px;
    }

    .dropdown {
        position: absolute;
        top: 100%;
        left: 0;
        right: 0;
        max-height: 200px;
        overflow-y: auto;
        background: white;
        border: 1px solid #ddd;
        border-radius: 4px;
        box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        z-index: 10;
    }

    .dropdown-item {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 0.5rem;
        width: 100%;
        text-align: left;
        border: none;
        background: none;
        cursor: pointer;
        color: #666;
    }

    .dropdown-item:hover {
        background: #f8f8f8;
    }

    .dropdown-item.loading,
    .dropdown-item.error,
    .dropdown-item.no-results {
        color: #666;
        font-style: italic;
    }

    .level {
        color: #666;
        font-size: 0.875rem;
    }    .detail {
        color: #666;
        font-size: 0.875rem;
    }
</style> 
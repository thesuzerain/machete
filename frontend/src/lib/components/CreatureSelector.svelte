<script lang="ts">
    import { onMount } from 'svelte';

    interface LibraryCreature {
        id: number;
        name: string;
        level: number;
    }

    export let campaignId: number;
    export let creatureType: 'creature' | 'hazard';
    export let onSelect: (creatureId: number) => void;
    export let placeholder = "Search creatures...";

    let creatures: LibraryCreature[] = [];
    let searchTerm = '';
    let loading = true;
    let error: string | null = null;
    let showDropdown = false;

    onMount(async () => {
        try {
            const response = await fetch(`/api/library/${creatureType}s`);
            if (!response.ok) throw new Error(`Failed to fetch ${creatureType}s`);
            creatures = await response.json();
        } catch (e) {
            error = e instanceof Error ? e.message : `Failed to load ${creatureType}s`;
        } finally {
            loading = false;
        }
    });

    $: filteredCreatures = creatures.filter(creature => 
        creature.name.toLowerCase().includes(searchTerm.toLowerCase())
    );

    function handleSelect(creature: LibraryCreature) {
        console.log("selected:", creature)
        onSelect(creature.id);
        searchTerm = '';
        showDropdown = false;
    }
</script>

<div class="creature-selector">
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
            {:else if filteredCreatures.length === 0}
                <div class="dropdown-item no-results">No matches found</div>
            {:else}
                {#each filteredCreatures as creature}
                    <button
                        class="dropdown-item"
                        on:click={() => handleSelect(creature)}
                    >
                        <span class="name">{creature.name}</span>
                        <span class="level">Lv: {creature.level}</span>
                    </button>
                {/each}
            {/if}
        </div>
    {/if}
</div>

<style>
    .creature-selector {
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
    }
</style> 
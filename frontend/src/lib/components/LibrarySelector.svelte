<script lang="ts">
    import type { Currency } from '$lib/types/library';
    import { onMount } from 'svelte';
    import { debounce } from '$lib/utils';

    interface LibraryEntity {
        id: number;
        name: string;
        level?: number;
        price?: Currency;
    }

    export let entityType: 'creature' | 'hazard' | 'item' | 'class';
    export let onSelect: (entityId: number) => void;
    export let placeholder = "Search...";
    export let initialIds: number[] = [];

    let entities: Map<number, LibraryEntity> = new Map();
    let searchTerm = '';
    let loading = true;
    let error: string | null = null;
    let showDropdown = false;
    let page = 0;
    let hasMore = true;
    let loadingMore = false;

    const routePart = {
        'creature': 'creatures',
        'hazard': 'hazards',
        'item': 'items',
        'class': 'classes'
    };

    async function fetchEntities(params: Record<string, string>) {
        const endpoint = routePart[entityType];
        const queryString = new URLSearchParams(params).toString();
        const response = await fetch(`/api/library/${endpoint}?${queryString}`);
        if (!response.ok) throw new Error(`Failed to fetch ${entityType}s`);
        const data = await response.json();
        
        data.forEach((entity: LibraryEntity) => {
            entities.set(entity.id, entity);
        });
        entities = entities;

        hasMore = data.length === 100;
        return data;
    }

    const debouncedSearch = debounce(async (searchTerm: string) => {
        try {
            loading = true;
            page = 0;
            entities = new Map();
            const data = await fetchEntities({ 
                name: searchTerm,
                page: '0'
            });
            data.forEach((entity: LibraryEntity) => {
                entities.set(entity.id, entity);
            });
            entities = entities;
        } catch (e) {
            error = e instanceof Error ? e.message : `Failed to load ${entityType}s`;
        } finally {
            loading = false;
        }
    }, 300);

    async function loadMore() {
        if (loadingMore || !hasMore) return;
        
        try {
            loadingMore = true;
            page += 1;
            await fetchEntities({ 
                name: searchTerm,
                page: page.toString()
            });
        } catch (e) {
            error = e instanceof Error ? e.message : `Failed to load more ${entityType}s`;
            hasMore = false;
        } finally {
            loadingMore = false;
        }
    }

    function handleScroll(event: Event) {
        const target = event.target as HTMLDivElement;
        if (target.scrollHeight - target.scrollTop <= target.clientHeight + 50) {
            loadMore();
        }
    }

    onMount(async () => {
        try {
            if (initialIds.length > 0) {
                await fetchEntities({ ids: initialIds.join(',') });
            }
            
            await fetchEntities({ page: '0' });
        } catch (e) {
            error = e instanceof Error ? e.message : `Failed to load ${entityType}s`;
        } finally {
            loading = false;
        }
    });

    $: filteredEntities = Array.from(entities.values())
        .filter(entity => entity.name.toLowerCase().includes(searchTerm.toLowerCase()));

    $: if (searchTerm) {
        debouncedSearch(searchTerm);
    }

    function handleSelect(entity: LibraryEntity) {
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
        <div class="dropdown" on:scroll={handleScroll}>
            {#if loading && !loadingMore}
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
                        {#if entity.price?.gold !== undefined}
                            <span class="detail">{entity.price.gold} gp</span>
                        {/if}
                    </button>
                {/each}
                {#if loadingMore}
                    <div class="dropdown-item loading">Loading more...</div>
                {/if}
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
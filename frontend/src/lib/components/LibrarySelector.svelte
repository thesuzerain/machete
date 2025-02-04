<script lang="ts">
    import { onMount, tick } from 'svelte';
    import { debounce } from '$lib/utils';
    import { API_URL } from '$lib/config';
    import { classStore, creatureStore, hazardStore, itemStore } from '$lib/stores/libraryStore';
  import type { LibraryEntity, LibrarySearchRequest } from '$lib/types/library';

  
    export let entityType: 'creature' | 'hazard' | 'item' | 'class';
    export let onSelect: (entityId: number) => void;
    export let placeholder = "Search...";
    export let initialIds: number[] = [];
    export let showSelected : number | null = null;

    let shownEntities: LibraryEntity[] = [];

    let searchTerm = '';
    let loading = true;
    let error: string | null = null;
    let showDropdown = false;
    let page = 0;
    let hasMore = true;
    let loadingMore = false;

    let selectedIndex = 0;

    let isMouseActive = false; // Flag to track mouse activity

    const routePart = {
        'creature': 'creatures',
        'hazard': 'hazards',
        'item': 'items',
        'class': 'classes'
    };

    const stores = {
        creature: creatureStore,
        hazard: hazardStore,
        item: itemStore,
        class: classStore
    };

    $: storeEntities = {
        creature: $creatureStore.entities,
        hazard: $hazardStore.entities,
        item: $itemStore.entities,
        class: $classStore.entities
    };

    // Searches for entities
    // Favours exact start of name (eg: "drag" for "dragon") but will follow up with similar matches
    // So "lich" will return "lich" first, but "demlich" (typo intentional) will be in the list after, even though it's alphabetically first
    async function searchEntities(query: string | null, ids : number[] | null, page: string = '0') {
        const endpoint = routePart[entityType];
        let params : Record<string, string> = {
            page,
        }
        if (query) {
            params.query = query;
            params.favor_exact_start = "true";
            params.min_similarity = "0.1";
        }
        if (ids) {
            params.ids = ids.join(','); // Add initial ids
        }

        // TODO: Use store for this, make sure oladed values are in store
        const queryString = new URLSearchParams(params).toString();

        let returnedEntities : LibraryEntity[];
        if (query) {
            let url = `${API_URL}/library/${endpoint}/search?${queryString}`;

            const response = await fetch(url);
            if (!response.ok) throw new Error(`Failed to fetch ${entityType}s`);
            const data : LibrarySearchRequest<LibraryEntity> = await response.json();

            // We provide one query, so we can return the relevant element
            returnedEntities = data[query] || [];

        } else {
            let url = `${API_URL}/library/${endpoint}?${queryString}`;

            const response = await fetch(url);
            if (!response.ok) throw new Error(`Failed to fetch ${entityType}s`);
            const data : LibraryEntity[] = await response.json();
            returnedEntities = data;
        }
       
        shownEntities = returnedEntities;
        hasMore = returnedEntities.length === 100;
        return returnedEntities;
    }

    const debouncedSearch = debounce(async (searchTerm: string) => {
        try {
            loading = true;
            page = 0;
            shownEntities = [];
            showDropdown = true;
            await searchEntities(searchTerm, null);
        } catch (e) {
            error = e instanceof Error ? e.message : `Failed to load ${entityType}s`;
        } finally {
            loading = false;
        }
    }, 100);

    async function loadMore() {
        if (loadingMore || !hasMore) return;
        
        try {
            loadingMore = true;
            page += 1;
            await searchEntities(searchTerm, null, page.toString());
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

    function handleMouseMove() {
        isMouseActive = true;
    }

    onMount(async () => {
        try {
            if (initialIds.length > 0) {
                await searchEntities(null, initialIds);
            }
            
            await searchEntities(null, null);
        } catch (e) {
            error = e instanceof Error ? e.message : `Failed to load ${entityType}s`;
        } finally {
            loading = false;
        }
    });

    $: if (searchTerm) {
        debouncedSearch(searchTerm);
    }

    function handleSelect(entity: LibraryEntity) {
        // Add to store
        stores[entityType].insertEntity(entity);
        
        onSelect(entity.id);
        searchTerm = '';
        showDropdown = false;
    }

    function handleMouseEnter(index: number) {
        if (isMouseActive) {
            selectedIndex = index;
        }
    }

    function handleKeydown(event: KeyboardEvent) {
        if (!showDropdown || shownEntities.length === 0) return;
        
        isMouseActive = false; // Reset mouse activity on keydown

        switch (event.key) {
            case 'ArrowDown':
                event.preventDefault();
                selectedIndex = (selectedIndex + 1) % shownEntities.length;
                scrollToSelected();
                break;
            case 'ArrowUp':
                event.preventDefault();
                selectedIndex = (selectedIndex - 1 + shownEntities.length) % shownEntities.length;
                scrollToSelected();
                break;
            case 'Enter':
                event.preventDefault();
                if (shownEntities[selectedIndex]) {
                    handleSelect(shownEntities[selectedIndex]);
                }
                break;
            case 'Escape':
                event.preventDefault();
                showDropdown = false;
                break;
        }
    }

    async function scrollToSelected() {
        await tick(); // Wait for the DOM to update
        const dropdown = document.querySelector('.dropdown');
        const selectedItem = dropdown?.querySelector('.dropdown-item.selected');
        selectedItem?.scrollIntoView({ block: 'nearest', inline: 'nearest' });
    }

    $: if (shownEntities) {
        selectedIndex = 0;
    }
</script>

<div class="entity-selector">
    {#if showSelected && storeEntities[entityType].get(+showSelected)}
        <input
        type="text"
        placeholder={storeEntities[entityType].get(+showSelected)?.name}
        bind:value={searchTerm}
        on:focus={() => showDropdown = true}
        class="selected-input"
        />
    {:else}
        <input
        type="text"
        placeholder={placeholder}
        bind:value={searchTerm}
        on:focus={() => showDropdown = true}
        on:keydown={handleKeydown}
        class="unselected-input"
        />
    {/if}
    {#if showDropdown && searchTerm.length > 0}
        <div class="dropdown" on:scroll={handleScroll} on:mousemove={handleMouseMove}>
            {#if loading && !loadingMore}
                <div class="dropdown-item loading">Loading...</div>
            {:else if error}
                <div class="dropdown-item error">{error}</div>
            {:else if shownEntities.length === 0}
                <div class="dropdown-item no-results">No matches found</div>
            {:else}
                {#each shownEntities as entity, i}
                    <button
                        class="dropdown-item"
                        class:selected={selectedIndex === i}
                        on:click={() => handleSelect(entity)}
                        on:mouseenter={() => handleMouseEnter(i)}
                        on:mousemove={() => handleMouseMove()}
                    >
                        <span class="name">{entity.name}</span>
                        {#if entity.level !== undefined}
                            <span class="detail">Lv: {entity.level}</span>
                        {/if}
                        {#if entity.price !== undefined}
                            <span class="detail">{entity.price} gp</span>
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

    .dropdown-item.loading,
    .dropdown-item.error,
    .dropdown-item.no-results {
        color: #666;
        font-style: italic;
    }

    .detail {
        color: #666;
        font-size: 0.875rem;
    }

    .selected-input {
        color: #000;
        font-size: 600000;
    }

    .selected-input::placeholder {
        color: #000;
        font-weight: bolder;
    }

    .unselected-input {
        color: #000;
        font-size: 600000;
    }

    .dropdown-item.selected {
        background: #f0f0f0;
    }
</style> 
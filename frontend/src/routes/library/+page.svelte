<script lang="ts">
    import { onMount } from 'svelte';
    import type { 
        LibraryEntity,
        LibraryEntityType,
        TableColumn,
        LibraryResponse
    } from '$lib/types/library';
    import { fade } from 'svelte/transition';
    import InfiniteScroll from "svelte-infinite-scroll";

    let activeTab: LibraryEntityType = 'class';
    let loading = false;
    let error: string | null = null;
    let searchQuery = '';
    let filterRarity: string = '';
    let filterLevel: string = '';
    let page = 0;
    const LIMIT = 100;

    console.log("library page");
    // Data stores
    let entities: LibraryEntity[] = [];
    let hasMore = true;
    let total = 0;

    // Column definitions for each entity type
    const columns: Record<LibraryEntityType, TableColumn[]> = {
        class: [
            { key: 'name', label: 'Name' },
            { key: 'rarity', label: 'Rarity' },
            { key: 'hp', label: 'HP' },
            { key: 'traditions', label: 'Traditions', formatter: (traditions: string[]) => traditions.join(', ') },
        ],
        spell: [
            { key: 'name', label: 'Name' },
            { key: 'rarity', label: 'Rarity' },
            { key: 'rank', label: 'Level' },
            { key: 'traditions', label: 'Traditions', formatter: (traditions: string[]) => traditions.join(', ') },
        ],
        creature: [
            { key: 'name', label: 'Name' },
            { key: 'rarity', label: 'Rarity' },
            { key: 'level', label: 'Level' },
            { key: 'size', label: 'Size' },
            { key: 'alignment', label: 'Alignment' }
        ],
        hazard: [
            { key: 'name', label: 'Name' },
            { key: 'rarity', label: 'Rarity' },
            { key: 'level', label: 'Level' },
        ],
        item: [
            { key: 'name', label: 'Name' },
            { key: 'rarity', label: 'Rarity' },
            { key: 'level', label: 'Level' },
            { key: 'price', label: 'Price', formatter: (price: any) => 
                price ? `${price.gold || 0}g ${price.silver || 0}s ${price.copper || 0}c` : ''
            },
            { key: 'category', label: 'Category' },
            { key: 'bulk', label: 'Bulk' }
        ]
    };

    const pluralizations: Record<LibraryEntityType, string> = {
        class: 'classes',
        spell: 'spells',
        creature: 'creatures',
        hazard: 'hazards',
        item: 'items'
    };

    async function fetchLibraryData(reset: boolean = false) {
        console.log("fetching library data");
        if (reset) {
            page = 0;
            entities = [];
            hasMore = true;
        }

        if (!hasMore || loading) return;

        loading = true;
        try {
            console.log("fetching library data2", page);
            const params = new URLSearchParams({
                page: page.toString(),
                limit: LIMIT.toString(),
                ...(searchQuery && { search: searchQuery }),
                ...(filterRarity && { rarity: filterRarity }),
                ...(filterLevel && { level: filterLevel })
            });

            const pluralType = pluralizations[activeTab];
            const response = await fetch(`/api/library/${pluralType}?${params}`);
            
            if (!response.ok) throw new Error(`Failed to fetch ${pluralType}`);
            
            const data: LibraryEntity[] = await response.json();
            console.log(data);
            if (reset) {
                entities = data;
            } else {
                entities = [...entities, ...data];
            }
            
            hasMore = data.length > 0;
            total = data.length;
            page++;
            
        } catch (e) {
            error = e instanceof Error ? e.message : 'Failed to load library data';
        } finally {
            loading = false;
        }
    }

    // Reset and refetch when filters change
    $: {
        if (searchQuery !== undefined || filterRarity !== undefined || filterLevel !== undefined) {
            fetchLibraryData(true);
        }
    }

    // Reset and refetch when tab changes
    $: {
        if (activeTab) {
            fetchLibraryData(true);
        }
    }

    onMount(() => fetchLibraryData(true));
</script>

<div class="library-page">
    <h1>Game Library</h1>

    {#if error}
        <div class="error" transition:fade>{error}</div>
    {/if}

    <div class="controls">
        <div class="tabs">
            {#each Object.keys(columns) as tab}
                <button 
                    class="tab-button" 
                    class:active={activeTab === tab}
                    on:click={() => activeTab = tab as LibraryEntityType}
                >
                    {tab.charAt(0).toUpperCase() + tab.slice(1)}s
                </button>
            {/each}
        </div>

        <div class="filters">
            <input
                type="text"
                placeholder="Search..."
                bind:value={searchQuery}
                class="search-input"
            />

            <select bind:value={filterRarity} class="filter-select">
                <option value="">All Rarities</option>
                <option value="common">Common</option>
                <option value="uncommon">Uncommon</option>
                <option value="rare">Rare</option>
                <option value="unique">Unique</option>
            </select>

            <select bind:value={filterLevel} class="filter-select">
                <option value="">All Levels</option>
                {#each Array(20) as _, i}
                    <option value={i + 1}>{i + 1}</option>
                {/each}
            </select>
        </div>

    </div>

    <div class="table-container">
        <table>
            <thead>
                <tr>
                    {#each columns[activeTab] as column}
                        <th>{column.label}</th>
                    {/each}
                </tr>
            </thead>
            <tbody>
                {#each entities as entity (entity.id)}
                    <tr>
                        {#each columns[activeTab] as column}
                            <td>
                                {#if column.formatter}
                                    {@html column.formatter(entity[column.key as keyof typeof entity])}
                                {:else}
                                    {entity[column.key as keyof typeof entity]}
                                {/if}
                            </td>
                        {/each}
                    </tr>
                {/each}
            </tbody>
        </table>

        <InfiniteScroll
            hasMore={hasMore}
            threshold={100}
            on:loadMore={() => {page++;fetchLibraryData()}}
        />
    </div>

    {#if loading}
        <div class="loading">Loading more items...</div>
    {/if}
</div>

<style>
    .library-page {
        padding: 2rem;
        max-width: 1400px;
        margin: 0 auto;
    }

    .controls {
        margin-bottom: 2rem;
    }

    .tabs {
        display: flex;
        gap: 0.5rem;
        margin-bottom: 1rem;
        border-bottom: 2px solid #e5e7eb;
        padding-bottom: 0.5rem;
    }

    .tab-button {
        padding: 0.5rem 1rem;
        border: none;
        background: none;
        cursor: pointer;
        font-size: 1rem;
        color: #6b7280;
        border-radius: 0.375rem;
        transition: all 0.2s;
    }

    .tab-button:hover {
        background: #f3f4f6;
        color: #111827;
    }

    .tab-button.active {
        background: #3b82f6;
        color: white;
    }

    .filters {
        display: flex;
        gap: 1rem;
        margin-bottom: 1rem;
    }

    .search-input {
        flex: 1;
        padding: 0.5rem;
        border: 1px solid #d1d5db;
        border-radius: 0.375rem;
        font-size: 0.875rem;
    }

    .filter-select {
        padding: 0.5rem;
        border: 1px solid #d1d5db;
        border-radius: 0.375rem;
        font-size: 0.875rem;
        min-width: 150px;
    }

    .table-container {
        background: white;
        border-radius: 0.5rem;
        box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
        overflow-x: auto;
        max-height: 70vh;
    }

    table {
        width: 100%;
        border-collapse: collapse;
        font-size: 0.875rem;
    }

    th, td {
        padding: 0.75rem 1rem;
        text-align: left;
        border-bottom: 1px solid #e5e7eb;
    }

    th {
        background: #f9fafb;
        font-weight: 600;
        color: #374151;
        position: sticky;
        top: 0;
        z-index: 10;
    }

    th.sortable {
        cursor: pointer;
        user-select: none;
    }

    th.sortable:hover {
        background: #f3f4f6;
    }

    .sort-indicator {
        display: inline-block;
        margin-left: 0.25rem;
        transition: transform 0.2s;
    }

    th.sorted.desc .sort-indicator {
        transform: rotate(180deg);
    }

    tr:hover {
        background: #f9fafb;
    }

    .error {
        background: #fee2e2;
        color: #ef4444;
        padding: 1rem;
        border-radius: 0.375rem;
        margin-bottom: 1rem;
    }

    .loading {
        text-align: center;
        color: #6b7280;
        padding: 2rem;
    }

    @media (max-width: 768px) {
        .filters {
            flex-direction: column;
        }

        .filter-select {
            width: 100%;
        }
    }

    .results-count {
        color: #6b7280;
        font-size: 0.875rem;
        margin-top: 0.5rem;
    }

</style> 
<script lang="ts">
    import { onMount } from 'svelte';
    import type { 
        LibraryEntity,
        LibraryEntityType,
        TableColumn,
        LibraryResponse,
        getFullUrl
    } from '$lib/types/library';
    import { fade, slide } from 'svelte/transition';
    import InfiniteScroll from "svelte-infinite-scroll";

    let activeTab: LibraryEntityType = 'class';
    let loading = false;
    let error: string | null = null;
    let searchQuery = '';
    let filterRarity: string = '';
    let filterLevel: string = '';
    let page = 0;
    const LIMIT = 100;

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

    // Add state for expanded row and preview
    let expandedRow: number | null = null;
    let previewEntity: LibraryEntity | null = null;
    let previewPosition = { x: 0, y: 0 };

    // Modify encounter state handling
    let currentEncounter: any | null = null;
    let isEncounterMode = false;

    export let data: { activeEncounter: boolean | null, startTab: string | null };

    onMount(async () => {
        await fetchLibraryData(true);
        
        // If we have an active encounter ID from URL, activate encounter mode
        if (data.activeEncounter) {
            await loadEncounter();
            isEncounterMode = true;
        } else {
            // Still load the encounter but don't activate mode
            await loadEncounter();
        }

        if (data.startTab) {
            activeTab = data.startTab as LibraryEntityType;
        }
    });

    async function loadEncounter() {
        try {
            const response = await fetch(`/api/encounters/draft`);
            if (response.ok) {
                currentEncounter = await response.json();
            }
        } catch (e) {
            console.error('Failed to load encounter');
        }
    }

    function activateEncounterMode() {
        isEncounterMode = true;
    }

    function exitEncounterMode() {
        isEncounterMode = false;
    }

    function handleRowClick(entity: LibraryEntity) {
        if (expandedRow === entity.id) {
            expandedRow = null;
        } else {
            expandedRow = entity.id;
        }
    }

    function handleMouseMove(event: MouseEvent, entity: LibraryEntity) {
        previewEntity = entity;
        previewPosition = {
            x: event.clientX + 20,
            y: event.clientY + 20
        };
    }

    function handleMouseLeave() {
        previewEntity = null;
    }

    async function addToEncounter(entity: LibraryEntity, type: 'enemy' | 'hazard' | 'treasure') {
        if (!currentEncounter) {
            error = 'No encounter in progress';
            return;
        }

        try {
            // Update the draft with the new entity
            const updatedDraft = { ...currentEncounter };
            
            switch (type) {
                case 'enemy':
                    updatedDraft.enemies = [...updatedDraft.enemies, entity.id];
                    break;
                case 'hazard':
                    updatedDraft.hazards = [...updatedDraft.hazards, entity.id];
                    break;
                case 'treasure':
                    updatedDraft.treasure_items = [...updatedDraft.treasure_items, entity.id];
                    break;
            }

            const response = await fetch(`/api/encounters/draft`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify(updatedDraft)
            });

            if (!response.ok) throw new Error(`Failed to add ${type} to encounter`);
            
            const responseGet = await fetch(`/api/encounters/draft`);
            currentEncounter = await responseGet.json();
            
            // Show success message
            // TODO: Add toast notification
            console.log(`Added ${entity.name} to encounter as ${type}`);
        } catch (e) {
            console.error(e);
            error = e instanceof Error ? e.message : `Failed to add to encounter`;
        }
    }

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
</script>

<div class="library-page">
    <div class="header">
        <h1>Game Library</h1>
        {#if !isEncounterMode && currentEncounter}
            <button 
                class="activate-mode-button"
                on:click={activateEncounterMode}
            >
                Start Adding to Encounter
            </button>
        {:else if isEncounterMode}
            <div class="mode-indicator">
                <span class="mode-badge">Encounter Mode</span>
                <span class="encounter-name">{currentEncounter?.name || 'Unnamed Encounter'}</span>
                <button class="exit-mode" on:click={exitEncounterMode}>Exit</button>
            </div>
        {/if}
    </div>

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
                    <th></th> <!-- Column for expand/collapse -->
                    {#each columns[activeTab] as column}
                        <th>{column.label}</th>
                    {/each}
                    {#if isEncounterMode && (activeTab === 'creature' || activeTab === 'hazard' || activeTab === 'item')}
                        <th>Actions</th>
                    {/if}
                </tr>
            </thead>
            <tbody>
                {#each entities as entity (entity.id)}
                    <tr 
                        class:expanded={expandedRow === entity.id}
                        on:mouseenter={(e) => handleMouseMove(e, entity)}
                        on:mouseleave={handleMouseLeave}
                    >
                        <td>
                            <button 
                                class="expand-button"
                                on:click={() => handleRowClick(entity)}
                            >
                                {expandedRow === entity.id ? '−' : '+'}
                            </button>
                        </td>
                        {#each columns[activeTab] as column}
                            <td>
                                {#if column.formatter}
                                    {@html column.formatter(entity[column.key as keyof typeof entity])}
                                {:else}
                                    {entity[column.key as keyof typeof entity]}
                                {/if}
                            </td>
                        {/each}
                        {#if isEncounterMode && (activeTab === 'creature' || activeTab === 'hazard' || activeTab === 'item')}
                            <td class="actions">
                                <button 
                                    class="add-button"
                                    on:click={() => addToEncounter(
                                        entity,
                                        activeTab === 'creature' ? 'enemy' :
                                        activeTab === 'hazard' ? 'hazard' : 'treasure'
                                    )}
                                >
                                    Add to Encounter
                                </button>
                            </td>
                        {/if}
                    </tr>
                    {#if expandedRow === entity.id}
                        <tr class="detail-row" transition:slide>
                            <td colspan={columns[activeTab].length + 2}>
                                <div class="entity-details">
                                    {#if entity.description}
                                        <div class="description">
                                            <h4>Description</h4>
                                            <p>{entity.description}</p>
                                        </div>
                                    {/if}
                                    {#if entity.url}
                                        <div class="external-link">
                                            <a href={getFullUrl(entity.url)} target="_blank" rel="noopener noreferrer">
                                                View More Details ↗
                                            </a>
                                        </div>
                                    {/if}
                                </div>
                            </td>
                        </tr>
                    {/if}
                {/each}
            </tbody>
        </table>

        <InfiniteScroll
            hasMore={hasMore}
            threshold={100}
            on:loadMore={() => {page++;fetchLibraryData()}}
        />
    </div>

    {#if previewEntity && !expandedRow}
        <div 
            class="preview-card"
            style="left: {previewPosition.x}px; top: {previewPosition.y}px"
            transition:fade
        >
            <h3>{previewEntity.name}</h3>
            {#if previewEntity.description}
                <p>{previewEntity.description.substring(0, 200)}...</p>
            {/if}
        </div>
    {/if}

    {#if loading}
        <div class="loading">Loading more items...</div>
    {/if}

    {#if isEncounterMode}
        <div class="encounter-indicator" transition:fade>
            <span class="mode-badge">Editing: {currentEncounter?.name || 'Unnamed Encounter'}</span>
            <a href="/encounters" class="view-encounter">View Encounter</a>
            <button class="exit-mode" on:click={exitEncounterMode}>Exit</button>
        </div>
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

    .expand-button {
        background: none;
        border: none;
        cursor: pointer;
        font-size: 1.2rem;
        color: #6b7280;
        padding: 0.25rem 0.5rem;
        border-radius: 0.25rem;
    }

    .expand-button:hover {
        background: #f3f4f6;
    }

    tr.expanded {
        background: #f8fafc;
    }

    .detail-row {
        background: #f8fafc;
    }

    .entity-details {
        padding: 1rem;
        display: grid;
        gap: 1rem;
    }

    .description h4 {
        margin: 0 0 0.5rem 0;
        color: #374151;
    }

    .description p {
        color: #4b5563;
        line-height: 1.5;
    }

    .external-link a {
        color: #3b82f6;
        text-decoration: none;
        display: inline-flex;
        align-items: center;
        gap: 0.25rem;
    }

    .external-link a:hover {
        text-decoration: underline;
    }

    .preview-card {
        position: fixed;
        background: white;
        padding: 1rem;
        border-radius: 0.5rem;
        box-shadow: 0 4px 6px -1px rgb(0 0 0 / 0.1), 0 2px 4px -2px rgb(0 0 0 / 0.1);
        max-width: 300px;
        z-index: 50;
    }

    .preview-card h3 {
        margin: 0 0 0.5rem 0;
        color: #111827;
    }

    .preview-card p {
        margin: 0;
        color: #4b5563;
        font-size: 0.875rem;
        line-height: 1.4;
    }

    .actions {
        display: flex;
        gap: 0.5rem;
    }

    .add-button {
        background: #3b82f6;
        color: white;
        border: none;
        padding: 0.5rem 1rem;
        border-radius: 0.375rem;
        font-size: 0.875rem;
        cursor: pointer;
        transition: background-color 0.2s;
        white-space: nowrap;
    }

    .add-button:hover {
        background: #2563eb;
    }

    .header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 2rem;
    }

    .mode-indicator {
        display: flex;
        align-items: center;
        gap: 1rem;
    }

    .mode-badge {
        background: #3b82f6;
        color: white;
        padding: 0.25rem 0.5rem;
        border-radius: 0.375rem;
        font-size: 0.875rem;
        font-weight: 500;
    }

    .encounter-name {
        color: #374151;
        font-weight: 500;
    }

    .start-draft {
        background: #3b82f6;
        color: white;
        border: none;
        padding: 0.5rem 1rem;
        border-radius: 0.375rem;
        cursor: pointer;
        font-size: 0.875rem;
        transition: background-color 0.2s;
    }

    .start-draft:hover {
        background: #2563eb;
    }

    .exit-mode {
        background: #6b7280;
        color: white;
        border: none;
        padding: 0.25rem 0.75rem;
        border-radius: 0.375rem;
        cursor: pointer;
        font-size: 0.875rem;
        transition: background-color 0.2s;
    }

    .exit-mode:hover {
        background: #4b5563;
    }

    .encounter-indicator {
        position: fixed;
        bottom: 1rem;
        right: 1rem;
        background: white;
        padding: 0.75rem 1rem;
        border-radius: 0.5rem;
        box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        display: flex;
        align-items: center;
        gap: 0.75rem;
        z-index: 50;
    }

    .view-encounter {
        color: #3b82f6;
        text-decoration: none;
        font-size: 0.875rem;
    }

    .view-encounter:hover {
        text-decoration: underline;
    }

    .activate-mode-button {
        background: #3b82f6;
        color: white;
        border: none;
        padding: 0.5rem 1rem;
        border-radius: 0.375rem;
        cursor: pointer;
        font-size: 0.875rem;
        transition: background-color 0.2s;
    }

    .activate-mode-button:hover {
        background: #2563eb;
    }
</style> 
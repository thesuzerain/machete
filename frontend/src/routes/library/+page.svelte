<script lang="ts">
    import { onMount } from 'svelte';
    import { 
        type LibraryEntity,
        type LibraryEntityType,
        type TableColumn,
        type LibraryResponse,
        

        formatCurrency,

        getFullUrl


    } from '$lib/types/library';
    import { fade, slide } from 'svelte/transition';
    import InfiniteScroll from "svelte-infinite-scroll";
    import { getExperienceFromLevel } from '$lib/utils/encounter';
    import { API_URL } from '$lib/config';

    export let data: { activeEncounter: boolean | null, startTab: string | null };

    let activeTab: LibraryEntityType = 'class';
    if (data.startTab) {
        activeTab = data.startTab as LibraryEntityType;
    }
    let loading = false;
    let error: string | null = null;
    let searchQuery = '';
    let filterRarity: string = '';

    let minLevel = -1;
    let maxLevel = 30;
    let minMinLevel: number = -1;
    let maxMaxLevel: number = 30;

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
            { key: 'price', label: 'Price', formatter: (price: any) =>  formatCurrency(price)
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

    // Add state for notification
    let notification: string | null = null;

    let lockToCommonRange = false;

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
    });

    async function loadEncounter() {
        try {
            const response = await fetch(`${API_URL}/encounters/draft`, {
                credentials: 'include'
            });
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

            const response = await fetch(`${API_URL}/encounters/draft`, {
                method: 'POST',
                credentials: 'include',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify(updatedDraft)
            });

            if (!response.ok) throw new Error(`Failed to add ${type} to encounter`);
            
            const responseGet = await fetch(`${API_URL}/encounters/draft`, {
                credentials: 'include'
            });
            currentEncounter = await responseGet.json();
            
            // Show success message
            notification = `Added ${entity.name} to encounter as ${type}`;
            setTimeout(() => notification = null, 3000); // Clear notification after 3 seconds
        } catch (e) {
            console.error(e);
            error = e instanceof Error ? e.message : `Failed to add to encounter`;
        }
    }

    async function fetchLibraryData(reset: boolean = false) {
        if (reset) {
            page = 0;
            entities = [];
            hasMore = true;
        }

        if (!hasMore || loading) return;

        loading = true;
        try {
            const params = new URLSearchParams({
                page: page.toString(),
                limit: LIMIT.toString(),
                ...(searchQuery && { name: searchQuery }),
                ...(filterRarity && { rarity: filterRarity }),
                ...(minLevel && { min_level: minLevel }),
                ...(maxLevel && { max_level: maxLevel })
            });

            const pluralType = pluralizations[activeTab];
            const response = await fetch(`${API_URL}/library/${pluralType}?${params}`);
            
            if (!response.ok) throw new Error(`Failed to fetch ${pluralType}`);
            
            const data: LibraryEntity[] = await response.json();
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
        if (searchQuery !== undefined || filterRarity !== undefined || minLevel !== undefined || maxLevel !== undefined) {
            fetchLibraryData(true);
        }
    }

    // Reset and refetch when tab changes
    $: {
        if (activeTab) {
            fetchLibraryData(true);
        }
    }

    $: {
        if (lockToCommonRange && currentEncounter) {
            minMinLevel = Math.max(currentEncounter.party_level - 4, -1);
            maxMaxLevel = currentEncounter.party_level + 3;

            minLevel = Math.max(minLevel || minMinLevel, minMinLevel);
            maxLevel = Math.min(maxLevel || maxMaxLevel, maxMaxLevel);
        } else {
            minMinLevel = -1;
            maxMaxLevel = 100;
        }
    }

    function toggleCommonRange() {
        lockToCommonRange = !lockToCommonRange;
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
                Start adding to current encounter
            </button>
        {:else if isEncounterMode}
            <div class="mode-indicator-container">
                <div class="mode-indicator">
                    <span class="mode-badge">Encounter Mode</span>
                    <span class="encounter-name">{currentEncounter?.name || 'Unnamed Encounter'}</span>
                    <button class="exit-mode" on:click={exitEncounterMode}>Exit</button>
                </div>
                <div class="toggle-range-container">
                    <label class="toggle-range-label">
                        <input type="checkbox" bind:checked={lockToCommonRange} />
                        Lock to Common Range
                    </label>
                </div>
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

            <select bind:value={minLevel} class="filter-select">
                <option value=-2>Min Level</option>
                {#each Array(33) as _, i}
                    {#if i-3 + 1 >= minMinLevel && i-3 + 1 <= maxMaxLevel}
                        <option value={i-3 + 1}>{i-3 + 1}</option>
                    {/if}
                {/each}
            </select>

            <select bind:value={maxLevel} class="filter-select">
                <option value=-2>Max Level</option>
                {#each Array(33) as _, i}
                    {#if i-3 + 1 >= minMinLevel && i-2 + 1 <= maxMaxLevel}
                        <option value={i-2 + 1}>{i-2 + 1}</option>
                    {/if}
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
                    {#if isEncounterMode && activeTab === 'creature'} <!-- Conditional rendering for Experience column -->
                        <th>Experience</th>
                    {/if}
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
                        on:click={() => handleRowClick(entity)}
                    >
                        <td>
                            <button 
                                class="expand-button"
                                on:click={(e) => { e.stopPropagation(); handleRowClick(entity); }}
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
                        {#if isEncounterMode && activeTab === 'creature'}
                            <td>
                                {getExperienceFromLevel(currentEncounter.party_level, entity.level)}
                            </td>
                        {/if}
                        {#if isEncounterMode && (activeTab === 'creature' || activeTab === 'hazard' || activeTab === 'item')}
                            <td class="actions">
                                <button 
                                    class="add-button"
                                    on:click={(e) => { e.stopPropagation(); addToEncounter(
                                        entity,
                                        activeTab === 'creature' ? 'enemy' :
                                        activeTab === 'hazard' ? 'hazard' : 'treasure'
                                    ); }}
                                >
                                    Add to Encounter
                                </button>
                            </td>
                        {/if}
                    </tr>
                    {#if expandedRow === entity.id}
                        <tr class="detail-row" transition:slide>
                            <td colspan={columns[activeTab].length + (isEncounterMode && activeTab === 'creature' ? 1 : 0) + 2}>
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
            <a href="/encounters" class="view-encounter"><span class="mode-badge">Editing: {currentEncounter?.name || 'Unnamed Encounter'}</span></a>
            <button class="exit-mode" on:click={exitEncounterMode}>Exit</button>
        </div>
    {/if}

    {#if notification}
        <div class="notification" transition:fade>{notification}</div>
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

    .mode-indicator-container {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
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

    .notification {
        position: fixed;
        top: 1rem;
        right: 1rem;
        background: #34d399; /* Green background for success */
        color: white;
        padding: 0.75rem 1rem;
        border-radius: 0.5rem;
        box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        z-index: 100;
    }

    .toggle-range-container {
        margin-top: 0.5rem;
    }

    .toggle-range-label {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        font-size: 0.875rem;
        color: #374151;
    }

    .toggle-range-label input {
        cursor: pointer;
    }

    .table-container tr:nth-child(even) {
        background-color: #f9fafb; /* Light background for even rows */
    }

    .table-container tr:nth-child(odd) {
        background-color: white; /* Default background for odd rows */
    }
</style> 
<script lang="ts">
    import { onMount } from 'svelte';
    import { 
        type LibraryEntity,
        type LibraryEntityType,
        type TableColumn,
        type LibraryResponse,
        formatCurrency,
        getFullUrl,
        type Rune,
        formatAlignment,
        type LibraryCreature,
        type LibraryHazard,
        type LibraryItem
    } from '$lib/types/library';
    import { fade, slide } from 'svelte/transition';
    import InfiniteScroll from "svelte-infinite-scroll";
    import { getCreatureExperienceFromLevel } from '$lib/utils/encounter';
    import { API_URL } from '$lib/config';
    import { goto } from '$app/navigation';
    import type { CreateOrReplaceEncounter } from '$lib/types/encounters';
    import { creatureStore, hazardStore, itemStore } from '$lib/stores/libraryStore';
    import Button from '../core/Button.svelte';
    import { notificationStore } from '$lib/stores/notifications';
    import MultiSelect from '../selectors/MultiSelect.svelte';
    import { libraryTagsStore } from '$lib/stores/libraryTags';
    import Card from '../core/Card.svelte';
    import { SolanaTokenListResolutionStrategy } from '@solana/spl-token-registry';
    import { SvelteMap } from 'svelte/reactivity';
    import DropdownButton from '../core/DropdownButton.svelte';

    export let allowedTabs: LibraryEntityType[] = ['class', 'spell', 'creature', 'hazard', 'item'];
    export let activeTab: LibraryEntityType  = allowedTabs[0];
    export let editingEncounter: CreateOrReplaceEncounter | null = null;

    $: libraryTraits = $libraryTagsStore[activeTab]?.traits || [];

    let loading = false;
    let error: string | null = null;
    let searchQuery = '';
    let filterRarity: string = '';
    let filterLegacy = 'remaster';

    let searchTraits: string[] = [];
    let searchRequiresAll = false;
    
    let minLevel = -1;
    let maxLevel = 30;
    let minMinLevel: number = -1;
    let maxMaxLevel: number = 30;

    let sortBy : SvelteMap<LibraryEntityType, string> = new SvelteMap();

    let orderBy : SvelteMap<LibraryEntityType, 'asc' | 'desc' > = new SvelteMap();

    let page = 0;
    const LIMIT = 100;

    // Data stores
    let entities: LibraryEntity[] = [];
    let hasMore = true;
    let total = 0;

    // Column definitions for each entity type
    const columns: Record<LibraryEntityType, TableColumn[]> = {
        class: [
            { key: 'name', label: 'Name', sortable: true },
            { key: 'rarity', label: 'Rarity', sortable: true },
            { key: 'hp', label: 'HP', sortable: true },
            { key: 'traditions', label: 'Traditions', formatter: (traditions: string[]) => traditions.join(', ') },
        ],
        spell: [
            { key: 'name', label: 'Name', sortable: true },
            { key: 'rarity', label: 'Rarity', sortable: true },
            { key: 'rank', label: 'Rank', sortable: true },
            { key: 'traditions', label: 'Traditions', formatter: (traditions: string[]) => traditions.join(', ') },
            { key: 'legacy', label: 'Legacy', formatter: booleanFormatter },
        ],
        creature: [
            { key: 'name', label: 'Name', sortable: true },
            { key: 'rarity', label: 'Rarity', sortable: true},
            { key: 'level', label: 'Level', sortable: true },
            { key: 'size', label: 'Size', sortable: true},
            { key: 'traits', label: 'Traits', formatter: (traits: string[]) => traits.join(', ') },
            { key: 'alignment', label: 'Alignment', sortable: true },
            { key: 'legacy', label: 'Legacy', formatter: booleanFormatter },
        ],
        hazard: [
            { key: 'name', label: 'Name', sortable: true },
            { key: 'rarity', label: 'Rarity', sortable: true },
            { key: 'level', label: 'Level', sortable: true },
            { key: 'complex', label: 'Complex', formatter: booleanFormatter },
            { key: 'haunt', label: 'Haunt', formatter: booleanFormatter },
            { key: 'legacy', label: 'Legacy', formatter: booleanFormatter },
        ],
        item: [
            { key: 'name', label: 'Name', sortable: true },
            { key: 'rarity', label: 'Rarity', sortable: true },
            { key: 'level', label: 'Level', sortable: true },
            { key: 'price', label: 'Price', sortable: true, formatter: (price: any) => formatCurrency(price) },
            { key: 'item_categories', label: 'Categories', formatter: (categories: string[]) => categories.join(', ') },
            { key: 'item_type', label: 'Type' },
            { key: 'traits', label: 'Traits', formatter: (traits: string[]) => traits.join(', ') },
            { key: 'runes', label: 'Runes', formatter: runeFormatter },
            { key: 'consumable', label: 'Consumable', formatter: booleanFormatter },
            { key: 'magical', label: 'Magical', formatter: booleanFormatter },
            { key: 'legacy', label: 'Legacy', formatter: booleanFormatter },
        ]
    };

    const defaultColumns: Record<LibraryEntityType, string[]> = {
        class: ['name', 'rarity', 'hp', 'traditions'],
        spell: ['name', 'rarity', 'rank', 'traditions'],
        creature: ['name', 'rarity', 'level', 'traits', 'alignment'],
        hazard: ['name', 'rarity', 'level', 'complex'],
        item: ['name', 'rarity', 'level', 'price', 'item_type', 'traits', 'runes', 'magical']
    };

    const ignoredColumns: Record<LibraryEntityType, string[]> = {
        class: ['id', 'url', 'description', 'remastering_alt_id'],
        spell: ['id', 'url', 'description', 'remastering_alt_id'],
        creature: ['id', 'url', 'description', 'remastering_alt_id'],
        hazard: ['id', 'url', 'description', 'remastering_alt_id'],
        item: ['id', 'url', 'description', 'remastering_alt_id']
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
    // TODO: Simplify this- don't need the variable 'isEncounterMode' if we can just use the editingEncounter variable
    $: isEncounterMode = editingEncounter;

    if (editingEncounter) {
            minMinLevel = Math.max(editingEncounter.party_level - 4, -1);
            maxMaxLevel = editingEncounter.party_level + 3;

            minLevel = Math.max(minLevel || minMinLevel, minMinLevel);
            maxLevel = Math.min(maxLevel || maxMaxLevel, maxMaxLevel);
        } else {
            minMinLevel = -3;
            maxMaxLevel = 30;
    }


    // Add these near the top with other state variables
    let showFilterDetails = false;
    let visibleColumns: Record<LibraryEntityType, Set<string>> = {
        class: new Set(defaultColumns.class),
        spell: new Set(defaultColumns.spell),
        creature: new Set(defaultColumns.creature),
        hazard: new Set(defaultColumns.hazard),
        item: new Set(defaultColumns.item)
    };

    function toggleColumn(type: LibraryEntityType, columnKey: string) {
        const newSet = new Set(visibleColumns[type]);
        if (newSet.has(columnKey)) {
            newSet.delete(columnKey);
        } else {
            newSet.add(columnKey);
        }
        visibleColumns[type] = newSet;
    }

    // Formatter function for boolean values
    function booleanFormatter(value: boolean): string {
        return value ? '✔️' : ''; // Checkmark for true, blank for false
    }

    // Formatter function for runes 
    function runeFormatter(runes: Rune[]): string {
        return runes.map(rune => {
            let name = rune.property ? rune.property : rune.type;
            return `${name} ${rune.potency}`;
        }).join(', ');
    }

    onMount(async () => {

        await Promise.all([
            fetchLibraryData(true),
            libraryTagsStore.fetch()
        ]);
    });

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

    async function addToEncounter(entity: LibraryEntity, type: LibraryEntityType) {
        // TODO: There may be a better way to do type assignemnts w.r.t LibraryEntity
        // currently: creatureStore.insertEntity(entity as LibraryCreature);
        if (!editingEncounter) return;
        switch (type) {
            case 'creature':
                if (editingEncounter.enemies) editingEncounter.enemies.push({
                    id: entity.id,
                    level_adjustment: 0
                });
                creatureStore.insertEntity(entity as LibraryCreature);
                break;
            case 'hazard':
            if (editingEncounter.hazards) editingEncounter.hazards.push(entity.id);
                hazardStore.insertEntity(entity as LibraryHazard);
                break;
            case 'item':
            if (editingEncounter.treasure_items) editingEncounter.treasure_items.push(entity.id);
                itemStore.insertEntity(entity as LibraryItem);
                break;
            // TODO: Spell
        }
            
        notificationStore.success(`Added ${entity.name} to encounter as ${type}`, 3000);
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
            let params = new URLSearchParams({
                page: page.toString(),
                limit: LIMIT.toString(),
                ...(searchQuery && { name: searchQuery }),
                ...(filterRarity && { rarity: filterRarity }),
                ...(minLevel && { min_level: minLevel.toString() }),
                ...(maxLevel && { max_level: maxLevel.toString() }),
                ...(sortBy.get(activeTab) && { sort_by: sortBy.get(activeTab) }),
                ...(orderBy.get(activeTab) && { order_by: orderBy.get(activeTab) }),
                legacy: filterLegacy
            });
            if (searchTraits.length > 0) {
                if (searchRequiresAll) {
                    for (const trait of searchTraits) {
                        params.append('traits_all', trait);
                    }
                } else {
                    for (const trait of searchTraits) {
                        params.append('traits_any', trait);
                    }
                }
            }


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
        if (searchQuery !== undefined || filterRarity !== undefined || 
            minLevel !== undefined || maxLevel !== undefined || 
            searchTraits.length > 0 ||
            filterLegacy !== undefined) {
            fetchLibraryData(true);
        }
    }

    // Reset and refetch when tab changes
    $: {
        if (activeTab) {
            fetchLibraryData(true);
        }
    }

    function toggleAnyAllTraits() {
        searchRequiresAll = !searchRequiresAll;
        if (searchTraits.length > 0) {
            fetchLibraryData(true);
        }
    }

    function formatDetailLabel(key: string): string {
        return key
            .split('_')
            .map(word => word.charAt(0).toUpperCase() + word.slice(1))
            .join(' ');
    }

</script>

<div class="library-page">
    {#if error}
    <div class="error" transition:fade>{error}</div>
{/if}

    <div class="controls">
        {#if allowedTabs.length > 1}
        <div class="tabs">
            {#each allowedTabs as tab}
                <button 
                    class="tab-button" 
                    class:active={activeTab === tab}
                    on:click={() => activeTab = tab as LibraryEntityType}
                >
                    {tab.charAt(0).toUpperCase() + tab.slice(1)}s
                </button>
            {/each}
        </div>
        {/if}

        <div class="filters">
            <input
                type="text"
                placeholder="Search name..."
                bind:value={searchQuery}
                class="search-input"
            />

            <input type="number" bind:value={minLevel} min={-2} max={maxMaxLevel} placeholder="Min Level" />
            <input type="number" bind:value={maxLevel} min={-2} max={maxMaxLevel} placeholder="Max Level" />

            <DropdownButton outlined colour='grey' label="Rarities">
                <div class="dropdown-content">
                    <Card tight>
                        <label class="rarity-checkbox">
                            <input type="checkbox" bind:group={filterRarity} value="common" />
                        Common
                        </label>
                    </Card>
                    <Card tight>
                        <label class="rarity-checkbox">
                            <input type="checkbox" bind:group={filterRarity} value="uncommon" />
                        Uncommon
                        </label>
                    </Card>
                    <Card tight>
                        <label class="rarity-checkbox">
                            <input type="checkbox" bind:group={filterRarity} value="rare" />
                        Rare
                        </label>
                    </Card>
                    <Card tight>
                        <label class="rarity-checkbox">
                            <input type="checkbox" bind:group={filterRarity} value="unique" />
                        Unique
                        </label>
                    </Card>
                </div>
            </DropdownButton>
            
            <select bind:value={filterLegacy}>
            >
                <option value="remaster">Remastered</option>
                <option value="remaster">Legacy</option>
                <option value="all">All Versions</option>
                <option value="legacy_only">Legacy Only</option>
                <option value="remaster_only">Remastered Only</option>
            </select>
        </div>

        <div class="filters">
            <Button colour='black' onclick={() => showFilterDetails = !showFilterDetails} >
                {showFilterDetails ? 'Hide' : 'Show'} details
            </Button>

            <Button
                colour='black'
                onclick={toggleAnyAllTraits}
            >
                {searchRequiresAll ? 'Require ALL' : 'Require ANY'}
            </Button>
            <MultiSelect
            bind:selected={searchTraits}
            options={libraryTraits}
        />
        </div>

        <div class="filter-details">
                {#if showFilterDetails}
                    <Card>
                        <h4>Toggle Visible Columns</h4>
                        <div class="column-options">
                            {#each columns[activeTab] as column}
                                <label class="column-option">
                                    <input
                                        type="checkbox"
                                        checked={visibleColumns[activeTab].has(column.key)}
                                        on:change={() => toggleColumn(activeTab, column.key)}
                                    />
                                    {column.label}
                                </label>
                            {/each}
                        </div>
                    </Card>
                {/if}
        </div>
        

    </div>

    <div class="table-container">
        <table>
            <thead>
                <tr>
                    <th></th> <!-- Column for expand/collapse -->
                    {#each columns[activeTab] as column}
                        {#if visibleColumns[activeTab].has(column.key)}
                            <th>
                                <span class="column-label-display">
                                {column.label}
                                {#if column.sortable}
                                    <button
                                        class="sort-button"
                                        on:click={() => {
                                            if (sortBy.get(activeTab) === column.key) {
                                                orderBy.set(activeTab,orderBy.get(activeTab) === 'asc' ? 'desc' : 'asc');
                                            } else {
                                                sortBy.set(activeTab,column.key);
                                                orderBy.set(activeTab,'desc');
                                            }
                                            fetchLibraryData(true);
                                        }}
                                    >
                                        {#if sortBy.get(activeTab) === column.key}
                                            {#if orderBy.get(activeTab) === 'asc'}
                                                ↑
                                            {:else}
                                                ↓
                                            {/if}
                                        {:else}
                                            ⇅
                                        {/if}
                                    </button>
                                {/if}
                            </span>

                            </th>
                        {/if}
                    {/each}
                    {#if isEncounterMode && activeTab === 'creature'} <!-- Conditional rendering for Experience column -->
                        <th>Experience</th>
                    {/if}
                    {#if isEncounterMode}
                        <th>Actions</th>
                    {/if}
                </tr>
            </thead>
            <tbody>
                {#if entities.length === 0}
                    <tr>
                        <td colspan={columns[activeTab].length + (isEncounterMode && activeTab === 'creature' ? 1 : 0) + 2}>
                            No results found.
                        </td>
                    </tr>
                {:else}
                    
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
                            {#if visibleColumns[activeTab].has(column.key)}
                                <td>
                                    {#if column.key === 'rarity'}
                                        <span class="rarity-label {entity[column.key as keyof typeof entity]}">
                                            {entity[column.key as keyof typeof entity]}
                                        </span>
                                    {:else if column.formatter}
                                        {@html column.formatter(entity[column.key as keyof typeof entity])}
                                    {:else}
                                        {entity[column.key as keyof typeof entity]}
                                    {/if}
                                </td>
                            {/if}
                        {/each}
                        {#if isEncounterMode && activeTab === 'creature'}
                            <td>
                                {#if editingEncounter && entity.level !== undefined}
                                {getCreatureExperienceFromLevel(editingEncounter.party_level, entity.level)}
                                {/if}
                            </td>
                        {/if}
                        {#if isEncounterMode && (activeTab === 'creature' || activeTab === 'hazard' || activeTab === 'item')}
                            <td class="actions">
                                <Button colour='blue'
                                onclick={() => {addToEncounter(
                                    entity,
                                    activeTab
                                ); }}
                            >
                                Add to Encounter
                            </Button>

                            </td>
                        {/if}
                    </tr>
                    {#if expandedRow === entity.id}
                        <tr class="detail-row" transition:slide>
                            <td colspan={columns[activeTab].length + (isEncounterMode && activeTab === 'creature' ? 1 : 0) + 2}>
                                <div class="entity-details">
                                    {#if entity.description}
                                        <div class="detail-section">
                                            <h4>Description</h4>
                                            <p>{entity.description}</p>
                                        </div>
                                    {/if}

                                    <div class="detail-section detail-grid">
                                        {#each Object.entries(entity) as [key, value]}
                                            {#if !ignoredColumns[activeTab as keyof typeof ignoredColumns]?.includes(key) && value !== undefined && value !== null}
                                                <div class="detail-item">
                                                    <span class="detail-label">{formatDetailLabel(key)}</span>
                                                    <span>
                                                        {#if key === 'alignment'}
                                                            {formatAlignment(value)}
                                                        {:else if key === 'price'}
                                                            {formatCurrency(value)}
                                                        {:else if key === 'rarity'}
                                                            <span class="rarity-label {value}">{value}</span>
                                                        {:else if key === 'runes' && value.length > 0}
                                                                {runeFormatter(value)}
                                                        {:else if Array.isArray(value)}
                                                            {value.join(', ')}
                                                        {:else if typeof value === 'boolean'}
                                                            {value ? '✔️' : '❌'}
                                                        {:else}
                                                            {value}
                                                        {/if}
                                                    </span>
                                                </div>
                                            {/if}
                                        {/each}
                                    </div>

                                    {#if entity.url}
                                        <div class="detail-section">
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
                {/if}

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
</div>

<style>
    .library-page {
        padding: 1rem;
        max-width: 80rem;
        margin: 0 auto;
    }

    .controls {
        margin-bottom: 2rem;
    }

    .tabs {
        display: flex;
        gap: 0.5rem;
        margin-bottom: 1rem;
        border-bottom: 2px solid var(--color-bg-light-raised-border);
        padding-bottom: 0.5rem;
    }

    .tab-button {
        padding: 0.5rem 1rem;
        border: none;
        background: none;
        cursor: pointer;
        font-size: 1rem;
        color: var(--color-text);
        border-radius: 0.375rem;
        transition: all 0.2s;
    }

    .tab-button:hover {
        background: var(  --color-bg-raised);
        color: var(--color-text);
    }

    .tab-button.active {
        background: var( --color-bg-selected);
        color: var(--color-text-light);
    }

    .filters {
        display: flex;
        gap: 1rem;
        margin-bottom: 1rem;
    }


    .filters input[type="number"] {
        max-width: 5rem;
    }

    .filters select {
        max-width: 10rem;
    }

    .search-input {
        flex: 1;
        padding: 0.5rem;
        border: 1px solid var(--color-bg-light-raised-border);

        border-radius: 0.375rem;
        font-size: 0.875rem;
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
        background: var(--color-bg-light-raised);
        font-weight: 600;
        color: var(--color-text);
        position: sticky;
        top: 0;
        z-index: 10;
    }


    .sort-indicator {
        display: inline-block;
        margin-left: 0.25rem;
        transition: transform 0.2s;
    }

    tr:hover {
        background: #f9fafb;
    }

    .error {
        background: #fee2e2;
        color: var(--color-bg-error);
        padding: 1rem;
        border-radius: 0.375rem;
        margin-bottom: 1rem;
    }

    .loading {
        text-align: center;
        color: var(--color-text);
        padding: 2rem;
    }

    .results-count {
        font-size: 0.875rem;
        margin-top: 0.5rem;
    }

    .expand-button {
        background: none;
        border: none;
        cursor: pointer;
        font-size: 1.2rem;
        color: var(--color-text);
        padding: 0.25rem 0.5rem;
        border-radius: 0.25rem;
    }

    .expand-button:hover {
        background: var(--color-bg-raised);
    }

    tr.expanded {
        background: #f8fafc;
    }

    .detail-row {
        background: #f8fafc;
    }

    .entity-details {
        padding: 1.5rem;
        background: white;
        border-radius: 0.5rem;
    }

    .detail-section {
        margin-bottom: 1.5rem;
    }

    .detail-section:last-child {
        margin-bottom: 0;
    }

    .detail-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
        gap: 1rem;
    }

    .detail-item {
        display: flex;
        flex-direction: column;
        gap: 0.25rem;
    }

    .detail-label {
        font-weight: 600;
        font-size: 0.875rem;
    }

    .preview-card {
        position: fixed;
        background: var(--color-bg);
        padding: 1rem;
        border-radius: 0.5rem;
        box-shadow: 0 4px 6px -1px rgb(0 0 0 / 0.1), 0 2px 4px -2px rgb(0 0 0 / 0.1);
        max-width: 300px;
        z-index: 50;
    }

    .preview-card h3 {
        margin: 0 0 0.5rem 0;
    }

    .preview-card p {
        margin: 0;
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

    .table-container tr:nth-child(even) {
        background-color: var(--color-bg-light-raised); /* Light background for even rows */
    }

    .table-container tr:nth-child(odd) {
        background-color: var(--color-bg); /* Default background for odd rows */
    }

    .filter-details {
        display: flex;
    }


    .column-selector {
        background: var(--color-bg);
        border: 1px solid var(--color-bg-light-raised);
        border-radius: 0.375rem;
        padding: 1rem;
        margin-top: 0.5rem;
    }

    .column-selector h4 {
        margin: 0 0 0.75rem 0;
    }

    .column-options {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    .column-option {
        display: flex;
        align-items: flex-start;
        justify-content: left;
        width: fit-content;
        gap: 0.5rem;
        font-size: 0.875rem;
        cursor: pointer;
    }

    .column-option input {
        cursor: pointer;
    }

    .column-label-display {
        display: flex;
        align-items: center;
    }

    .sort-button {
        background: none;
        border: none;
        cursor: pointer;
        font-size: 0.875rem;
        color: var(--color-text);
        padding: 0.25rem 0.5rem;
        border-radius: 0.25rem;
        overflow: hidden;
    }

.rarity-checkbox {
        display: flex;
        cursor: pointer;
        gap: 0.5rem;
        margin-right: 0.5rem;
        margin-left: 0.5rem;
        width: max-content;
    }

    .rarity-label {
        display: inline-block;
        padding: 0.25rem 0.5rem;
        border-radius: 0.375rem;
        font-size: 0.75rem;
        font-weight: 500;
        text-transform: uppercase;
        letter-spacing: 0.05em;
        color: var(--color-text-light);
    }

    .rarity-label.common {
        background-color: var(--color-rarity-common);
    }

    .rarity-label.uncommon {
        background-color: var(--color-rarity-uncommon);
    }

    .rarity-label.rare {
        background-color: var(--color-rarity-rare); 
    }

    .rarity-label.unique {
        background-color: var(--color-rarity-unique); 
    }
</style> 
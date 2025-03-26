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
        formatAlignment
    } from '$lib/types/library';
    import { fade, slide } from 'svelte/transition';
    import InfiniteScroll from "svelte-infinite-scroll";
    import { getCreatureExperienceFromLevel } from '$lib/utils/encounter';
    import { API_URL } from '$lib/config';
    import { goto } from '$app/navigation';
    import type { Encounter } from '$lib/types/encounters';
    import Library from '$lib/components/library/Library.svelte';

    export let data: { activeEncounterId: number | null, startTab: string | null };

    let activeTab: LibraryEntityType = 'class';
    if (data.startTab) {
        activeTab = data.startTab as LibraryEntityType;
    }

    let loading = false;
    let error: string | null = null;
    let searchQuery = '';
    let filterRarity: string = '';
    let filterLegacy = 'remaster'; // default to match Rust enum default

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
            { key: 'legacy', label: 'Legacy', formatter: booleanFormatter },
        ],
        creature: [
            { key: 'name', label: 'Name' },
            { key: 'rarity', label: 'Rarity' },
            { key: 'level', label: 'Level' },
            { key: 'size', label: 'Size' },
            { key: 'alignment', label: 'Alignment' },
            { key: 'legacy', label: 'Legacy', formatter: booleanFormatter },
        ],
        hazard: [
            { key: 'name', label: 'Name' },
            { key: 'rarity', label: 'Rarity' },
            { key: 'level', label: 'Level' },
            { key: 'legacy', label: 'Legacy', formatter: booleanFormatter },
        ],
        item: [
            { key: 'name', label: 'Name' },
            { key: 'rarity', label: 'Rarity' },
            { key: 'level', label: 'Level' },
            { key: 'price', label: 'Price', formatter: (price: any) => formatCurrency(price) },
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
        creature: ['name', 'rarity', 'level', 'size', 'alignment'],
        hazard: ['name', 'rarity', 'level'],
        item: ['name', 'rarity', 'level', 'price', 'item_categories', 'item_type', 'traits', 'runes', 'magical']
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
    let currentEncounter: Encounter | null = null;
    let isEncounterMode = false;

    // Add state for notification
    let notification: string | null = null;

    let lockToCommonRange = false;

    // Add these near the top with other state variables
    let showColumnSelector = false;
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
        await fetchLibraryData(true);
        // If we have an active encounter ID from URL, activate encounter mode
        if (data.activeEncounterId) {
            await loadEncounter(data.activeEncounterId);
            if (currentEncounter) isEncounterMode = true;
        } 
    });

    
    async function loadEncounter(encounterId: number) {
        try {
            const response = await fetch(`${API_URL}/encounters/${encounterId}`, {
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

    function exitEncounterModeReturn() {
        goto(`/encounters?encounterId=${data.activeEncounterId}`);
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
            const updatedDraft = {
                enemies: [...currentEncounter.enemies || []],
                hazards: [...currentEncounter.hazards || []],
                treasure_items: [...currentEncounter.treasure_items]
            };
            switch (type) {
                case 'enemy':
                    updatedDraft.enemies  = [...updatedDraft.enemies, {
                        id: entity.id,
                        // TODO: For now, we just do level adjustment 0 (no elite or weak) when adding from library
                        level_adjustment: 0
                    }];
                    break;
                case 'hazard':
                    updatedDraft.hazards = [...updatedDraft.hazards, entity.id];
                    break;
                case 'treasure':
                    updatedDraft.treasure_items = [...updatedDraft.treasure_items, entity.id];
                    break;
            }

            // TODO: Use store functions
            const response = await fetch(`${API_URL}/encounters/${currentEncounter.id}`, {
                method: 'PATCH',
                credentials: 'include',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify(updatedDraft)
            });

            if (!response.ok) throw new Error(`Failed to add ${type} to encounter`);
            
            const responseGet = await fetch(`${API_URL}/encounters/${currentEncounter.id}`, {
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
                ...(minLevel && { min_level: minLevel.toString() }),
                ...(maxLevel && { max_level: maxLevel.toString() }),
                legacy: filterLegacy
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
        if (searchQuery !== undefined || filterRarity !== undefined || 
            minLevel !== undefined || maxLevel !== undefined || 
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

    <Library activeTab={activeTab} />
</div>

<style>
    .library-page {
        padding: 2rem;
        max-width: 1600px;
        margin: 0 auto;
    }

    .error {
        background: #fee2e2;
        color: #ef4444;
        padding: 1rem;
        border-radius: 0.375rem;
        margin-bottom: 1rem;
    }
</style> 
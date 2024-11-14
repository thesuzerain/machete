<script lang="ts">
    import type { LibraryEntity } from '$lib/types/library';
    
    export let entityType: 'creature' | 'hazard' | 'item';
    export let entityId: number;

    let entity: LibraryEntity | null = null;
    let loading = true;
    let error: string | null = null;

    async function loadEntity() {
        try {
            const response = await fetch(`/api/library/${entityType}s/${entityId}`);
            if (!response.ok) throw new Error('Failed to load entity');
            entity = await response.json();
        } catch (e) {
            error = e instanceof Error ? e.message : 'Failed to load entity';
        } finally {
            loading = false;
        }
    }

    $: if (entityId) {
        loadEntity();
    }
</script>

{#if loading}
    <span class="loading">Loading...</span>
{:else if error}
    <span class="error" title={error}>Error loading {entityType}</span>
{:else if entity}
<span class="entity-name">{entity.name}</span>
{:else}
    <span class="not-found">Unknown {entityType}</span>
{/if}

<style>
    .loading {
        color: #666;
        font-style: italic;
    }

    .error {
        color: #ef4444;
        cursor: help;
    }

    .not-found {
        color: #666;
    }

    .entity-name {
        font-weight: 500;
    }
</style> 
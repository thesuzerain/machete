<script lang="ts">
    import { creatureStore, hazardStore, itemStore } from '$lib/stores/libraryStore';
  import type { LibraryEntity } from '$lib/types/types';
    
    export let entityType: 'creature' | 'hazard' | 'item';
    export let entityId: number;

    const stores = {
        creature: creatureStore,
        hazard: hazardStore,
        item: itemStore
    };

    const store = stores[entityType];
    
    let unsubscribe: () => void;
    let entity : LibraryEntity | null = null;
    let loading = false;
    let error : string | null = null;

    $: if (entityId) {
        unsubscribe = store.subscribe(state => {
            entity = state.entities.get(entityId) || null;
            loading = state.loading;
            error = state.error;
        });
        
        if (!entity && !loading) {
            store.getEntity(entityId);
        }
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
<script lang="ts">
    import { createEventDispatcher, onMount } from 'svelte';
    import { fade } from 'svelte/transition';
    import Modal from './Modal.svelte';
    import Library from './Library.svelte';
    import LibraryEntityName from './LibraryEntityName.svelte';
    import type { LibraryEntityType } from '$lib/types/library';
    import type { LibraryEntity } from '$lib/types/types';
    import type { CreateOrReplaceEncounter, Encounter } from '$lib/types/encounters';

    interface Props {
        show: boolean;
        allowedTabs: LibraryEntityType[];
        addEntityToEncounter?: (entityType : LibraryEntityType, entity : LibraryEntity) => Promise<void>;
        editingEncounter?: CreateOrReplaceEncounter;
    }

    let { 
        show = $bindable(),
        allowedTabs,
        addEntityToEncounter = $bindable(),
        editingEncounter = $bindable()
    } : Props = $props();
    
    const dispatch = createEventDispatcher();
    let error: string | null = $state(null);

    function closeModal() {
        show = false;
        error = null;
        dispatch('close');
    }
</script>

{#if show}
    <Modal bind:show={show} on:close={closeModal} bind:error={error}>
        <Library allowedTabs={allowedTabs} addEntityToEncounter={addEntityToEncounter} bind:editingEncounter={editingEncounter} />
    </Modal>
{/if}

<style>

</style> 
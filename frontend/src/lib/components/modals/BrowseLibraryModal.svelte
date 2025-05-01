<script lang="ts">
    import { createEventDispatcher, onMount } from 'svelte';
    import { fade } from 'svelte/transition';
    import Modal from '../core/Modal.svelte';
    import Library from '../library/Library.svelte';
    import LibraryEntityName from '../library/LibraryEntityName.svelte';
    import type { LibraryEntityType } from '$lib/types/library';
    import type { LibraryEntity } from '$lib/types/types';
    import type { CreateOrReplaceEncounter, Encounter } from '$lib/types/encounters';

    interface Props {
        show: boolean;
        allowedTabs: LibraryEntityType[];
        editingEncounter?: CreateOrReplaceEncounter;
    }

    let { 
        show = $bindable(),
        allowedTabs,
        editingEncounter = $bindable(),
    } : Props = $props();
    
    export function showWithTabs(tabs: LibraryEntityType[]) {
        allowedTabs = tabs;
        show = true;
    }

    const dispatch = createEventDispatcher();
    let error: string | null = $state(null);

    function closeModal() {
        show = false;
        error = null;
        dispatch('close');
    }
</script>

{#if show}
    <Modal bind:show={show} on:close={closeModal} bind:error={error} closeButton>
        <div slot="header">
        </div>
        <div class="modal-shape">
            <Library allowedTabs={allowedTabs} bind:editingEncounter={editingEncounter} />
        </div>
    </Modal>
{/if}

<style>
.modal-shape {
    width: 130vh;
    height: 100vh;
}
</style> 
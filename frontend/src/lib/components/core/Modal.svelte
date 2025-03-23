<script lang="ts">
    import { createEventDispatcher, onMount } from 'svelte';
    import { fade } from 'svelte/transition';

    interface Props {
        show: boolean;
        error?: string;
    }

    let { 
        show = $bindable(),
        error = $bindable(),
    } : Props = $props();
    
    const dispatch = createEventDispatcher();
    function closeModal() {
        show = false;
        error = undefined;
        dispatch('close');
    }
</script>

{#if show}
    {#if error}
        <div class="error-message" transition:fade>{error}</div>
    {/if}

    <div class="modal-backdrop" on:click={closeModal} transition:fade>
        <div class="modal-content" on:click|stopPropagation>
             <slot />
        </div>
    </div>
{/if}

<style>

    .modal-backdrop {
        position: fixed;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        background: rgba(0, 0, 0, 0.5);
        display: flex;
        align-items: flex-start;
        justify-content: center;
        z-index: 1000;
        padding: 2rem;
        overflow-y: auto;
        
    }

    /*
    Decide what to set width, etc. Some contenders:
    - login modal
    - signup modal
    -new character modal
    - edit campaign/new campaign modal
    - reorder modal
    - library modal
    - encounterviewer modal
    -n ew campaign modal

    Needs to look good with all of them

    */

    .modal-content {
        background: var(--color-bg);
        padding: 1.5rem;
        border-radius: 0.5rem;
        max-width: 95vh;
        max-height: 100vh;
        overflow-y: auto;
        margin: auto;
    }
</style> 
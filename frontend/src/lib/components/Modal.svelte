<script lang="ts">
    import { createEventDispatcher, onMount } from 'svelte';
    import { fade } from 'svelte/transition';

    interface Props {
        show: boolean;
        error: string | null;
    }

    let { 
        show = $bindable(),
        error = $bindable(),
        ...other
    } : Props = $props();

    
    const dispatch = createEventDispatcher();
    function closeModal() {
        show = false;
        error = null;
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

    .modal-content {
        background: white;
        padding: 2rem;
        border-radius: 0.5rem;
        width: 90%;
        max-width: 1000px;
        max-height: 90vh;
        overflow-y: auto;
        margin: auto;
    }
</style> 
<script lang="ts">
    import { createEventDispatcher, onMount } from 'svelte';
    import { fade } from 'svelte/transition';

    interface Props {
        show: boolean;
        closeButton?: boolean;
        error?: string;
    }

    let { 
        show = $bindable(),
        closeButton = false,
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
            <div class="modal-header">
                <slot name="header" />
                {#if closeButton}
                    <button class="close-button" on:click={closeModal}>×</button>
                {/if}
            </div>

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

    .modal-header {
        display: flex;
        align-items: center;
        justify-content: space-between;
    }

    .close-button {
        background: none;
        border: none;
        font-size: 1.5rem;
        cursor: pointer;
        color: #666;
    }

    .modal-content {
        background: var(--color-bg);
        padding: 1.5rem;
        border-radius: 0.5rem;
        max-width: 95vw;
        max-height: 95vh;
        overflow-y: auto;
        margin: auto;
    }
</style> 
<script lang="ts">
    import { createEventDispatcher, onMount } from 'svelte';
    import { fade } from 'svelte/transition';
    import Modal from './Modal.svelte';
    import Card from './Card.svelte';

    interface Props {
        show: boolean;
        error: string | null;
        confirmationString: string | null;
    }

    let { 
        show = $bindable(),
        error = $bindable(),
        confirmationString = $bindable(),
    } : Props = $props();

    
    const dispatch = createEventDispatcher();
    function confirm() {
        show = false;
        error = null;
        dispatch('confirm');
    }
    
    function cancel() {
        show = false;
        error = null;
        dispatch('close');
    }

</script>

<div class="confirmation-modal-content">
<Modal bind:show={show} bind:error={error} on:close={cancel}>

    <div class="modal-header">
        <h2>Confirmation</h2>
    </div>
   <slot />
    <div class="modal-footer">
        <button class="cancel-button" on:click={
            cancel
        }>Cancel</button>
        <button class="confirm-button" on:click={confirm}>{confirmationString ?? "OK"}</button>
    </div>
</Modal>
</div>

<style>
    :global(.card) {
        background-color: red;
    }

    /* TODO: Is this a code smell? Not sure of optimal practice for overwriting. Seems like other options are passing variables or $$restProps */
    .confirmation-modal-content :global(.modal-content) {
        max-width: 50rem;
    }

    .modal-footer {
        display: flex;
        justify-content: space-between;
        gap: 1rem;
    }
    .cancel-button {
        color: white;
        padding: 14px 20px;
        margin: 8px 0;
        border: none;
        cursor: pointer;
        width: 50%;
    }

    .confirm-button {
        background-color: #f44336;
        color: white;
        padding: 14px 20px;
        margin: 8px 0;
        border: none;
        cursor: pointer;
        width: 50%;
    }

</style> 
<script lang="ts">
    import { createEventDispatcher, onMount } from 'svelte';
    import { fade } from 'svelte/transition';
    import Modal from '../core/Modal.svelte';
    import Card from '../core/Card.svelte';
    import Button from '../core/Button.svelte';

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
    <Card>
        <slot />
    </Card>
    <div class="modal-footer">
        <Button large colour='black' onclick={cancel}>Cancel</Button>
        <Button large colour='red' onclick={cancel}>{confirmationString ?? "OK"}}</Button>
    </div>
</Modal>
</div>

<style>
/* TODO: Is this a code smell? Not sure of optimal practice for overwriting. Seems like other options are passing variables or $$restProps */
.confirmation-modal-content :global(.modal-content) {
    max-width: 50rem;
}

.modal-footer {
    display: flex;
    justify-content: center;
    gap: 1rem;
}
</style> 
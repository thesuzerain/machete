<script lang="ts">
    import { createEventDispatcher } from 'svelte';
    import { fade } from 'svelte/transition';
    import { API_URL } from '$lib/config';
    import type { Character } from '$lib/types/types';
    import LibrarySelector from '../selectors/LibrarySelector.svelte';
    import { characterStore } from '$lib/stores/characters';
    import { classStore } from '$lib/stores/libraryStore';
    import Modal from '../core/Modal.svelte';
    import Button from '../core/Button.svelte';

    export let show = false;
    export let campaignId: number | null;
    export let editingCharacter: Character | null = null;

    let classes = $classStore.entities;

    const dispatch = createEventDispatcher();

    let name = '';
    let classId: number | null = null;
    let className = '';
    let error: string | null = null;

    // Reset form when modal opens or editingCharacter changes
    $: if (show) {
        if (editingCharacter) {
            name = editingCharacter.name;
            classId = editingCharacter.class;
            className = editingCharacter.class_name;
        } else {
            name = '';
            classId = null;
            className = '';
        }
    }

    async function handleSubmit() {
        if (!campaignId) {
            error = "No campaign selected";
            return;
        }

        if (!classId) {
            error = "Please select a class";
            return;
        }

        try {
            const characterData = {
                name,
                class: classId,
                campaign_id: campaignId,
            };

            if (editingCharacter) {
                const url = `${API_URL}/campaign/${campaignId}/characters/${editingCharacter.id}`

                const response = await fetch(url, {
                    method: 'PUT',
                    credentials: 'include',
                    headers: {
                        'Content-Type': 'application/json',
                    },
                    body: JSON.stringify(characterData),
                });

                if (!response.ok) throw new Error('Failed to edit character');

                // Get characters
                await characterStore.fetchCharacters(campaignId);

            } else {
                const url = `${API_URL}/campaign/${campaignId}/characters`

                const response = await fetch(url, {
                    method: 'POST',
                    credentials: 'include',
                    headers: {
                        'Content-Type': 'application/json',
                    },
                    body: JSON.stringify([characterData]),
                });

                if (!response.ok) throw new Error('Failed to add character');

                // Get characters
                await characterStore.fetchCharacters(campaignId);
            }
            closeModal();
        } catch (e) {
            error = e instanceof Error ? e.message : 'An error occurred';
        }
    }

    function closeModal() {
        show = false;
        error = null;
        dispatch('close');
    }
</script>


<Modal bind:show={show} bind:error={error} closeButton>
    <div slot="header">
        <h2>{editingCharacter ? 'Edit' : 'New'} Character</h2>

    </div>
    <form on:submit|preventDefault={handleSubmit}>
        <div class="form-group">
            <label for="name">Name</label>
            <input 
                type="text" 
                id="name" 
                bind:value={name}
                required
            />
        </div>


        <div class="form-group inline">
            <label for="class">Class:</label>
            <select bind:value={classId}>
                <option value={0}>Select Class</option>
                {#each Array.from(classes.values()) as classOption}
                    <option value={classOption.id}>{classOption.name}</option>
                {/each}
            </select>

        </div> 

        <div class="modal-actions">
            <Button onclick={closeModal} colour="red">
                Cancel
            </Button>
            <Button submit colour="green">
                {editingCharacter ? 'Save' : 'Create'} Character
            </Button>
        </div> 
    </form>

    </Modal>


<style>
    .form-group {
        margin-bottom: 1rem;
    }

    .form-group label {
        display: block;
        margin-bottom: 0.5rem;
        color: var(--color-text-secondary);
        font-weight: 500;
    }

    .form-group input {
        width: 100%;
        padding: 0.5rem;
        border: 1px solid #e5e7eb;
        border-radius: 0.375rem;
        font-size: 1rem;
    }

    .modal-actions {
        display: flex;
        justify-content: flex-end;
        gap: 1rem;
        margin-top: 2rem;
    }

    .form-group.inline {
        display: flex;
        gap: 0.5rem;
        align-items: center;
    }

</style> 
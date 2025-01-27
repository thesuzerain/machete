<script lang="ts">
    import { createEventDispatcher } from 'svelte';
    import { fade } from 'svelte/transition';
    import { API_URL } from '$lib/config';
    import type { Campaign } from '$lib/types/types';
    import { campaignStore } from '$lib/stores/campaigns';

    export let show = false;
    export let editingCampaign: Campaign | null = null;

    const dispatch = createEventDispatcher();

    let name = '';
    let description = '';
    let error: string | null = null;

    // Reset form when modal opens or editingCampaign changes
    $: if (show) {
        if (editingCampaign) {
            name = editingCampaign.name;
            description = editingCampaign.description || '';
        } else {
            name = '';
            description = '';
        }
    }

    async function handleSubmit() {
        try {
            const campaignData = {
                name,
                description
            };

            const url = editingCampaign 
                ? `${API_URL}/campaign/${editingCampaign.id}`
                : `${API_URL}/campaign`;

            const response = await fetch(url, {
                method: editingCampaign ? 'PUT' : 'POST',
                credentials: 'include',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(campaignData),
            });

            if (!response.ok) throw new Error('Failed to save campaign');

            // Get campaigns
            await campaignStore.fetchCampaigns();
            
            // Get id 
            const data = await response.json();

            // TODO: Switch active modal to use new campaign
            dispatch('saved', data.id);

            closeModal();
        } catch (e) {
            console.error('Error saving campaign:', e);
            error = e instanceof Error ? e.message : 'An error occurred';
        }
    }

    function closeModal() {
        show = false;
        error = null;
        dispatch('close');
    }
</script>

{#if show}
    <div class="modal-backdrop" on:click={closeModal} transition:fade>
        <div class="modal-content" on:click|stopPropagation>
            <h2>{editingCampaign ? 'Edit' : 'New'} Campaign</h2>
            
            {#if error}
                <div class="error-message">{error}</div>
            {/if}

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

                <div class="form-group">
                    <label for="description">Description</label>
                    <textarea 
                        id="description" 
                        bind:value={description}
                        rows="4"
                    ></textarea>
                </div>

                <div class="modal-actions">
                    <button type="button" class="cancel-btn" on:click={closeModal}>
                        Cancel
                    </button>
                    <button type="submit" class="save-btn">
                        {editingCampaign ? 'Save' : 'Create'} Campaign
                    </button>
                </div>
            </form>
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
        align-items: center;
        justify-content: center;
        z-index: 1000;
    }

    .modal-content {
        background: white;
        padding: 2rem;
        border-radius: 0.5rem;
        width: 90%;
        max-width: 500px;
    }

    .form-group {
        margin-bottom: 1rem;
    }

    .form-group label {
        display: block;
        margin-bottom: 0.5rem;
        color: #4b5563;
        font-weight: 500;
    }

    .form-group input,
    .form-group textarea {
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

    .cancel-btn {
        padding: 0.5rem 1rem;
        border-radius: 0.375rem;
        background: #6b7280;
        color: white;
    }

    .save-btn {
        padding: 0.5rem 1rem;
        border-radius: 0.375rem;
        background: #22c55e;
        color: white;
    }

    .error-message {
        background: #fee2e2;
        color: #991b1b;
        padding: 1rem;
        border-radius: 0.375rem;
        margin-bottom: 1rem;
    }
</style> 
<script lang="ts">
    import { createEventDispatcher } from 'svelte';
    import { fade } from 'svelte/transition';
    import { API_URL } from '$lib/config';
    import type { Campaign } from '$lib/types/types';
    import { campaignStore } from '$lib/stores/campaigns';

    export let show = false;
    export let editingCampaign: Campaign | null = null;

    const dispatch = createEventDispatcher();

    let activeTab: 'create' | 'import' = 'create';
    let name = '';
    let description = '';
    let importJson = '';
    let error: string | null = null;

    // Reset form when modal opens or editingCampaign changes
    $: if (show) {
        if (editingCampaign) {
            name = editingCampaign.name;
            description = editingCampaign.description || '';
            activeTab = 'create'; // Force create tab when editing
        } else {
            name = '';
            description = '';
            importJson = '';
        }
    }

    async function handleSubmit() {
        try {
            if (activeTab === 'create') {
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
                const data = await response.json();
                await handleSuccess(data.id);
            } else {
                try {
                    JSON.parse(importJson);
                    let id = await campaignStore.importCampaign(importJson);
                    await handleSuccess(id);
                } catch (e) {
                    throw new Error('Invalid JSON format');
                }
            }
        } catch (e) {
            console.error('Error saving campaign:', e);
            error = e instanceof Error ? e.message : 'An error occurred';
        }
    }

    async function handleSuccess(id: number) {
        await campaignStore.fetchCampaigns();
        dispatch('saved', id);
        closeModal();
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
            
            {#if !editingCampaign}
                <div class="tabs">
                    <button 
                        class="tab-button" 
                        class:active={activeTab === 'create'}
                        on:click={() => activeTab = 'create'}
                    >
                        Create New
                    </button>
                    <button 
                        class="tab-button" 
                        class:active={activeTab === 'import'}
                        on:click={() => activeTab = 'import'}
                    >
                        Import
                    </button>
                </div>
            {/if}
            
            {#if error}
                <div class="error-message">{error}</div>
            {/if}

            <form on:submit|preventDefault={handleSubmit}>
                {#if activeTab === 'create'}
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
                {:else}
                    <div class="form-group">
                        <label for="import">Campaign JSON</label>
                        <textarea 
                            id="import" 
                            bind:value={importJson}
                            rows="10"
                            placeholder="Paste your campaign JSON here..."
                            required
                        ></textarea>
                    </div>
                {/if}

                <div class="modal-actions">
                    <button type="button" class="cancel-btn" on:click={closeModal}>
                        Cancel
                    </button>
                    <button type="submit" class="save-btn">
                        {#if activeTab === 'create'}
                            {editingCampaign ? 'Save' : 'Create'} Campaign
                        {:else}
                            Import Campaign
                        {/if}
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

    .tabs {
        display: flex;
        gap: 0.5rem;
        margin-bottom: 1.5rem;
        border-bottom: 2px solid #e5e7eb;
        padding-bottom: 0.5rem;
    }

    .tab-button {
        padding: 0.5rem 1rem;
        border: none;
        background: none;
        cursor: pointer;
        font-size: 1rem;
        color: #6b7280;
        border-radius: 0.375rem;
        transition: all 0.2s;
    }

    .tab-button:hover {
        background: #f3f4f6;
        color: #111827;
    }

    .tab-button.active {
        background: #3b82f6;
        color: white;
    }

    textarea#import {
        font-family: monospace;
        font-size: 0.875rem;
    }
</style> 
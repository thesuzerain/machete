<script lang="ts">
    import { fade } from 'svelte/transition';
    import { characterStore } from '$lib/stores/characters';
  import { campaignSessionStore } from '$lib/stores/campaignSessions';
  import { campaignStore, selectedCampaignStore } from '$lib/stores/campaigns';
  import type { Campaign } from '$lib/types/types';
  import CampaignModal from './CampaignModal.svelte';
  import { onMount } from 'svelte';

    export let error: string | null;

    let showNewCampaignModal = false;
    let editingCampaign: Campaign | null = null;

    $: campaigns = $campaignStore;
    $: selectedCampaign = $selectedCampaignStore ? $campaignStore.get($selectedCampaignStore) : null;

    onMount(async () => {
        await campaignStore.fetchCampaigns();
        let campaigns = $campaignStore;

        // If no campaign is selected, select the first one
        if (!$selectedCampaignStore) {
            $selectedCampaignStore = (campaigns.size > 0 ? campaigns.keys().next().value : null) || null;
        }
    });
    </script>

<div class="campaign-header" transition:fade>
    {#if error}
        <div class="error-message" transition:fade>{error}</div>
    {/if}

    <div class="campaign-selector">
        <select bind:value={$selectedCampaignStore}>
            <option value={null}>Select a campaign...</option>
            {#each campaigns as [id, campaign]}
                <option value={id}>{campaign.name}</option>
            {/each}
        </select>
        {#if $selectedCampaignStore}
            <button class="new-campaign-btn" on:click={() => {
                editingCampaign = selectedCampaign ? selectedCampaign : null
                showNewCampaignModal = true
            }}>
                Edit Campaign
            </button>
        {/if}
        <button class="new-campaign-btn" on:click={() => showNewCampaignModal = true}>
            New Campaign
        </button>
    </div>
</div>

<CampaignModal
    bind:show={showNewCampaignModal}
    bind:editingCampaign
    on:saved={(e : CustomEvent<number>) => {
        showNewCampaignModal = false;
        editingCampaign = null;
        
        if (e.detail) {
            $selectedCampaignStore = e.detail;
        }
    }}
    on:close={() => {
        showNewCampaignModal = false;
        editingCampaign = null;
    }}
/>

<style>
    .campaign-selector {
        display: flex;
        gap: 1rem;
        margin-bottom: 2rem;
    }

    .campaign-selector select {
        flex: 1;
        padding: 0.5rem;
        font-size: 1rem;
        border: 1px solid #e2e8f0;
        border-radius: 0.375rem;
    }

    .new-campaign-btn {
        background: #3b82f6;
        color: white;
        padding: 0.5rem 1rem;
        border-radius: 0.375rem;
        font-weight: 500;
        white-space: nowrap;
    }

    .error-message {
        background: #fee2e2;
        color: #991b1b;
        padding: 1rem;
        border-radius: 0.375rem;
        margin-bottom: 1rem;
    }
</style> 
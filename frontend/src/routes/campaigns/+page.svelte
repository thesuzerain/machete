<script lang="ts">
    import { onMount } from 'svelte';
    import { fade } from 'svelte/transition';
    import { campaignStore } from '$lib/stores/campaigns';
    import { characterStore } from '$lib/stores/characters';
    import type { Campaign, InsertEvent, Log } from '$lib/types/types';
    import CampaignModal from '$lib/components/CampaignModal.svelte';
    import CampaignCharactersTab from '$lib/components/CampaignCharactersTab.svelte';
    import CampaignLogsTab from '$lib/components/CampaignLogsTab.svelte';
    import { classStore } from '$lib/stores/libraryStore';
    import { API_URL } from '$lib/config';
    import { requireAuth } from '$lib/guards/auth';
    import CampaignImportTab from '$lib/components/CampaignImportTab.svelte';
    import { campaignSessionStore } from '$lib/stores/campaignSessions';
    import CampaignSummaryTab from '$lib/components/CampaignSummaryTab.svelte';

    let selectedCampaignId: number | null = null;
    let loading = true;
    let error: string | null = null;
    let showNewCampaignModal = false;
    let editingCampaign: Campaign | null = null;
    let activeTab: 'summary' | 'characters' | 'logs' | 'import' = 'summary';
    let campaignLogs: Log[] = [];

    // Subscribe to stores
    $: campaigns = $campaignStore;
    $: characters = selectedCampaignId ? $characterStore.get(selectedCampaignId) || [] : [];
    $: campaignSessions = selectedCampaignId ? $campaignSessionStore.get(selectedCampaignId) || [] : [];

    console.log(campaignSessions);

    async function fetchLogs() {
        if (!selectedCampaignId) return;
        
        try {
            const response = await fetch(`${API_URL}/campaign/${selectedCampaignId}/logs`, {
                credentials: 'include',
            });
            if (!response.ok) throw new Error('Failed to fetch logs');
            campaignLogs = await response.json();
        } catch (e) {
            error = e instanceof Error ? e.message : 'Failed to fetch logs';
        }
    }

    // Update the watch for campaign changes
    $: if (selectedCampaignId) {
        Promise.all([
            characterStore.fetchCharacters(selectedCampaignId),
            campaignSessionStore.fetchCampaignSessions(selectedCampaignId),
            fetchLogs(),
        ]).catch(e => {
            error = e instanceof Error ? e.message : 'An error occurred';
        });
    }

    // Update onMount to include library data
    onMount(async () => {
        requireAuth();

        try {
            await Promise.all([
                campaignStore.fetchCampaigns(),
                classStore.fetchEntities({}),
            ]);

            if (campaigns.size > 0) {
                // TODO: Way to set most recent campaign as default. Currently just selects first campaign in mapo
                selectedCampaignId = campaigns.keys().next()?.value || null;
            }
        } catch (e) {
            error = e instanceof Error ? e.message : 'Failed to load campaigns';
        } finally {
            loading = false;
        }
    });
</script>

<div class="campaigns-page">
    {#if error}
        <div class="error-message" transition:fade>{error}</div>
    {/if}

    <div class="campaign-selector">
        <select bind:value={selectedCampaignId}>
            <option value={null}>Select a campaign...</option>
            {#each campaigns as [id, campaign]}
                <option value={id}>{campaign.name}</option>
            {/each}
        </select>
        <button class="new-campaign-btn" on:click={() => showNewCampaignModal = true}>
            New Campaign
        </button>
    </div>

    {#if selectedCampaignId}
        <div class="campaign-metadata" transition:fade>
            <div class="metadata-item">
                <span class="label">Total Sessions</span>
                <!-- <span class="value">{metadata.total_sessions}</span> -->
            </div>
            <div class="metadata-item">
                <span class="label">Average Level</span>
                <!-- <span class="value">{metadata.average_level.toFixed(1)}</span> -->
            </div>
            <div class="metadata-item">
                <span class="label">Total XP</span>
                <!-- <span class="value">{metadata.total_experience}</span> -->
            </div>
            <div class="metadata-item">
                <span class="label">Last Session</span>
                <span class="value">
                    <!-- {metadata.last_session ? new Date(metadata.last_session).toLocaleDateString() : 'Never'} -->
                </span>
            </div>
        </div>

    <div class="tabs">
        <button 
            class="tab-button {activeTab === 'summary' ? 'active' : ''}"
            on:click={() => activeTab = 'summary'}
        >
            Summary
        </button>
        <button 
        class="tab-button {activeTab === 'characters' ? 'active' : ''}"
        on:click={() => activeTab = 'characters'}
    >
        Characters
    </button>

        <button 
            class="tab-button {activeTab === 'logs' ? 'active' : ''}"
            on:click={() => activeTab = 'logs'}
        >
            Logs
        </button>
        <button 
            class="tab-button {activeTab === 'import' ? 'active' : ''}"
            on:click={() => activeTab = 'import'}
        >
            Import
        </button>
    </div>

        {#if activeTab === 'summary'}
            <CampaignSummaryTab
                {selectedCampaignId}
                bind:error
            />
        {:else if activeTab === 'characters'}
            <CampaignCharactersTab
                        {selectedCampaignId}
                        bind:error
            />
        {:else if activeTab === 'logs'}
            <CampaignLogsTab
                {selectedCampaignId}
                {campaignLogs}
                {characters}
                {fetchLogs}
                bind:error
            />
        {:else}
            <CampaignImportTab
                {selectedCampaignId}
                {characters}
                bind:error
                {fetchLogs}
            />
        {/if}
    {/if}
</div>

<CampaignModal
    bind:show={showNewCampaignModal}
    bind:editingCampaign
    on:saved={(e : CustomEvent<number>) => {
        showNewCampaignModal = false;
        editingCampaign = null;
        
        if (e.detail) {
            selectedCampaignId = e.detail;
        }
    }}
    on:close={() => {
        showNewCampaignModal = false;
        editingCampaign = null;
    }}
/>

<style>
    .campaigns-page {
        padding: 2rem;
        max-width: 1200px;
        margin: 0 auto;
    }

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

    .campaign-metadata {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
        gap: 1.5rem;
        padding: 1.5rem;
        background: white;
        border-radius: 0.5rem;
        box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
        margin-bottom: 2rem;
    }

    .metadata-item {
        display: flex;
        flex-direction: column;
        align-items: center;
        text-align: center;
    }

    .metadata-item .label {
        font-size: 0.875rem;
        color: #64748b;
        margin-bottom: 0.5rem;
    }

    .metadata-item .value {
        font-size: 1.5rem;
        font-weight: 600;
        color: #1e293b;
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

    .tabs {
        display: flex;
        gap: 1rem;
        margin-bottom: 1rem;
        border-bottom: 1px solid #e2e8f0;
        padding-bottom: 0.5rem;
    }

    .tab-button {
        padding: 0.5rem 1rem;
        border: none;
        background: none;
        font-size: 1rem;
        color: #64748b;
        cursor: pointer;
        border-bottom: 2px solid transparent;
        transition: all 0.2s;
    }

    .tab-button:hover {
        color: #1e293b;
    }

    .tab-button.active {
        color: #3b82f6;
        border-bottom-color: #3b82f6;
    }
</style> 
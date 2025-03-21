<script lang="ts">
    import { onMount } from 'svelte';
    import { fade } from 'svelte/transition';
    import { campaignStore, selectedCampaignStore } from '$lib/stores/campaigns';
    import { characterStore } from '$lib/stores/characters';
    import type { Campaign, InsertEvent, Log } from '$lib/types/types';
    import CampaignModal from '$lib/components/CampaignModal.svelte';
    import CampaignCharactersTab from '$lib/components/CampaignCharactersTab.svelte';
    import CampaignLogsTab from '$lib/components/CampaignLogsTab.svelte';
    import { classStore } from '$lib/stores/libraryStore';
    import { API_URL } from '$lib/config';
    import { requireAuth } from '$lib/guards/auth';
    import CampaignImportTab from '$lib/components/CampaignImportTab.svelte';
    import CampaignExportTab from '$lib/components/CampaignExportTab.svelte';
    import { campaignSessionStore } from '$lib/stores/campaignSessions';
    import CampaignSessionsTab from '$lib/components/CampaignSessionsTab.svelte';
  import { encounterStore } from '$lib/stores/encounters';
  import CampaignSummaryTab from '$lib/components/CampaignSummaryTab.svelte';
import { statsStore } from '$lib/stores/stats';
    import { page } from '$app/stores';

    let loading = true;
    let error: string | null = null;
    let showNewCampaignModal = false;
    let editingCampaign: Campaign | null = null;
    let activeTab: 'summary' | 'sessions' | 'characters' | 'logs' | 'import' | 'export' = 'summary';
    let campaignLogs: Log[] = [];

    // Subscribe to stores
    $: selectedCampaignId = $selectedCampaignStore;
    $: campaigns = $campaignStore;
    $: characters = selectedCampaignId ? $characterStore.get(selectedCampaignId) || [] : [];
    $: campaignSessions = selectedCampaignId ? $campaignSessionStore.get(selectedCampaignId) || [] : [];
    $: stats = selectedCampaignId ? $statsStore.get(selectedCampaignId) : null;

    // Default session to snap to (pass to this page with a query parameter)
    // TODO: Svelte solution for parsing query parameters to a page?
    let defaultSessionIdString = $page.url.searchParams.get('sessionId');
    let defaultSessionId: number | null = null;
    if (defaultSessionIdString) {
        let sessionId = parseInt(defaultSessionIdString);

        // If we want a default session, set the active tab to sessions and set the session ID
        if (sessionId) {
            activeTab = 'sessions';
            defaultSessionId = sessionId;
        }
    }


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
            encounterStore.fetchEncounters(),
            fetchLogs(),
            statsStore.fetchStats(selectedCampaignId),
        ]).catch(e => {
            error = e instanceof Error ? e.message : 'An error occurred';
        });
    }

        // TODO: Refactor, this is used in a few places
        function getTreasureFraction(totalTreasure : number | undefined, expectedTreasure : number | undefined) {
        let expectedTreasureSanity = expectedTreasure || 0;
        let fraction = (totalTreasure || 0) / expectedTreasureSanity;
        if (isNaN(fraction)) {
            fraction = 1;
        }
        return fraction;
    }


    $: totalCombinedTreasure = stats?.total_combined_treasure || 0;
    $: treasureFractionCurrent = getTreasureFraction(stats?.total_combined_treasure, stats?.total_expected_combined_treasure);


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

    // TODO: This logic is also used in CampaignSummaryTab- refactor.
    function getClassColorForFraction(fraction: number) {
        if (fraction < 0.8) {
            return 'large-deficit-colour';
        } else if (fraction < 0.9) {
            return 'small-deficit-colour';
        } else if (fraction < 1.1) {
            return 'no-deficit-colour';
        } else if (fraction < 1.2) {
            return 'small-surplus-colour';
        } else {
            return 'large-surplus-colour';
        }
    }
</script>

<div class="campaigns-page">
    {#if selectedCampaignId && stats}
        <div class="campaign-metadata" transition:fade>
            <div class="metadata-item">
                <div class="metadata-content">
                    <span class="label">Campaign Level</span>
                    <div class="value-group">
                        <span class="value">{stats.level}</span>
                        <div class="mini-progress-bar">
                            <div class="progress" style="width: {(stats.experience_this_level / 1000) * 100}%"></div>
                        </div>
                        <div class="subtext">Experience: {stats?.experience_this_level || 0}</div>
                    </div>
                </div>
            </div>
            <div class="metadata-item">
                <div class="metadata-content">
                    <span class="label">Sessions</span>
                    <div class="value-group">
                        <span class="value">{stats.num_sessions}</span>
                        <span class="subtext">({stats.num_combat_encounters} encounters)</span>
                    </div>
                </div>
            </div>
            <div class="metadata-item">
                <div class="metadata-content">
                    <span class="label">Treasure Balance</span>
                    <div class="values-group">
                        <div class="value-group">
                        <span class="value">
                            {totalCombinedTreasure}
                        </span>
                        <span class="subtext">gold</span>
                    </div>
                    <div class="value-group">
                        <span class="value {getClassColorForFraction(treasureFractionCurrent)}">
                            {(treasureFractionCurrent * 100).toFixed(1)}%
                        </span>
                        <span class="subtext">of approximate expected gold</span>
                    </div>
                </div>
                </div>
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
            class="tab-button {activeTab === 'sessions' ? 'active' : ''}"
            on:click={() => activeTab = 'sessions'}
        >
            Sessions
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
        <button 
            class="tab-button {activeTab === 'export' ? 'active' : ''}"
            on:click={() => activeTab = 'export'}
        >
            Export
        </button>
    </div>

    {#if activeTab === 'summary'}
    <CampaignSummaryTab
        {selectedCampaignId}
    />
    {:else if activeTab === 'sessions'}
        <CampaignSessionsTab
            {selectedCampaignId}
            defaultSessionId={defaultSessionId}
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
        {:else if activeTab === 'export'}
            <CampaignExportTab
                campaignId={selectedCampaignId}
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
        gap: 1rem;
        padding: 1rem;
        background: white;
        border-radius: 0.5rem;
        box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
        margin-bottom: 2rem;
    }

    .metadata-item {
        padding: 0.75rem;
        border-radius: 0.375rem;
        background: #f8fafc;
    }

    .metadata-content {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    .metadata-item .label {
        font-size: 0.75rem;
        color: #64748b;
        text-transform: uppercase;
        letter-spacing: 0.05em;
    }

    .values-group {
        display: flex;
        justify-content: space-between;
    }

    .value-group {
        display: flex;
        align-items: baseline;
        gap: 0.5rem;
    }

    .metadata-item .value {
        font-size: 1.25rem;
        font-weight: 600;
    }

    .metadata-item .subtext {
        font-size: 0.75rem;
        color: #64748b;
    }

    .mini-progress-bar {
        width: 40px;
        height: 3px;
        background: #e2e8f0;
        border-radius: 2px;
        overflow: hidden;
        margin-top: 0.25rem;
    }

    .mini-progress-bar .progress {
        height: 100%;
        background: #3b82f6;
        transition: width 0.3s ease;
    }

    .deficit {
        color: #ef4444;
    }

    .surplus {
        color: #22c55e;
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

    .large-deficit-colour {
        color: #ef4444;
    }
    .small-deficit-colour {
        color: rgb(250, 107, 107);
    }


    .no-deficit-colour {
        color: rgb(99, 192, 133);
    }

    .small-surplus-colour {
        color: #ca9a22;
    }

    .large-surplus-colour {
        color: #f0de0d;
    }
</style> 
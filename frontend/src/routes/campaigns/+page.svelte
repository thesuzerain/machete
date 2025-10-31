<script lang="ts">
    import { onMount } from 'svelte';
    import { fade } from 'svelte/transition';
    import { campaignStore, selectedCampaignStore } from '$lib/stores/campaigns';
    import { characterStore } from '$lib/stores/characters';
    import type { Campaign, InsertEvent, Log } from '$lib/types/types';
    import NewCampaignModal from '$lib/components/modals/NewCampaignModal.svelte';
    import CampaignCharactersTab from '$lib/components/tabs/CampaignCharactersTab.svelte';
    import { classStore } from '$lib/stores/libraryStore';
    import { API_URL } from '$lib/config';
    import { requireAuth } from '$lib/guards/auth';
    import CampaignExportTab from '$lib/components/tabs/CampaignExportTab.svelte';
    import { campaignSessionStore } from '$lib/stores/campaignSessions';
    import CampaignSessionsTab from '$lib/components/tabs/CampaignSessionsTab.svelte';
  import { encounterStore } from '$lib/stores/encounters';
  import CampaignSummaryTab from '$lib/components/tabs/CampaignSummaryTab.svelte';
import { statsStore } from '$lib/stores/stats';
    import { page } from '$app/stores';
    import Card from '$lib/components/core/Card.svelte';

    let loading = true;
    let error: string | null = null;
    let showNewNewCampaignModal = false;
    let editingCampaign: Campaign | null = null;
    let activeTab: 'summary' | 'sessions' | 'characters' | 'export' = 'summary';

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

    // Update the watch for campaign changes
    $: if (selectedCampaignId) {
        Promise.all([
            characterStore.fetchCharacters(selectedCampaignId),
            campaignSessionStore.fetchCampaignSessions(selectedCampaignId),
            encounterStore.fetchEncounters(),
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
    <Card background="light" tight>
        <div class="campaign-metadata">
            <Card tight shadowed={false}>
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
            </Card>
            <Card tight shadowed={false}>
                <div class="metadata-content">
                    <span class="label">Sessions</span>
                    <div class="value-group">
                        <span class="value">{stats.num_sessions}</span>
                        <span class="subtext">({stats.num_combat_encounters} encounters)</span>
                    </div>
                </div>
            </Card>


            <Card tight shadowed={false}>
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
            </Card>
            
        </div>
    </Card>
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
        {:else if activeTab === 'export'}
            <CampaignExportTab
                campaignId={selectedCampaignId}
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

    .campaign-metadata {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
        gap: 1rem;
    }

    .metadata-content {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    .metadata-content .label {
        font-size: 0.75rem;
        color: var(--color-text-secondary);
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

    .metadata-content .value {
        font-size: 1.25rem;
        font-weight: 600;
    }

    .metadata-content .subtext {
        font-size: 0.75rem;
        color: var(--color-text-secondary);
    }

    .mini-progress-bar {
        width: 40px;
        height: 3px;
        background: var(--color-bg-raised);
        border-radius: 2px;
        overflow: hidden;
        margin-top: 0.25rem;
    }

    .mini-progress-bar .progress {
        height: 100%;
        background: var(--color-bg-success);
        transition: width 0.3s ease;
    }

    .tabs {
        display: flex;
        gap: 1rem;
        margin-top: 1rem;
        margin-bottom: 1rem;
        border-bottom: 1px solid var(--color-bg-border);
        padding-bottom: 0.5rem;
    }

    .tab-button {
        padding: 0.5rem 1rem;
        border: none;
        background: none;
        font-size: 1rem;
        color: var(--color-text-secondary);
        cursor: pointer;
        border-bottom: 2px solid transparent;
        transition: all 0.2s;
    }

    .tab-button:hover {
        background-color: var(--color-bg-hover);
    }

    .tab-button.active {
        /* TODO: Maybe make all tab selectors work this way? It looks VERY nice compared to the straight blue buttons */
        color: var(--color-bg-selected);
        border-bottom-color: var(--color-bg-selected);
    }

    .large-deficit-colour {
        color: var(--color-large-deficit);
    }
    .small-deficit-colour {
        color: var(--color-small-deficit);
    }
    .no-deficit-colour {
        color: var(--color-no-deficit);
    }
    .small-surplus-colour {
        color: var(--color-small-surplus);
    }
    .large-surplus-colour {
        color: var(--color-large-surplus);
    }
</style> 
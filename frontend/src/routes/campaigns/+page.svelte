<script lang="ts">
    import { onMount } from 'svelte';
    import { fade } from 'svelte/transition';
    import { campaignStore } from '$lib/stores/campaigns';
    import { characterStore } from '$lib/stores/characters';
    import type { Campaign, Log } from '$lib/types/types';
    import CampaignModal from '$lib/components/CampaignModal.svelte';
    import CampaignCharactersTab from '$lib/components/CampaignCharactersTab.svelte';
    import CampaignLogsTab from '$lib/components/CampaignLogsTab.svelte';
    import { classStore } from '$lib/stores/libraryStore';
    import { API_URL } from '$lib/config';
    import { requireAuth } from '$lib/guards/auth';

    let selectedCampaignId: number | null = null;
    let loading = true;
    let error: string | null = null;
    let showNewCampaignModal = false;
    let editingCampaign: Campaign | null = null;
    let activeTab: 'characters' | 'logs' = 'characters';
    let campaignLogs: Log[] = [];

    // Subscribe to stores
    $: campaigns = $campaignStore;
    $: characters = $characterStore;

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

            if (campaigns.length > 0) {
                selectedCampaignId = campaigns[0].id;
            }
        } catch (e) {
            error = e instanceof Error ? e.message : 'Failed to load campaigns';
        } finally {
            loading = false;
        }
    });

    // Add logs-related helper functions
    function generateEvents(characterIds: number[]): InsertEvent[] {
        const events: InsertEvent[] = [];
        
        for (const enemy of enemies) {
            for (const characterId of characterIds) {
                events.push({
                    character: characterId,
                    event_type: enemy.type === 'enemy' ? 'EnemyDefeated' : 'HazardDefeated',
                    description: `Defeated ${enemy.count} ${enemy.type}`,
                    data: {
                        id: enemy.id,
                        count: enemy.count
                    }
                });

                events.push({
                    character: characterId,
                    event_type: 'ExperienceGain',
                    description: `Gained experience from ${enemy.type}`,
                    data: {
                        experience: getExperienceFromLevel(enemy.level || 0, characters.find(c => c.id === characterId)?.level || 0)
                    }
                });
            }
        }

        for (const treasure of treasures) {
            for (const characterId of characterIds) {
                events.push({
                    character: characterId,
                    event_type: treasure.type === 'currency' ? 'CurrencyGain' : 'ItemGain',
                    description: treasure.type === 'currency' 
                        ? `Gained ${treasure.amount} currency`
                        : `Gained item`,
                    data: treasure.type === 'currency' 
                        ? { currency: { gold: treasure.amount } }
                        : { id: treasure.itemId }
                });
            }
        }

        return events;
    }

    async function handleCharacterDelete(id: number) {
        try {
            if (selectedCampaignId) {
                await characterStore.deleteCharacter(selectedCampaignId, id);
            }
        } catch (e) {
            error = e instanceof Error ? e.message : 'Failed to delete character';
        }
    }
</script>

<div class="campaigns-page">
    {#if error}
        <div class="error-message" transition:fade>{error}</div>
    {/if}

    <div class="campaign-selector">
        <select bind:value={selectedCampaignId}>
            <option value={null}>Select a campaign...</option>
            {#each campaigns as campaign}
                <option value={campaign.id}>{campaign.name}</option>
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
    </div>

        {#if activeTab === 'characters'}
            <CampaignCharactersTab
                {selectedCampaignId}
                bind:error
            />
        {:else}
            <CampaignLogsTab
                {selectedCampaignId}
                {campaignLogs}
                {characters}
                {fetchLogs}
                bind:error
            />
        {/if}
    {/if}
</div>

<CampaignModal
    bind:show={showNewCampaignModal}
    bind:editingCampaign
    on:saved={async () => {
        await campaignStore.fetchCampaigns();
        showNewCampaignModal = false;
        editingCampaign = null;
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

    .characters-section {
        background: white;
        border-radius: 0.5rem;
        box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
        padding: 1.5rem;
    }

    .characters-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 1.5rem;
    }

    .character-list {
        display: flex;
        flex-direction: column;
        gap: 1.5rem;
    }

    .character-row {
        background: white;
        border-radius: 0.5rem;
        box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
        overflow: hidden;
    }

    .character-main {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 1.5rem;
        background: #f8fafc;
        border-bottom: 1px solid #e2e8f0;
    }
    
    .character-identity {
        display: flex;
        flex-direction: row;
        gap: 1rem;
    }

    .character-identity h3 {
        margin: 0;
        font-size: 1.25rem;
        color: #1e293b;
    }

    .character-subtitle {
        color: #64748b;
        font-size: 0.875rem;
        margin-top: 0.25rem;
    }

    .character-content {
        padding: 1.5rem;
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
        gap: 1.5rem;
    }

    .content-section {
        padding: 1rem;
        background: #f8fafc;
        border-radius: 0.375rem;
    }

    .content-section h4 {
        margin: 0 0 1rem 0;
        color: #475569;
        font-size: 0.875rem;
        text-transform: uppercase;
        letter-spacing: 0.05em;
    }

    .xp-display {
        background: #e2e8f0;
        border-radius: 9999px;
        height: 1.5rem;
        overflow: hidden;
        position: relative;
    }

    .xp-bar {
        background: #3b82f6;
        height: 100%;
        width: var(--progress);
        transition: width 0.3s ease;
    }

    .xp-text {
        position: absolute;
        left: 50%;
        top: 50%;
        transform: translate(-50%, -50%);
        color: white;
        font-size: 0.875rem;
        font-weight: 500;
        text-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
    }

    .stats-grid {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
        gap: 1rem;
    }

    .stat-item {
        display: flex;
        flex-direction: column;
        align-items: center;
        text-align: center;
    }

    .stat-label {
        font-size: 0.75rem;
        color: #64748b;
        margin-bottom: 0.25rem;
    }

    .stat-value {
        font-size: 1.25rem;
        font-weight: 600;
        color: #1e293b;
    }

    .activity-placeholder {
        height: 100px;
        background: #e2e8f0;
        border-radius: 0.375rem;
        display: flex;
        align-items: center;
        justify-content: center;
        color: #64748b;
        font-size: 0.875rem;
    }

    .add-character-btn {
        background: #22c55e;
        color: white;
        padding: 0.5rem 1rem;
        border-radius: 0.375rem;
        font-weight: 500;
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

    .logs-section {
        background: white;
        border-radius: 0.5rem;
        box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
        padding: 1.5rem;
    }

    .logs-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 1.5rem;
    }

    .logs-preview {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    .add-log-btn {
        background: #3b82f6;
        color: white;
        padding: 0.5rem 1rem;
        border-radius: 0.375rem;
        font-weight: 500;
    }

    .logs-controls {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 1rem;
        padding: 1rem;
        background: #f9fafb;
        border-radius: 0.375rem;
    }

    .filter-sort {
        display: flex;
        gap: 1rem;
        align-items: center;
    }

    .filter-input {
        padding: 0.5rem;
        border: 1px solid #e2e8f0;
        border-radius: 0.375rem;
        min-width: 200px;
    }

    .sort-controls {
        display: flex;
        gap: 0.5rem;
        align-items: center;
    }

    .sort-direction {
        padding: 0.25rem 0.5rem;
        background: white;
        border: 1px solid #e2e8f0;
        border-radius: 0.375rem;
        cursor: pointer;
    }

    .new-log-btn {
        background: #3b82f6;
        color: white;
        padding: 0.5rem 1rem;
        border-radius: 0.375rem;
        font-weight: 500;
    }

    .modal {
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
        max-width: 800px;
        width: 90%;
        max-height: 90vh;
        overflow-y: auto;
    }

    .form-actions {
        display: flex;
        justify-content: flex-end;
        gap: 1rem;
        margin-top: 2rem;
    }

    .form-actions button {
        padding: 0.5rem 1rem;
        border-radius: 0.375rem;
        font-weight: 500;
    }

    .form-actions button[type="submit"] {
        background: #3b82f6;
        color: white;
    }

    .form-actions button[type="button"] {
        background: #e5e7eb;
        color: #374151;
    }

    .enemy-entry, .treasure-entry {
        display: flex;
        gap: 0.5rem;
        align-items: center;
        margin-bottom: 0.5rem;
        padding: 0.5rem;
        background: #f9fafb;
        border-radius: 0.375rem;
    }

    .enemy-entry input[type="number"],
    .treasure-entry input[type="number"] {
        width: 100px;
    }

    .item-name {
        min-width: 150px;
        font-weight: 500;
    }
</style> 
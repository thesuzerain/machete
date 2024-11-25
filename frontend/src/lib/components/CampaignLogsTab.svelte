<script lang="ts">
    import { fade } from 'svelte/transition';
    import type { Log, InsertEvent, Character } from '$lib/types/types';
    import EventManager from '$lib/components/EventManager.svelte';
    import EventCreator from '$lib/components/EventCreator.svelte';
    import LogCreationModal from './LogCreationModal.svelte';
    import { API_URL } from '$lib/config';

    export let selectedCampaignId: number;
    export let campaignLogs: Log[];
    export let characters: Character[];
    export let error: string | null;
    export let fetchLogs: () => Promise<void>;

    let logNewOpen = false;
    let showingEventsForLog: Log | null = null;
    let logFilter = '';
    let logSort: 'date' | 'name' = 'date';
    let sortDirection: 'asc' | 'desc' = 'desc';
    let selectedLogs: number[] = [];

    $: filteredAndSortedLogs = campaignLogs
        .filter(log => log.name.toLowerCase().includes(logFilter.toLowerCase()))
        .sort((a, b) => {
            const direction = sortDirection === 'asc' ? 1 : -1;
            switch (logSort) {
                case 'date':
                    return direction * (new Date(a.timestamp).getTime() - new Date(b.timestamp).getTime());
                case 'name':
                    return direction * a.name.localeCompare(b.name);
                default:
                    return 0;
            }
        });

        async function viewLogEvents(log: Log) {
        showingEventsForLog = log;
    }

</script>

<div class="logs-section" transition:fade>
    <div class="logs-header">
        <h2>Logs</h2>
        <button class="add-log-btn" on:click={() => logNewOpen = true}>
            Add Log
        </button>
    </div>

    <div class="logs-controls">
        <input type="text" placeholder="Filter logs" bind:value={logFilter} />
        <select bind:value={logSort}>
            <option value="date">Date</option>
            <option value="name">Name</option>
        </select>
        <select bind:value={sortDirection}>
            <option value="asc">Ascending</option>
            <option value="desc">Descending</option>
        </select>
    </div>

    <div class="logs-list">
        {#each filteredAndSortedLogs as log}
            <div class="log-row" transition:fade>
                <div class="log-row">
                    <input
                        type="checkbox"
                        value={log.id}
                        bind:group={selectedLogs}
                        on:click|stopPropagation
                    />
                    <div class="log-summary" on:click={() => viewLogEvents(log)}>
                        <div class="log-title">
                            <span class="log-name">{log.name}</span>
                            <span class="log-events-count">({log.events.length} events)</span>
                        </div>
                        <div class="log-meta">
                            <span class="timestamp">{new Date(log.timestamp).toLocaleString()}</span>
                            
                        </div>
                    </div>
                </div>

            </div>
        {/each}
    </div>
</div>

<LogCreationModal
    bind:show={logNewOpen}
    {selectedCampaignId}
    {characters}
    {fetchLogs}
/>

{#if showingEventsForLog}
    <div class="modal" on:click={() => showingEventsForLog = null}>
        <div class="modal-content" on:click|stopPropagation>
            <div class="modal-header">
                <h2>Events for: {showingEventsForLog.name}</h2>
                <button class="close-button" on:click={() => showingEventsForLog = null}>Close</button>
            </div>
            <p>{showingEventsForLog.description}</p>
            
            <!-- Add EventCreator above the EventManager -->
            <div class="modal-event-creator">
                <EventCreator 
                    characters={characters}
                    onEventCreate={async (event) => {
                        try {
                            const response = await fetch(`${API_URL}/campaign/${selectedCampaignId}/events`, {
                                method: 'POST',
                                credentials: 'include',
                                headers: { 'Content-Type': 'application/json' },
                                body: JSON.stringify({
                                    ...event,
                                    event_group: showingEventsForLog?.id
                                }),
                            });
                            if (!response.ok) throw new Error('Failed to create event');
                            await fetchLogs();
                        } catch (e) {
                            error = e instanceof Error ? e.message : 'Failed to create event';
                        }
                    }}
                />
            </div>

            <EventManager 
                events={showingEventsForLog.events}
                characters={characters}
                campaignId={selectedCampaignId}
                groupId={showingEventsForLog.id.toString()}
                onEventsUpdate={fetchLogs}
            />
        </div>
    </div>
{/if}


<style>
    .logs-page {
        padding: 2rem;
        max-width: 1200px;
        margin: 0 auto;
    }

    .log-form {
        display: flex;
        flex-direction: column;
        background: #f8f8f8;
        padding: 1.5rem;
        border-radius: 8px;
        margin-bottom: 2rem;
        gap: 1rem;
    }

    .template-description {
        display: block;
        color: #666;
        margin-top: 0.25rem;
    }

    .logs-section {
        margin-top: 2rem;
        background: white;
        border-radius: 8px;
        padding: 1.5rem;
        box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
    }

    .logs-controls {
        margin-bottom: 1.5rem;
        padding: 1rem;
        background: #f9fafb;
        border-radius: 4px;
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    .logs-list {
        max-height: 600px;
        overflow-y: auto;
        border: 1px solid #e5e7eb;
        border-radius: 4px;
    }

    .log-row {
        display: flex;
        align-items: center;
        padding: 0.5rem 1rem;
        border-bottom: 1px solid #e5e7eb;
        background: white;
    }

    .log-row:hover {
        background: #f9fafb;
    }

    .log-summary {
        display: flex;
        justify-content: space-between;
        align-items: center;
        flex: 1;
        cursor: pointer;
        margin-left: 1rem;
    }

    .log-title {
        display: flex;
        align-items: center;
        gap: 0.5rem;
    }

    .log-events-count {
        color: #6b7280;
        font-size: 0.875rem;
    }

    .log-meta {
        display: flex;
        align-items: center;
        gap: 1rem;
    }

    .log-actions {
        display: flex;
        gap: 0.5rem;
    }

    .log-actions button {
        padding: 0.25rem 0.5rem;
        font-size: 0.875rem;
    }

    .log-details {
        padding: 1rem;
        background: #f9fafb;
        border-bottom: 1px solid #e5e7eb;
    }

    .delete-selected {
        background: #ef4444;
        color: white;
        border: none;
        padding: 0.5rem 1rem;
        border-radius: 4px;
        cursor: pointer;
    }

    .delete-selected:hover {
        background: #dc2626;
    }

    .modal {
        position: fixed;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        background: rgba(0,0,0,0.5);
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .modal-content {
        background: white;
        padding: 2rem;
        border-radius: 8px;
        max-width: 800px;
        width: 90%;
        max-height: 90vh;
        overflow-y: auto;
    }

    .enemy-entry, .treasure-entry {
        display: flex;
        gap: 0.5rem;
        margin-bottom: 0.5rem;
        align-items: center;
    }

    .enemies-section, .treasure-section {
        margin: 1rem 0;
        padding: 1rem;
        background: #f0f0f0;
        border-radius: 4px;
    }

    .advanced-section {
        margin-top: 1rem;
        padding-top: 1rem;
        border-top: 1px solid #ddd;
        
    }

    .manual-events {
        margin-top: 1rem;
        padding: 1rem;
        background: #f8f8f8;
        border-radius: 4px;
    }

    .events-preview {
        margin-top: 1rem;
        border-top: 1px solid #ddd;
        padding-top: 1rem;
    }

    .event-preview {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 0.5rem;
        background: white;
        border-radius: 4px;
        margin-bottom: 0.5rem;
    }

    .event-info {
        display: flex;
        gap: 1rem;
        align-items: center;
    }

    .event-type {
        font-weight: 500;
        min-width: 120px;
    }

    .event-character {
        color: #666;
        min-width: 100px;
    }

    .remove-event {
        background: #ef4444;
        color: white;
        border: none;
        padding: 0.25rem 0.5rem;
        border-radius: 4px;
        cursor: pointer;
    }

    .remove-event:hover {
        background: #dc2626;
    }

    .modal-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    .item-name {
        font-weight: 500;
        white-space: nowrap;
    }

    .form-header {
        display: flex;
        justify-content: space-between;
        gap: 1rem;
        align-items: center;
        background: #f9fafb;

        border-radius: 4px;
        align-items: stretch;
        height: 100%;
    }

    .form-card {
        display: flex;
        justify-content: flex-start;
        align-items: stretch; /* Changed from flex-start to stretch to fill height */
        flex-direction: column;
        margin-bottom: 0.5rem; /* Fixed the comment syntax */
        padding: 1rem;
        background: #f0f0f0;
        border-radius: 4px;
        width: 100%;
    }

    .form-card h3 {
        margin: 0;
        min-width: 200px;
    }

    .form-card input {
        width: 100%;
    }
    
    .form-card textarea {
        width: 100%;
        height: 100%;
        resize: none;
    }

    .form-group {
        width: 100%;
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    .character-checkbox {
        display: flex;
        flex-direction: row;        
        white-space: nowrap;
        gap: 0.5rem;
        border-radius: 4px;
    }

    .character-checkbox input[type="checkbox"] {
       width: auto;
       cursor: pointer;
    }

    .log-form-container {
        height: 100%;
    }

    .create-log-button {
        margin-top: 1rem;
    }
    </style> 
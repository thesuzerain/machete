<script lang="ts">
    import { fade } from 'svelte/transition';
    import type { Character, CampaignSession } from '$lib/types/types';
    import CharacterModal from '$lib/components/CharacterModal.svelte';
    import { characterStore } from '$lib/stores/characters';
    import { classStore, itemStore } from '$lib/stores/libraryStore';
    import { campaignSessionStore } from '$lib/stores/campaignSessions';
    import { encounterStore } from '$lib/stores/encounters';
    import { goto } from '$app/navigation';
    import { dndzone, type DndEvent } from 'svelte-dnd-action';
    import { getRewardForLevelSeverity } from '$lib/utils/encounter';
    import { onMount } from 'svelte';
    import { id } from 'date-fns/locale';

    export let selectedCampaignId: number;

    let showNewCharacterModal = false;
    let showSessionOrderModal = false;
    let editingName = false;
    let editingDescription = false;
    let selectedSessionId: number | null = null;
    let tempName = '';
    let tempDescription = '';

    $: items = $itemStore;
    $: campaignSessions = ($campaignSessionStore.get(selectedCampaignId)) || [];
    $: selectedSession = campaignSessions.find(s => s.id === selectedSessionId);
    $: sessionEncounters = selectedSession ? ($encounterStore.filter(e => selectedSession.encounter_ids.includes(e.id))) : [];

    // Calculate total rewards for the session
    $: totalSessionRewards = sessionEncounters.reduce((acc, enc) => {
        const gold = Math.floor(enc.treasure_currency);
        const silver = Math.floor((enc.treasure_currency - gold) * 10);
        const copper = Math.floor(((enc.treasure_currency - gold) * 10 - silver) * 10);
        return ({
        xp: acc.xp + enc.total_experience,
        currency: {
            gold: acc.currency.gold + gold,
            silver: acc.currency.silver + silver,
            copper: acc.currency.copper + copper
        }
    })
    
    }, { xp: 0, currency: { gold: 0, silver: 0, copper: 0 } });

    // Set to most recent session by default
    // TODO: Cache where user were recently
    onMount(async () => {
        await handleCampaignChange(selectedCampaignId);
    });

    // TODO: Bad pattern?
    $: handleCampaignChange(selectedCampaignId);

    async function handleCampaignChange(id: number) {
        await campaignSessionStore.fetchCampaignSessions(id);
        if (campaignSessions.length > 0) {
            selectedSessionId = campaignSessions[campaignSessions.length - 1].id;
        }
    }

    async function updateSessionName() {
        if (!selectedSession || !tempName) return;
        await campaignSessionStore.updateCampaignSession(selectedCampaignId, { ...selectedSession, name: tempName });
        editingName = false;
    }

    async function updateSessionDescription() {
        if (!selectedSession || !tempDescription) return;
        await campaignSessionStore.updateCampaignSession(selectedCampaignId, { ...selectedSession, description: tempDescription });
        editingDescription = false;
    }

    async function removeEncounterFromSession(encounterId: number) {
        if (!selectedSession) return;
        await encounterStore.updateEncounter(encounterId, {
            session_id: null,
        });
        await campaignSessionStore.fetchCampaignSessions(selectedCampaignId);
    }

    async function handleTemporarySessionReorder(e: CustomEvent<DndEvent<CampaignSession>>) {
        let sessions = e.detail.items;
        let acc = 0;
        let sessionOrders = sessions.map(s => {
            acc++;
            return { ...s, session_order: acc };
        });
        campaignSessions = sessionOrders;
    }

    async function handleSessionReorder(e: CustomEvent<DndEvent<CampaignSession>>) {
        let sessions = e.detail.items;
        let acc = 0;
        let sessionOrders = sessions.map(s => {
            acc++;
            return { ...s, session_order: acc };
        });
        await campaignSessionStore.updateCampaignSessions(selectedCampaignId, sessionOrders);
    }

    async function createNewSession() {
        const highestSessionOrder = campaignSessions.reduce((acc, s) => s.session_order > acc ? s.session_order : acc, 0);
        await campaignSessionStore.addCampaignSessions(selectedCampaignId, [{
            name: `New session`,
            description: '',
            session_order: highestSessionOrder + 1,
            encounter_ids: [],
        }]);

        await campaignSessionStore.fetchCampaignSessions(selectedCampaignId);

        // Go to the new session
        selectedSessionId = campaignSessions[campaignSessions.length - 1].id;
    }

    function createNewEncounter() {
        goto('/encounters?sessionId=' + selectedSessionId);
    }

    function editEncounter(encounterId: number) {
        // TODO
        goto(`/encounters/${encounterId}`);
    }



</script>

<div class="characters-section" transition:fade>
    <div class="session-selector">
        <select bind:value={selectedSessionId}>
            <option value={null}>Select a session...</option>
            {#each campaignSessions as session, ind}
                <option value={session.id}>Session {ind}: {session.name}</option>
            {/each}
        </select>
        <button class="edit-button" on:click={() => showSessionOrderModal = true}>
            Edit sessions
        </button>
        <button class="add-button" on:click={createNewSession}>
            New session
        </button>
    </div>

    {#if selectedSession}
        <div class="session-header">
            <div class="session-title">
                {#if editingName}
                    <input 
                        bind:value={tempName} 
                        on:blur={updateSessionName}
                        on:keydown={(e) => e.key === 'Enter' && updateSessionName()}
                    />
                {:else}
                    <h2 on:click={() => {
                        editingName = true;
                        tempName = selectedSession.name;
                    }}>{selectedSession.name}</h2>
                {/if}
            </div>
            
            <div class="session-description">
                {#if editingDescription}
                    <textarea 
                        bind:value={tempDescription}
                        on:blur={updateSessionDescription}
                        rows="3"
                    />
                {:else}
                    <p on:click={() => {
                        editingDescription = true;
                        tempDescription = selectedSession.description || '';
                    }}>{selectedSession.description || 'Click to add description...'}</p>
                {/if}
            </div>
        </div>

        <div class="session-summary">
            <div class="summary-box">
                <h4>Session Rewards</h4>
                <div class="reward-details">
                    <p>Experience: {totalSessionRewards.xp} XP</p>
                    <p>Treasure: {totalSessionRewards.currency.gold}g {totalSessionRewards.currency.silver}s {totalSessionRewards.currency.copper}c</p>
                </div>
            </div>
        </div>

        <div class="encounters-section">
            <div class="section-header">
                <h3>Session Encounters</h3>
                <button class="add-button" on:click={createNewEncounter}>
                    Create New Encounter
                </button>
            </div>

            <div class="encounters-list">
                {#each sessionEncounters as encounter}
                    <div class="encounter-card">
                        <div class="encounter-info">
                            <h4>{encounter.name}</h4>
                            <p>XP: {encounter.total_experience}</p>
                        </div>
                        <div class="encounter-actions">
                            <button class="edit-button" on:click={() => editEncounter(encounter.id)}>
                                Edit
                            </button>
                            <button class="remove-button" on:click={() => removeEncounterFromSession(encounter.id)}>
                                Remove
                            </button>
                        </div>
                    </div>
                {/each}
            </div>
        </div>

        <div class="misc-section">
            <h3>Reward Assignments</h3>
            <div class="item-division">
                <h4>Items</h4>
                <div class="item-list">
                    {#each sessionEncounterItems as item}
                        <div class="item-card">
                            <p>{item.name}</p>
                        </div>
                    {/each}
                </div>
            </div>

            <div class="gold-division">
                TODO
            </div>
        </div>
    {/if}
</div>

{#if showSessionOrderModal}
    <div class="modal">
        <div class="modal-content">
            <h2>Reorder Sessions</h2>
            <div use:dndzone={{items: campaignSessions}} on:consider="{handleTemporarySessionReorder}" on:finalize="{handleSessionReorder}">
                {#each campaignSessions as session, ix (session.id)}
                    <div class="session-order-item" draggable="true">
                        <span class="drag-handle">⋮⋮</span>
                        <span>Session {ix} {session.name}</span>
                    </div>
                {/each}
            </div>
            <div class="modal-actions">
                <button class="cancel-button" on:click={() => showSessionOrderModal = false}>
                    Close
                </button>
            </div>
        </div>
    </div>
{/if}

<style>
    .session-selector {
        display: flex;
        gap: 1rem;
        margin-bottom: 2rem;
    }

    .session-selector select {
        flex: 1;
        padding: 0.5rem;
        font-size: 1rem;
        border: 1px solid #e2e8f0;
        border-radius: 0.375rem;
    }

    .characters-section {
        background: white;
        border-radius: 0.5rem;
        box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
        padding: 1.5rem;
    }

    .session-header {
        margin-bottom: 2rem;
    }

    .session-title input {
        font-size: 1.5rem;
        width: 100%;
        padding: 0.5rem;
        border: 1px solid #e5e7eb;
        border-radius: 0.375rem;
    }

    .session-description textarea {
        width: 100%;
        padding: 0.5rem;
        border: 1px solid #e5e7eb;
        border-radius: 0.375rem;
        resize: vertical;
    }

    .summary-box {
        background: #f9fafb;
        padding: 1rem;
        border-radius: 0.5rem;
        margin-bottom: 2rem;
    }

    .reward-details {
        display: flex;
        gap: 2rem;
        margin-top: 0.5rem;
    }

    .encounters-list {
        display: grid;
        gap: 1rem;
        margin-top: 1rem;
    }

    .encounter-card {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 1rem;
        background: #f9fafb;
        border-radius: 0.5rem;
    }

    .encounter-actions {
        display: flex;
        gap: 0.5rem;
    }

    .session-order-item {
        display: flex;
        align-items: center;
        gap: 1rem;
        padding: 0.75rem;
        background: white;
        border: 1px solid #e5e7eb;
        margin-bottom: 0.5rem;
        border-radius: 0.375rem;
        cursor: move;
    }

    .drag-handle {
        color: #9ca3af;
        cursor: move;
    }

    .add-button {
        background: #22c55e;
        color: white;
        border: none;
        padding: 0.5rem 1rem;
        border-radius: 0.375rem;
        cursor: pointer;
    }

    .edit-button {
        background: #3b82f6;
        color: white;
        border: none;
        padding: 0.5rem 1rem;
        border-radius: 0.375rem;
        cursor: pointer;
    }

    .remove-button {
        background: #ef4444;
        color: white;
        border: none;
        padding: 0.5rem 1rem;
        border-radius: 0.375rem;
        cursor: pointer;
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
        max-width: 500px;
        width: 90%;
    }

    .section-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 1rem;
    }
</style> 
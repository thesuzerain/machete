<script lang="ts">
    import { fade } from 'svelte/transition';
    import type { CampaignSession, CompiledRewards } from '$lib/types/types';
    import { characterStore } from '$lib/stores/characters';
    import { itemStore } from '$lib/stores/libraryStore';
    import { campaignSessionStore } from '$lib/stores/campaignSessions';
    import { encounterStore } from '$lib/stores/encounters';
    import { goto } from '$app/navigation';
    import { dndzone, SHADOW_PLACEHOLDER_ITEM_ID, type DndEvent } from 'svelte-dnd-action';
    import { onMount } from 'svelte';
    import RangeSlider from 'svelte-range-slider-pips';
    import { compile } from 'svelte/compiler';

    interface Props {
        selectedCampaignId: number;
    }
    let { 
        selectedCampaignId = $bindable(),

     } : Props = $props();

    let showSessionOrderModal = $state(false);
    let editingName = $state(false);
    let editingDescription = $state(false);
    let selectedSessionId: number | null = $state(null);

    let tempName = $state('');
    let tempDescription = $state('');

    let items = $derived($itemStore);
    let campaignSessions = $derived(($campaignSessionStore.get(selectedCampaignId)) || []);
    let selectedSession = $derived(campaignSessions.find(s => s.id === selectedSessionId));
    let sessionEncounters = $derived(selectedSession ? ($encounterStore.filter(e => selectedSession.encounter_ids.includes(e.id))) : []);
    let campaignCharacters = $derived(($characterStore.get(selectedCampaignId)) || []);

    // Calculate total rewards for the session
    interface TotalRewards {
        xp: number;
        currency: number;
        items: Record<number, number[]>;
        total_treasure_value: number;
    }
    let totalSessionRewards = $derived(sessionEncounters.reduce((acc, encounter) => {
        acc.xp += encounter.total_experience;
        acc.currency += encounter.treasure_currency;
        acc.items[encounter.id]= encounter.treasure_items;
        acc.total_treasure_value += encounter.total_treasure_value;
        return acc;
    }, { xp: 0, currency: 0, items: {}, total_treasure_value: 0 } as TotalRewards));
    
    // character -> items and character -> gold
    // -1 -> unassigned
    interface DndRewardItem {
        // For drag and drop. Almost always will parse to itemId
        id: string;
        itemId: number;
    }

    let compiledGoldRewards : Record<number, number> = $state({});
    let compiledGoldTotal = $derived(Object.values(compiledGoldRewards).reduce((acc, curr) => acc + curr, 0));

    let compiledItemRewardsWithIds : Record<number, DndRewardItem[]> = $state({});
    let compiledItemRewardsApi = $derived(Object.fromEntries(Object.entries(compiledItemRewardsWithIds).map(([key, value]) => [key, value.map(v => v.itemId)]
    ))) as Record<number, number[]>;
    let compiledItemRewardsIter = $derived(Object.entries(compiledItemRewardsWithIds).map((a,b) => {
        return [Number(a[0]), a[1]];
    }) as [number, DndRewardItem[]][]);
    let compiledItemRewardsTotal = $derived(Object.values(compiledItemRewardsWithIds).flat().length);

    // Set to most recent session by default
    // TODO: Cache where user were recently
    onMount(async () => {
        await handleCampaignChange(selectedCampaignId);
    });

    // TODO: Bad pattern? Onupdate?
    $effect(() => {
        handleCampaignChange(selectedCampaignId);
    });

    async function handleCampaignChange(id: number) {
        await campaignSessionStore.fetchCampaignSessions(id);
        if (campaignSessions.length > 0) {
            selectedSessionId = campaignSessions[campaignSessions.length - 1].id;
        }

        // TODO: This is not right. Because when we 'switch' the campaign it will all reset...
        await handleEncountersUpdate();
    }

    async function handleSessionChange() {
        await handleEncountersUpdate();
    }

    async function handleEncountersUpdate() {
        // TODO: Refactor
        const requiredItems = sessionEncounters.reduce((acc, encounter) => {
            return acc.concat(encounter.treasure_items);
        }, [] as number[]);
        await itemStore.fetchEntities({
            ids: requiredItems.join(','),
        })

        // Update compiled rewards.
        let session = campaignSessions.find(s => s.id === selectedSessionId);
        if (!session) return;

        compiledItemRewardsWithIds = {};
        compiledGoldRewards = {};

        // Populate with items and gold
        Object.entries(session.compiled_rewards).forEach(([characterId, characterRewards]) => {
            // Populate with gold
            compiledGoldRewards[Number(characterId)] = characterRewards.gold;

            // Populate with items
            compiledItemRewardsWithIds[Number(characterId)] = characterRewards.items.map((iid, ix) => {
                return {
                    id: iid.toString()+'-'+ix,
                    itemId: iid,
                } as DndRewardItem;
            });
        });

        // Add unassigned ones as -1
        compiledItemRewardsWithIds[-1] = session.unassigned_item_rewards.map((iid, ix) => {
            return {
                id: iid.toString()+'-'+ix,
                itemId: iid,
            } as DndRewardItem;
        });
        compiledGoldRewards[-1] = session.unassigned_gold_rewards;

        // For every character in this session, ensure they have a key in both compilations
        // TODO: Not every character is in every session
        campaignCharacters.forEach(c => {
            compiledItemRewardsWithIds[c.id] = compiledItemRewardsWithIds[c.id] || [];
            compiledGoldRewards[c.id] = compiledGoldRewards[c.id] || 0;
        });
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
        await campaignSessionStore.unlinkEncounterFromSession(selectedCampaignId, selectedSession.id, encounterId);

        // Update the session encounters (gold, etc, changes so we need to re-assign)
        await handleEncountersUpdate();
    }

    let temporarySessionOrder : CampaignSession[] = $state([]);

    async function initializeSessionReorder() {
        temporarySessionOrder = campaignSessions;
        showSessionOrderModal = true;
    }

    async function handleTemporarySessionReorder(e: CustomEvent<DndEvent<CampaignSession>>) {
        let sessions = e.detail.items;
        let acc = 0;
        let sessionOrders = sessions.map(s => {
            acc++;
            return { ...s, session_order: acc };
        });
        temporarySessionOrder = sessionOrders;
    }

    async function handleSessionReorder(e: CustomEvent<DndEvent<CampaignSession>>) {
        let sessions = e.detail.items;
        let acc = 0;
        let sessionOrders = sessions.map(s => {
            acc++;
            return { ...s, session_order: acc };
        });
        temporarySessionOrder = sessionOrders;
        await campaignSessionStore.updateCampaignSessions(selectedCampaignId, sessionOrders);   
    }

    async function createNewSession() {
        const highestSessionOrder = campaignSessions.reduce((acc, s) => s.session_order > acc ? s.session_order : acc, 0);
        await campaignSessionStore.addCampaignSessions(selectedCampaignId, [{
            name: `New session`,
            description: '',
            session_order: highestSessionOrder + 1,
            encounter_ids: [],
            unassigned_gold_rewards: 0,
            unassigned_item_rewards: [],
            compiled_rewards: [],
        }]);

        await campaignSessionStore.fetchCampaignSessions(selectedCampaignId);

        // Go to the new session
        selectedSessionId = campaignSessions[campaignSessions.length - 1].id;
    }

    function createNewEncounter() {
        goto('/encounters?sessionId=' + selectedSessionId);
    }

    function editEncounter(encounterId: number) {
        goto(`/encounters?encounterId=${encounterId}`);
    }

    function dragItemAssignmentConsider(cid : number, e: CustomEvent<DndEvent<DndRewardItem>>) {
        compiledItemRewardsWithIds[cid] = e.detail.items.filter(i => i.id !== SHADOW_PLACEHOLDER_ITEM_ID);
    }

    function dragItemAssignmentFinalize(cid : number, e: CustomEvent<DndEvent<DndRewardItem>>) {
        compiledItemRewardsWithIds[cid] = e.detail.items.filter(i => i.id !== SHADOW_PLACEHOLDER_ITEM_ID);
        updateRewardAssignments();
    }
    
    function modifyGoldReward(cid : number, e: {detail: {value: number}}) {
        compiledGoldRewards[cid] = e.detail.value;
        reassignGoldWithMaximum(cid);
        updateRewardAssignments();
    }

    function reassignGoldWithMaximum(cidEdited: number) {
        let randomNum = Math.random();

        // Ensure we don't exceed the total gold rewards
        const totalCompiledGoldRewards = Object.values(compiledGoldRewards).reduce((acc, curr) => acc + curr, 0);
        let amountToReduce = totalCompiledGoldRewards - totalSessionRewards.currency;

        while (amountToReduce !== 0) {
            // Remove from unassigned if we can, otherwise remove from the first character
            // todo: why is this an array
            const firstOtherKeyWithGold = Object.entries(compiledGoldRewards).filter(([key, value]) => value > 0 && Number(key) !== cidEdited).map(([key, value]) => Number(key));
            const firstOtherKeyWithSpace = Object.entries(compiledGoldRewards).filter(([key, value]) => value < totalSessionRewards.currency && Number(key) !== cidEdited).map(([key, value]) => Number(key));

            if (firstOtherKeyWithGold.length > 0 && amountToReduce > 0) {
                let singularReduction = Math.min(compiledGoldRewards[firstOtherKeyWithGold[0]], amountToReduce);
                compiledGoldRewards[firstOtherKeyWithGold[0]] -= singularReduction;
                amountToReduce -= singularReduction;
            } else if (firstOtherKeyWithSpace.length > 0 && amountToReduce < 0) {
                let singularReduction = Math.min(totalSessionRewards.currency - compiledGoldRewards[firstOtherKeyWithSpace[0]], amountToReduce);
                compiledGoldRewards[firstOtherKeyWithSpace[0]] -= singularReduction;
                amountToReduce -= singularReduction;
            } else {
                break;
            }
        }
    }

    async function updateRewardAssignments() {
        if (!selectedSession) return;

        let itemRewards : Record<number,number[]> = $state.snapshot(compiledItemRewardsApi);
        delete itemRewards[-1];

        let goldRewards : Record<number,number> = $state.snapshot(compiledGoldRewards);
        delete goldRewards[-1];

        let compiledRewards : Record<number,CompiledRewards> = {};
        Object.entries(itemRewards).forEach(([cid, items]) => {
            compiledRewards[Number(cid)] = {
                gold: goldRewards[Number(cid)] || 0,
                items: items,
            }
        });

        await campaignSessionStore.updateEncounterLinksMetadata(selectedCampaignId, selectedSession.id,
        { ...selectedSession, compiled_rewards: compiledRewards });
    }
</script>

<div class="characters-section" transition:fade>
    <div class="session-selector">
        <select bind:value={selectedSessionId} on:change={handleSessionChange}>
            <option value={null}>Select a session...</option>
            {#each campaignSessions as session, ind}
                <option value={session.id}>Session {ind}: {session.name}</option>
            {/each}
        </select>
        <button class="edit-button" on:click={() => initializeSessionReorder()}>
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
                            <div class="encounter-info-row"><p>XP: {encounter.total_experience}</p><p>Gold: {encounter.treasure_currency}</p></div>
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
            <div class="reward-assignments-header">
                <h3>Reward Assignments</h3>
            </div>
            <div class="summary-box">
                <h4>Session Rewards</h4>
                <div class="reward-details">
                    <p>Experience: {totalSessionRewards.xp} XP</p>
                    <p>Gold: {totalSessionRewards.currency}g</p>
                    <p>Total treasure value: {totalSessionRewards.total_treasure_value}</p>
                </div>
            </div>

            <div class="item-division-characters">
                {#each compiledItemRewardsIter as [cid, characterItems]}
                <div class="item-division-character-column">

                    {#if compiledGoldTotal > 0 || compiledItemRewardsTotal > 0}
                    <h4>{ cid === -1 ? 'Unassigned' : campaignCharacters.find(c => c.id === cid)?.name}</h4>
                    {/if}
                    {#if compiledItemRewardsTotal > 0}
                        <section use:dndzone="{{items: characterItems}}" on:consider="{(e) => dragItemAssignmentConsider(cid, e)}" on:finalize="{(e) => dragItemAssignmentFinalize(cid, e)}" class="item-division-character-dnd">
                            {#each characterItems as item(item.id)}
                                {#if items.entities.get(item.itemId)}
                                <div class="session-order-item" draggable="true">
                                    <span class="drag-handle">⋮⋮</span>
                                    <span>{items.entities.get(item.itemId)!.name}</span>
                                </div>
                                {:else}
                                <div class="session-order-item" draggable="true">
                                    <span class="drag-handle">⋮⋮</span>
                                    <span>Failed to find item: {item.id}</span>
                                </div>

                                    
                                {/if}
                            {/each}
                        </section>
                    {/if}
                    fresh {compiledGoldTotal}
                    {#if compiledGoldTotal > 0 && totalSessionRewards.currency > 0}
                    <div class="gold-division">
                        <input type="number" bind:value={compiledGoldRewards[cid]} min={0} max={Math.ceil(totalSessionRewards.currency)} 
                        on:change={(e) => reassignGoldWithMaximum(cid)}
                        />
                        <RangeSlider value={compiledGoldRewards[cid]} all='label' 
                        float="true" pipstep={Math.ceil(totalSessionRewards.currency/10)} springValues={[0.1, 0.1]} pips 
                        on:change={(e) => modifyGoldReward(cid, e)}
                        min={0} max={Math.ceil(totalSessionRewards.currency)} />
                        <p>gold</p>
                    </div>
                    {/if}
                </div>
                {/each}
                </div>
        </div>
    {/if}
</div>

<!-- TODO: Extract to component?-->
{#if showSessionOrderModal}
    <div class="modal">
        <div class="modal-content">
            <h2>Reorder Sessions</h2>
            <div use:dndzone={{items: temporarySessionOrder}} on:consider="{handleTemporarySessionReorder}" on:finalize="{handleSessionReorder}">
                {#each temporarySessionOrder as session, ix (session.id)}
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

    .item-division-characters {
        display: flex;
        flex-direction: row;
        align-items: stretch;
        gap: 1rem;
    }

    .item-division-character-column {
        display: flex;
        flex-direction: column;
        align-items: stretch;
        gap: 1rem;
        flex: 1;
    }

    .item-division-character-dnd {
        flex: 1;
        background: #f9fafb;
        border: 1px solid #e5e7eb;
        border-radius: 0.5rem;
        overflow: hidden;
    }

    .gold-division {
        display: grid;
        grid-template-columns: 0.1fr 0.9fr 0.1fr;
        gap: rem;
    }

    .gold-division input {
        height: 50%;
        padding: 0.5rem;
        border: 1px solid #e5e7eb;

        padding: 0.5rem;
        border: 1px solid #e5e7eb;
        border-radius: 0.375rem;
    }

    .gold-division p {
        padding-top: 0.5rem;
        text-align: center;
    }

    .misc-section {
        margin-top: 2rem;
    }

    .reward-assignments-header {
        margin-bottom: 1rem;
    }

</style> 
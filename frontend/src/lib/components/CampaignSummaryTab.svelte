<script lang="ts">
    import { fade } from 'svelte/transition';
    import type { CampaignSession } from '$lib/types/types';
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
    }
    let totalSessionRewards = $derived(sessionEncounters.reduce((acc, encounter) => {
        acc.xp += encounter.total_experience;
        acc.currency += encounter.treasure_currency;
        acc.items[encounter.id]= encounter.treasure_items;
        return acc;
    }, { xp: 0, currency: 0, items: {} } as TotalRewards));
    
    // character -> items and character -> gold
    // -1 -> unassigned
    interface DndRewardItem {
        // For drag and drop. Almost always will parse to itemId
        id: string;
        encounterId: number;
        itemId: number;
    }

    let compiledGoldRewards : Record<number, number> = $state({});
    let compiledGoldTotal = $derived(Object.values(compiledGoldRewards).reduce((acc, curr) => acc + curr, 0));
    let [compiledGoldRewardsApi, compiledLeftoverGoldApi] = $derived.by(() => {
        // It doesn't really matter which encounter the gold is assigned to, as long as it doesn't surpass the total
        let finishedCharacters : Record<number, Record<number, number>> = {};
        let finishedLeftover : Record<number, number> = {};

        const clonedCompiledGoldRewards = { ...$state.snapshot(compiledGoldRewards) };
        const totalTreasures = Object.fromEntries(sessionEncounters.map(e => [e.id, e.treasure_currency]));

        // iterate thru clonedCompiledGoldRewards
        Object.entries(clonedCompiledGoldRewards).forEach(([cid, gold]) => {
            if (gold > 0) {
                // Find the first encounter that has gold left
                const remainingEncounter = Object.entries(totalTreasures).find(([_, treasure]) => treasure > 0) || null;
                const characterId = Number(cid);
                if (remainingEncounter) {
                    const [eid, treasure] = remainingEncounter;
                    let encounterId = Number(eid);
                    finishedCharacters[encounterId] = finishedCharacters[encounterId] || {};
                    if (characterId === -1) {
                        finishedLeftover[encounterId] = gold;
                    } else {
                        finishedCharacters[encounterId][characterId] = gold;
                        totalTreasures[eid] -= gold;
                    }
                } 
            }
        });

        // Assign the remaining gold to unassigned
        Object.entries(totalTreasures).forEach(([eid, treasure]) => {
            if (finishedLeftover[Number(eid)]) {
                finishedLeftover[Number(eid)] += treasure;
            } else {
                finishedLeftover[Number(eid)] = treasure;
            }
        });

        return [finishedCharacters, finishedLeftover];
    });


    let compiledItemRewardsWithIds : Record<number, DndRewardItem[]> = $state({});
    let compiledItemRewardsIter = $derived(Object.entries(compiledItemRewardsWithIds).map((a,b) => {
        return [Number(a[0]), a[1]];
    }) as [number, DndRewardItem[]][]);

    // Original encounterId -> characterId -> itemIds
    let [compiledItemRewardsApi, compiledLeftoverItemApi] = $derived.by(() => {
        Object.entries(compiledItemRewardsWithIds)
        let finishedCharacters : Record<number, Record<number, number[]>> = {};
        let finishedLeftover : Record<number, number[]> = {};

        Object.entries(compiledItemRewardsWithIds).forEach(([cid, characterItems]) => {
            let characterId = Number(cid);
            characterItems.forEach(item => {
                finishedLeftover[item.encounterId] = finishedLeftover[item.encounterId] || [];
                finishedCharacters[item.encounterId] = finishedCharacters[item.encounterId] || {};
                if (characterId === -1) {
                    finishedLeftover[item.encounterId].push(item.itemId);
                } else {
                    finishedCharacters[item.encounterId][characterId] = finishedCharacters[item.encounterId][characterId] || [];
                    finishedCharacters[item.encounterId][characterId].push(item.itemId);
                }
            });
        });

        return [finishedCharacters, finishedLeftover];
    });

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

        // Populate with items
        Object.entries(session.compiled_item_rewards).forEach(([encounterId, characterItemIds]) => {
            Object.entries(characterItemIds).forEach(([characterId, itemIds]) => {
                let a = itemIds.map((iid, ix) => {
                    return {
                        id: iid.toString()+'-'+encounterId+'-'+ix,
                        encounterId: Number(encounterId),
                        itemId: iid,
                    } as DndRewardItem;
                });
                compiledItemRewardsWithIds[Number(characterId)] = (compiledItemRewardsWithIds[Number(characterId)] || []).concat(a);
            });
        });

        // Add unassigned ones as -1
        Object.entries(session.unassigned_item_rewards).forEach(([eId, itemIds], ix) => {
            let a = itemIds.map((iid, ix) => {
                return {
                    id: iid.toString()+'-'+eId+'-'+ix,
                    encounterId: Number(eId),
                    itemId: iid,
                } as DndRewardItem;
            });
            compiledItemRewardsWithIds[-1] = (compiledItemRewardsWithIds[-1] || []).concat(a);
        });

        // Populate with gold
        Object.entries(session.compiled_gold_rewards).forEach(([eId, characterGolds]) => {
            Object.entries(characterGolds).forEach(([characterId, gold]) => {
                compiledGoldRewards[Number(characterId)] = (compiledGoldRewards[Number(characterId)] || 0) + gold;
            });
        });

        // Add unassigned ones as -1
        Object.entries(session.unassigned_gold_rewards).forEach(([eId, gold], ix) => {
            compiledGoldRewards[-1] = (compiledGoldRewards[-1] || 0) + gold;
        });

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
    }

    let temporarySessionOrder: CampaignSession[] = [];

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
        await campaignSessionStore.updateCampaignSessions(selectedCampaignId, sessionOrders);   
    }

    async function createNewSession() {
        const highestSessionOrder = campaignSessions.reduce((acc, s) => s.session_order > acc ? s.session_order : acc, 0);
        await campaignSessionStore.addCampaignSessions(selectedCampaignId, [{
            name: `New session`,
            description: '',
            session_order: highestSessionOrder + 1,
            encounter_ids: [],
            unassigned_gold_rewards: {},
            unassigned_item_rewards: {},
            compiled_gold_rewards: {},
            compiled_item_rewards: {},
        }]);

        await campaignSessionStore.fetchCampaignSessions(selectedCampaignId);

        // Go to the new session
        selectedSessionId = campaignSessions[campaignSessions.length - 1].id;
    }

    function createNewEncounter() {
        // TODO
        goto('/encounters?sessionId=' + selectedSessionId);
    }

    function editEncounter(encounterId: number) {
        // TODO
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
        // Ensure we don't exceed the total gold rewards
        const totalCompiledGoldRewards = Object.values(compiledGoldRewards).reduce((acc, curr) => acc + curr, 0);
        const amountToReduce = totalCompiledGoldRewards - totalSessionRewards.currency;

        if (amountToReduce !== 0) {
            // Remove from unassigned if we can, otherwise remove from the first character
            const firstOtherKeyWithGold = Object.entries(compiledGoldRewards).filter(([key, value]) => value > 0 && Number(key) !== cidEdited).map(([key, value]) => Number(key));
            const firstOtherKeyWithSpace = Object.entries(compiledGoldRewards).filter(([key, value]) => value < totalSessionRewards.currency && Number(key) !== cidEdited).map(([key, value]) => Number(key));

            if (firstOtherKeyWithGold.length > 0 && amountToReduce > 0) {
                compiledGoldRewards[firstOtherKeyWithGold[0]] -= amountToReduce;
            } else if (firstOtherKeyWithSpace.length > 0 && amountToReduce < 0) {
                compiledGoldRewards[firstOtherKeyWithSpace[0]] -= amountToReduce;
            }
        }
    }

    async function updateRewardAssignments() {
        if (!selectedSession) return;
        await campaignSessionStore.updateEncounterLinksMetadata(selectedCampaignId, selectedSession.id,
        { ...selectedSession, compiled_gold_rewards: compiledGoldRewardsApi, 
            unassigned_gold_rewards: compiledLeftoverGoldApi, compiled_item_rewards: compiledItemRewardsApi, unassigned_item_rewards: compiledLeftoverItemApi });
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
            <div class="reward-assignments-header">
                <h3>Reward Assignments</h3>
            </div>
            <div class="summary-box">
                <h4>Session Rewards</h4>
                <div class="reward-details">
                    <p>Experience: {totalSessionRewards.xp} XP</p>
                    <p>Treasure: {totalSessionRewards.currency}g</p>
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
                    {#if compiledGoldTotal > 0}
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
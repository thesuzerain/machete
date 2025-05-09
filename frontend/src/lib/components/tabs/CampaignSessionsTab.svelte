<script lang="ts">
    import { fade } from 'svelte/transition';
    import type { CampaignSession, CompiledRewards } from '$lib/types/types';
    import { characterStore } from '$lib/stores/characters';
    import { creatureStore, hazardStore, itemStore } from '$lib/stores/libraryStore';
    import { campaignSessionStore } from '$lib/stores/campaignSessions';
    import { encounterStore } from '$lib/stores/encounters';
    import { goto } from '$app/navigation';
    import { dndzone, SHADOW_PLACEHOLDER_ITEM_ID, type DndEvent } from 'svelte-dnd-action';
    import { onMount } from 'svelte';
    import RangeSlider from 'svelte-range-slider-pips';
    import { compile } from 'svelte/compiler';
    import EncounterViewer from '../encounter/EncounterViewerModal.svelte';
    import type { AccomplishmentLevel, Encounter } from '$lib/types/encounters';
    import Card from '../core/Card.svelte';
    import Button from '../core/Button.svelte';
    import Modal from '../core/Modal.svelte';
    import QuickAccomplishment from '../encounter/QuickAccomplishment.svelte';
    import EncounterLinkerModal from '../modals/EncounterLinkerModal.svelte';
    import EncounterSummary from '../encounter/EncounterSummary.svelte';
    import { color } from 'd3';
    import DropdownButton from '../core/DropdownButton.svelte';
    import ConfirmationModal from '../modals/ConfirmationModal.svelte';

    interface Props {
        selectedCampaignId: number;
        defaultSessionId: number | null;
    }
    let { 
        selectedCampaignId = $bindable(),
        defaultSessionId = $bindable(),
     } : Props = $props();


    let showSessionOrderModal = $state(false);
    let deletingSession = $state(false);
    let editingName = $state(false);
    let editingDescription = $state(false);
    let selectedSessionId: number | null = $state(null);
    let showCharacterSelector = $state(false);
    let showNonPresent = $state(false);

    let tempName = $state('');
    let tempDescription = $state('');

    let items = $derived($itemStore);
    let creatures = $derived($creatureStore);
    let campaignSessions = $derived(($campaignSessionStore.get(selectedCampaignId)) || []);
    let selectedSession : CampaignSession | null = $derived(campaignSessions.find(s => s.id === selectedSessionId)|| null);
    let sessionEncounters = $derived(selectedSession ? ($encounterStore.filter(e => selectedSession.encounter_ids.includes(e.id))) : []);
    let campaignCharacters = $derived(($characterStore.get(selectedCampaignId)) || []);

    // Track which characters are present in the session separately from rewards
    let presentCharacters = $state(new Set<number>());

    // Level at 'start' of session- get previous
    let sessionIx = $derived.by(() => {
        let sessionIx: Map<number, number> = new Map();
        campaignSessions.forEach((session, ix) => {
            sessionIx.set(session.id, ix);
        });
        return sessionIx;
    });
    let levelAtStart = $derived.by(() => {
        if (!selectedSessionId) return 0;
        const sessionIxValue = sessionIx.get(selectedSessionId);
        if (sessionIxValue === undefined) return 0;
        const previousSession = campaignSessions[sessionIxValue - 1];
        return previousSession.level_at_end;
    });

    let thisSessionIx = $derived.by(() => {
        if (!selectedSessionId) return 0;
        const sessionIxValue = sessionIx.get(selectedSessionId);
        if (sessionIxValue === undefined) return 0;
        return sessionIxValue;
    });

    let nextSession = $derived.by(() => {
        if (campaignSessions.length <= thisSessionIx) return undefined;
        return campaignSessions[thisSessionIx + 1];
    });

    let prevSession = $derived.by(() => {
        if (thisSessionIx === 0) return undefined;
        return campaignSessions[thisSessionIx - 1];
    });

    let firstSession = $derived.by(() => {
        if (campaignSessions.length === 0) return undefined;
        return campaignSessions[0];
    });

    let lastSession = $derived.by(() => {
        if (campaignSessions.length === 0) return undefined;
        return campaignSessions[campaignSessions.length - 1];
    });

    // Calculate total rewards for the session
    interface TotalRewards {
        xp: number;
        currency: number;
        items: Record<number, number[]>;
        total_items_value: number;
    }
    let totalSessionRewards = $derived(sessionEncounters.reduce((acc, encounter) => {
        acc.xp += encounter.total_experience;
        acc.currency += encounter.treasure_currency;
        acc.items[encounter.id]= encounter.treasure_items;
        acc.total_items_value += encounter.total_items_value;
        return acc;
    }, { xp: 0, currency: 0, items: {}, total_items_value: 0 } as TotalRewards));
    
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
            if (defaultSessionId) {
                selectedSessionId = defaultSessionId;
            } else {
                selectedSessionId = campaignSessions[campaignSessions.length - 1].id;
            }
        }

        // TODO: This is not right. Because when we 'switch' the campaign it will all reset...
        await handleSessionChange();
    }

    async function handleSessionChange() {
        await handleEncountersUpdate();
        // Update present characters based on compiled rewards
        if (selectedSession) {
            console.log('selectedSession', selectedSession);
            presentCharacters = new Set(
                Object.keys(selectedSession.compiled_rewards)
                    .map(Number)
                    .filter(id => id !== -1)
                    .filter(id => selectedSession.compiled_rewards[id].present)
            );
        } else {
            presentCharacters = new Set();
        }
        showCharacterSelector = false;
        showAccomplishmentForm = false;
    }

    async function handleEncountersUpdate() {
        // TODO: Refactor, modular?
        const requiredItems = sessionEncounters.reduce((acc, encounter) => {
            return acc.concat(encounter.treasure_items);
        }, [] as number[]);

        const requiredCreatures = sessionEncounters.reduce((acc, encounter) => {
            return acc.concat((encounter.enemies ?? []).map(e => e.id));
        }, [] as number[]);

        const requiredHazards = sessionEncounters.reduce((acc, encounter) => {
            return acc.concat(encounter.hazards ?? []);
        }, [] as number[]);
        
        await Promise.all([
            itemStore.fetchEntities({
            ids: requiredItems.join(','),
        }), 
            creatureStore.fetchEntities({
                ids: requiredCreatures.join(','),
            }),
            hazardStore.fetchEntities({
                ids: requiredHazards.join(','),
            }),
    ]); 

        // Update compiled rewards.
        let session = campaignSessions.find(s => s.id === selectedSessionId);
        if (!session) return;

        compiledItemRewardsWithIds = {};
        compiledGoldRewards = {};

        // Add unassigned rewards
        compiledItemRewardsWithIds[-1] = session.unassigned_item_rewards.map((iid, ix) => {
            return {
                id: iid.toString()+'-'+ix,
                itemId: iid,
            } as DndRewardItem;
        });
        compiledGoldRewards[-1] = session.unassigned_gold_rewards;

        // Add entries for all characters  characters
        for (const charId of campaignCharacters.map(c => c.id)) {
            const rewards = session.compiled_rewards[charId] || { gold: 0, items: [] };
            compiledGoldRewards[charId] = rewards.gold;
            compiledItemRewardsWithIds[charId] = rewards.items.map((iid, ix) => {
                return {
                    id: iid.toString()+'-'+ix,
                    itemId: iid,
                } as DndRewardItem;
            });
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
        await campaignSessionStore.unlinkEncounterFromSession(selectedCampaignId, selectedSession.id, encounterId);

        // Update the session encounters (gold, etc, changes so we need to re-assign)
        await handleEncountersUpdate();
    }

    async function deleteEncounter(encounterId: number) {
        await encounterStore.deleteEncounter(encounterId);
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
        
        // Get characters from most recent session, or all characters if no sessions exist
        let initialCharacterIds: number[] = [];
        if (campaignSessions.length > 0) {
            const mostRecentSession = campaignSessions[campaignSessions.length - 1];
            initialCharacterIds = Object.keys(mostRecentSession.compiled_rewards).map(Number);
        } else {
            initialCharacterIds = campaignCharacters.map(c => c.id);
        }

        await campaignSessionStore.addCampaignSessions(selectedCampaignId, [{
            name: `New session`,
            description: '',
            session_order: highestSessionOrder + 1,
            characters: initialCharacterIds,
        }]);

        await campaignSessionStore.fetchCampaignSessions(selectedCampaignId);

        // Go to the new session
        selectedSessionId = campaignSessions[campaignSessions.length - 1].id;

        handleSessionChange();
    }

    function createNewEncounter() {
        goto(`/encounters?sessionId=${selectedSessionId}&returnToSessionId=${selectedSessionId}`);
    }

    function editEncounter(encounterId: number) {
        goto(`/encounters?encounterId=${encounterId}&returnToSessionId=${selectedSessionId}`);
    }

    function correctEncounter(encounterId: number) {
        // Quick correct for party size and level
        const encounter = sessionEncounters.find(e => e.id === encounterId);
        if (!encounter) return;
        const partySize = encounter.party_size || 0;
        const partyLevel = encounter.party_level || 0;
        const newPartySize = presentCharacters.size;
        const newPartyLevel = levelAtStart;
        encounterStore.updateEncounter(encounterId, {
            party_size: newPartySize,
            party_level: newPartyLevel,
        });
        encounter.party_size = newPartySize;
        encounter.party_level = newPartyLevel;
    }

    function dragItemAssignmentConsider(cid : number, e: CustomEvent<DndEvent<DndRewardItem>>) {
        compiledItemRewardsWithIds[cid] = e.detail.items;
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

        const updatedCompiledRewards: Record<number, CompiledRewards> = {};
        
        // Include rewards for all present characters
        for (const charId of campaignCharacters.map(c => c.id)) {
            updatedCompiledRewards[charId] = {
                gold: compiledGoldRewards[charId] || 0,
                present: presentCharacters.has(charId),
                items: compiledItemRewardsWithIds[charId]?.map(item => item.itemId) || []
            };
        }

        await campaignSessionStore.updateEncounterLinksMetadata(selectedCampaignId,  
            selectedSession.id, {
            compiled_rewards: updatedCompiledRewards
        });
    }

    // Add these state variables
    let viewingEncounter : Encounter | null = $state(null);
    let showEncounterViewer = $state(false);
    let showEncounterLinker = $state(false);

    // Add this function
    function viewEncounter(encounter : Encounter) {
        viewingEncounter = encounter;
        showEncounterViewer = true;
    }

    // Accomplishment state
    let accomplishmentName = $state('');
    let accomplishmentDescription = $state('');
    let accomplishmentType: AccomplishmentLevel | null = $state('moderate');
    let useCustomXP = $state(false);
    let customXPAmount = $state(0);

    let showAccomplishmentForm = $state(false);

    let canAddAccomplishment = $derived(
        useCustomXP ? customXPAmount > 0 : accomplishmentType !== null
    );

    function setCustomXP() {
        useCustomXP = true;
        accomplishmentType = null;
        if (!customXPAmount) customXPAmount = 10; // Default value
    }

    function setAccomplishmentType(type: AccomplishmentLevel) {
        accomplishmentType = type;
        useCustomXP = false;
    }


    async function updateSessionCharacters(characterId: number, present: boolean) {
        if (!selectedSession) return;
        if (present) {
            presentCharacters.add(characterId);
        } else {
            // Treat as if we set rewards to 0
            compiledGoldRewards[characterId] = 0;
            compiledItemRewardsWithIds[characterId] = [];

            presentCharacters.delete(characterId);
        }
        await updateRewardAssignments();
        presentCharacters = presentCharacters; // trigger reactivity

        // Update the UI state to match
        await handleEncountersUpdate();
    }

    async function handleSessionDelete() {
        if (!selectedSession) return;
        const newSession = prevSession?.id;
        await campaignSessionStore.deleteCampaignSession(selectedCampaignId, selectedSession.id);
        console.log('newSession', newSession);
        if (newSession) {
            selectedSessionId = newSession;
            await handleSessionChange();
        } else {
            selectedSessionId = null;
        }
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

        <!-- Next session-->
         
         {#if selectedSessionId !== null}
         <Button
         colour="blue"
         disabled={!firstSession || selectedSessionId === firstSession.id}
         onclick={() => {
             if (firstSession) {
                 selectedSessionId = firstSession.id;
                 handleSessionChange();
             }
         }}
     >&lt;&lt;</Button>

        <Button
        colour="blue"
        disabled={!prevSession}
        onclick={() => {
            if (prevSession) {
                selectedSessionId = prevSession.id;
                handleSessionChange();
            }
        }}
    >&lt;</Button>

            <Button
            colour="blue"
            disabled={!nextSession}
            onclick={() => {
                if (nextSession) {
                    selectedSessionId = nextSession.id;
                    handleSessionChange();
                }
            }}
        >&gt;</Button>


        <Button
            colour="blue"
            disabled={!lastSession || selectedSessionId === lastSession.id}
            onclick={() => {
                if (lastSession) {
                    selectedSessionId = lastSession.id;
                    handleSessionChange();
                }
            }}
        >&gt;&gt;</Button>


         {/if}
 
        <DropdownButton colour="blue" label="Session options">
            <Button left colour="blue" onclick={createNewSession} >Create session</Button>
            <Button left colour="blue"
            onclick={() => initializeSessionReorder()}
            >Reorder sessions</Button>
            <Button left colour="blue" onclick={() => {
                deletingSession = true;
            }}>Delete session</Button>
        </DropdownButton>
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

            <div class="character-selector">
                <Button onclick={() => showCharacterSelector = !showCharacterSelector}>
                    {showCharacterSelector ? 'Hide' : 'Show'} Present Characters ({presentCharacters.size})

                </Button>

                {#if showCharacterSelector}
                    <Card>
                        <div class="character-checkboxes">
                            {#each campaignCharacters as character}
                                <label class="character-checkbox">
                                    <input 
                                        type="checkbox" 
                                        checked={presentCharacters.has(character.id)}
                                        on:change={(e) => updateSessionCharacters(character.id, (e.target as HTMLInputElement).checked)}
                                    />
                                    {character.name}
                                </label>
                            {/each}
                        </div>

                    </Card  >
                    
                {/if}
            </div>
        </div>


        <div class="encounters-section">
            <div class="section-header">
                <h3>Session Encounters</h3>
                <div class="header-buttons">
                    {#if showAccomplishmentForm}

                    <Button colour="red" onclick={() => showAccomplishmentForm = !showAccomplishmentForm}>
                        Cancel
                    </Button>
                    {:else}
                    <Button colour="green" onclick={() => showAccomplishmentForm = !showAccomplishmentForm}>
                        Add Accomplishment
                    </Button>
                    {/if}
                    <Button colour="blue" onclick={() => showEncounterLinker = true}>
                        Link Encounter
                    </Button>
                    <Button colour="green" onclick={createNewEncounter}>
                        Create New Encounter
                    </Button>                    
                </div>
            </div>

            {#if showAccomplishmentForm && selectedSessionId}
                <QuickAccomplishment {selectedCampaignId} {selectedSessionId} onAddEncounter={
                    () => {
                        showAccomplishmentForm = false;
                        handleEncountersUpdate();
                    }
                }/>
            {/if}

            <!-- Regular Encounters -->
            <div class="encounters-list">
                <h4>Combat & Other Encounters</h4>
                {#each sessionEncounters.filter(e => e.encounter_type !== 'accomplishment') as encounter}
                    <EncounterSummary encounter={encounter} size='normal' expectedPartySize={presentCharacters.size} expectedPartyLevel={levelAtStart}>

                        <Button colour="black" onclick={() => viewEncounter(encounter)}>
                            View
                        </Button>
                        {#if selectedSessionId && (encounter.party_size != presentCharacters.size || encounter.party_level != levelAtStart)}
                            <Button colour="blue" onclick={() => correctEncounter(encounter.id)}>
                                Correct party information
                            </Button>
                        {/if}
                        <Button colour="blue" onclick={() => editEncounter(encounter.id)}>
                            Edit
                        </Button>
                        <Button colour="red" onclick={() => removeEncounterFromSession(encounter.id)}>
                            Unlink
                        </Button>

                    </EncounterSummary>
                {/each}
            </div>
            {#if sessionEncounters.some(e => e.encounter_type === 'accomplishment')}
                <div class="accomplishments-list accomplishments">
                    <h4>Accomplishments</h4>
                    {#each sessionEncounters.filter(e => e.encounter_type === 'accomplishment') as encounter}
                    <EncounterSummary size='short' encounter={encounter}>
                        <Button colour="blue" onclick={() => editEncounter(encounter.id)}>
                            Edit
                        </Button>
                        <Button colour="red" onclick={() => deleteEncounter(encounter.id)}>
                            Remove
                        </Button>
                    </EncounterSummary>

                    {/each}
                </div>
            {/if}
        </div>

        <div class="misc-section">
            <div class="reward-assignments-header">
                <h3>Reward Assignments</h3>
            </div>
            <Card>
                <h4>Session Rewards</h4>
                <div class="reward-details">
                    <p>Experience: {totalSessionRewards.xp} XP</p>
                    <p>Gold: {totalSessionRewards.currency}g</p>
                    <p>Total item treasure value: {totalSessionRewards.total_items_value}</p>
                    <p>At end of session, we are level {selectedSession.level_at_end} with {selectedSession.experience_at_end} XP</p>
                </div>
            </Card>
            {#if compiledItemRewardsTotal > 0}
            <div class="non-present-checkbox">
                
                <input type="checkbox" bind:checked={showNonPresent} />
                <label on:click={() => showNonPresent = !showNonPresent}>
                    
                
                Show non-present characters</label>
            </div>
            {/if}

            <div class="item-division-characters">
                {#each compiledItemRewardsIter as [cid, characterItems]}
                {#if showNonPresent || presentCharacters.has(cid) || cid === -1}

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
                    {#if compiledGoldTotal > 0 && totalSessionRewards.currency > 0}
                    <div class="gold-division">
                        <input type="number" bind:value={compiledGoldRewards[cid]} min={0} max={Math.ceil(totalSessionRewards.currency)} 
                        on:change={(e) => reassignGoldWithMaximum(cid)}
                        />
                        <RangeSlider value={compiledGoldRewards[cid]} all='label' 
                        float pipstep={Math.ceil(totalSessionRewards.currency/10)} springValues={[0.1, 0.1]} pips 
                        on:change={(e) => modifyGoldReward(cid, e)}
                        min={0} max={Math.ceil(totalSessionRewards.currency)} />
                        <p>gold</p>
                    </div>
                    {/if}
                </div>

                {/if}
                {/each}
                </div>
        </div>
    {/if}
</div>

    <Modal bind:show={showSessionOrderModal} closeButton>
        <div slot="header">
            <h2>Reorder Sessions</h2>
        </div>
        <div class="modal-content">
            <div use:dndzone={{items: temporarySessionOrder}} on:consider="{handleTemporarySessionReorder}" on:finalize="{handleSessionReorder}" class="item-division-session-dnd">
                {#each temporarySessionOrder as session, ix (session.id)}
                    <div class="session-order-item" draggable="true">
                        <span class="drag-handle">⋮⋮</span>
                        <span>Session {ix} {session.name}</span>
                    </div>
                {/each}
            </div>
        </div>

    </Modal>


<EncounterViewer   
    bind:encounter={viewingEncounter}
    bind:show={showEncounterViewer}
/>

<EncounterLinkerModal
    bind:show={showEncounterLinker}
    bind:sessionId={selectedSessionId}
/>

<ConfirmationModal
    bind:show={deletingSession}
    on:confirm={handleSessionDelete}
    on:cancel={() => deletingSession = false}
    confirmationString="Delete session"
> Are you sure you want to delete this session? All linked encounters will be unlinked. This action cannot be undone. </ConfirmationModal>

<style>
    .session-selector {
        display: flex;
        gap: 0.5rem;
        margin-bottom: 2rem;
    }

    .characters-section {
        background:vare(--color-bg);
        border-radius: 0.5rem;
        box-shadow: var(--shadow);
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
    .reward-details {
        display: flex;
        gap: 2rem;
        margin-top: 0.5rem;
    }

    .encounters-list {
        display: grid;
        gap: 1rem;
        margin-top: 1rem;
        margin-bottom: 1rem;
    }

    .accomplishments-list {
        display: grid;
        gap: 0.25rem;
        margin: 0;
    }

    .encounter-card {
        display: flex;
        justify-content: space-between;
        align-items: center;
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
        background: var(--color-bg);
        border: 1px solid var(--color-bg-light-raised-border);
        margin-bottom: 0.5rem;
        border-radius: 0.375rem;
        cursor: move;
    }

    .drag-handle {
        color: var(--color-text);
        cursor: move;
    }


    .modal-content {
        background: var(--color-bg);
        padding: 2rem;
        border-radius: 8px;
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
        background: var(--color-bg-light-raised);
        border: 1px solid var(--color-bg-light-raised-border);
        border-radius: 0.5rem;
        overflow: hidden;
    }

    .item-division-session-dnd {
        flex: 1;
        min-width: 80vh;
        background: var(--color-bg-light-raised);
        border: 1px solid var(--color-bg-light-raised-border);
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
        display: flex;
        flex-direction: column;
        margin-top: 2rem;
        gap: 1rem;
    }

    .reward-assignments-header {
        margin-bottom: 1rem;
    }

    .header-buttons {
        display: flex;
        gap: 0.5rem;
    }

    .encounters-list h4 {
        margin-bottom: 0.75rem;
        color: var(--color-text-secondary);
    }


    .character-selector {
        margin: 1rem 0;
    }

    .character-checkboxes {
        display: flex;
        flex-wrap: wrap;
        gap: 1rem;
    }

    .character-checkbox {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        cursor: pointer;
    }

    .character-checkbox input[type="checkbox"] {
        width: 1rem;
        height: 1rem;
    }

    .non-present-checkbox {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        width: fit-content;
        cursor: pointer;
    }

    .non-present-checkbox input[type="checkbox"] {
        width: 1rem;
        height: 1rem;
        cursor: pointer;
    }

    .non-present-checkbox label {
        cursor: pointer;
    }


</style> 
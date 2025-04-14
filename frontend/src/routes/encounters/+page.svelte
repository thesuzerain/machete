<script lang="ts">
    import { requireAuth } from '$lib/guards/auth';
    import { page } from '$app/stores';
    import { onMount } from 'svelte';
    import type { Character } from '$lib/types/types';
    import LibrarySelector from '$lib/components/selectors/LibrarySelector.svelte';
    import EncounterCreator from "$lib/components/encounter/EncounterCreator.svelte";
    import { id } from 'date-fns/locale';
    import { 
        getCreatureExperienceFromLevel, 
        getSeverityFromRawExperience, 
        getRewardForLevelSeverity,
        EncounterDifficulty, 
        getSeverityFromFinalExperience,
        getHazardExperienceFromLevel


    } from '$lib/utils/encounter';
    import type { Encounter } from '$lib/types/encounters';
    import { getFullUrl } from '$lib/types/library';
    import { fade } from 'svelte/transition';
    import { FontAwesomeIcon } from '@fortawesome/svelte-fontawesome';
    import {
    type IconLookup,
    type IconDefinition,
    findIconDefinition,
    library
  } from '@fortawesome/fontawesome-svg-core'
  import {
    faLink,
  } from '@fortawesome/free-solid-svg-icons'
  import { API_URL } from '$lib/config';
  import { encounterStore } from '$lib/stores/encounters';
  import { campaignStore, selectedCampaignStore } from '$lib/stores/campaigns';
  import { creatureStore, hazardStore, itemStore } from '$lib/stores/libraryStore';
  import { characterStore } from '$lib/stores/characters';
  import { campaignSessionStore } from '$lib/stores/campaignSessions';
    import Card from '$lib/components/core/Card.svelte';
    import Button from '$lib/components/core/Button.svelte';
    import ConfirmationModal from '$lib/components/modals/ConfirmationModal.svelte';
    import Modal from '$lib/components/core/Modal.svelte';

library.add(faLink)

    let loading = $state(true);
    let error: string | null = $state(null);
    
    let encounterCreator: EncounterCreator;

    let hideAccomplishments = $state(true);

    // Variables for encounter display
    let encountersListClosed = $state(false);
    let encounterOpenStates: { [key: number]: boolean } = $state({});
    let encounterFilter = $state('');
    let encounterSort: 'name' | 'level' | 'xp' = $state('name');
    let sortDirection: 'asc' | 'desc' = $state('asc');

    // Form values for editing/completing encounters
    let editingEncounter: Encounter | null = $state(null);
    let linkingEncounter: Encounter | null = $state(null);
    let selectedLinkingSession: number | null = $state(null);

    // Subscribe to the stores
    let encounters = $derived($encounterStore);
    let campaigns = $derived( $campaignStore);
    let libraryEnemies = $derived($creatureStore);
    let libraryHazards = $derived($hazardStore);
    let libraryItems = $derived($itemStore);
    let globalCampaignId = $derived($selectedCampaignStore);
    let campaignSessions = $derived($campaignSessionStore.get(globalCampaignId || 0) || []);


    // Default session (pass to this page with a query parameter) (to attach to an encounter)
    // TODO: Svelte solution for parsing query parameters to a page?
    let chosenSessionId : number | null = $state(null);
    let sessionIdString = $page.url.searchParams.get('sessionId');
    if (sessionIdString) {
        chosenSessionId = parseInt(sessionIdString);
    }

    // Return to this session after creating an encounter
    // TODO: Svelte solution for parsing query parameters to a page?
    // TODO: Is there ever a case where this and chosenSessionId are different?
    let returnToSessionId : number | null = $state(null);
    let returnToSessionIdString = $page.url.searchParams.get('returnToSessionId');
    if (returnToSessionIdString) {
        returnToSessionId = parseInt(returnToSessionIdString);
    }

    let deletingEncounter: number | null = $state(null);
    let deletingEncounterName: string | null = $derived.by(() => {
        if (deletingEncounter === null) return null;
        return encounters.find(e => e.id === deletingEncounter)?.name || null;
    });
    
    async function deleteEncounter(id: number | null) {
        if (id === null) return;
        await encounterStore.deleteEncounter(id);
        deletingEncounter = null;
    }

    let sessionIx = $derived.by(() => {
        let sessionIx : Map<number, number> = new Map();
        campaignSessions.forEach((session, ix) => {
            sessionIx.set(session.id, ix);
        });
        return sessionIx;
    });

    // Fetch campaigns data
    async function fetchCampaigns() {
        await campaignStore.fetchCampaigns();
    }

    async function loadLibraryData() {
        try {
            // TODO: This pattern is repeated in multiple places, consider refactoring
            // Load any enemies that are in current encounters
            const enemyIds = new Set(
                encounters.flatMap(e => e.enemies ?? []).map((e) => e?.id)
            );
            
            if (enemyIds.size > 0) {
                await creatureStore.fetchEntities({
                    ids: Array.from(enemyIds).join(',')
                })
            }

            // Load any hazards that are in current encounters
            const hazardIds = new Set(
                encounters.flatMap(e => e.hazards ?? [])
            );
            if (hazardIds.size > 0) {
                await hazardStore.fetchEntities({
                    ids: Array.from(hazardIds).join(',')
                });
            }

            // Load any items that are in current encounters
            const itemIds = new Set(
                encounters.flatMap(e => e.treasure_items ?? [])
            );
            if (itemIds.size > 0) {
                await itemStore.fetchEntities({
                    ids: Array.from(itemIds).join(',')
                }); 
            }
        } catch (e) {
            console.error(e);
            error = e instanceof Error ? e.message : 'Failed to load library data';
        }
    }

    onMount(async () => {
        requireAuth();

        try {
            await Promise.all([
                fetchEncounters(),
                fetchCampaigns(),
            ]);
            await loadLibraryData();

            // TODO: Svelte solution for parsing query parameters to a page?
            let encounterIdString = $page.url.searchParams.get('encounterId');
            if (encounterIdString) {
                // Load the encounter into the editor.
                // editingEncounteris bound to EncounterCreator
                editingEncounter = encounters.find(e => e.id === parseInt(encounterIdString)) || null;
            }

        } catch (e) {
            console.error(e);
            error = e instanceof Error ? e.message : 'An error occurred';
        } finally {
            loading = false;
        }
    });

    function getEnemyDetails(id: number) {
        return libraryEnemies.entities.get(id);
    }

    function getHazardDetails(id: number) {
        return libraryHazards.entities.get(id);
    }

    function getItemDetails(id: number) {
        return libraryItems.entities.get(id);
    }

    async function fetchEncounters() {
        await encounterStore.fetchEncounters();
    }

    // Add this reactive statement to sort and filter encounters
    let filteredAndSortedEncounters = $derived(encounters
    .filter(enc => hideAccomplishments ? (enc.encounter_type != 'accomplishment' && enc.encounter_type != 'unknown' && enc.encounter_type != 'rewardInitialization') : true)
    .filter(enc => enc.name.toLowerCase().includes(encounterFilter.toLowerCase()))
    .sort((a, b) => {
            const direction = sortDirection === 'asc' ? 1 : -1;
            switch (encounterSort) {
                case 'name':
                    return direction * a.name.localeCompare(b.name);
                // TODO: Other sorts require not draftEncounter but encounter-specific data
                case 'level':
                    return direction * (a.party_level - b.party_level);
                case 'xp':
                    return direction * (a.total_experience - b.total_experience);
                default:
                    return 0;
            }
        })); 

    // TODO: modularize, along with css classes
    function getClassForDifficulty(difficulty: EncounterDifficulty): string {
        switch (difficulty) {
            case 'Trivial':
                return 'difficulty-trivial';
            case 'Low':
                return 'difficulty-low';
            case 'Moderate':
                return 'difficulty-moderate';
            case 'Severe':
                return 'difficulty-severe';
            case 'Extreme':
                return 'difficulty-extreme';
            default:
                return '';
        }
    }

    function getAdjustmentName(adjustment: number): string {
        if (adjustment === 0) return 'Normal';
        return adjustment > 0 ? 'Elite' : 'Weak';
    }

    function linkEncounterToSession(encounter: Partial<Encounter>, sessionId: number | null) {
        if (!encounter.id || !globalCampaignId) return;
        if (sessionId && globalCampaignId) {
            campaignSessionStore.linkEncounterToSession(globalCampaignId, sessionId, encounter.id);
        } else {
            encounterStore.unlinkEncounterFromSession(encounter.id);
        }
    }

    let encounterEditor : HTMLDivElement;
    const scrollToEncounterEditor = async () => {
        encounterEditor.scrollIntoView({
            behavior: 'smooth',
            block: 'start',
            inline: 'nearest'
        });
    }; 
    
</script>

<div class="encounters-page">
    <h1>Campaign Encounters</h1>
    {#if error}
        <div class="error">{error}</div>
    {/if}

    <div class="creator">
        <div bind:this={encounterEditor}>
            <EncounterCreator  bind:editingEncounter bind:chosenSessionId bind:returnToSessionId bind:this={encounterCreator} />
        </div>
</div>
    {#if loading}
        <div class="loading">Loading encounters...</div>
    {:else}
        <Card bind:collapsed={encountersListClosed}>
            <div slot="header">
                <h2>
                    Existing Encounters ({filteredAndSortedEncounters.length})
                </h2>
            </div>
                <div class="filter-sort">
                    <input
                        type="text"
                        placeholder="Filter encounters..."
                        bind:value={encounterFilter}
                        class="filter-input"
                    />
                    <div class="sort-controls">
                        <select bind:value={encounterSort}>
                            <option value="name">Sort by Name</option>
                            <option value="level">Sort by Level</option>
                            <option value="xp">Sort by XP</option>
                        </select>
                        <Button colour='white' onclick={() => sortDirection = sortDirection === 'asc' ? 'desc' : 'asc'}>
                           {sortDirection === 'asc' ? '↑' : '↓'}

                        </Button>
   
                    </div>
                </div>
                <div class="hide-accomplishments">
                    <input type="checkbox" bind:checked={hideAccomplishments} />
                    <span>Hide Accomplishments</span>
                </div>
            <Card background="light">
                {#each filteredAndSortedEncounters as encounter (encounter.id)}
                <Card bind:collapsed={
                        () => encounterOpenStates[encounter.id]  ?? true,
                        (val) => encounterOpenStates[encounter.id] = val}
                    >
                    <div slot="header" class="encounter-summary">
                        <h3>{encounter.name}</h3>
                        <div class="encounter-meta">
                            {#if encounter.session_id}
                                <span class="status linked">Linked: Session {sessionIx.get(encounter.session_id)}</span>
                            {:else}
                                <span class="status prepared">Prepared</span>
                            {/if}
                            <span class="xp">XP: {encounter.total_experience} (<span class="{getClassForDifficulty(getSeverityFromFinalExperience(encounter.total_experience, encounter.extra_experience))}">{getSeverityFromFinalExperience(encounter.total_experience, encounter.extra_experience).toWellFormed()}</span>)</span>
                            <span class="party">Level {encounter.party_level} ({encounter.party_size} players)</span>
                        </div>
                    </div>
                    <!-- TODO: You have an encounter viewer modal, switch it out for this-->
                    <div class="encounter-details">
                        <p>{encounter.description}</p>
                        
                        <div class="details">
                            {#if encounter.enemies}
                            <div class="detail-section">
                                <h4>Enemies ({encounter.enemies.length})</h4>
                                <ul>
                                    {#each encounter.enemies as encounterEnemy : EncounterEnemy}
                                        {#if getEnemyDetails(encounterEnemy.id)}
                                            <li>{getEnemyDetails(encounterEnemy.id)?.name} 
                                                {#if encounterEnemy.level_adjustment !== 0}
                                                    ({getAdjustmentName(encounterEnemy.level_adjustment)})
                                                {/if}
                                                (Level {(getEnemyDetails(encounterEnemy.id)?.level || 0) + encounterEnemy.level_adjustment})
                                                (XP: {getCreatureExperienceFromLevel(encounter.party_level, getEnemyDetails(encounterEnemy.id)?.level || 0)})</li>
                                        {/if}
                                    {/each}
                                </ul>
                            </div>
                            {/if}
                            {#if encounter.hazards}

                            <div class="detail-section">
                                <h4>Hazards ({encounter.hazards.length})</h4>
                                <ul>
                                    {#each encounter.hazards as hazardId}
                                        {@const hazardDetails = getHazardDetails(hazardId)}

                                        {#if hazardDetails}
                                            <li>{getHazardDetails(hazardId)?.name} (XP: {getHazardExperienceFromLevel(encounter.party_level, hazardDetails.level || 0, hazardDetails.complex)})</li>
                                        {/if}
                                    {/each}
                                </ul>
                            </div>
                            {/if}
                            {#if encounter.subsystem_type}
                            <div class="detail-section">
                                <h4>Subsystem</h4>
                                <p>Subsystem Type: {encounter.subsystem_type}</p>
                                <ul>
                                    {#each encounter.subsystem_checks || [] as check}
                                        <li> {check.name} 

                                            ({#each check.roll_options as roll, i}
                                            {roll.skill} DC {roll.dc}{#if i < check.roll_options.length - 1},&nbsp;{/if} 
                                            {/each})

                                        </li>
                                        
                                        
                                    {/each}
                                </ul>
                            </div>


                            {/if}

                            <div class="detail-section">
                                <h4>Treasure</h4>
                                <p>Currency: {encounter.treasure_currency}</p>
                                <ul>
                                    {#each encounter.treasure_items as itemId}
                                        {#if getItemDetails(itemId)}
                                            <li>{getItemDetails(itemId)?.name}</li>
                                        {/if}
                                    {/each}
                                </ul>
                            </div>
                        </div>
                        <div class="actions">

                        {#if !encounter.session_id}

                        <Button colour='green' onclick={() => linkingEncounter = encounter}>
                            Link to session
                        </Button>
                        
                        {:else}
                        
                
                        <Button colour='red' onclick={() => linkEncounterToSession(encounter, null)}>
                            Unlink from session
                        </Button>
                        {/if}

                                <Button colour='blue' onclick={() => {editingEncounter = encounter; scrollToEncounterEditor()}}>
                                    Edit
                                </Button>
                                
                            <Button colour='blue' onclick={() =>  encounterCreator.loadEncounterCopyToDraft(encounter)}>
                                Clone into draft
                            </Button>

                        <Button colour='red' onclick={() => deletingEncounter = encounter.id}>
                            Delete
                        </Button>


                            </div>
                            
                    </div>

                </Card>
                {/each}
            </Card>
        </Card>
    {/if}
    
</div>

<ConfirmationModal show={!!deletingEncounter && !!deletingEncounterName}
 on:confirm={() => deleteEncounter(deletingEncounter)} on:close={() => deletingEncounter = null}
 confirmationString="Delete"
>

    <p>Are you sure you want to delete this encounter?</p>
    <p>It will be deleted from any attached sessions and campaigns.</p>

</ConfirmationModal>

<Modal show={!!linkingEncounter} closeButton on:close={() => linkingEncounter = null}>
    <div slot="header">
    <h2>Link Encounter to Session</h2>
    </div>

    <p>Choose a session to link this encounter to:</p>

    <select bind:value={selectedLinkingSession}>
        <option value={null}>Select a session...</option>
        {#each campaignSessions as session, ix}
            <option value={session.id}>Session {ix}: {session.name}</option>
        {/each}
    </select>

    <div class="modal-actions">

        <Button 
            onclick={() => linkingEncounter = null}
        >
            Cancel
</Button>

        <Button 
            colour='green'
            disabled={selectedLinkingSession === null}
            onclick={() => {
                if (linkingEncounter === null || selectedLinkingSession === null) return;
                linkEncounterToSession(linkingEncounter, selectedLinkingSession);
                linkingEncounter = null;
            }}
        >
            Link
        </Button>
    </div>

</Modal>


<style>
    .encounters-page {
        padding: 2rem;
        max-width: 1200px;
        margin: 0 auto;
    }

    .creator {
        margin-bottom: 2rem;
    }

    .encounter-summary {
        display: flex;
        align-items: center;
        gap: 2rem;
        flex: 1;
    }

    .encounter-summary h3 {
        margin: 0;
        min-width: 200px;
    }

    .encounter-meta {
        display: flex;
        gap: 2rem;
        align-items: center;
        font-size: 0.875rem;
        color: var(--color-text-secondary);
    }

    .encounter-details {
        padding: 1.5rem;
    }

    .status {
        padding: 0.25rem 0.75rem;
        border-radius: 999px;
        font-size: 0.75rem;
        font-weight: 500;
        text-transform: uppercase;
        letter-spacing: 0.05em;
    }

    .status.prepared { 
        background: #dbeafe; 
        color: #1e40af; 
    }

    .status.linked { 
        background: #dcfce7; 
        color: #166534; 
    }


    .filter-sort {
        margin-bottom: 1.5rem;
        padding: 1rem;
        border-radius: 4px;
        display: flex;
        gap: 1rem;
        align-items: center;
    }

    .filter-input {
        flex: 1;
        padding: 0.5rem 1rem;
        border: 1px solid #e5e7eb;
        border-radius: 4px;
        font-size: 0.875rem;
    }

    .sort-controls {
        display: flex;
        gap: 0.5rem;
        align-items: center;
    }

    .detail-section {
        margin-bottom: 1.5rem;
    }

    .detail-section:last-child {
        margin-bottom: 0;
    }

    .detail-section h4 {
        margin-bottom: 0.5rem;
        color: #374151;
    }

    .detail-section ul {
        list-style: none;
        padding: 0;
        margin: 0;
    }

    .detail-section li {
        padding: 0.25rem 0;
        color: var(--color-text-secondary);
    }

    .actions {
        display: flex;
        gap: 0.75rem;
        margin-top: 1.5rem;
        padding-top: 1rem;
        border-top: 1px solid var(--color-bg-light-raised-border);
    }

    .modal-actions {
        display: flex;
        gap: 1rem;
        margin-top: 1.5rem;
        justify-content: flex-end;
    }


    .difficulty-trivial {
        color: var(--color-difficulty-trivial);
    }

    .difficulty-low {
        color: var(--color-difficulty-low);
    }

    .difficulty-moderate {
        color: var(--color-difficulty-moderate);
    }

    .difficulty-severe {
        color: var(--color-difficulty-severe);
    }

    .difficulty-extreme {
        color: var(--color-difficulty-extreme);
    }

    .hide-accomplishments {
        margin-top: 1rem;
        width: fit-content;
        display: flex;
        flex-direction: row;
        align-items: center;  
        justify-content: flex-start;
        gap: 0.5rem;
        font-size: 0.875rem;
        color: var(--color-text-secondary);
    }

    .hide-accomplishments input {
        margin-right: 0.5rem;
    }

    .hide-accomplishments span {
        font-size: 0.875rem;
        color: var(--color-text-secondary);
        white-space: nowrap;
    }



</style> 
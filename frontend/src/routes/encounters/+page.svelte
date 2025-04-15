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
    import EncounterList from '$lib/components/encounter/EncounterList.svelte';
    import EncounterViewer from '$lib/components/encounter/EncounterViewerModal.svelte';

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


    async function fetchEncounters() {
        await encounterStore.fetchEncounters();
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

    let encounterList : EncounterViewer | null = $state(null);
    let encounterFilterCounter = $state(0);
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
                    Existing Encounters ({encounterFilterCounter})
                </h2>
            </div>
            <div>
                <EncounterList onupdatefilter={(es) => {encounterFilterCounter = es.length}} let:encounter>
                    {#if !encounter.session_id}
                    <Button colour='green' onclick={() => linkingEncounter = encounter}>
                        Link to session
                    </Button>
                    {:else}
                    <Button colour='red' onclick={() => linkEncounterToSession(encounter, null)}>
                        Unlink from session
                    </Button>
                {/if}
    
                        <Button colour='blue' onclick={() => {editingEncounter = encounter; scrollToEncounterEditor() }}>

                            Edit
                        </Button>
                        
                    <Button colour='blue' onclick={() => {encounterCreator.loadEncounterCopyToDraft(encounter); scrollToEncounterEditor() }}>
                        Clone into draft
                    </Button>
    
                <Button colour='red' onclick={() => deletingEncounter = encounter.id}>
                    Delete
                </Button>
    

                </EncounterList>
            </div>
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
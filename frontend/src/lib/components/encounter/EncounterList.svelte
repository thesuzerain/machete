<script lang="ts">
    import { goto } from "$app/navigation";
    // This is a Svelte component for displaying a list of encounters in a campaign.
    // Items passed in slot are displayed as buttons for each encounter.

    import {
        campaignStore,
        selectedCampaignStore,
    } from "$lib/stores/campaigns";
    import { campaignSessionStore } from "$lib/stores/campaignSessions";
    import { encounterStore } from "$lib/stores/encounters";
    import {
        creatureStore,
        hazardStore,
        itemStore,
    } from "$lib/stores/libraryStore";
    import type { Encounter } from "$lib/types/encounters";
    import {
        getCreatureExperienceFromLevel,
        getHazardExperienceFromLevel,
        type EncounterDifficulty,
    } from "$lib/utils/encounter";
    import Button from "../core/Button.svelte";
    import Card from "../core/Card.svelte";
    import Modal from "../core/Modal.svelte";
    import ConfirmationModal from "../modals/ConfirmationModal.svelte";
    import EncounterSummary from "./EncounterSummary.svelte";

    interface Props {
        onupdatefilter?: (encounters: Encounter[]) => void;
        restrictToCampaign?: boolean;
        forceHideUnlinked?: boolean;
    }

    let { onupdatefilter = () => {}, forceHideUnlinked = false, restrictToCampaign = false }: Props =
        $props();

    let hideAccomplishments = $state(true);
    let hideUnlinked = $state(forceHideUnlinked ? true : false);
    let hideOtherCampaigns = $state(true);

    // Subscribe to the stores
    let campaigns = $derived($campaignStore);
    let libraryEnemies = $derived($creatureStore);
    let libraryHazards = $derived($hazardStore);
    let libraryItems = $derived($itemStore);
    let globalCampaignId = $derived($selectedCampaignStore);
    let campaignSessions = $derived(
        $campaignSessionStore.get(globalCampaignId || 0) || [],
    );

    let encounters = $derived.by(() => {
        let innerEncounters = $encounterStore;
        if (restrictToCampaign) {
            innerEncounters = innerEncounters.filter(
                (encounter) => encounter.session_id ? campaignSessions.some((session) => session.id === encounter.session_id) : false,
            );
        }
        return innerEncounters;
    });


    let editingEncounter: Encounter | null = $state(null);
    let linkingEncounter: Encounter | null = $state(null);
    let selectedLinkingSession: number | null = $state(null);

    // Variables for encounter display
    let encountersListClosed = $state(false);
    let encounterOpenStates: { [key: number]: boolean } = $state({});
    let encounterFilter = $state("");
    let encounterSort: "name" | "level" | "xp" = $state("name");
    let sortDirection: "asc" | "desc" = $state("asc");

    // Add this reactive statement to sort and filter encounters
    let filteredAndSortedEncounters = $derived(
        encounters
            .filter((enc) =>
                hideAccomplishments
                    ? enc.encounter_type != "accomplishment" &&
                      enc.encounter_type != "unknown" &&
                      enc.encounter_type != "rewardInitialization"
                    : true,
            )
            .filter((enc) => {
                if (hideOtherCampaigns) {
                    return (
                        enc.campaign_id === globalCampaignId ||
                        enc.campaign_id === null ||
                        enc.session_id === null
                    );
                }
                return true;
            })
            .filter((enc) => (hideUnlinked ? enc.session_id === null : true))
            .filter((enc) =>
                enc.name.toLowerCase().includes(encounterFilter.toLowerCase()),
            )
            .sort((a, b) => {
                const direction = sortDirection === "asc" ? 1 : -1;
                switch (encounterSort) {
                    case "name":
                        return direction * a.name.localeCompare(b.name);
                    // TODO: Other sorts require not draftEncounter but encounter-specific data
                    case "level":
                        return direction * (a.party_level - b.party_level);
                    case "xp":
                        return (
                            direction *
                            (a.total_experience - b.total_experience)
                        );
                    default:
                        return 0;
                }
            }),
    );

    // This will be called whenever the filteredAndSortedEncounters changes
    $effect(() => {
        if (onupdatefilter) {
            onupdatefilter(filteredAndSortedEncounters);
        }
    });

    let deletingEncounter: number | null = $state(null);

    let sessionIx = $derived.by(() => {
        let sessionIx: Map<number, number> = new Map();
        campaignSessions.forEach((session, ix) => {
            sessionIx.set(session.id, ix);
        });
        return sessionIx;
    });

    // TODO: modularize, along with css classes
    function getClassForDifficulty(difficulty: EncounterDifficulty): string {
        switch (difficulty) {
            case "Trivial":
                return "difficulty-trivial";
            case "Low":
                return "difficulty-low";
            case "Moderate":
                return "difficulty-moderate";
            case "Severe":
                return "difficulty-severe";
            case "Extreme":
                return "difficulty-extreme";
            default:
                return "difficulty-unknown";
        }
    }

    function getAdjustmentName(adjustment: number): string {
        if (adjustment === 0) return "Normal";
        return adjustment > 0 ? "Elite" : "Weak";
    }

    function getEnemyDetails(id: number) {
        return libraryEnemies.entities.get(id);
    }

    function getHazardDetails(id: number) {
        return libraryHazards.entities.get(id);
    }

    function getItemDetails(id: number) {
        return libraryItems.entities.get(id);
    }
</script>

<div class="encounters-list">
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
            <Button
                colour="white"
                onclick={() =>
                    (sortDirection = sortDirection === "asc" ? "desc" : "asc")}
            >
                {sortDirection === "asc" ? "↑" : "↓"}
            </Button>
        </div>
    </div>
    <div class="hide-accomplishments">
        <input type="checkbox" bind:checked={hideAccomplishments} />
        <span>Hide accomplishments</span>
    </div>
    {#if !forceHideUnlinked}
        <div class="hide-accomplishments">
            <input type="checkbox" bind:checked={hideUnlinked} />
            <span>Hide unlinked encounters</span>
        </div>
    {/if}
    <div class="hide-accomplishments">
        <input type="checkbox" bind:checked={hideOtherCampaigns} />
        <span>Hide other campaigns</span>
    </div>

    <Card background="grey">
        {#if filteredAndSortedEncounters.length === 0}
            <div class="empty-state">
                <p>
                    No {forceHideUnlinked ? "unlinked" : ""} encounters found.
                </p>
                <p>Try adjusting your filters or adding new encounters.</p>
            </div>
        {:else}
            {#each filteredAndSortedEncounters as encounter (encounter.id)}
                <Card
                    background="light"
                    bind:collapsed={
                        () => encounterOpenStates[encounter.id] ?? true,
                        (val) => (encounterOpenStates[encounter.id] = val)
                    }
                >
                    <div slot="header">
                        <EncounterSummary {encounter} size="title" />
                    </div>
                    <EncounterSummary {encounter} size="detailed">
                        <slot {encounter} />
                    </EncounterSummary>
                </Card>
            {/each}
        {/if}
    </Card>
</div>

<style>
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

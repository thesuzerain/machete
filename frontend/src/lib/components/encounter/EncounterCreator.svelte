<script lang="ts">
    import { requireAuth } from "$lib/guards/auth";
    import { page } from "$app/stores";
    import { onMount } from "svelte";
    import type { Character, LibraryEntity } from "$lib/types/types";
    import LibrarySelector from "$lib/components/selectors/LibrarySelector.svelte";
    import { id } from "date-fns/locale";
    import {
        getCreatureExperienceFromLevel,
        getSeverityFromRawExperience,
        EncounterDifficulty,
        getAdjustedExperienceFromPartySize,
        getHazardExperienceFromLevel,
    } from "$lib/utils/encounter";
    import type {
        Encounter,
        CreateOrReplaceEncounter,
        CreateEncounterFinalized,
        EncounterEnemy,
        CreateOrReplaceEncounterExtended,
        EncounterType,
        SubsystemCategory,
        SkillCheck,
    } from "$lib/types/encounters";
    import {
        getFullUrl,
        getFullUrlWithAdjustment,
        type LibraryCreature,
        type LibraryEntityType,
        type LibraryHazard,
        type LibraryItem,
    } from "$lib/types/library";
    import { fade } from "svelte/transition";
    import { FontAwesomeIcon } from "@fortawesome/svelte-fontawesome";
    import {
        type IconLookup,
        type IconDefinition,
        findIconDefinition,
        library,
    } from "@fortawesome/fontawesome-svg-core";
    import { faLink } from "@fortawesome/free-solid-svg-icons";
    import { API_URL } from "$lib/config";
    import { encounterStore } from "$lib/stores/encounters";
    import {
        campaignStore,
        selectedCampaignStore,
    } from "$lib/stores/campaigns";
    import { writable, type Writable } from "svelte/store";
    import {
        creatureStore,
        hazardStore,
        itemStore,
    } from "$lib/stores/libraryStore";
    import { characterStore } from "$lib/stores/characters";
    import { campaignSessionStore } from "$lib/stores/campaignSessions";
    import { goto } from "$app/navigation";
    import { skills } from "$lib/types/types";
    import BrowseLibraryModal from "../modals/BrowseLibraryModal.svelte";
    import Card from "../core/Card.svelte";
    import Button from "../core/Button.svelte";
    import EncounterLibraryItemSelector from "./EncounterLibraryItemSelector.svelte";
    import ConfirmationModal from "../modals/ConfirmationModal.svelte";
    import EncounterXpCalculatorInfoModal from "../modals/EncounterXpCalculatorInfoModal.svelte";
    import EncounterDifficultyBar from "./EncounterDifficultyBar.svelte";

    interface Props {
        editingEncounter: Encounter | null;
        chosenSessionId: number | null;
        returnToSessionId: number | null;
    }
    let {
        editingEncounter = $bindable(),
        chosenSessionId = $bindable(),
        returnToSessionId = $bindable(),
    }: Props = $props();

    const localStorageKey = "draftEncounter";

    library.add(faLink);

    let loading = $state(true);
    let error: string | null = $state(null);
    let libraryModal : BrowseLibraryModal | null = $state(null);

    // Add new state for auto-saving
    let saveTimeout: NodeJS.Timeout;
    const AUTOSAVE_DELAY = 250;

    // Add subsystem state variables
    let skillChecks = $state<SkillCheck[]>([]);
    let subsystemSectionClosed = $state(false);

    // Modify the wipEncounter state to include encounter_type and subsystem fields
    let wipEncounter: CreateOrReplaceEncounter = $state({
        name: "",
        description: "",
        encounter_type: "combat",
        enemies: [],
        hazards: [],
        treasure_items: [],
        treasure_currency: 0,
        extra_experience: 0,
        party_level: 1,
        party_size: 4,
        subsystem_type: "chase",
        subsystem_checks: [],
    });

    let lastEncounterSet : CreateOrReplaceEncounter | null = $state(null);
    function setWipEncounterAs(encounter: Encounter) {
        if (lastEncounterSet && lastEncounterSet === encounter) {
            return;
        }
        wipEncounter = {
            name: encounter.name,
            description: encounter.description,
            encounter_type: encounter.encounter_type || "combat",
            enemies: encounter.enemies?.map((e) => ({
                id: e.id,
                level_adjustment: e.level_adjustment,
            })),
            hazards: encounter.hazards || [],
            treasure_items: encounter.treasure_items,
            treasure_currency: encounter.treasure_currency,
            extra_experience: encounter.extra_experience,
            party_level: encounter.party_level,
            party_size: encounter.party_size,
            subsystem_type: encounter.subsystem_type || "chase",
            subsystem_checks: encounter.subsystem_checks || [],
        };
        lastEncounterSet = encounter;

        // Set session id
        chosenSessionId = encounter.session_id || null;

        // Update local skillChecks for UI
        skillChecks = encounter.subsystem_checks || [];
    }
    if (editingEncounter) {
        setWipEncounterAs(editingEncounter);
    }

    // TODO: Effect is a smell, refactor when possible. (But needs to update when editingEncounter updates from inside here or elsewhere)
    $effect(() => {
        if (editingEncounter) {
            setWipEncounterAs(editingEncounter);
        }
    });

    // Add these variables near the top of the script section, with the other state variables
    let enemiesSectionClosed = $state(false);
    let hazardsSectionClosed = $state(false);
    let treasureSectionClosed = $state(false);

    let showUpdateEncounterWarningModal = $state(false);

    // Subscribe to the stores
    let encounters = $derived($encounterStore);
    let libraryEnemies = $derived($creatureStore);
    let libraryHazards = $derived($hazardStore);
    let libraryItems = $derived($itemStore);
    let selectedCampaignId = $derived($selectedCampaignStore);
    let campaignSessions = $derived.by(() => {
        if (selectedCampaignId) {
            return $campaignSessionStore.get(selectedCampaignId) || null;
        } else {
            return null;
        }
    });

    let chosenSessionIndex = $derived.by(() => {
        if (campaignSessions && chosenSessionId) {
            return campaignSessions.findIndex((s) => s.id === chosenSessionId);
        } else {
            return -1;
        }
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
                    ids: Array.from(enemyIds).join(","),
                });
            }

            // Load any hazards that are in current encounters
            const hazardIds = new Set(
                encounters.flatMap(e => e.hazards ?? [])
            );
            if (hazardIds.size > 0) {
                await hazardStore.fetchEntities({
                    ids: Array.from(hazardIds).join(","),
                });
            }

            // Load any items that are in current encounters
            const itemIds = new Set(
                encounters.flatMap(e => e.treasure_items ?? [])
            );
            if (itemIds.size > 0) {
                await itemStore.fetchEntities({
                    ids: Array.from(itemIds).join(","),
                });
            }
        } catch (e) {
            console.error(e);
            error =
                e instanceof Error ? e.message : "Failed to load library data";
        }
    }

    export function loadEncounterCopyToDraft(encounter: Encounter) {
        editingEncounter = null;
        setWipEncounterAs(encounter);
    }

    onMount(async () => {
        try {
            // First check for any in-progress draft encounter, if we are not editing an existing one
            const inProgressRaw = localStorage.getItem(localStorageKey);
            // Parse the encounter from localStorage
            let inProgress : CreateOrReplaceEncounter | null = null;
            if (inProgressRaw) {
                try {
                    inProgress = JSON.parse(inProgressRaw);
                } catch (e) {
                    console.error("Failed to parse draft encounter:", e);
                }
            }

            // If we have an in-progress encounter, set it as the wipEncounter
            if (inProgress && !editingEncounter) {
                wipEncounter = inProgress;
            }

            // Setting sessions
            if (selectedCampaignId) {
                await campaignSessionStore.fetchCampaignSessions(
                    selectedCampaignId,
                );

                // If we have a chosen session, set the number of players and party level to those in the encounter generator by default
                if (chosenSessionId) {
                    const session = campaignSessions?.find(
                        (s) => s.id === chosenSessionId,
                    );
                    if (session) {
                        wipEncounter.party_size = Object.keys(
                            session.compiled_rewards,
                        ).length;
                        wipEncounter.party_level = session.level_at_end; // Level at end by default. For an empty session, this will be the same as the start, but this allows it to sorta change mid-session if needed.
                    }
                }
            }

            // Then load other encounters and campaigns
            await Promise.all([
                fetchEncounters(),
                fetchCampaigns(),
                loadLibraryData(),
            ]);
        } catch (e) {
            error = e instanceof Error ? e.message : "An error occurred";
        } finally {
            loading = false;
        }
    });

    // Auto-save function
    async function autoSave() {
        console.log("Auto-saving draft encounter...");
        if (saveTimeout) clearTimeout(saveTimeout);

        // Do not autosave if we are editing an existing encounter
        if (editingEncounter) return;

        saveTimeout = setTimeout(async () => {
            try {
                localStorage.setItem(
                    localStorageKey,
                    JSON.stringify(wipEncounter),
                );
                console.log("Draft encounter auto-saved");
                            } catch (e) {
                error = e instanceof Error ? e.message : "Failed to save draft";
            }
        }, AUTOSAVE_DELAY);
    }

    function returnToSession(sessionId: number) {
        goto(`/campaigns?sessionId=${sessionId}`);
    }

    // Modify the createEncounter function
    async function createEncounter() {

        try {
            // Prepare the encounter data based on type
            const encounterData: CreateOrReplaceEncounterExtended = {
                ...wipEncounter,
                total_experience: totalEarnedXP,
                total_items_value: totalTreasure,
            };

            if (editingEncounter) {
                await encounterStore.updateEncounter(
                    editingEncounter.id,
                    encounterData,
                );

                // If we're editing an encounter and it's linked to a session, we need to update the session
                if (
                    editingEncounter.session_id &&
                    chosenSessionId &&
                    editingEncounter.session_id !== chosenSessionId
                ) {
                    // Unlink from old session
                    await encounterStore.unlinkEncounterFromSession(
                        editingEncounter.id,
                    );

                    // Link to new session
                    if (chosenSessionId && selectedCampaignId) {
                        await campaignSessionStore.linkEncounterToSession(
                            selectedCampaignId,
                            chosenSessionId,
                            editingEncounter.id,
                        );
                    }
                } else if (editingEncounter.session_id && !chosenSessionId) {
                    // Unlink from session
                    await encounterStore.unlinkEncounterFromSession(
                        editingEncounter.id,
                    );
                } else if (
                    !editingEncounter.session_id &&
                    chosenSessionId &&
                    selectedCampaignId
                ) {
                    // Link to new session
                    await campaignSessionStore.linkEncounterToSession(
                        selectedCampaignId,
                        chosenSessionId,
                        editingEncounter.id,
                    );
                }

                // Reset form
                editingEncounter = null;

                // Reset draft
                wipEncounter = {
                    name: "",
                    description: "",
                    encounter_type: "combat",
                    enemies: [],
                    hazards: [],
                    treasure_items: [],
                    extra_experience: 0,
                    treasure_currency: 0,
                    party_level: 1,
                    party_size: 4,
                    subsystem_type: "chase",
                    subsystem_checks: [],
                };

                // Reset localStorage
                localStorage.setItem(
                    localStorageKey,
                    JSON.stringify(wipEncounter),
                );

                skillChecks = [];
            } else {
                // Creating a new encounter
                const finalizedEncounter: CreateEncounterFinalized = {
                    ...encounterData,
                    session_id: chosenSessionId,
                };

                await encounterStore.addEncounter(finalizedEncounter);

                // Reset form
                wipEncounter = {
                    name: "",
                    description: "",
                    encounter_type: "combat",
                    enemies: [],
                    hazards: [],
                    treasure_items: [],
                    extra_experience: 0,
                    treasure_currency: 0,
                    party_level: 1,
                    party_size: 4,
                    subsystem_type: "chase",
                    subsystem_checks: [],
                };

                skillChecks = [];
            }

            // Reset localStorage
            localStorage.setItem(
                localStorageKey,
                JSON.stringify(wipEncounter),
            );

            // If we are editing an encounter, also, return to the page.
            // TODO: Can probably remove above resetting, but not sure what we want to keep here.
            if (returnToSessionId) {
                returnToSession(returnToSessionId);
            }
        } catch (e) {
            console.error("Error creating encounter:", e);
            error = e instanceof Error ? e.message : "An error occurred";
        }
    }

    function getEnemyDetails(id: number): LibraryCreature | null {
        return libraryEnemies.entities.get(id) || null;
    }

    function getHazardDetails(id: number): LibraryHazard | null {
        return libraryHazards.entities.get(id) || null;
    }

    function getItemDetails(id: number): LibraryItem | null {
        return libraryItems.entities.get(id) || null;
    }

    let totalTreasure: number = $derived.by(() => {
        let total = wipEncounter.treasure_currency || 0;
        wipEncounter.treasure_items.forEach((itemId) => {
            const item = getItemDetails(itemId);
            if (item && item.price) {
                total += item.price;
            }
        });
        return total;
    });

    async function fetchEncounters() {
        await encounterStore.fetchEncounters();
    }

    let subtotalXPEnemies: number = $derived.by(() => {
        return (wipEncounter.enemies || []).reduce((total, encounterEnemy) => {
            const enemy = getEnemyDetails(encounterEnemy.id);
            if (enemy?.level != undefined) {
                return (
                    total +
                    getCreatureExperienceFromLevel(
                        wipEncounter.party_level,
                        enemy.level + encounterEnemy.level_adjustment,
                    )
                );
            }
            return total;
        }, 0);
    });

    let subtotalXPHazards: number = $derived(
        (wipEncounter.hazards || []).reduce((total, hazardId) => {
            const hazard = getHazardDetails(hazardId);
            if (hazard?.level) {
                return (
                    total +
                    getHazardExperienceFromLevel(
                        wipEncounter.party_level,
                        hazard.level,
                        hazard.complex
                    )
                );
            }
            return total;
        }, 0),
    );

    async function pickDefaultCampaignForParty(newCampaignId: number | null) {
        if (newCampaignId) {
            const campaign = $campaignStore.get(newCampaignId);
            const level = campaign?.level || 1;
            await characterStore.fetchCharacters(newCampaignId); // Fetch characters for the selected campaign
            const characters = $characterStore.get(newCampaignId);
            if (characters) {
                // Set PartyConfig to the player count and level of the campaign
                wipEncounter.party_size = characters.length;
                wipEncounter.party_level = characters.reduce(
                    (max, char) => Math.max(max, level),
                    1,
                );
            }
        }
    }
    selectedCampaignStore.subscribe(pickDefaultCampaignForParty);

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

    function toggleEnemyAdjustment(enemyId: number) {
        let cycleOrder = new Map([
            [0, 1],
            [1, -1],
            [-1, 0],
        ]);
        if (!wipEncounter.enemies) return;
        let encounterEnemy = wipEncounter.enemies[enemyId];
        encounterEnemy.level_adjustment =
            cycleOrder.get(encounterEnemy.level_adjustment) || 0;
    }

    function getAdjustmentName(levelAdjustment: number) {
        switch (levelAdjustment) {
            case 1:
                return "Elite";
            case -1:
                return "Weak";
            default:
                return "Normal";
        }
    }

    function getAdjustmentColour(levelAdjustment: number) {
        switch (levelAdjustment) {
            case 1:
                return "red";
            case -1:
                return "green";
            default:
                return "grey";
        }
    }

    // Add reactive statement for difficulty calculation
    let encounterDifficulty: EncounterDifficulty = $derived(
        getSeverityFromRawExperience(
            subtotalXPEnemies + subtotalXPHazards,
            wipEncounter.party_size,
        ),
    );

    // Sum up the total XP for the encounter, adjusted by party
    // Does not include extra experience
    let subtotalXPPartyAdjusted: number = $derived(
        getAdjustedExperienceFromPartySize(
            subtotalXPEnemies + subtotalXPHazards,
            wipEncounter.party_size,
        ),
    );
    let adjustedXPAmount: number = $derived(
        subtotalXPPartyAdjusted - (subtotalXPEnemies + subtotalXPHazards),
    );
    let totalEarnedXP: number = $derived(
        subtotalXPPartyAdjusted + wipEncounter.extra_experience,
    );

    // Add reactive statements for auto-saving
    // TODO: Are these code smell usages of effect? I don't know enough Svelte yet to say so.
    $effect(() => {
        autoSave();
        if (
            wipEncounter.name &&
            wipEncounter.description &&
            wipEncounter.enemies &&
            wipEncounter.hazards &&
            wipEncounter.treasure_items &&
            wipEncounter.extra_experience
        ) {
            autoSave();
        }
    });

    // Handle encounter type change
    function handleEncounterTypeChange() {
        // Set new encounter type metadat to include needed fields if they don't exist
        if (!wipEncounter.enemies || !wipEncounter.hazards) {
            wipEncounter.enemies = [];
            wipEncounter.hazards = [];
        }

        if (
            !wipEncounter.subsystem_type ||
            !wipEncounter.subsystem_checks ||
            !wipEncounter.party_level ||
            !wipEncounter.party_size
        ) {
            skillChecks = [];
            wipEncounter.subsystem_type = "chase";
            wipEncounter.subsystem_checks = [];
        }

        // Save the draft with the new encounter type
        localStorage.setItem(
            localStorageKey,
            JSON.stringify(wipEncounter),
        );
    }

    // Update addSkillCheck function
    function addSkillCheck() {
        const newSkillCheck: SkillCheck = {
            name: `Check ${(wipEncounter.subsystem_checks?.length || 0) + 1}`,
            roll_options: [
                {
                    skill: "Acrobatics",
                    dc: 15,
                },
            ],
            vp: 5,
        };

        wipEncounter.subsystem_checks = [
            ...(wipEncounter.subsystem_checks || []),
            newSkillCheck,
        ];
    }

    // Update addRollOption function
    function addRollOption(checkIndex: number) {
        if (!wipEncounter.subsystem_checks?.[checkIndex]) return;

        wipEncounter.subsystem_checks[checkIndex].roll_options = [
            ...wipEncounter.subsystem_checks[checkIndex].roll_options,
            { skill: "Acrobatics", dc: 15 },
        ];
        wipEncounter.subsystem_checks = [...wipEncounter.subsystem_checks];
    }

    function resetToNewEncounter() {
        editingEncounter = null;
        chosenSessionId = null;
        wipEncounter = {
            name: "",
            description: "",
            encounter_type: "combat",
            enemies: [],
            hazards: [],
            treasure_items: [],
            extra_experience: 0,
            treasure_currency: 0,
            party_level: 1,
            party_size: 4,
            subsystem_type: "chase",
            subsystem_checks: [],
        };
    }

    let libraryTabs: LibraryEntityType[] = $state(["creature"]);
    let showLibraryModal = $state(false);
    let showExperienceInformationModal = $state(false);
    function openLibrary(entityType: LibraryEntityType) {
        //TODO: reset
        if (entityType === "creature") {
            libraryTabs = ["creature"];
        } else if (entityType === "hazard") {
            libraryTabs = ["hazard"];
        } else if (entityType === "item") {
            // TODO: Include spells for future use of scrolls, wands
            libraryTabs = ["item", "spell"];
        } else {
            return;
        }
        showLibraryModal = true;
    }

    // TODO: Edit this maybe with a better LibraryEntity definition
    async function addEntityFromLibrary(
        entityType: LibraryEntityType,
        entity: LibraryEntity,
    ) {
        // TODO: Maybe remove this? might be better to keep in library
        return;
        if (entityType === "creature") {
            wipEncounter.enemies = [
                ...(wipEncounter.enemies || []),
                { id: entity.id, level_adjustment: 0 },
            ];
        } else if (entityType === "hazard") {
            wipEncounter.hazards = [...(wipEncounter.hazards || []), entity.id];
        } else if (entityType === "item") {
            wipEncounter.treasure_items = [
                ...(wipEncounter.treasure_items || []),
                entity.id,
            ];
        }

        // TODO: Spells

        // TODO: patch request here
        // Do we want to do a patch request here? I dont think we need to
    }
</script>

<form>
    <Card>
        <div class="encounter-header">
            <h2>Create New Encounter</h2>
            <Button large colour="black" onclick={resetToNewEncounter}
                >Reset encounter editor</Button
            >
        </div>
        <div class="encounter-form-container">
            <Card background="light">
                <h3>Encounter Configuration</h3>

                <div class="form-group">
                    <label for="name">Name</label>
                    <input
                        type="text"
                        id="name"
                        class="name-input"
                        bind:value={wipEncounter.name}
                        required
                    />
                </div>

                <label for="description">Description</label>
                <div class="form-group">
                    <textarea
                        id="description"
                        bind:value={wipEncounter.description}
                    ></textarea>
                </div>
            </Card>
            <Card background="light">
                <h3>Party Configuration</h3>
                <div class="party-config-row">
                    <div class="party-config-input">
                        <label for="playerCount">Number of Players</label>
                        <input
                            type="number"
                            id="playerCount"
                            disabled={editingEncounter && chosenSessionId ? true : false}
                            bind:value={wipEncounter.party_size}
                            min="1"
                            max="6"
                        />
                    </div>
                    <div class="party-config-input">
                        <label for="partyLevel">Party Level</label>
                        <input
                            type="number"
                            id="partyLevel"
                            disabled={editingEncounter ? true : false}
                            bind:value={wipEncounter.party_level}
                            min="1"
                            max="20"
                        />
                    </div>
                </div>

                <div
                    class="difficulty-indicator {encounterDifficulty.toLowerCase()}"
                >
                <div>
                    <div>
                        Total earned XP: <b>{totalEarnedXP}</b>
                        ({subtotalXPEnemies} + {subtotalXPHazards} + {adjustedXPAmount}
                        + {wipEncounter.extra_experience})
                    </div>
                    {#if wipEncounter.encounter_type === "combat"}
                        This is a <b
                            class={getClassForDifficulty(encounterDifficulty)}
                            >{encounterDifficulty.toLowerCase()}</b
                        >
                        difficulty encounter for
                        <b>{wipEncounter.party_size}</b>
                        level <b>{wipEncounter.party_level}</b> players
                    {/if}
                </div>
                    <Button tight onclick={() => showExperienceInformationModal = true}> Learn more </Button>
                </div>
                <EncounterDifficultyBar experience={subtotalXPEnemies + subtotalXPHazards} partySize={wipEncounter.party_size} />
            </Card>
        </div>

        <!-- Encounter Type Selection -->
        <Card background="light">
            <h3>Encounter Type</h3>
            <div class="encounter-type-selector">
                <div class="encounter-type-options">
                    <label class="encounter-type-option">
                        <input
                            type="radio"
                            name="encounterType"
                            value="combat"
                            bind:group={wipEncounter.encounter_type}
                            on:change={handleEncounterTypeChange}
                        />
                        <span class="encounter-type-label">Combat</span>
                        <span class="encounter-type-description"
                            >Standard encounter with enemies and/or hazards</span
                        >
                    </label>

                    <label class="encounter-type-option">
                        <input
                            type="radio"
                            name="encounterType"
                            value="accomplishment"
                            bind:group={wipEncounter.encounter_type}
                            on:change={handleEncounterTypeChange}
                        />
                        <span class="encounter-type-label">Reward</span>
                        <span class="encounter-type-description"
                            >Just treasure and XP, no combat</span
                        >
                    </label>

                    <label class="encounter-type-option">
                        <input
                            type="radio"
                            name="encounterType"
                            value="subsystem"
                            bind:group={wipEncounter.encounter_type}
                            on:change={handleEncounterTypeChange}
                        />
                        <span class="encounter-type-label">Subsystem</span>
                        <!-- TODO: Social may not actually be a subsystem. Double check this- I think its actually a whole other thing with enemies..?-->
                        <span class="encounter-type-description"
                            >Chase, infiltration, research, or social challenge</span
                        >
                    </label>
                </div>
            </div>
        </Card>

        <!-- Subsystem section - Only shown for subsystem encounters -->
        {#if wipEncounter.encounter_type === "subsystem"}
            <Card background="light" collapsed={subsystemSectionClosed}>
                <div slot="header">
                    <h3>Subsystem Challenge</h3>
                </div>
                <h2>
                    TODO: Experience is not provided by subsystems, but will be
                    added as accomplishments are added (either simulatenously,
                    or as accomplishments directly).
                </h2>

                <div class="section-content" transition:fade>
                    <div class="form-group">
                        <label for="subsystemCategory">Challenge Type</label>
                        <select
                            id="subsystemCategory"
                            bind:value={wipEncounter.subsystem_type}
                        >
                            <option value="chase">Chase</option>
                            <option value="infiltration">Infiltration</option>
                            <option value="research">Research</option>
                        </select>
                    </div>

                    <div class="skill-checks-list">
                        {#each wipEncounter.subsystem_checks || [] as check, checkIndex}
                            <Card outlined>
                                <div class="skill-check-header">
                                    <input
                                        type="text"
                                        class="check-name-input"
                                        bind:value={check.name}
                                        placeholder="Check name"
                                    />
                                    <div class="vp-input-container">
                                        <label for="vp-{checkIndex}"
                                            >Victory Points:</label
                                        >
                                        <input
                                            type="number"
                                            id="vp-{checkIndex}"
                                            class="vp-input"
                                            bind:value={check.vp}
                                            min="0"
                                        />
                                    </div>
                                    <Button
                                        colour="red"
                                        onclick={() => {
                                            wipEncounter.subsystem_checks =
                                                wipEncounter.subsystem_checks?.filter(
                                                    (_, i) => i !== checkIndex,
                                                );
                                        }}>Remove Check</Button
                                    >
                                </div>

                                <div class="roll-options-list">
                                    {#each check.roll_options as option, optionIndex}
                                        <Card background="light" tight outlined>
                                            <div class="roll-option">
                                                <select
                                                    bind:value={option.skill}
                                                >
                                                    {#each skills as skill}
                                                        <option value={skill}
                                                            >{skill}</option
                                                        >
                                                    {/each}
                                                </select>

                                                <div class="dc-input-container">
                                                    <label
                                                        for="dc-{checkIndex}-{optionIndex}"
                                                        >DC:</label
                                                    >
                                                    <input
                                                        type="number"
                                                        id="dc-{checkIndex}-{optionIndex}"
                                                        class="dc-input"
                                                        bind:value={option.dc}
                                                        min="1"
                                                    />
                                                </div>

                                                <Button
                                                    colour="red"
                                                    onclick={() => {
                                                        check.roll_options =
                                                            check.roll_options.filter(
                                                                (_, i) =>
                                                                    i !==
                                                                    optionIndex,
                                                            );
                                                        wipEncounter.subsystem_checks =
                                                            [
                                                                ...(wipEncounter.subsystem_checks ||
                                                                    []),
                                                            ];
                                                    }}>Remove</Button
                                                >
                                            </div>
                                        </Card>
                                    {/each}

                                    <Button
                                        colour="blue"
                                        onclick={() =>
                                            addRollOption(checkIndex)}
                                        >Add Roll Option</Button
                                    >
                                </div>
                            </Card>
                        {/each}

                        <Button colour="blue" onclick={addSkillCheck}
                            >Add Skill Check</Button
                        >
                    </div>
                </div>
            </Card>
        {/if}

        {#if wipEncounter.encounter_type === "combat"}
            {#if wipEncounter.enemies}
                <Card background="light" bind:collapsed={enemiesSectionClosed}>
                    <div slot="header">
                        <h3>
                            Enemies ({wipEncounter.enemies.length}) - {subtotalXPEnemies}
                            XP
                        </h3>
                    </div>
                    <div class="section-content">
                        {#if wipEncounter.enemies}
                            <h4>Enemies</h4>

                            {#if libraryModal}
                                <EncounterLibraryItemSelector libraryObjectType='creature' {libraryModal} partyLevel={wipEncounter.party_level} bind:data={wipEncounter.enemies} />
                            {/if}                        
                        {/if}
                    </div>
                </Card>
            {/if}

            {#if wipEncounter.hazards}
                <Card background="light" bind:collapsed={hazardsSectionClosed}>
                    <div slot="header">
                        <h3>
                            Hazards ({wipEncounter.hazards.length}) - {subtotalXPHazards}
                            XP
                        </h3>
                    </div>
                    <div class="section-content">
                        <h4>Hazards</h4>

                        {#if libraryModal}
                            <EncounterLibraryItemSelector libraryObjectType='hazard' partyLevel={wipEncounter.party_level} {libraryModal} bind:data={wipEncounter.hazards} />
                        {/if}                        
                    </div>
                </Card>
            {/if}
        {/if}

        <Card background="light" bind:collapsed={treasureSectionClosed}>
            <div slot="header">
                <h3>
                    Treasure - {totalTreasure} gold
                </h3>
            </div>
            <div class="section-content">
                <div class="form-group-treasure-row">
                    <label for="currency">Currency</label>
                    <input
                        type="number"
                        id="currency"
                        class="currency-input"
                        bind:value={wipEncounter.treasure_currency}
                        min="0"
                    />
                </div>
                <div class="form-group-treasure-row">
                    <label for="currency">Experience</label>
                    <input
                        type="number"
                        id="currency"
                        class="currency-input"
                        bind:value={wipEncounter.extra_experience}
                    />
                </div>

                <h4>Items</h4>
                <div class="list-items">
                    {#if libraryModal}
                        <EncounterLibraryItemSelector libraryObjectType='item' partyLevel={wipEncounter.party_level} {libraryModal} bind:data={wipEncounter.treasure_items} />
                    {/if}
                </div>
            </div>
        </Card>

        <!-- Session selection -->
        {#if campaignSessions && campaignSessions.length > 0}
            <div class="session-selector">
                {#if editingEncounter && campaignSessions[chosenSessionIndex]}
                    <label for="sessionSelect"
                        >Linked to session {chosenSessionIndex + 1}: {campaignSessions[
                            chosenSessionIndex
                        ].name}</label
                    >

                    <Button
                        colour="red"
                        onclick={async () => {
                            await encounterStore.unlinkEncounterFromSession(
                                editingEncounter!.id,
                            );
                            chosenSessionId = null;
                        }}>Unlink from session</Button
                    >
                {:else}
                    <label for="sessionSelect">Add to Session:</label>
                    <select id="sessionSelect" bind:value={chosenSessionId}>
                        <option value={null}>None</option>
                        {#each campaignSessions as session, i}
                            <option value={session.id}
                                >Session {i}: {session.name}</option
                            >
                        {/each}
                    </select>
                {/if}
            </div>
        {/if}

        {#if editingEncounter} 
        <Button large onclick={() => {if (chosenSessionId) {
            showUpdateEncounterWarningModal = true;
        } else {
            createEncounter();
         }}} colour="blue"
            >Update Encounter</Button
        >
         {:else}
         <Button large onclick={() => createEncounter()} colour="blue"
            >Create Encounter</Button
        >

        {/if}
    </Card>
</form>

<BrowseLibraryModal
    bind:this={libraryModal}
    bind:show={showLibraryModal}
    allowedTabs={libraryTabs}
    bind:editingEncounter={wipEncounter}
/>

<ConfirmationModal
    bind:show={showUpdateEncounterWarningModal}
    on:confirm={() => {
        showUpdateEncounterWarningModal = false;
        createEncounter();
    }}
    on:cancel={() => {
        showUpdateEncounterWarningModal = false;
    }}
>You are updating a session-linked encounter. This will clear any item or gold assignments related to this encounter in the session. Are you sure? </ConfirmationModal>

<EncounterXpCalculatorInfoModal bind:show={showExperienceInformationModal} />

<style>
    .encounters-page {
        padding: 2rem;
        max-width: 1200px;
        margin: 0 auto;
    }

    .list-item {
        display: grid;
        grid-template-columns: auto minmax(200px, 1fr) auto auto auto auto;
        gap: 1rem;
        border-radius: 4px;
        align-items: center;
    }

    .entity-name {
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }

    .entity-xp,
    .entity-level {
        white-space: nowrap;
    }

    .encounters-list {
        display: flex;
        flex-direction: column;
        gap: 1rem;
        max-width: 100%;
    }

    .encounter-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        border-radius: 4px;
    }

    .list-items {
        display: flex;
        flex-direction: column;
        margin-bottom: 1rem;
        gap: 0.25rem;
    }

    .section-content {
        padding-top: 1rem;
    }

    .library-selector-container {
        margin-top: 1rem;
        display: flex;
        gap: 0.5rem;
    }

    .encounter-form-container {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 1rem;
    }

    .party-config-row {
        display: grid;
        grid-template-columns: 1fr 1fr;
        padding-top: 0.5rem;
    }

    .party-config-input {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    .session-selector {
        display: flex;
        gap: 1rem;
        margin: 1rem;
        align-items: center;
    }

    .difficulty-indicator {
        padding-left: 1rem;
        display: flex;
        justify-content: space-between;
    }

    .name-input {
        width: 100%;
        font-size: 1.2rem;
        font-family: inherit;
        border: 1px solid #e5e7eb;
        border-radius: 4px;
    }

    .currency-input {
        padding: 0.5rem;
        border: 1px solid #e5e7eb;
        border-radius: 4px;
        font-size: 1rem;
    }

    .form-group {
        display: flex;
        align-items: center;
        gap: 1rem;
    }

    .form-group-treasure-row {
        display: grid;
        grid-template-columns: 0.1fr 0.4fr 0.5fr;
        gap: 1rem;
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

    .encounter-type-selector {
        margin-bottom: 1.5rem;
    }

    .encounter-type-options {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
        gap: 1rem;
    }

    .encounter-type-option {
        display: flex;
        flex-direction: column;
        padding: 1rem;
        /* TODO: This should be consistent with the encounter type card- make it raised */
        border: 1px solid var(--color-bg-light-raised-border);
        border-radius: 0.5rem;
        cursor: pointer;
        transition: all 0.2s;
    }

    .encounter-type-option:hover {
        /* TODO: This should be consistent with the encounter type card- make it raised */
        background: var(--color-bg-hover);
    }

    .encounter-type-option input[type="radio"] {
        width: auto;
        margin-right: 0.5rem;
    }

    .encounter-type-label {
        font-weight: 600;
        margin-bottom: 0.25rem;
    }

    .encounter-type-description {
        font-size: 0.875rem;
        color: var(--color-text-secondary);
    }

    .skill-checks-list {
        display: flex;
        flex-direction: column;
        gap: 1rem;
        margin-top: 1rem;
    }

    .skill-check-header {
        display: grid;
        grid-template-columns: 1fr auto auto;
        gap: 1rem;
        align-items: center;
        padding-bottom: 1rem;
        border-bottom: 1px solid var(--color-bg-light-raised-border);
    }

    .check-name-input {
        font-size: 1rem;
        padding: 0.5rem;
        border: 1px solid #e5e7eb;
        border-radius: 0.375rem;
        width: 100%;
    }

    .vp-input-container {
        display: flex;
        align-items: center;
        gap: 0.5rem;
    }

    .vp-input {
        width: 5rem;
        padding: 0.5rem;
        border: 1px solid #e5e7eb;
        border-radius: 0.375rem;
    }

    .roll-options-list {
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
    }

    .roll-option {
        display: grid;
        grid-template-columns: 1fr auto auto;
        gap: 1rem;
        align-items: center;
    }

    .dc-input-container {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        white-space: nowrap;
    }

    .dc-input {
        width: 5rem;
        padding: 0.5rem;
        border: 1px solid #e5e7eb;
        border-radius: 0.375rem;
    }
</style>

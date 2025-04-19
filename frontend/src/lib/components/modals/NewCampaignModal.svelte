<script lang="ts">
    import { createEventDispatcher, onMount } from "svelte";
    import { fade } from "svelte/transition";
    import { API_URL } from "$lib/config";
    import type { Campaign, InsertInitialCampaignData } from "$lib/types/types";
    import { campaignStore } from "$lib/stores/campaigns";
    import { classStore, itemStore } from "$lib/stores/libraryStore";
    import LibrarySelector from "../selectors/LibrarySelector.svelte";
    import Modal from "../core/Modal.svelte";
    import Button from "../core/Button.svelte";
    import Card from "../core/Card.svelte";

    interface Props {
        show: boolean;
    }

    let { show = $bindable() }: Props = $props();

    interface InitialCharacter {
        name: string;
        class: number;
        gold: number;
        items: number[];
    }

    const dispatch = createEventDispatcher();

    let itemDetails = $derived($itemStore);

    let activeTab: "create" | "import" = $state("create");
    let name = $state("");
    let description = $state("");
    let importJson = $state("");
    let error: string | null = $state(null);
    let initName = $state("");
    let initDescription = $state("");
    let currentLevel = $state(1);
    let remainderXP = $state(0);
    let completedSessions = $state(0);
    let characters = $state<InitialCharacter[]>([]);
    let partyGold = $state(0);
    let partyItems = $state<number[]>([]);
    let totalSteps = $state(2);
    let currentStep = $state(1);

    // on load, load classstore
    onMount(async () => {
        await classStore.fetchEntities({});
    });

    function nextStep() {
        if (currentStep === 1) {
            if (!initName.trim()) {
                error = "Campaign name is required";
                return;
            }
        }
        error = null;
        currentStep += 1;
    }
    function prevStep() {
        currentStep -= 1;
    }
    function addCharacter() {
        characters = [
            ...characters,
            {
                name: "",
                class: 0,
                gold: 0,
                items: [],
                isCollapsed: false,
            },
        ];
    }
    function removeCharacter(index: number) {
        // TODO: remove character from list
        console.log("removeCharacter", index);
    }

    $effect(() => {
        if (show) {
            name = "";
            description = "";
            importJson = "";
        }
    });

    async function handleSubmit() {
        try {
            if (activeTab === "import") {
                try {
                    let json = JSON.parse(importJson);
                    // TODO: Validate JSON
                    if (!json) throw new Error("Unparseable JSON");
                    let id = await campaignStore.importCampaign(importJson);
                    await handleSuccess(id);
                } catch (e) {
                    console.log("Error importing campaign", e);
                    throw new Error("Could not import campaign", e);
                }
            } else if (activeTab === "create") {
                const url = `${API_URL}/campaign`;

                const ret = await campaignStore.addCampaign(
                    {
                        name: initName,
                        description: initDescription,
                        experience: currentLevel * 1000 + remainderXP,
                    },
                    {
                        gold: partyGold,
                        items: partyItems,
                        characters: characters.map((c) => ({
                            name: c.name,
                            class: c.class,
                            gold: c.gold,
                            items: c.items,
                        })),
                    },
                );

                await handleSuccess(ret.id);
            }
        } catch (e) {
            console.error("Error saving campaign:", e);
            error = e instanceof Error ? e.message : "An error occurred";
        }
    }

    async function handleSuccess(id: number) {
        await campaignStore.fetchCampaigns();
        dispatch("saved", id);
        closeModal();
    }

    function closeModal() {
        show = false;
        error = null;
        dispatch("close");
    }
</script>

<Modal bind:show bind:error closeButton>
    <div slot="header">
        <h2>New Campaign</h2>
    </div>
    <div class="modal-size">
        <div class="tabs">
            <button
                class="tab-button"
                class:active={activeTab === "create"}
                on:click={() => (activeTab = "create")}
            >
                Initialize Existing
            </button>

            <button
                class="tab-button"
                class:active={activeTab === "import"}
                on:click={() => (activeTab = "import")}
            >
                Import
            </button>
        </div>

        <form on:submit|preventDefault={handleSubmit}>
            {#if activeTab === "import"}
                <Card>
                    <label for="import">Campaign JSON</label>
                    <textarea
                        id="import"
                        bind:value={importJson}
                        rows="10"
                        placeholder="Paste your campaign JSON here..."
                        required
                    ></textarea>
                </Card>
                <Button onclick={handleSubmit} large colour="blue"
                    >Initialize Campaign</Button
                >
            {:else if activeTab === "create"}
                <Card tight>
                    <div class="wizard-progress">
                        {#each Array(totalSteps) as _, i}
                            <div
                                class="step"
                                class:active={currentStep === i + 1}
                                class:completed={currentStep > i + 1}
                            >
                                <div class="step-number">{i + 1}</div>
                                <div class="step-label">
                                    {i === 0
                                        ? "Campaign Info"
                                        : "Characters & Resources"}
                                </div>
                            </div>
                        {/each}
                    </div>
                </Card>

                <div class="wizard-content">
                    {#if currentStep === 1}
                        <!-- Step 1: Campaign Info -->
                        <Card>
                            <div class="form-group">
                                <label for="initName">Campaign Name</label>
                                <input
                                    type="text"
                                    id="initName"
                                    bind:value={initName}
                                    required
                                />
                            </div>

                            <div class="form-group">
                                <label for="initDescription">Description</label>
                                <textarea
                                    id="initDescription"
                                    bind:value={initDescription}
                                    rows="3"
                                ></textarea>
                            </div>

                            <div class="form-row">
                                <div class="form-group">
                                    <label for="currentLevel"
                                        >Current Level</label
                                    >
                                    <input
                                        type="number"
                                        id="currentLevel"
                                        bind:value={currentLevel}
                                        min="1"
                                        max="20"
                                        on:input={(e) => {
                                            const val = parseInt(
                                                e.currentTarget.value,
                                            );
                                            if (val < 1) currentLevel = 1;
                                            if (val > 20) currentLevel = 20;
                                        }}
                                        required
                                    />
                                </div>

                                <div class="form-group">
                                    <label for="remainderXP"
                                        >Experience into this level</label
                                    >
                                    <input
                                        type="number"
                                        id="remainderXP"
                                        bind:value={remainderXP}
                                        min="0"
                                        max="999"
                                        required
                                    />
                                    <div class="help-text">
                                        Total XP: {(currentLevel - 1) * 1000 +
                                            remainderXP}
                                    </div>
                                </div>
                            </div>
                        </Card>
                    {:else if currentStep === 2}
                        <!-- Step 2: Characters and Resources -->
                        <Card outlined shadowed={false}>
                            Assign character's items and gold (and ones
                            unassigned to any particular character). A basic
                            encounter will be generated containing all of these.
                            You will be able to edit this later, if you want to
                            fill them in on a session-by-session basis.
                        </Card>
                        <div class="characters-section">
                            <div class="characters-section-button">
                                <Button colour="blue" onclick={addCharacter}
                                    >Add Character</Button
                                >
                            </div>

                            {#each characters as character, i}
                                <Card
                                    bind:collapsed={character.isCollapsed}
                                    outlined
                                    shadowed={false}
                                >
                                    <div slot="header">
                                        <div class="header-content">
                                            <h4>
                                                {character.name ||
                                                    "New Character"}
                                            </h4>
                                        </div>
                                    </div>
                                    <div class="character-content">
                                        <div class="character-basic-info">
                                            <div class="form-group inline">
                                                <input
                                                    type="text"
                                                    placeholder="Character Name"
                                                    bind:value={character.name}
                                                    required
                                                />
                                                <Button
                                                    colour="red"
                                                    onclick={() =>
                                                        removeCharacter(i)}
                                                    >Remove character</Button
                                                >
                                            </div>

                                            <div class="form-group inline">
                                                <label for={`charClass${i}`}
                                                    >Class:</label
                                                >
                                                <select
                                                    id={`charClass${i}`}
                                                    bind:value={character.class}
                                                >
                                                    <option value={0}
                                                        >Select Class</option
                                                    >
                                                    {#each Array.from($classStore.entities.values()) as classOption}
                                                        <option
                                                            value={classOption.id}
                                                            >{classOption.name}</option
                                                        >
                                                    {/each}
                                                </select>

                                                <label for={`charGold${i}`}
                                                    >Gold:</label
                                                >
                                                <input
                                                    type="number"
                                                    id={`charGold${i}`}
                                                    bind:value={character.gold}
                                                    min="0"
                                                />
                                            </div>
                                        </div>

                                        <div>
                                            <h5>Items</h5>

                                            {#if character.items.length > 0}
                                                <div class="item-list">
                                                    {#each character.items as itemId}
                                                        {#if itemDetails.entities.get(itemId)}
                                                            {@const item =
                                                                itemDetails.entities.get(
                                                                    itemId,
                                                                )}
                                                            {#if item}
                                                                <div
                                                                    class="item-entry"
                                                                >
                                                                    <div
                                                                        class="item-name"
                                                                    >
                                                                        {item.name}
                                                                    </div>
                                                                    <div
                                                                        class="item-details"
                                                                    >
                                                                        Level {item.level}
                                                                        • {#if item.price}{item.price}
                                                                            gp{/if}
                                                                    </div>
                                                                    <Button
                                                                        colour="red"
                                                                        onclick={() => {
                                                                            // TODO: This doesn't work with duplicates- no filters!
                                                                            character.items =
                                                                                character.items.filter(
                                                                                    (
                                                                                        id,
                                                                                    ) =>
                                                                                        id !==
                                                                                        itemId,
                                                                                );
                                                                        }}
                                                                        >Remove</Button
                                                                    >
                                                                </div>
                                                            {/if}
                                                        {/if}
                                                    {/each}
                                                </div>
                                            {:else}
                                                <div class="help-text">
                                                    No items added yet
                                                </div>
                                            {/if}

                                            <div class="form-group">
                                                <LibrarySelector
                                                    entityType="item"
                                                    onSelect={(id) => {
                                                        character.items = [
                                                            ...character.items,
                                                            id,
                                                        ];
                                                    }}
                                                    placeholder="Search for items..."
                                                />
                                            </div>
                                        </div>
                                    </div>
                                </Card>
                            {/each}
                        </div>

                        <Card outlined shadowed={false}>
                            <h4>Unassigned Party Resources</h4>

                            <div class="form-group">
                                <label for="partyGold">Party Gold</label>
                                <input
                                    type="number"
                                    id="partyGold"
                                    bind:value={partyGold}
                                    min="0"
                                />
                                <div class="help-text">
                                    Gold that hasn't been assigned to specific
                                    characters
                                </div>
                            </div>

                            <div class="form-group">
                                <label>Party Items</label>

                                {#if partyItems.length > 0}
                                    <div class="item-list">
                                        {#each partyItems as itemId}
                                            {#if itemDetails.entities.get(itemId)}
                                                {@const item =
                                                    itemDetails.entities.get(
                                                        itemId,
                                                    )}
                                                {#if item}
                                                    <div class="item-entry">
                                                        <div class="item-name">
                                                            {item.name}
                                                        </div>
                                                        <div
                                                            class="item-details"
                                                        >
                                                            Level {item.level} •
                                                            {#if item.price}{item.price}
                                                                gp{/if}
                                                        </div>

                                                        <Button
                                                            colour="red"
                                                            onclick={() => {
                                                                partyItems =
                                                                    partyItems.filter(
                                                                        (id) =>
                                                                            id !==
                                                                            itemId,
                                                                    );
                                                            }}>Remove</Button
                                                        >
                                                    </div>
                                                {/if}
                                            {/if}
                                        {/each}
                                    </div>
                                {:else}
                                    <div class="help-text">
                                        No party items added yet
                                    </div>
                                {/if}

                                <div class="form-group">
                                    <LibrarySelector
                                        entityType="item"
                                        onSelect={(id) => {
                                            partyItems = [...partyItems, id];
                                        }}
                                        placeholder="Search for items..."
                                    />
                                </div>
                            </div>
                        </Card>
                    {/if}
                </div>

                <div class="wizard-actions">
                    {#if currentStep > 1}
                        <Button onclick={prevStep} large>Previous</Button>
                    {:else}
                        <div></div>
                        <!-- Empty div to maintain layout -->
                    {/if}

                    {#if currentStep < totalSteps}
                        <Button onclick={nextStep} large colour="blue"
                            >Next</Button
                        >
                    {:else}
                        <Button onclick={handleSubmit} large colour="blue"
                            >Initialize Campaign</Button
                        >
                    {/if}
                </div>
            {/if}
        </form>
    </div>
</Modal>

<style>
    .modal-size {
        width: 50vw;
        max-height: 70vh;
    }

    .wizard-actions {
        display: flex;
        justify-content: space-between;
        margin-top: 1rem;
    }

    .form-group label {
        display: block;
        margin-bottom: 0.5rem;
        color: #4b5563;
        font-weight: 500;
    }

    .tabs {
        display: flex;
        gap: 0.5rem;
        margin-bottom: 1.5rem;
        border-bottom: 2px solid var(--color-bg-light-raised-border);
        padding-bottom: 0.5rem;
    }

    .tab-button {
        padding: 0.5rem 1rem;
        border: none;
        background: none;
        cursor: pointer;
        font-size: 1rem;
        color: var(--color-text);
        border-radius: 0.375rem;
        transition: all 0.2s;
    }

    .tab-button:hover {
        background: var(--color-bg-hover);
    }

    .tab-button.active {
        background: var(--color-bg-selected);
        color: var(--color-text-light);
    }

    textarea#import {
        font-family: monospace;
        font-size: 0.875rem;
    }

    .wizard-progress {
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    .wizard-content {
        padding-top: 1rem;
    }

    .step {
        display: flex;
        gap: 0.5rem;
        padding: 0.5rem 1rem;
        border-radius: 0.375rem;
        background: none;
        cursor: pointer;
        font-size: 1rem;
        color: var(--color-text-secondary);
        transition: all 0.2s;
    }

    .step.active {
        background: var(--color-bg-selected);
        color: var(--color-text-light);
    }

    .step.completed {
        background: var(--color-bg);
        color: var(--color-text-secondary);
    }

    .header-content {
        display: flex;
        align-items: center;
        gap: 0.5rem;
    }

    .character-basic-info {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    .form-row {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
        gap: 1rem;
        margin-bottom: 1rem;
    }

    .item-list {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
        margin-bottom: 1rem;
    }

    .item-entry {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 0.5rem;
        background: white;
        border-radius: 0.375rem;
        border: 1px solid #e5e7eb;
    }

    .item-name {
        font-weight: 500;
    }

    .item-details {
        color: #6b7280;
        font-size: 0.875rem;
    }

    .help-text {
        font-size: 0.875rem;
        color: var(--color-text-secondary);
        margin-top: 0.25rem;
    }

    .characters-section {
        margin-bottom: 1.5rem;
    }

    .characters-section-button {
        margin-top: 1.5rem;
    }

    .form-group.inline {
        display: flex;
        gap: 0.5rem;
        align-items: center;
    }

    .form-group.inline label {
        margin-bottom: 0;
        white-space: nowrap;
    }

    .form-group.inline select,
    .form-group.inline input {
        flex: 1;
    }
</style>

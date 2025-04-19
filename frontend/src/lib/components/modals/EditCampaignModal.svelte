<script lang="ts">
    import { createEventDispatcher, onMount } from "svelte";
    import type { Campaign } from "$lib/types/types";
    import { campaignStore } from "$lib/stores/campaigns";
    import { classStore, itemStore } from "$lib/stores/libraryStore";
    import Modal from "../core/Modal.svelte";
    import Button from "../core/Button.svelte";
    import Card from "../core/Card.svelte";
    import ConfirmationModal from "./ConfirmationModal.svelte";

    interface Props {
        show: boolean;
        editingCampaign: Campaign | null;
    }

    let { show = $bindable(), editingCampaign = $bindable() }: Props = $props();

    const dispatch = createEventDispatcher();

    let name = $state("");
    let description = $state("");
    let error: string | null = $state(null);
    let showDeleteConfirmation = $state(false);

    // on load, load classstore
    onMount(async () => {
        await classStore.fetchEntities({});
    });

    // Reset form when modal opens or editingCampaign changes
    $effect(() => {
        console.log("Bump");
        if (show) {
            if (editingCampaign) {
                console.log(">Editing campaign", editingCampaign);
                name = editingCampaign.name;
                description = editingCampaign.description || "";
            } 
        }
    });

    async function handleSubmit() {
        try {
            if (!editingCampaign) return;
                await campaignStore.editCampaign(
                    editingCampaign?.id,
                    {
                        name,
                        description,
                    }
                );

                await campaignStore.fetchCampaigns();
                closeModal();
        } catch (e) {
            console.error("Error saving campaign:", e);
            error = e instanceof Error ? e.message : "An error occurred";
        }
    }

    async function handleDelete() {
        if (!editingCampaign) return;
        try {
            await campaignStore.deleteCampaign(editingCampaign.id);
            await campaignStore.fetchCampaigns();
            closeModal();
        } catch (e) {
            console.error("Error deleting campaign:", e);
            error = e instanceof Error ? e.message : "An error occurred";
        }
    }

    function closeModal() {
        show = false;
        error = null;
        dispatch("close");
    }
</script>

<Modal bind:show closeButton>
    <div slot="header">
        <h2>{editingCampaign ? "Edit" : "New"} Campaign</h2>
    </div>
    <div class="modal-size">

        <form on:submit|preventDefault={handleSubmit}>
                <div class="wizard-content">
                        <Card>
                            <div class="form-group">
                                <label for="initName">Campaign Name</label>
                                <input
                                    type="text"
                                    id="initName"
                                    bind:value={name}
                                    required
                                />
                            </div>

                            <div class="form-group">
                                <label for="initDescription">Description</label>
                                <textarea
                                    id="initDescription"
                                    bind:value={description}
                                    rows="3"
                                ></textarea>
                            </div>
                        </Card>
                    
                </div>

                <div class="wizard-actions">
                    <Button
                        onclick={() => (showDeleteConfirmation = true)}
                        colour="red"
                        >Delete Campaign</Button
                    >
                    <Button onclick={handleSubmit} large colour="blue"
                    >Save Campaign</Button
                >

                </div>
        </form>
    </div>
</Modal>

<ConfirmationModal
    bind:show={showDeleteConfirmation}
    confirmationString="Delete Campaign"
    on:confirm={handleDelete}
    on:cancel={() => (showDeleteConfirmation = false)}
    >Are you sure you want to delete this campaign? This action cannot be undone. All encounters will be unlinked from this campaign. All other data will be permanently deleted</ConfirmationModal>

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

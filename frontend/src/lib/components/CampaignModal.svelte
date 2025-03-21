<script lang="ts">
    import { createEventDispatcher, onMount } from 'svelte';
    import { fade } from 'svelte/transition';
    import { API_URL } from '$lib/config';
    import type { Campaign, InsertInitialCampaignData } from '$lib/types/types';
    import { campaignStore } from '$lib/stores/campaigns';
    import { classStore, itemStore } from '$lib/stores/libraryStore';
    import LibrarySelector from './LibrarySelector.svelte';

    interface Props {
        show: boolean;
        editingCampaign: Campaign | null;
    }

    let { 
        show = $bindable(),
        editingCampaign = $bindable()
    } : Props = $props();
    
    interface InitialCharacter {
        name: string;
        class: number;
        gold: number;
        items: number[];
    }

    const dispatch = createEventDispatcher();

    let itemDetails = $derived($itemStore);

    let activeTab: 'create' | 'import' | 'initialize' = $state('create');
    let name = $state('');
    let description = $state('');
    let importJson = $state('');
    let error: string | null = $state(null);
    let initName = $state('');
    let initDescription = $state('');
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
    })

    function nextStep() {
        if (currentStep === 1) {
            if (!initName.trim()) {
                error = 'Campaign name is required';
                return;
            }
            // Validate all character names
            if (characters.some(c => !c.name.trim())) {
                error = 'All characters must have names';
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
        characters = [...characters, { 
            name: '', 
            class: 0, 
            gold: 0, 
            items: [],
            isCollapsed: false 
        }];
    }
    function removeCharacter(index: number) {
        // TODO: remove character from list
        console.log('removeCharacter', index);
    }

    // Reset form when modal opens or editingCampaign changes
    $effect(() => {
        if (show) {
            if (editingCampaign) {
                name = editingCampaign.name;
                description = editingCampaign.description || '';
                activeTab = 'create'; // Force create tab when editing
            } else {
                name = '';
                description = '';
                importJson = '';
            }
        }
    })
        
    async function handleSubmit() {
        try {
            if (activeTab === 'create') {
                const campaignData = {
                    name,
                    description
                };

                const url = editingCampaign 
                    ? `${API_URL}/campaign/${editingCampaign.id}`
                    : `${API_URL}/campaign`;

                const response = await fetch(url, {
                    method: editingCampaign ? 'PUT' : 'POST',
                    credentials: 'include',
                    headers: {
                        'Content-Type': 'application/json',
                    },
                    body: JSON.stringify(campaignData),
                });

                if (!response.ok) throw new Error('Failed to save campaign');
                const data = await response.json();
                await handleSuccess(data.id);
            } else if (activeTab === 'import') {
                try {
                    let json = JSON.parse(importJson);
                    // TODO: Validate JSON
                    if (!json) throw new Error('Unparseable JSON');
                    let id = await campaignStore.importCampaign(importJson);
                    await handleSuccess(id);
                } catch (e) {
                    console.log('Error importing campaign', e);
                    throw new Error('Could not import campaign', e);
                }
            } else if (activeTab === 'initialize') {
                const url = `${API_URL}/campaign`;
                
                const ret = await campaignStore.addCampaign({
                    name: initName,
                    description: initDescription,
                    experience: currentLevel * 1000 + remainderXP
                }, {
                    gold: partyGold,
                    items: partyItems,
                    characters: characters.map(c => ({
                            name: c.name,
                            class: c.class,
                            gold: c.gold,
                            items: c.items
                        }))
                    }
                );

                await handleSuccess(ret.id);
            }
        } catch (e) {
            console.error('Error saving campaign:', e);
            error = e instanceof Error ? e.message : 'An error occurred';
        }
    }

    async function handleSuccess(id: number) {
        await campaignStore.fetchCampaigns();
        dispatch('saved', id);
        closeModal();
    }

    function closeModal() {
        show = false;
        error = null;
        dispatch('close');
    }
</script>

{#if show}
    <div class="modal-backdrop" on:click={closeModal} transition:fade>
        <div class="modal-content" on:click|stopPropagation>
            <h2>{editingCampaign ? 'Edit' : 'New'} Campaign</h2>
            
            {#if !editingCampaign}
                <div class="tabs">
                    <button 
                        class="tab-button" 
                        class:active={activeTab === 'create'}
                        on:click={() => activeTab = 'create'}
                    >
                        Create New
                    </button>
                    <button 
                        class="tab-button" 
                        class:active={activeTab === 'import'}
                        on:click={() => activeTab = 'import'}
                    >
                        Import
                    </button>
                    <button 
                        class="tab-button" 
                        class:active={activeTab === 'initialize'}
                        on:click={() => activeTab = 'initialize'}
                    >
                        Initialize Existing
                    </button>
                </div>
            {/if}
            
            {#if error}
                <div class="error-message">{error}</div>
            {/if}

            <form on:submit|preventDefault={handleSubmit}>
                {#if activeTab === 'create'}
                    <div class="form-group">
                        <label for="name">Name</label>
                        <input 
                            type="text" 
                            id="name" 
                            bind:value={name}
                            required
                        />
                    </div>

                    <div class="form-group">
                        <label for="description">Description</label>
                        <textarea 
                            id="description" 
                            bind:value={description}
                            rows="4"
                        ></textarea>
                    </div>
                    <button 
                    type="button"
                    class="finish-btn"
                    on:click={handleSubmit}
                >
                    Initialize Campaign
                </button>

                {:else if activeTab === 'import'}
                    <div class="form-group">
                        <label for="import">Campaign JSON</label>
                        <textarea 
                            id="import" 
                            bind:value={importJson}
                            rows="10"
                            placeholder="Paste your campaign JSON here..."
                            required
                        ></textarea>
                    </div>
                    <button 
                    type="button"
                    class="finish-btn"
                    on:click={handleSubmit}
                >
                    Initialize Campaign
                </button>

                {:else if activeTab === 'initialize'}
                    <!-- Initialize Existing Campaign Wizard -->
                    <div class="wizard-container">
                        <div class="wizard-progress">
                            {#each Array(totalSteps) as _, i}
                                <div class="step" class:active={currentStep === i + 1} class:completed={currentStep > i + 1}>
                                    <div class="step-number">{i + 1}</div>
                                    <div class="step-label">
                                        {i === 0 ? 'Campaign Info' : 'Characters & Resources'}
                                    </div>
                                </div>
                            {/each}
                        </div>
                        
                        <div class="progress-bar">
                            <div class="progress" style="width: {((currentStep - 1) / (totalSteps - 1)) * 100}%"></div>
                        </div>
                    </div>
                    
                    <div class="wizard-content">
                        {#if currentStep === 1}
                            <!-- Step 1: Campaign Info -->
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
                                    <label for="currentLevel">Current Level</label>
                                    <input 
                                        type="number" 
                                        id="currentLevel" 
                                        bind:value={currentLevel}
                                        min="1"
                                        max="20"
                                        on:input={(e) => {
                                            const val = parseInt(e.currentTarget.value);
                                            if (val < 1) currentLevel = 1;
                                            if (val > 20) currentLevel = 20;
                                        }}
                                        required
                                    />
                                </div>

                                <div class="form-group">
                                    <label for="remainderXP">Experience into this level</label>
                                    <input 
                                        type="number" 
                                        id="remainderXP" 
                                        bind:value={remainderXP}
                                        min="0"
                                        max="999"
                                        required
                                    />
                                    <div class="help-text">
                                        Total XP: {(currentLevel - 1) * 1000 + remainderXP}
                                    </div>
                                </div>
                            </div>
                        {:else if currentStep === 2}
                            <!-- Step 2: Characters and Resources -->
                             <div class="description">
                                Assign character's items and gold (and ones unassigned to any particular character). A basic encounter will be generated containing all of these. You will be able to edit this later, if you want to fill them in on a session-by-session basis.
                             </div>
                            <div class="characters-section">
                                <button 
                                    type="button"
                                    class="add-character-btn"
                                    on:click={addCharacter}
                                >
                                    Add Character
                                </button>
                                
                                {#each characters as character, i}
                                    <div class="character-card">
                                        <div class="character-header" 
                                             on:click={() => character.isCollapsed = !character.isCollapsed}>
                                            <div class="header-content">
                                                <h4>{character.name || 'New Character'}</h4>
                                                <span class="collapse-indicator">
                                                    {character.isCollapsed ? '▼' : '▲'}
                                                </span>
                                            </div>
                                            {#if characters.length > 1}
                                                <button 
                                                    type="button" 
                                                    class="remove-button"
                                                    on:click|stopPropagation={() => removeCharacter(i)}
                                                >
                                                    Remove
                                                </button>
                                            {/if}
                                        </div>
                                        
                                        {#if !character.isCollapsed}
                                            <div class="character-content">
                                                <div class="character-basic-info">
                                                    <div class="form-group">
                                                        <input 
                                                            type="text" 
                                                            placeholder="Character Name"
                                                            bind:value={character.name}
                                                            required
                                                        />
                                                    </div>
                                                    
                                                    <div class="form-group inline">
                                                        <label for={`charClass${i}`}>Class:</label>
                                                        <select id={`charClass${i}`} bind:value={character.class}>
                                                            <option value={0}>Select Class</option>
                                                            {#each Array.from($classStore.entities.values()) as classOption}
                                                                <option value={classOption.id}>{classOption.name}</option>
                                                            {/each}
                                                        </select>
                                                        
                                                        <label for={`charGold${i}`}>Gold:</label>
                                                        <input 
                                                            type="number" 
                                                            id={`charGold${i}`} 
                                                            bind:value={character.gold}
                                                            min="0"
                                                        />
                                                    </div>
                                                </div>
                                                
                                                <div class="character-items">
                                                    <h5>Items</h5>
                                                    
                                                    {#if character.items.length > 0}
                                                        <div class="item-list">
                                                            {#each character.items as itemId}
                                                                {#if itemDetails.entities.get(itemId)}
                                                                    {@const item = itemDetails.entities.get(itemId)}
                                                                    {#if item}
                                                                        <div class="item-entry">
                                                                            <div class="item-name">{item.name}</div>
                                                                            <div class="item-details">Level {item.level} • {#if item.price}{item.price} gp{/if}</div>
                                                                            <button 
                                                                                type="button"
                                                                                class="remove-button"
                                                                                on:click={() => {
                                                                                    character.items = character.items.filter(id => id !== itemId);
                                                                                }}
                                                                            >
                                                                                Remove
                                                                            </button>
                                                                        </div>
                                                                    {/if}
                                                                {/if}
                                                            {/each}
                                                        </div>
                                                    {:else}
                                                        <div class="help-text">No items added yet</div>
                                                    {/if}
                                                    
                                                    <div class="form-group">
                                                        <LibrarySelector
                                                            entityType="item"
                                                            onSelect={(id) => {
                                                                character.items = [...character.items, id];
                                                            }}
                                                            placeholder="Search for items..."
                                                        />
                                                    </div>
                                                </div>
                                            </div>
                                        {/if}
                                    </div>
                                {/each}
                            </div>
                            
                            <div class="party-resources">
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
                                        Gold that hasn't been assigned to specific characters
                                    </div>
                                </div>
                                
                                <div class="form-group">
                                    <label>Party Items</label>
                                    
                                    {#if partyItems.length > 0}
                                        <div class="item-list">
                                            {#each partyItems as itemId}
                                                {#if itemDetails.entities.get(itemId)}
                                                    {@const item = itemDetails.entities.get(itemId)}
                                                    {#if item}
                                                        <div class="item-entry">
                                                            <div class="item-name">{item.name}</div>
                                                            <div class="item-details">Level {item.level} • {#if item.price}{item.price} gp{/if}</div>
                                                            <button 
                                                                type="button"
                                                                class="remove-button"
                                                                on:click={() => {
                                                                    partyItems = partyItems.filter(id => id !== itemId);
                                                                }}
                                                            >
                                                                Remove
                                                            </button>
                                                        </div>
                                                    {/if}
                                                {/if}
                                            {/each}
                                        </div>
                                    {:else}
                                        <div class="help-text">No party items added yet</div>
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
                            </div>
                        {/if}
                    </div>
                    
                    <div class="wizard-actions">
                        {#if currentStep > 1}
                            <button 
                                type="button"
                                class="prev-btn"
                                on:click={prevStep}
                            >
                                Previous
                            </button>
                        {:else}
                            <div></div> <!-- Empty div to maintain layout -->
                        {/if}
                        
                        {#if currentStep < totalSteps}
                            <button 
                                type="button"
                                class="next-btn"
                                on:click={nextStep}
                            >
                                Next
                            </button>
                        {:else}
                            <button 
                                type="button"
                                class="finish-btn"
                                on:click={handleSubmit}
                            >
                                Initialize Campaign
                            </button>
                        {/if}
                    </div>
                {/if}
                </form>
            </div>
    </div>
{/if}

<style>
    .modal-backdrop {
        position: fixed;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        background: rgba(0, 0, 0, 0.5);
        display: flex;
        align-items: flex-start;
        justify-content: center;
        z-index: 1000;
        padding: 2rem;
        overflow-y: auto;
    }

    .modal-content {
        background: white;
        padding: 2rem;
        border-radius: 0.5rem;
        width: 90%;
        max-width: 1000px;
        max-height: 90vh;
        overflow-y: auto;
        margin: auto;
    }

    .form-group {
        margin-bottom: 1rem;
    }

    .form-group label {
        display: block;
        margin-bottom: 0.5rem;
        color: #4b5563;
        font-weight: 500;
    }

    .form-group input,
    .form-group textarea {
        width: 100%;
        padding: 0.5rem;
        border: 1px solid #e5e7eb;
        border-radius: 0.375rem;
        font-size: 1rem;
    }

    .modal-actions {
        display: flex;
        justify-content: flex-end;
        gap: 1rem;
        margin-top: 2rem;
    }

    .cancel-btn {
        padding: 0.5rem 1rem;
        border-radius: 0.375rem;
        background: #6b7280;
        color: white;
    }

    .save-btn {
        padding: 0.5rem 1rem;
        border-radius: 0.375rem;
        background: #22c55e;
        color: white;
    }

    .error-message {
        background: #fee2e2;
        color: #991b1b;
        padding: 1rem;
        border-radius: 0.375rem;
        margin-bottom: 1rem;
    }

    .tabs {
        display: flex;
        gap: 0.5rem;
        margin-bottom: 1.5rem;
        border-bottom: 2px solid #e5e7eb;
        padding-bottom: 0.5rem;
    }

    .tab-button {
        padding: 0.5rem 1rem;
        border: none;
        background: none;
        cursor: pointer;
        font-size: 1rem;
        color: #6b7280;
        border-radius: 0.375rem;
        transition: all 0.2s;
    }

    .tab-button:hover {
        background: #f3f4f6;
        color: #111827;
    }

    .tab-button.active {
        background: #3b82f6;
        color: white;
    }

    textarea#import {
        font-family: monospace;
        font-size: 0.875rem;
    }

    .wizard-container {
        padding: 1.5rem;
        margin-bottom: 1rem;
        border: 1px solid #e5e7eb;
    }
    
    .wizard-progress {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 1rem;
    }
    
    .step {
        display: flex;
        gap: 0.5rem;
        padding: 0.5rem 1rem;
        border-radius: 0.375rem;
        background: none;
        cursor: pointer;
        font-size: 1rem;
        color: #6b7280;
        transition: all 0.2s;
    }
    
    .step.active {
        background: #3b82f6;
        color: white;
    }
    
    .step.completed {
        background: #d1d5db;
        color: #6b7280;
    }
    
    .progress-bar {
        height: 0.25rem;
        background: #e5e7eb;
        border-radius: 0.125rem;
        overflow: hidden;
    }
    
    .progress {
        height: 100%;
        background: #3b82f6;
    }
    
    .wizard-content {
        padding: 1.5rem;
        margin-bottom: 1rem;
    }
    
    .character-header {
        padding: 1rem;
        cursor: pointer;
        background: #f9fafb;
        border-radius: 0.5rem 0.5rem 0 0;
        display: flex;
        justify-content: space-between;
        align-items: center;
    }
    
    .header-content {
        display: flex;
        align-items: center;
        gap: 0.5rem;
    }
    
    .character-content {
        padding: 1rem;
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
    
    .character-items {
        margin-top: 1rem;
    }
    
    .character-items h5 {
        margin-top: 0;
        margin-bottom: 0.75rem;
        color: #4b5563;
        font-size: 0.875rem;
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
    
    .add-character-btn {
        background: #3b82f6;
        color: white;
        border: none;
        padding: 0.5rem 1rem;
        border-radius: 0.375rem;
        font-weight: 500;
        cursor: pointer;
        margin-bottom: 1.5rem;
    }
    
    .remove-button {
        background: #ef4444;
        color: white;
        border: none;
        padding: 0.25rem 0.5rem;
        border-radius: 0.375rem;
        font-size: 0.875rem;
        cursor: pointer;
    }
    
    .help-text {
        font-size: 0.875rem;
        color: #6b7280;
        margin-top: 0.25rem;
    }
    
    .party-resources {
        background: #f9fafb;
        border-radius: 0.5rem;
        padding: 1.5rem;
        margin-top: 1.5rem;
        border: 1px solid #e5e7eb;
    }

    .description { 
        background: #f9fafb;
        border-radius: 0.5rem;
        padding: 1.5rem;
        margin-top: 1.5rem;
        margin-bottom: 1.5rem;
        border: 1px solid #e5e7eb;
    }

    .characters-section {
        margin-bottom: 1.5rem;
    }

    .character-card {
        border: 1px solid #e5e7eb;
        border-radius: 0.5rem;
        margin-bottom: 1rem;
        background: white;
    }

    .collapse-indicator {
        color: #6b7280;
        margin-left: 0.5rem;
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
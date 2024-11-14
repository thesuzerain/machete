<script lang="ts">
    import type { Character } from '$lib/types/types';
    import type { InsertEvent } from '$lib/types/types';
    import LibrarySelector from '$lib/components/LibrarySelector.svelte';
    import type { Currency } from '$lib/types/library';

    export let characters: Character[] = [];
    export let onEventCreate: (eventData: InsertEvent) => void;

    let selectedType: string = '';
    let selectedCharacter: number | null = null;
    let eventData: any = {};

    const eventTypes = [
        { value: 'CurrencyGain', label: 'Currency Gain' },
        { value: 'ExperienceGain', label: 'Experience Gain' },
        { value: 'EnemyDefeated', label: 'Enemy Defeated' },
        { value: 'HazardDefeated', label: 'Hazard Defeated' },
        { value: 'ItemGain', label: 'Item Gain' }
    ];

    function resetForm() {
        selectedType = '';
        selectedCharacter = null;
        eventData = {};
    }

    function handleSubmit() {
        if (!selectedCharacter || !selectedType) return;

        const event: InsertEvent = {
            character_id: selectedCharacter,
            event_type: selectedType,
            description: '',
            data: eventData
        };

        onEventCreate(event);
        resetForm();
    }

    // Reset event data when type changes
    $: if (selectedType) {
        switch (selectedType) {
            case 'CurrencyGain':
                eventData = { currency: { gold: 0, silver: 0, copper: 0 } };
                break;
            case 'ExperienceGain':
                eventData = { experience: 0 };
                break;
            case 'EnemyDefeated':
            case 'HazardDefeated':
            case 'ItemGain':
                eventData = { id: null };
                break;
        }
    }
</script>

<div class="event-creator">
    <h3>Add Event</h3>
    <form on:submit|preventDefault={handleSubmit}>
        <div class="form-group">
            <label for="eventType">Event Type</label>
            <select 
                id="eventType"
                bind:value={selectedType}
                required
            >
                <option value="">Select event type...</option>
                {#each eventTypes as type}
                    <option value={type.value}>{type.label}</option>
                {/each}
            </select>
        </div>

        <div class="form-group">
            <label for="character">Character</label>
            <select 
                id="character"
                bind:value={selectedCharacter}
                required
            >
                <option value={null}>Select character...</option>
                {#each characters as character}
                    <option value={character.id}>{character.name}</option>
                {/each}
            </select>
        </div>

        {#if selectedType}
            <div class="form-group">
                {#if selectedType === 'CurrencyGain'}
                    <div class="currency-fields">
                        <div class="currency-field">
                            <label>Gold</label>
                            <input 
                                type="number"
                                bind:value={eventData.currency.gold}
                                min="0"
                            />
                        </div>
                        <div class="currency-field">
                            <label>Silver</label>
                            <input 
                                type="number"
                                bind:value={eventData.currency.silver}
                                min="0"
                            />
                        </div>
                        <div class="currency-field">
                            <label>Copper</label>
                            <input 
                                type="number"
                                bind:value={eventData.currency.copper}
                                min="0"
                            />
                        </div>
                    </div>
                {:else if selectedType === 'ExperienceGain'}
                    <label>Experience</label>
                    <input 
                        type="number"
                        bind:value={eventData.experience}
                        min="0"
                        required
                    />
                {:else if selectedType === 'EnemyDefeated'}
                    <label>Enemy</label>
                    <LibrarySelector
                        entityType="creature"
                        onSelect={(id) => eventData.id = id}
                        placeholder="Select enemy..."
                    />
                {:else if selectedType === 'HazardDefeated'}
                    <label>Hazard</label>
                    <LibrarySelector
                        entityType="hazard"
                        onSelect={(id) => eventData.id = id}
                        placeholder="Select hazard..."
                    />
                {:else if selectedType === 'ItemGain'}
                    <label>Item</label>
                    <LibrarySelector
                        entityType="item"
                        onSelect={(id) => eventData.id = id}
                        placeholder="Select item..."
                    />
                {/if}
            </div>
        {/if}

        <button 
            type="submit" 
            disabled={!selectedType || !selectedCharacter || 
                     (selectedType.includes('Defeated') && !eventData.id) ||
                     (selectedType === 'ItemGain' && !eventData.id)}
        >
            Add Event
        </button>
    </form>
</div>

<style>
    .event-creator {
        background: #f8f8f8;
        padding: 1rem;
        border-radius: 4px;
        margin-bottom: 1rem;
    }

    .form-group {
        margin-bottom: 1rem;
    }

    label {
        display: block;
        margin-bottom: 0.5rem;
        font-weight: 500;
    }

    select, input {
        width: 100%;
        padding: 0.5rem;
        border: 1px solid #ddd;
        border-radius: 4px;
        margin-bottom: 0.5rem;
    }

    .currency-fields {
        display: grid;
        grid-template-columns: repeat(3, 1fr);
        gap: 1rem;
    }

    .currency-field {
        display: flex;
        flex-direction: column;
    }

    button {
        background: #3b82f6;
        color: white;
        border: none;
        padding: 0.5rem 1rem;
        border-radius: 4px;
        cursor: pointer;
    }

    button:disabled {
        background: #9ca3af;
        cursor: not-allowed;
    }

    h3 {
        margin-top: 0;
        margin-bottom: 1rem;
    }
</style> 
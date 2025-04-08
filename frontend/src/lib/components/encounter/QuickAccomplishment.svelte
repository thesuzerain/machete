<script lang="ts">
    import { fade } from 'svelte/transition';
    import { encounterStore } from '$lib/stores/encounters';
    import { experienceForAccomplishment, type AccomplishmentLevel, type CreateEncounterFinalized, type CreateOrReplaceEncounter } from '$lib/types/encounters';
    import Card from '../core/Card.svelte';
    import Button from '../core/Button.svelte';
    import EncounterLibraryItemSelector from './EncounterLibraryItemSelector.svelte';
    import BrowseLibraryModal from '../modals/BrowseLibraryModal.svelte';
    import { itemStore } from '$lib/stores/libraryStore';
    import { campaignSessionStore } from '$lib/stores/campaignSessions';
    import { statsStore } from '$lib/stores/stats';

    interface Props {
        selectedCampaignId: number;
        selectedSessionId: number;
        onAddEncounter: () => void;
    }
    let { 
        selectedCampaignId,
        selectedSessionId,
        onAddEncounter
     } : Props = $props();

     let libraryModal : BrowseLibraryModal | null = $state(null);
    let showLibraryModal = $state(false);
    let libraryTabs = $state([]);

    let items = $derived($itemStore);
    let sessions = $derived($campaignSessionStore);

    // Accomplishment state
    let accomplishmentType: AccomplishmentLevel | null = $state('moderate');
    let useCustomXP = $state(false);
    let customXPAmount = $state(0);

    // Accomplishment Encounter state
    let wipEncounter: CreateOrReplaceEncounter = $state({
        name: "",
        description: "",
        encounter_type: "accomplishment",
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

    let showAccomplishmentForm = $state(false);

    let canAddAccomplishment = $derived(
        (useCustomXP && customXPAmount != 0) || accomplishmentType !== null || wipEncounter.treasure_items.length > 0
    );

    function setCustomXP() {
        useCustomXP = true;
        accomplishmentType = null;
        if (!customXPAmount) customXPAmount = 10; // Default value
        wipEncounter.extra_experience = customXPAmount;
    }

    function setAccomplishmentType(type: AccomplishmentLevel | null) {
        accomplishmentType = type;
        useCustomXP = false;
        wipEncounter.extra_experience = experienceForAccomplishment(type);
    }

    async function addAccomplishment() {
        console.log("Adding accomplishment");
        if (!canAddAccomplishment) return;
        
        let xpUnparsed = useCustomXP ? customXPAmount : accomplishmentType!;
        let xp : number;
        if (typeof xpUnparsed === 'number') {
            xp = xpUnparsed;
        } else {
            xp = experienceForAccomplishment(xpUnparsed);
        }

        let treasure_sum = 0;
        for (let item of wipEncounter.treasure_items) {
            treasure_sum += items.entities.get(item)?.price || 0;
        }

        // Creating a new encounter
        const finalizedEncounter: CreateEncounterFinalized = {
            ...wipEncounter,
            total_experience: xp,
            total_items_value: treasure_sum,
            session_id: selectedSessionId,
        };

        await encounterStore.addEncounter(finalizedEncounter);
        await Promise.all([
            campaignSessionStore.fetchCampaignSessions(selectedCampaignId),
            statsStore.fetchStats(selectedCampaignId),
        ]);

        onAddEncounter();

        // Reset form name (don't reset type, so that we can quickly add more)
        showAccomplishmentForm = false;
        wipEncounter = {
            name: "",
            description: "",
            encounter_type: "accomplishment",
            enemies: [],
            hazards: [],
            treasure_items: [],
            treasure_currency: 0,
            extra_experience: 0,
            party_level: 1,
            party_size: 4,
            subsystem_type: "chase",
            subsystem_checks: [],
        }
    }


    </script>


<div transition:fade>
    <Card>
        <form onsubmit={addAccomplishment} class="accomplishment-inputs">
            <div class="name-description-row">
                <input 
                    type="text" 
                    placeholder="Name"
                    bind:value={wipEncounter.name}
                />
                <Button 
                colour="green" 
                onclick={addAccomplishment}
                disabled={!canAddAccomplishment}
            >
                Add Accomplishment
            </Button>

            </div>
            <div class="accomplishment-buttons">
                <Button colour='white' selectedColour='blue' selected={accomplishmentType === null && !useCustomXP} onclick={() => setAccomplishmentType(null)}>
                    None (0 XP)                           
                </Button>
                <Button colour='white' selectedColour='blue' selected={accomplishmentType === 'minor'} onclick={() => setAccomplishmentType('minor')}>
                    Minor (10 XP)                            
                </Button>
                <Button colour='white' selectedColour='blue' selected={accomplishmentType === 'moderate'} onclick={() => setAccomplishmentType('moderate')}>
                    Moderate (30 XP)
                </Button>
                <Button colour='white' selectedColour='blue' selected={accomplishmentType === 'major'} onclick={() => setAccomplishmentType('major')}>
                    Major (80 XP)
                </Button>
                <Button colour='white' selectedColour='blue' selected={useCustomXP} onclick={() => setCustomXP()}>
                    Custom XP
                </Button>
            
            {#if useCustomXP}
                <div class="custom-xp">
                    <input 
                        type="number" 
                        bind:value={customXPAmount}
                        min="0"
                        placeholder="Enter XP amount"
                    />
                </div>
            {/if}
        </div>

<div>
<div>
    {#if libraryModal}
    <EncounterLibraryItemSelector libraryObjectType='item' partyLevel={wipEncounter.party_level} {libraryModal} bind:data={wipEncounter.treasure_items} />
    {/if}
</div>
    </div>

    </form>
</Card>
</div>

<BrowseLibraryModal
    bind:this={libraryModal}
    bind:show={showLibraryModal}
    allowedTabs={libraryTabs}
    bind:editingEncounter={wipEncounter}
/>

<style>
    .name-description-row {
        display: flex;
        gap: 1rem;
        justify-content: space-between;
    }

    .accomplishment-inputs {
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
    }

    .accomplishment-buttons {
        display: grid;
        grid-template-columns: 1fr 1fr 1fr 1fr 1fr 1fr;
        gap: 0.5rem;
        flex-wrap: wrap;
    }

    .custom-xp {
        display: flex;
        justify-content: center;
    }

    .custom-xp input[type="number"] {
        width: 200px;
        padding: 0.5rem;
        border: 1px solid #e5e7eb;
        border-radius: 0.375rem;
        text-align: center;
    }

    </style>
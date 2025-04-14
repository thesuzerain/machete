<script lang="ts">
    import { goto } from '$app/navigation';
    import { selectedCampaignStore } from '$lib/stores/campaigns';
    import { campaignSessionStore } from '$lib/stores/campaignSessions';
    import { encounterStore } from '$lib/stores/encounters';
    import type { Encounter } from '$lib/types/encounters';
    import Button from '../core/Button.svelte';
    import Modal from '../core/Modal.svelte';
    import EncounterList from "../encounter/EncounterList.svelte";
import EncounterViewer from "../encounter/EncounterViewer.svelte";

    interface Props {
        show: boolean;
        error?: string;
        sessionId: number | null;
    }

    let { 
        show = $bindable(),
        error = $bindable(),
        sessionId = $bindable()
     } : Props = $props();

     let globalCampaignId = $derived($selectedCampaignStore);


     function linkEncounterToSession(encounter: Partial<Encounter>, sessionId: number | null) {
        if (!encounter.id || !globalCampaignId) return;
        if (sessionId && globalCampaignId) {
            campaignSessionStore.linkEncounterToSession(globalCampaignId, sessionId, encounter.id);
        } 
    }

    function gotoEncounter(encounterId: number) {
        goto(`/encounters?encounterId=${encounterId}&returnToSessionId=${sessionId}`);
    }

</script>

<Modal bind:show bind:error closeButton>
    
    <EncounterList let:encounter forceHideUnlinked>
        <Button colour='green' onclick={() => linkEncounterToSession(encounter, sessionId)}>
            Link to session
        </Button>
        <Button colour='blue' onclick={() => gotoEncounter(encounter.id)}>
            Go to encounter
        </Button>


    </EncounterList>


    
</Modal>


<style></style>
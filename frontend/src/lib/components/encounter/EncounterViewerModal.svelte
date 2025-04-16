<script lang="ts">
    import type { Encounter } from "$lib/types/encounters";
    import Modal from "../core/Modal.svelte";
    import EncounterSummary from "./EncounterSummary.svelte";

    interface Props {
        show: boolean;
        encounter: Encounter | null;
    }

    let { show = $bindable(), encounter = $bindable() }: Props = $props();
</script>

{#if encounter}
    <Modal bind:show closeButton>
        <div slot="header">
            <h2>{encounter.name}</h2>
        </div>
        <div class="modal-shape">
            <EncounterSummary {encounter} size="detailed">
                <slot />
            </EncounterSummary>
        </div>
    </Modal>
{/if}

<style>
    .modal-shape {
        min-width: 100vh;
    }
</style>

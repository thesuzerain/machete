<script lang="ts">
    import { fade } from 'svelte/transition';
    import type { Character } from '$lib/types/types';
    import CharacterModal from '$lib/components/CharacterModal.svelte';
    import { characterStore } from '$lib/stores/characters';
    import { classStore } from '$lib/stores/libraryStore';
    import { statsStore } from '$lib/stores/stats';
    import type { CharacterStats } from '$lib/types/stats';
    import { campaignStore } from '$lib/stores/campaigns';
    import { id } from 'date-fns/locale';
    import ConfirmationModal from './ConfirmationModal.svelte';

    export let selectedCampaignId: number;
    export let error: string | null;

    let showNewCharacterModal = false;
    let editingCharacter: Character | null = null;
    $: characters = $characterStore.get(selectedCampaignId) || [];
    $: classes = $classStore;
    $: campaign = $campaignStore.get(selectedCampaignId);

    $: stats = $statsStore.get(selectedCampaignId);
    $: characterStats = stats?.character_stats || {};

    let characterToDelete: number | null = null;

    function getCharacterStats(characterId: number): CharacterStats | undefined {
        return stats?.character_stats[characterId];
    }


    $: allIndividualGold = Object.values(stats?.character_stats || {}).map(c => c.total_combined_treasure).reduce((acc, val) => acc + val, 0);
    // TODO: refactor with CampaignSummaryTab
    function getEquityStats(character) {
        const charStats = characterStats[character.id];
        if (!charStats) return null;

        const expectedGoldShare = (allIndividualGold || 0) / (characters.length || 1);
        const goldDeno = expectedGoldShare ? expectedGoldShare : 1;
        
        return {
            goldShare: charStats.total_combined_treasure,
            expectedGoldShare,
            goldPercent: ((charStats.total_combined_treasure / goldDeno) * 100).toFixed(1),
            permanentItems: charStats.total_permanent_items.length,
            expectedPermanentItems: Object.values(charStats.expected_boosts || {}).length,
            availableBoosts: charStats.available_boosts,
            expectedBoosts: charStats.expected_boosts,
            missingBoosts: charStats.expected_boosts.filter(expected => 
                !charStats.available_boosts.some(available => 
                    available.boost_category_id === expected.boost_category_id && 
                    available.potency === expected.potency
                )
            )
        };
    }

    async function handleCharacterDelete(id: number) {
        try {
            await characterStore.deleteCharacter(selectedCampaignId, id);
        } catch (e) {
            error = e instanceof Error ? e.message : 'Failed to delete character';
        }
    }
</script>

<div class="characters-section" transition:fade>
    <div class="characters-header">
        <h2>Characters</h2>
        <button class="add-character-btn" on:click={() => showNewCharacterModal = true}>
            Add Character
        </button>
    </div>

    <div class="character-list">
        {#each characters as character}
            {@const equity = getEquityStats(character)}
            <div class="character-row" transition:fade>
                <div class="character-main">
                    <div class="character-identity">
                        <h3>{character.name}</h3>
                        <div class="character-subtitle">
                            Level {campaign?.level || '?'} {classes.entities.get(character.class)?.name}
                        </div>
                    </div>
                    <div class="character-actions">
                        <button class="edit-btn" on:click={() => {
                            editingCharacter = character;
                            showNewCharacterModal = true;
                        }}>Edit</button>
                        <button class="delete-btn" on:click={() => characterToDelete = character.id}>
                            Delete
                        </button>
                    </div>
                </div>

                <div class="character-content">
                    <div class="content-section">
                        <h4>Treasure & Items</h4>
                        {#if equity}
                            <div class="equity-stats">
                                <div class="equity-stat" class:deficit={equity.goldShare < equity.expectedGoldShare}
                                                  class:surplus={equity.goldShare >= equity.expectedGoldShare}>
                                    <span class="stat-label">Gold Share</span>
                                    <span class="stat-value">{equity.goldShare.toFixed(1)}</span>
                                    <span class="stat-subtext">({equity.goldPercent}% of fair share {equity.expectedGoldShare.toFixed(1)})</span>
                                </div>
                                <div class="equity-stat" class:deficit={equity.permanentItems < equity.expectedPermanentItems}
                                                  class:surplus={equity.permanentItems >= equity.expectedPermanentItems}>
                                    <span class="stat-label">Permanent Items</span>
                                    <span class="stat-value">{equity.permanentItems}/{equity.expectedPermanentItems}</span>
                                </div>
                            </div>
                        {/if}
                    </div>

                    <div class="content-section">
                        <h4>Boosts</h4>
                        {#if equity}
                            <div class="boosts-grid">
                                <div class="boost-section">
                                    <h5>Available Boosts</h5>
                                    {#if equity.availableBoosts.length}
                                        {#each equity.availableBoosts as boost}
                                            <div class="boost-item">
                                                <span class="boost-name">{boost.boost_category_name}</span>
                                                <span class="boost-potency">+{boost.potency}</span>
                                            </div>
                                        {/each}
                                    {:else}
                                        <div class="empty-state">No boosts available</div>
                                    {/if}
                                </div>
                                <div class="boost-section">
                                    <h5>Missing Boosts</h5>
                                    {#if equity.missingBoosts.length}
                                        {#each equity.missingBoosts as boost}
                                            <div class="boost-item missing">
                                                <span class="boost-name">{boost.boost_category_name}</span>
                                                <span class="boost-potency">+{boost.potency}</span>
                                            </div>
                                        {/each}
                                    {:else}
                                        <div class="empty-state">No missing boosts</div>
                                    {/if}
                                </div>
                            </div>
                        {/if}
                    </div>
                </div>
            </div>
        {/each}
    </div>
</div>

<CharacterModal
    bind:show={showNewCharacterModal}
    campaignId={selectedCampaignId}
    bind:editingCharacter
    on:saved={async (event) => {
        if (editingCharacter) {
            await characterStore.updateCharacter(selectedCampaignId, editingCharacter.id, event.detail);
        } else {
            await characterStore.addCharacters(selectedCampaignId, [event.detail]);
        }
        editingCharacter = null;
    }}
    on:close={() => {
        editingCharacter = null;
    }}
/>

<ConfirmationModal
    show={characterToDelete !== null}
    error={error}
    on:confirm={() => {
        if(characterToDelete) handleCharacterDelete(characterToDelete);
        characterToDelete = null;
    }}
    on:close={() => {
        characterToDelete = null
    }}
    confirmationString="Delete '{characters.find(c => c.id === characterToDelete)?.name}'"
    >
Are you sure you would like to delete the character "{
    characters.find(c => c.id === characterToDelete)?.name
}"?
    </ConfirmationModal>

<style>
    .characters-section {
        background: white;
        border-radius: 0.5rem;
        box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
        padding: 1.5rem;
    }

    .characters-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 1.5rem;
    }

    .character-list {
        display: flex;
        flex-direction: column;
        gap: 1.5rem;
    }

    .character-row {
        background: white;
        border-radius: 0.5rem;
        box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
        overflow: hidden;
    }

    .character-main {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 1.5rem;
        background: #f8fafc;
        border-bottom: 1px solid #e2e8f0;
    }
    
    .character-identity {
        display: flex;
        flex-direction: row;
        gap: 1rem;
    }

    .character-identity h3 {
        margin: 0;
        font-size: 1.25rem;
        color: #1e293b;
    }

    .character-subtitle {
        color: #64748b;
        font-size: 0.875rem;
        margin-top: 0.25rem;
    }

    .character-content {
        padding: 1.5rem;
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
        gap: 1.5rem;
    }

    .content-section {
        padding: 1rem;
        background: #f8fafc;
        border-radius: 0.375rem;
    }

    .content-section h4 {
        margin: 0 0 1rem 0;
        color: #475569;
        font-size: 0.875rem;
        text-transform: uppercase;
        letter-spacing: 0.05em;
    }

    .xp-display {
        background: #e2e8f0;
        border-radius: 9999px;
        height: 1.5rem;
        overflow: hidden;
        position: relative;
    }

    .xp-bar {
        background: #3b82f6;
        height: 100%;
        width: var(--progress);
        transition: width 0.3s ease;
    }

    .xp-text {
        position: absolute;
        left: 50%;
        top: 50%;
        transform: translate(-50%, -50%);
        color: white;
        font-size: 0.875rem;
        font-weight: 500;
        text-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
    }

    .equity-stats {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
        gap: 1rem;
    }

    .equity-stat {
        padding: 0.75rem;
        background: white;
        border-radius: 0.375rem;
        box-shadow: 0 1px 2px rgba(0, 0, 0, 0.05);
    }

    .stat-label {
        font-size: 0.75rem;
        color: #64748b;
        text-transform: uppercase;
        letter-spacing: 0.05em;
    }

    .stat-value {
        font-size: 1.25rem;
        font-weight: 600;
        margin: 0.25rem 0;
    }

    .stat-subtext {
        font-size: 0.75rem;
        color: #64748b;
    }

    .boosts-grid {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
        gap: 1rem;
    }

    .boost-section h5 {
        font-size: 0.875rem;
        color: #64748b;
        margin: 0 0 0.75rem 0;
    }

    .boost-item {
        display: flex;
        justify-content: space-between;
        padding: 0.5rem;
        background: white;
        border-radius: 0.25rem;
        margin-bottom: 0.5rem;
    }

    .boost-item.missing {
        background: #fee2e2;
        color: #991b1b;
    }

    .deficit .stat-value {
        color: #ef4444;
    }

    .surplus .stat-value {
        color: #22c55e;
    }

    .empty-state {
        color: #64748b;
        font-size: 0.875rem;
        text-align: center;
        padding: 1rem;
    }

    .add-character-btn {
        background: #22c55e;
        color: white;
        padding: 0.5rem 1rem;
        border-radius: 0.375rem;
        font-weight: 500;
    }
</style> 
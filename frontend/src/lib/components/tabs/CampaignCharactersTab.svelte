<script lang="ts">
    import { fade } from "svelte/transition";
    import type { Character } from "$lib/types/types";
    import CharacterModal from "$lib/components/modals/CharacterModal.svelte";
    import { characterStore } from "$lib/stores/characters";
    import { classStore, itemStore } from "$lib/stores/libraryStore";
    import { statsStore } from "$lib/stores/stats";
    import type { CharacterStats } from "$lib/types/stats";
    import { campaignStore } from "$lib/stores/campaigns";
    import { id } from "date-fns/locale";
    import ConfirmationModal from "../modals/ConfirmationModal.svelte";
    import Button from "../core/Button.svelte";
    import Card from "../core/Card.svelte";
    import { onMount } from "svelte";

    export let selectedCampaignId: number;
    export let error: string | null;

    let showNewCharacterModal = false;
    let editingCharacter: Character | null = null;
    $: characters = $characterStore.get(selectedCampaignId) || [];
    $: classes = $classStore;
    $: items = $itemStore;
    $: campaign = $campaignStore.get(selectedCampaignId);

    $: stats = $statsStore.get(selectedCampaignId);
    $: characterStats = stats?.character_stats || {};

    let characterToDelete: number | null = null;

    function getCharacterStats(
        characterId: number,
    ): CharacterStats | undefined {
        return stats?.character_stats[characterId];
    }

    $: allIndividualGold = Object.values(stats?.character_stats || {})
        .map((c) => c.total_combined_treasure)
        .reduce((acc, val) => acc + val, 0);
    // TODO: refactor with CampaignSummaryTab, and consider making a struct
    function getEquityStats(character: Character) {
        const charStats = characterStats[character.id];
        if (!charStats) return null;

        const expectedGoldShare =
            (allIndividualGold || 0) / (characters.length || 1);
        const goldDeno = expectedGoldShare ? expectedGoldShare : 1;

        return {
            goldShare: charStats.total_combined_treasure,
            expectedGoldShare,
            goldPercent: (
                (charStats.total_combined_treasure / goldDeno) *
                100
            ).toFixed(1),
            permanentItems: charStats.total_permanent_items.length,
            expectedPermanentItems: Object.values(
                charStats.expected_boosts || {},
            ).length,
            availableBoosts: charStats.available_boosts,
            expectedBoosts: charStats.expected_boosts,
            missingBoosts: charStats.expected_boosts.filter(
                (expected) =>
                    !charStats.available_boosts.some(
                        (available) =>
                            available.boost_category_id ===
                                expected.boost_category_id &&
                            available.potency === expected.potency,
                    ),
            ),
        };
    }

    async function handleCharacterDelete(id: number) {
        try {
            await characterStore.deleteCharacter(selectedCampaignId, id);
        } catch (e) {
            error =
                e instanceof Error ? e.message : "Failed to delete character";
        }
    }
    $: unassignedItemsValue = (stats?.unassigned_items || []).reduce(
        (acc, itemId) => {
            const item = items.entities.get(itemId);
            return acc + (item?.price || 0);
        },
        0,
    );

    onMount(() => {
        let ids: number[] = [];
        if (selectedCampaignId) {
            for (const character of characters) {
                if (characterStats[character.id]) {
                    ids = ids.concat(
                        characterStats[character.id].total_permanent_items,
                    );
                    ids = ids.concat(
                        characterStats[character.id].total_consumable_items,
                    );
                }
            }
            ids = ids.concat(stats?.unassigned_items || []);
            ids = [...new Set(ids)]; // Remove duplicates
            itemStore.fetchEntities({
                ids: ids.join(","),
            });
        }

        console.log("characterStats", characterStats);
    });
</script>

<div class="characters-section" transition:fade>
    <div class="characters-header">
        <h2>Characters</h2>
        <span>Total Treasure: {stats?.total_combined_treasure || 0}g</span>
        <span>Total Sum Gold: {stats?.total_gold || 0}g</span>
        <span
            >Total Sum Char Items Gold Value: {characters.reduce(
                (acc, char) => {
                    const charStats = getCharacterStats(char.id);
                    return (
                        acc +
                        (charStats?.total_treasure_items_value || 0) +
                        (charStats?.total_gold || 0)
                    );
                },
                0,
            )}g</span
        >
        <span
            >Total Unassigned Items Treasure Value : {unassignedItemsValue}g</span
        >
        <Button
            colour="green"
            onclick={() => {
                showNewCharacterModal = true;
                editingCharacter = null;
            }}>Add Character</Button
        >
    </div>

    <div class="character-list">
        {#each characters as character}
            {@const charStats = getCharacterStats(character.id)}
            {@const equity = getEquityStats(character)}
            {@const charItems = (charStats?.total_permanent_items || [])
                .concat(charStats?.total_consumable_items || [])
                .toSorted((a, b) => {
                    const itemA = items.entities.get(a);
                    const itemB = items.entities.get(b);
                    return (itemB?.price || 0) - (itemA?.price || 0);
                })}
            <div class="character-row" transition:fade>
                <div class="character-main">
                    <div class="character-identity">
                        <h3>{character.name}</h3>
                        <div class="character-subtitle">
                            Level {campaign?.level || "?"}
                            {classes.entities.get(character.class)?.name}
                        </div>
                    </div>
                    <div class="character-actions">
                        <Button
                            colour="blue"
                            onclick={() => {
                                editingCharacter = character;
                                showNewCharacterModal = true;
                            }}>Edit</Button
                        >
                        <Button
                            colour="red"
                            onclick={() => (characterToDelete = character.id)}
                        >
                            Delete
                        </Button>
                    </div>
                </div>

                <div class="character-content">
                    <Card>
                        <h4>Treasure & Items</h4>
                        {#if equity}
                            <div class="equity-stats">
                                <div
                                    class="equity-stat"
                                    class:deficit={equity.goldShare <
                                        equity.expectedGoldShare}
                                    class:surplus={equity.goldShare >=
                                        equity.expectedGoldShare}
                                >
                                    <span class="stat-label"
                                        >Treasure Share</span
                                    >
                                    <span class="stat-value"
                                        >{equity.goldShare.toFixed(1)}</span
                                    >
                                    <span class="stat-subtext"
                                        >({equity.goldPercent}% of fair share {equity.expectedGoldShare.toFixed(
                                            1,
                                        )})</span
                                    >
                                </div>
                                <div
                                    class="equity-stat"
                                    class:deficit={equity.permanentItems <
                                        equity.expectedPermanentItems}
                                    class:surplus={equity.permanentItems >=
                                        equity.expectedPermanentItems}
                                >
                                    <span class="stat-label"
                                        >Permanent Items</span
                                    >
                                    <span class="stat-value"
                                        >{equity.permanentItems}/{equity.expectedPermanentItems}</span
                                    >
                                </div>
                                <div class="equity-stat">
                                    <span class="stat-label"
                                        >Treasure currency</span
                                    >
                                    <span class="stat-value">
                                        {charStats?.total_gold || 0}g
                                    </span>
                                </div>
                                <div class="equity-stat">
                                    <span class="stat-label"
                                        >Treasure items</span
                                    >
                                    <span class="stat-value">
                                        {charStats?.total_treasure_items_value}g
                                    </span>
                                </div>
                            </div>
                        {/if}
                        {#if charStats}
                            <div class="equity-items">
                                {#each charItems as item}
                                    {@const itemStats =
                                        items.entities.get(item)}
                                    {#if itemStats}
                                        <Card tight>
                                            <div class="equity-item">
                                                <span class="equity-item-name"
                                                    >{itemStats?.name ||
                                                        "Unknown Item"}</span
                                                >
                                                <span class="equity-item-gold">
                                                    {#if itemStats?.price}
                                                        {itemStats.price}g
                                                    {:else}
                                                        No Price
                                                    {/if}
                                                </span>
                                            </div>
                                        </Card>
                                    {/if}
                                {/each}
                            </div>
                        {/if}
                    </Card>

                    <Card>
                        <h4>Boosts</h4>
                        {#if equity}
                            <div class="boosts-grid">
                                <Card shadowed={false} background="light">
                                    <div class="boost-section">
                                        <h5>Available Boosts</h5>
                                        {#if equity.availableBoosts.length}
                                            {#each equity.availableBoosts as boost}
                                                <div class="boost-item">
                                                    <span class="boost-name"
                                                        >{boost.boost_category_name}</span
                                                    >
                                                    <span class="boost-potency"
                                                        >+{boost.potency}</span
                                                    >
                                                </div>
                                            {/each}
                                        {:else}
                                            <div class="empty-state">
                                                No boosts available
                                            </div>
                                        {/if}
                                    </div>
                                </Card>
                                <Card shadowed={false} background="light">
                                    <div class="boost-section">
                                        <h5>Missing Boosts</h5>
                                        {#if equity.missingBoosts.length}
                                            {#each equity.missingBoosts as boost}
                                                <div class="boost-item missing">
                                                    <span class="boost-name"
                                                        >{boost.boost_category_name}</span
                                                    >
                                                    <span class="boost-potency"
                                                        >+{boost.potency}</span
                                                    >
                                                </div>
                                            {/each}
                                        {:else}
                                            <div class="empty-state">
                                                No missing boosts
                                            </div>
                                        {/if}
                                    </div>
                                </Card>
                            </div>
                        {/if}
                    </Card>
                </div>
            </div>
        {/each}

        <Card>
            <h3>Unassigned Items</h3>
            <div>
                <span class="stat-label">Total Unassigned Gold:</span>
                <span class="stat-value">{stats?.unassigned_gold || 0}g</span>
            </div>
            <div>
                <span class="stat-label">Total Unassigned Items Value:</span>
                <span class="stat-value">{unassignedItemsValue}g</span>
            </div>

            {@const unassignedItemsSorted = (
                stats?.unassigned_items || []
            ).toSorted((a, b) => {
                const itemA = items.entities.get(a);
                const itemB = items.entities.get(b);
                return (itemB?.price || 0) - (itemA?.price || 0);
            })}
            {#each unassignedItemsSorted as item}
                {@const itemStats = items.entities.get(item)}
                {#if itemStats}
                    <Card tight>
                        <div class="equity-item">
                            <span class="equity-item-name"
                                >{itemStats?.name || "Unknown Item"}</span
                            >
                            <span class="equity-item-gold">
                                {#if itemStats?.price}
                                    {itemStats.price}g
                                {:else}
                                    No Price
                                {/if}
                            </span>
                        </div>
                    </Card>
                {/if}
            {/each}
        </Card>
    </div>
</div>

<CharacterModal
    bind:show={showNewCharacterModal}
    campaignId={selectedCampaignId}
    bind:editingCharacter
    on:saved={async (event) => {
        if (editingCharacter) {
            await characterStore.updateCharacter(
                selectedCampaignId,
                editingCharacter.id,
                event.detail,
            );
        } else {
            await characterStore.addCharacters(selectedCampaignId, [
                event.detail,
            ]);
        }
        editingCharacter = null;
    }}
    on:close={() => {
        editingCharacter = null;
    }}
/>

<ConfirmationModal
    show={characterToDelete !== null}
    {error}
    on:confirm={() => {
        if (characterToDelete) handleCharacterDelete(characterToDelete);
        characterToDelete = null;
    }}
    on:close={() => {
        characterToDelete = null;
    }}
    confirmationString="Delete '{characters.find(
        (c) => c.id === characterToDelete,
    )?.name}'"
>
    Are you sure you would like to delete the character "{characters.find(
        (c) => c.id === characterToDelete,
    )?.name}"?
</ConfirmationModal>

<style>
    .characters-section {
        background: var(--color-bg);
        border-radius: 0.5rem;
        box-shadow: var(--shadow);
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
        background: var(--color-bg);
        border-radius: 0.5rem;
        box-shadow: var(--shadow);
        overflow: hidden;
    }

    .character-main {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 1.5rem;
        background: var(--color-bg-light-raised);
        border-bottom: 1px solid var(--color-bg-light-raised-border);
    }

    .character-identity {
        display: flex;
        flex-direction: row;
        gap: 1rem;
    }

    .character-identity h3 {
        margin: 0;
        font-size: 1.25rem;
    }

    .character-subtitle {
        color: var(--color-text-secondary);
        font-size: 0.875rem;
        margin-top: 0.25rem;
    }

    .character-content {
        padding: 1.5rem;
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
        gap: 1.5rem;
    }

    .equity-stats {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
        gap: 1rem;
    }

    .equity-stat {
        padding: 0.75rem;
        background: var(--color-bg);
        border-radius: 0.375rem;
        box-shadow: var(--shadow);
    }

    .equity-items {
        display: flex;
        flex-direction: column;
    }

    .equity-item {
        display: flex;
        justify-content: space-between;
        padding: 0.5rem;
        background: var(--color-bg-light-raised);
        border-radius: 0.25rem;
        margin-bottom: -0.5rem;
    }

    .stat-label {
        font-size: 0.75rem;
        color: var(--color-text-secondary);
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
        color: var(--color-text-secondary);
    }

    .boosts-grid {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
        gap: 1rem;
    }

    .boost-section h5 {
        font-size: 0.875rem;
        color: var(--color-text-secondary);
        margin: 0 0 0.75rem 0;
    }

    .boost-item {
        display: flex;
        justify-content: space-between;
        padding: 0.5rem;
        background: var(--color-bg);
        border-radius: 0.25rem;
        margin-bottom: 0.5rem;
    }

    .boost-item.missing {
        background: var(--color-bg-error);
        color: var(--color-text-error);
    }

    .deficit .stat-value {
        color: var(--color-text-error);
    }

    .surplus .stat-value {
        color: var(--color-text-success);
    }

    .empty-state {
        color: var(--color-text-secondary);
        font-size: 0.875rem;
        text-align: center;
        padding: 1rem;
    }
</style>

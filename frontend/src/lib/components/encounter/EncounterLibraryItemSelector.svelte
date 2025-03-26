<script lang="ts">
    import { creatureStore, hazardStore, itemStore } from "$lib/stores/libraryStore";
    import type { EncounterEnemy } from "$lib/types/encounters";
    import { getFullUrl, type LibraryCreature, type LibraryHazard, type LibraryItem } from "$lib/types/library";
    import { FontAwesomeIcon } from "@fortawesome/svelte-fontawesome";
    import Card from "../core/Card.svelte";
    import Button from "../core/Button.svelte";
    import LibrarySelector from "../selectors/LibrarySelector.svelte";
    import BrowseLibraryModal from "../modals/BrowseLibraryModal.svelte";
    import { getExperienceFromLevel } from "$lib/utils/encounter";

    interface Props {
        libraryObjectType : "item" | "creature" | "hazard"  ;
        partyLevel : number;
        data : (EncounterEnemy | number)[];
        libraryModal : BrowseLibraryModal;
    }
    let {
        libraryObjectType,
        partyLevel = 1,
        data = $bindable(),
        libraryModal
    }: Props = $props();

    let libraryCreatures = $derived($creatureStore);
    let libraryHazards = $derived($hazardStore);
    let libraryItems = $derived($itemStore);

    let dataAsIds = $derived(data.map(getId));

    function getObjectContext(id : number) : LibraryCreature | LibraryItem | LibraryHazard | null {
        switch (libraryObjectType) {
            case "creature":
                return libraryCreatures.entities.get(id) || null;
            case "item":
                return libraryItems.entities.get(id) || null;
            case "hazard":
                return libraryHazards.entities.get(id) || null;
        }
    }

    function addIdToData(id : number) {
        switch (libraryObjectType) {
            case "creature":
                data = [...data, {
                    id,
                    level_adjustment: 0
                }];
                break;
            case "item":
                data = [...data, id];
                break;
            case "hazard":
                data = [...data, id];
                break;
        }
    }

    function getId(o : EncounterEnemy | number) : number {
        return typeof o === "number" ? o : o.id;
    }

    function getEnemyAdjustment(object : EncounterEnemy | number) : number | undefined {
        if (libraryObjectType !== "creature") return undefined;
        if (typeof object === "number") return undefined;
        return object.level_adjustment;
    }

    function openLibrary() {
        switch (libraryObjectType) {
            case "creature":
                libraryModal.showWithTabs(["creature"]);
                break;
            case "item":
                libraryModal.showWithTabs(["item"]);
                break;
            case "hazard":
                libraryModal.showWithTabs(["hazard"]);
                break;
        }
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

    function toggleEnemyAdjustment(enemyId: number) {
        if (libraryObjectType !== "creature") return;

        let cycleOrder = new Map([
            [0, 1],
            [1, -1],
            [-1, 0],
        ]);
        let encounterEnemy = data[enemyId];
        if (typeof encounterEnemy !== "object") return;
        if ("level_adjustment" in encounterEnemy === false) return;
        encounterEnemy.level_adjustment =
            cycleOrder.get(encounterEnemy.level_adjustment) || 0;
    }

</script>

<div class="section-content">
   <div class="list-items">
         {#each data as o, i}
            {@const context = getObjectContext(getId(o))}
          {#if context}
               <Card tight>
                    <div class="list-item">
                        <div class="list-item-span-group">
                        {#if libraryObjectType === "creature"}
                            <div class="entity-adjustment">
                                <Button
                                    tight
                                    colour={getAdjustmentColour(
                                        getEnemyAdjustment(o) ?? 0,
                                    )}
                                    onclick={() => {
                                        toggleEnemyAdjustment(
                                            i,
                                        );
                                    }}
                                    >{getAdjustmentName(
                                        getEnemyAdjustment(o) ?? 0,
                                    )}</Button
                                >
                            </div>

                        {/if}
                        <span>{context.name}</span>
                        </div>

                        <div class="list-item-span-group">
                        <div class="entity-link">
                            <a
                                href={getFullUrl(
                                    context.url ||
                                        "",
                                )}
                                target="_blank"
                                rel="noopener noreferrer"
                            >
                                <FontAwesomeIcon
                                    icon={["fas", "link"]}
                                />
                            </a>
                        </div>
                        {#if libraryObjectType === "item"}
                            {#if 'price' in context && context.price}
                                <span
                                    >Value: {context?.price}g</span
                                >
                            {:else}
                                <span>Value: Priceless</span>
                            {/if}
                        {:else if libraryObjectType === "creature"}
                            <div class="entity-xp">
                                XP: {getExperienceFromLevel(
                                    partyLevel,
                                    (context?.level ?? 0) +
                                        (getEnemyAdjustment(o) ?? 0),
                                )}
                            </div>
                            <div class="entity-level">
                                Level {(context?.level || 0) +
                                    (getEnemyAdjustment(o) || 0)}
                            </div>
                        {/if}

                        <Button
                            colour="red"
                            onclick={() => {
                                data = data.filter(
                                    (_, index) =>
                                        index !== i,
                                );
                            }}>Remove</Button
                        >
                    </div>
                    </div>
                </Card>
            {/if}
        {/each}
    </div> 
    <div class="library-selector-container">
        <LibrarySelector
            entityType={libraryObjectType}
            onSelect={(id) => {addIdToData(id);}}
            placeholder="Search for items..."
            initialIds={dataAsIds}
        />
        <Button colour="blue" onclick={() => openLibrary()}
            >📚 Browse Library</Button
        >
    </div>
</div>


<style>
    .list-item {
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    .list-item-span-group {
        display: flex;
        flex-direction: row;
        gap: 1rem;
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
</style>
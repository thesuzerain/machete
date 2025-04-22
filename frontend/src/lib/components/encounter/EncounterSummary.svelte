<script lang="ts">
    import {
        creatureStore,
        hazardStore,
        itemStore,
    } from "$lib/stores/libraryStore";
    import type { Encounter } from "$lib/types/encounters";
    import { getFullUrl } from "$lib/types/library";
    import { FontAwesomeIcon } from "@fortawesome/svelte-fontawesome";
    import Card from "../core/Card.svelte";
    import {
        EncounterDifficulty,
        getCreatureExperienceFromLevel,
        getHazardExperienceFromLevel,
        getSeverityFromFinalExperience,
    } from "$lib/utils/encounter";
    import { faLink } from "@fortawesome/free-solid-svg-icons";
    import { campaignSessionStore } from "$lib/stores/campaignSessions";
    import {
        campaignStore,
        selectedCampaignStore,
    } from "$lib/stores/campaigns";
    import { onMount } from "svelte";

    interface Props {
        encounter: Encounter;
        size?: "title" | "short" | "normal" | "detailed";
        expectedPartySize?: number;
        expectedPartyLevel?: number;
    }
    let {
        encounter,
        size = "normal",
        expectedPartySize = encounter.party_size,
        expectedPartyLevel = encounter.party_level,
    }: Props = $props();

    let items = $derived($itemStore);
    let creatures = $derived($creatureStore);
    let globalCampaignId = $derived($selectedCampaignStore);
    let campaigns = $derived($campaignStore);
    let campaignSessions = $derived(
        $campaignSessionStore.get(globalCampaignId || 0) || [],
    );

    onMount(async () => {
        // TODO: Refactor, modular?

        await Promise.all([
            itemStore.fetchEntities({
                ids: encounter.treasure_items.join(","),
            }),
            creatureStore.fetchEntities({
                ids: (encounter.enemies ?? []).map((e) => e.id).join(","),
            }),
            hazardStore.fetchEntities({
                ids: (encounter.hazards ?? []).join(","),
            }),
        ]);
    });

    // TODO: modularize, along with css classes
    function getClassForDifficulty(difficulty: EncounterDifficulty): string {
        switch (difficulty) {
            case "Trivial":
                return "difficulty-trivial";
            case "Low":
                return "difficulty-low";
            case "Moderate":
                return "difficulty-moderate";
            case "Severe":
                return "difficulty-severe";
            case "Extreme":
                return "difficulty-extreme";
            default:
            return "difficulty-unknown";
        }
    }

    function getAdjustmentName(adjustment: number): string {
        if (adjustment === 0) return "Normal";
        return adjustment > 0 ? "Elite" : "Weak";
    }

    let sessionIx = $derived.by(() => {
        let sessionIx: Map<number, number> = new Map();
        campaignSessions.forEach((session, ix) => {
            sessionIx.set(session.id, ix);
        });
        return sessionIx;
    });
</script>

{#if size == "short"}
    <Card tight>
        <div class="short-card">
            <div class="short-info">
                <h4>{encounter.name}</h4>
                {#if encounter.total_experience > 0}
                    <span>XP: {encounter.total_experience}</span>
                {/if}
                {#if encounter.treasure_items.length > 0}
                    <span
                        >Items: {encounter.treasure_items
                            .map((item) => items.entities.get(item)?.name)
                            .join(", ")}</span
                    >
                {/if}
                {#if encounter.treasure_currency > 0}
                    <span>Currency: {encounter.treasure_currency}g</span>
                {/if}
            </div>
            <div class="encounter-actions">
                <slot />
            </div>
        </div>
    </Card>
{:else if size == "normal"}
    <Card
        ><div class="normal-card">
            <div class="normal-info">
                <div class="normal-info-row">
                    <h4>{encounter.name}</h4>
                    <span class="encounter-normal-title-row">
                        (<span
                            class={getClassForDifficulty(
                                getSeverityFromFinalExperience(
                                    encounter.total_experience,
                                    encounter.extra_experience,
                                ),
                            )}
                            >{getSeverityFromFinalExperience(
                                encounter.total_experience,
                                encounter.extra_experience,
                            )}</span
                        >)
                        {#if encounter.party_size != expectedPartySize}
                            (Party size: {encounter.party_size})
                        {/if}
                        {#if encounter.party_level != expectedPartyLevel}
                            (Party level: {encounter.party_level})
                        {/if}
                    </span>
                </div>
                {#if encounter.enemies && encounter.enemies.length > 0}
                    <div class="normal-info-row">
                        <h5>Enemies:</h5>
                        <div class="normal-info-names">
                            {#each encounter.enemies || [] as enemy, i}
                                {@const enemyData = creatures.entities.get(
                                    enemy.id,
                                )}
                                {#if enemyData}
                                    <div>
                                        <span>
                                            {#if i > 0},&nbsp;{/if}{enemyData.name}
                                            {#if enemy.level_adjustment > 0}(Elite){:else if enemy.level_adjustment < 0}(Weak){/if}</span
                                        >
                                    </div>{/if}
                            {/each}
                        </div>
                    </div>
                {/if}
                {#if encounter.hazards && encounter.hazards.length > 0}
                    <div class="normal-info-row">
                        <h5>Hazards:</h5>
                        <div class="normal-info-names">
                            {#each encounter.hazards || [] as hazard, i}
                                {@const hazardData =
                                    creatures.entities.get(hazard)}
                                {#if hazardData}
                                    <div>
                                        <span>
                                            {#if i > 0},&nbsp;{/if}{hazardData.name}
                                        </span>
                                    </div>{/if}
                            {/each}
                        </div>
                    </div>
                {/if}
                {#if encounter.treasure_items.length > 0}
                    <div class="normal-info-row">
                        <h5>Items:</h5>
                        <div class="normal-info-names">
                            {#each encounter.treasure_items || [] as item, i}
                                {@const itemData = items.entities.get(item)}
                                {#if itemData}
                                    <div>
                                        <span>
                                            {#if i > 0},&nbsp;{/if}{itemData.name}
                                        </span>
                                    </div>
                                {/if}
                            {/each}
                        </div>
                    </div>
                {/if}

                <div class="normal-info-row">
                    <p>
                        XP: {encounter.total_experience}, Gold: {encounter.treasure_currency}
                    </p>
                </div>
            </div>
            <div class="encounter-actions">
                <slot />
            </div>
        </div>
    </Card>
{:else if size == "detailed"}
    <div class="detailed-meta">
        <span class="xp"
            >XP: {encounter.total_experience}
            (<span
                class={getClassForDifficulty(
                    getSeverityFromFinalExperience(
                        encounter.total_experience,
                        encounter.extra_experience,
                    ),
                )}
                >{getSeverityFromFinalExperience(
                    encounter.total_experience,
                    encounter.extra_experience,
                )}</span
            >)
        </span>
        <span class="party"
            >Level {encounter.party_level} ({encounter.party_size} players)</span
        >
    </div>

    <div class="detailed-description">
        <p>{encounter.description}</p>
    </div>

    <div class="detailed-details">
        {#if encounter.enemies && encounter.enemies.length > 0}
            <Card>
                <h3>Enemies ({encounter.enemies.length})</h3>
                <ul>
                    {#each encounter.enemies as enemy}
                        {@const enemyDetails = $creatureStore.entities.get(
                            enemy.id,
                        )}
                        {#if enemyDetails}
                            <li class="enemy-item">
                                <span class="enemy-name"
                                    >{enemyDetails?.name}</span
                                >
                                {#if enemy.level_adjustment !== 0}
                                    <span class="adjustment"
                                        >({getAdjustmentName(
                                            enemy.level_adjustment,
                                        )})</span
                                    >
                                {/if}
                                <span class="enemy-level"
                                    >Level {(enemyDetails?.level || 0) +
                                        enemy.level_adjustment}</span
                                >
                                <span class="enemy-xp"
                                    >XP: {getCreatureExperienceFromLevel(
                                        encounter.party_level,
                                        enemyDetails?.level || 0,
                                    )}</span
                                >
                                <a
                                    href={getFullUrl(enemyDetails?.url || "")}
                                    target="_blank"
                                    rel="noopener noreferrer"
                                    class="entity-link"
                                >
                                    <FontAwesomeIcon icon={faLink} />
                                </a>
                            </li>
                        {/if}
                    {/each}
                </ul>
            </Card>
        {/if}

        {#if encounter.hazards && encounter.hazards.length > 0}
            <Card>
                <h3>Hazards ({encounter.hazards.length})</h3>
                <ul>
                    {#each encounter.hazards as hazardId}
                        {@const hazardDetails =
                            $hazardStore.entities.get(hazardId)}
                        {#if hazardDetails}
                            <li class="hazard-item">
                                <span class="hazard-name"
                                    >{hazardDetails.name}</span
                                >
                                <span class="hazard-xp"
                                    >XP: {getHazardExperienceFromLevel(
                                        encounter.party_level,
                                        hazardDetails.level || 0,
                                        hazardDetails.complex,
                                    )}</span
                                >
                                <a
                                    href={getFullUrl(hazardDetails?.url || "")}
                                    target="_blank"
                                    rel="noopener noreferrer"
                                    class="entity-link"
                                >
                                    <FontAwesomeIcon icon={faLink} />
                                </a>
                            </li>
                        {/if}
                    {/each}
                </ul>
            </Card>
        {/if}

        {#if encounter.subsystem_type}
            <Card>
                <h3>Subsystem Challenge</h3>
                <p class="subsystem-type">Type: {encounter.subsystem_type}</p>
                {#if encounter.subsystem_checks && encounter.subsystem_checks.length > 0}
                    <ul>
                        {#each encounter.subsystem_checks as check}
                            <li class="check-item">
                                <div class="check-header">
                                    <span class="check-name">{check.name}</span>
                                    <span>VP: {check.vp}</span>
                                </div>
                                <div class="check-options">
                                    {#each check.roll_options as roll, i}
                                        <span class="roll-option">
                                            {roll.skill} DC {roll.dc}{#if i < check.roll_options.length - 1},&nbsp;{/if}
                                        </span>
                                    {/each}
                                </div>
                            </li>
                        {/each}
                    </ul>
                {/if}
            </Card>
        {/if}

        {#if encounter.treasure_currency > 0 || encounter.treasure_items.length > 0}
            <Card>
                <h3>Treasure</h3>
                <p class="currency">
                    Currency: {encounter.treasure_currency}gp
                </p>
                {#if encounter.treasure_items && encounter.treasure_items.length > 0}
                    <ul>
                        {#each encounter.treasure_items as itemId}
                            {@const itemDetails =
                                $itemStore.entities.get(itemId)}
                            {#if itemDetails}
                                <li class="item-entry">
                                    <span class="item-name"
                                        >{itemDetails?.name}</span
                                    >
                                    <a
                                        href={getFullUrl(
                                            itemDetails?.url || "",
                                        )}
                                        target="_blank"
                                        rel="noopener noreferrer"
                                        class="entity-link"
                                    >
                                        <FontAwesomeIcon icon={faLink} />
                                    </a>
                                </li>
                            {/if}
                        {/each}
                    </ul>
                {/if}
            </Card>
        {/if}
    </div>
    <div class="encounter-actions">
        <slot />
    </div>

{:else if size == "title"}
    <div class="encounter-title">
        <h3>{encounter.name}</h3>
        <div class="encounter-title-meta">
            {#if encounter.session_id && encounter.campaign_id && encounter.campaign_id == globalCampaignId}
                <span class="status linked"
                    >Linked: Session {sessionIx.get(encounter.session_id)}</span
                >
            {:else if encounter.campaign_id}
                <span class="status linked"
                    >Linked: Campaign {campaigns.get(encounter.campaign_id)
                        ?.name}</span
                >
            {:else}
                <span class="status prepared">Prepared</span>
            {/if}
            <span class="xp"
                >XP: {encounter.total_experience} (<span
                    class={getClassForDifficulty(
                        getSeverityFromFinalExperience(
                            encounter.total_experience,
                            encounter.extra_experience,
                        ),
                    )}
                    >{getSeverityFromFinalExperience(
                        encounter.total_experience,
                        encounter.extra_experience,
                    ).toWellFormed()}</span
                >)</span
            >
            <span class="party"
                >Level {encounter.party_level} ({encounter.party_size} players)</span
            >
        </div>
    </div>
{/if}

<div></div>

<style>
    .short-card {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding-left: 0.5rem;
        padding-right: 0.5rem;
    }
    .short-info {
        display: flex;
        gap: 1rem;
    }
    .normal-card {
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    .encounter-actions {
        display: flex;
        gap: 0.5rem;
    }

    .normal-info h4 {
        margin-bottom: 0.75rem;
        color: var(--color-text-secondary);
    }

    .normal-info h5 {
        color: var(--color-text-secondary);
        font-size: 1rem;
    }

    .normal-info-row {
        display: flex;
        gap: 0.5rem;
    }

    .encounter-normal-title-row-piece {
        display: flex;
    }

    .normal-info-names {
        display: flex;
        flex-wrap: wrap;
    }

    .detailed-meta {
        display: flex;
        gap: 1rem;
        align-items: center;
        margin-bottom: 1.5rem;
    }

    .encounter-title-meta {
        display: flex;
        gap: 1rem;
        align-items: center;
        margin-bottom: 0rem;
    }

    .detailed-description {
        margin-bottom: 1.5rem;
    }

    .detail-section {
        margin-bottom: 2rem;
        padding: 1rem;
        background: var(--color-bg-light-raised);
        border-radius: 8px;
    }

    .detailed-details ul {
        list-style: none;
        padding: 0;
        margin: 0;
    }

    .detailed-details li {
        padding: 0.75rem;
        background: var(--color-bg);
        border-radius: 4px;
        margin-bottom: 0.5rem;
    }

    .enemy-item,
    .hazard-item,
    .item-entry {
        display: flex;
        align-items: center;
        gap: 1rem;
    }

    .enemy-name,
    .hazard-name,
    .item-name {
        font-weight: 500;
        flex: 1;
    }

    .adjustment {
        color: var(--color-text-secondary);
        font-style: italic;
    }

    .enemy-level,
    .enemy-xp,
    .hazard-xp {
        color: var(--color-text-secondary);
        white-space: nowrap;
    }

    .check-item {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    .check-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    .check-name {
        font-weight: 500;
    }

    .check-options {
        font-size: 0.9rem;
    }

    .entity-link {
        color: var(--color-text-link);
        text-decoration: none;
    }

    .entity-link:hover {
        color: var(--color-text-link-hover);
    }

    .currency {
        margin-bottom: 1rem;
    }

    .encounter-title {
        display: flex;
        align-items: center;
        gap: 2rem;
        flex: 1;
    }

    .encounter-title h3 {
        margin: 0;
        min-width: 200px;
    }

    .detailed-details {
        display: flex;
        flex-direction: column;
        gap: 1rem;
        margin-bottom: 2rem;
    }

    /* Difficulty colors */
    .difficulty-trivial {
        color: var(--color-difficulty-trivial);
    }
    .difficulty-low {
        color: var(--color-difficulty-low);
    }
    .difficulty-moderate {
        color: var(--color-difficulty-moderate);
    }
    .difficulty-severe {
        color: var(--color-difficulty-severe);
    }
    .difficulty-extreme {
        color: var(--color-difficulty-extreme);
    }


    .status {
        padding: 0.25rem 0.75rem;
        border-radius: 999px;
        font-size: 0.75rem;
        font-weight: 500;
        text-transform: uppercase;
        letter-spacing: 0.05em;
    }

    .status.prepared {
        background: #dbeafe;
        color: #1e40af;
    }

    .status.linked {
        background: #dcfce7;
        color: #166534;
    }
</style>

<script lang="ts">
    import { fade } from 'svelte/transition';
    import { FontAwesomeIcon } from '@fortawesome/svelte-fontawesome';
    import { faLink } from '@fortawesome/free-solid-svg-icons';
    import type { Encounter, EncounterEnemy } from '$lib/types/encounters';
    import { EncounterDifficulty, getExperienceFromLevel, getSeverityFromFinalExperience } from '$lib/utils/encounter';
    import { getFullUrl } from '$lib/types/library';
    import { creatureStore, hazardStore, itemStore } from '$lib/stores/libraryStore';
    import Modal from '../core/Modal.svelte';

    interface Props {
        show: boolean;
        encounter: Encounter | null;
    }

    let { 
        show = $bindable(),
        encounter = $bindable()
    } : Props = $props();


    let libraryEnemies = $derived($creatureStore);
    let libraryHazards = $derived($hazardStore);
    let libraryItems = $derived($itemStore);

    function getEnemyDetails(id: number) {
        return libraryEnemies.entities.get(id);
    }

    function getHazardDetails(id: number) {
        return libraryHazards.entities.get(id);
    }

    function getItemDetails(id: number) {
        return libraryItems.entities.get(id);
    }

        // TODO: modularize, along with css classes
    function getClassForDifficulty(difficulty: EncounterDifficulty): string {
        switch (difficulty) {
            case 'Trivial':
                return 'difficulty-trivial';
            case 'Low':
                return 'difficulty-low';
            case 'Moderate':
                return 'difficulty-moderate';
            case 'Severe':
                return 'difficulty-severe';
            case 'Extreme':
                return 'difficulty-extreme';
            default:
                return '';
        }
    }

    function getAdjustmentName(adjustment: number): string {
        if (adjustment === 0) return 'Normal';
        return adjustment > 0 ? 'Elite' : 'Weak';
    }

    function closeModal() {
        show = false;
    }
</script>

<Modal show={show && !!encounter}>
    <div class="modal-header">
        <h2>{encounter.name}</h2>
        <button class="close-button" on:click={closeModal}>×</button>
    </div>

    <div class="encounter-meta">
        <span class="status {encounter.status.toLowerCase()}">{encounter.status}</span>
        <span class="xp">XP: {encounter.total_experience} 
            (<span class="{getClassForDifficulty(getSeverityFromFinalExperience(encounter.total_experience, encounter.extra_experience))}"
            >{getSeverityFromFinalExperience(encounter.total_experience, encounter.extra_experience)}</span>)
        </span>
        <span class="party">Level {encounter.party_level} ({encounter.party_size} players)</span>
    </div>

    <div class="encounter-description">
        <p>{encounter.description}</p>
    </div>

    <div class="encounter-details">
        {#if encounter.enemies && encounter.enemies.length > 0}
            <div class="detail-section">
                <h3>Enemies ({encounter.enemies.length})</h3>
                <ul>
                    {#each encounter.enemies as enemy}
                        {#if getEnemyDetails(enemy.id)}
                            <li class="enemy-item">
                                <span class="enemy-name">{getEnemyDetails(enemy.id)?.name}</span>
                                {#if enemy.level_adjustment !== 0}
                                    <span class="adjustment">({getAdjustmentName(enemy.level_adjustment)})</span>
                                {/if}
                                <span class="enemy-level">Level {(getEnemyDetails(enemy.id)?.level || 0) + enemy.level_adjustment}</span>
                                <span class="enemy-xp">XP: {getExperienceFromLevel(encounter.party_level, getEnemyDetails(enemy.id)?.level || 0)}</span>
                                <a href={getFullUrl(getEnemyDetails(enemy.id)?.url || '')} target="_blank" rel="noopener noreferrer" class="entity-link">
                                    <FontAwesomeIcon icon={faLink} />
                                </a>
                            </li>
                        {/if}
                    {/each}
                </ul>
            </div>
        {/if}

        {#if encounter.hazards && encounter.hazards.length > 0}
            <div class="detail-section">
                <h3>Hazards ({encounter.hazards.length})</h3>
                <ul>
                    {#each encounter.hazards as hazardId}
                        {#if getHazardDetails(hazardId)}
                            <li class="hazard-item">
                                <span class="hazard-name">{getHazardDetails(hazardId)?.name}</span>
                                <span class="hazard-xp">XP: {getExperienceFromLevel(encounter.party_level, getHazardDetails(hazardId)?.level || 0)}</span>
                                <a href={getFullUrl(getHazardDetails(hazardId)?.url || '')} target="_blank" rel="noopener noreferrer" class="entity-link">
                                    <FontAwesomeIcon icon={faLink} />
                                </a>
                            </li>
                        {/if}
                    {/each}
                </ul>
            </div>
        {/if}

        {#if encounter.subsystem_type}
            <div class="detail-section">
                <h3>Subsystem Challenge</h3>
                <p class="subsystem-type">Type: {encounter.subsystem_type}</p>
                {#if encounter.subsystem_checks && encounter.subsystem_checks.length > 0}
                    <ul>
                        {#each encounter.subsystem_checks as check}
                            <li class="check-item">
                                <div class="check-header">
                                    <span class="check-name">{check.name}</span>
                                    <span class="check-vp">VP: {check.vp}</span>
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
            </div>
        {/if}

        <div class="detail-section">
            <h3>Treasure</h3>
            <p class="currency">Currency: {encounter.treasure_currency}gp</p>
            {#if encounter.treasure_items && encounter.treasure_items.length > 0}
                <ul>
                    {#each encounter.treasure_items as itemId}
                        {#if getItemDetails(itemId)}
                            <li class="item-entry">
                                <span class="item-name">{getItemDetails(itemId)?.name}</span>
                                <a href={getFullUrl(getItemDetails(itemId)?.url || '')} target="_blank" rel="noopener noreferrer" class="entity-link">
                                    <FontAwesomeIcon icon={faLink} />
                                </a>
                            </li>
                        {/if}
                    {/each}
                </ul>
            {/if}
        </div>
    </div>

</Modal>


<style>

    .modal-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 1rem;
    }

    .close-button {
        background: none;
        border: none;
        font-size: 1.5rem;
        cursor: pointer;
        padding: 0.5rem;
        color: #666;
    }

    .encounter-meta {
        display: flex;
        gap: 1rem;
        align-items: center;
        margin-bottom: 1.5rem;
    }

    .status {
        padding: 0.25rem 0.75rem;
        border-radius: 999px;
        font-size: 0.75rem;
        font-weight: 500;
        text-transform: uppercase;
        letter-spacing: 0.05em;
    }

    .status.prepared { background: #dbeafe; color: #1e40af; }
    .status.success { background: #dcfce7; color: #166534; }
    .status.failure { background: #fee2e2; color: #991b1b; }
    .status.archived { background: #f3f4f6; color: #1f2937; }
    .status.draft { background: #f3f4f6; color: #1f2937; }

    .encounter-description {
        margin-bottom: 1.5rem;
        color: #4b5563;
    }

    .detail-section {
        margin-bottom: 2rem;
        padding: 1rem;
        background: #f9fafb;
        border-radius: 8px;
    }

    .detail-section h3 {
        margin-bottom: 1rem;
        color: #111827;
    }

    ul {
        list-style: none;
        padding: 0;
        margin: 0;
    }

    li {
        padding: 0.75rem;
        background: white;
        border-radius: 4px;
        margin-bottom: 0.5rem;
    }

    .enemy-item, .hazard-item, .item-entry {
        display: flex;
        align-items: center;
        gap: 1rem;
    }

    .enemy-name, .hazard-name, .item-name {
        font-weight: 500;
        flex: 1;
    }

    .adjustment {
        color: #6b7280;
        font-style: italic;
    }

    .enemy-level, .enemy-xp, .hazard-xp {
        color: #6b7280;
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

    .check-vp {
        color: #6b7280;
    }

    .check-options {
        color: #6b7280;
        font-size: 0.9rem;
    }

    .entity-link {
        color: #3b82f6;
        text-decoration: none;
    }

    .entity-link:hover {
        color: #2563eb;
    }

    .currency {
        color: #6b7280;
        margin-bottom: 1rem;
    }

    /* Difficulty colors */
    .difficulty-trivial { color: #10b981; }
    .difficulty-low { color: #f59e0b; }
    .difficulty-moderate { color: #f59e0b; }
    .difficulty-severe { color: #ef4444; }
    .difficulty-extreme { color: #ef4444; }
</style> 
<script lang="ts">
    import { fade } from 'svelte/transition';
    import type { Character } from '$lib/types/types';
    import CharacterModal from '$lib/components/CharacterModal.svelte';
    import { characterStore } from '$lib/stores/characters';
    import { classStore } from '$lib/stores/libraryStore';

    export let selectedCampaignId: number;
    export let error: string | null;

    let showNewCharacterModal = false;
    let editingCharacter: Character | null = null;
    $: characters = $characterStore.get(selectedCampaignId) || [];
    $: classes = $classStore;

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
        
                    <div class="character-row" transition:fade>
                    <div class="character-main">
                            <div class="character-identity">
                                <h3>{character.name}</h3>
                                <div class="character-subtitle">
                                    Level {character.level} {classes.entities.get(character.class)?.name}
                                </div>
                            </div>
                            <div class="character-actions">
                                <button class="edit-btn" on:click={() => {
                                    editingCharacter = character;
                                    showNewCharacterModal = true;
                                }}>Edit</button>
                                <button class="delete-btn" on:click={() => handleCharacterDelete(character.id)}>
                                    Delete
                                </button>
                            </div>
                        </div>

                        <div class="character-content">
                            <div class="content-section">
                                <h4>Experience</h4>
                                <div class="xp-display">
                                    <div class="xp-bar" style="--progress: {(character.experience % 1000) / 1000 * 100}%">
                                        <span class="xp-text">{character.experience} / 1000 XP</span>
                                    </div>
                                </div>
                            </div>

                            <div class="content-section">
                                <h4>Recent Activity</h4>
                                <!-- Placeholder for activity graph -->
                                <div class="activity-placeholder">
                                    Activity graph coming soon
                                </div>
                            </div>

                            <div class="content-section">
                                <h4>Statistics</h4>
                                <div class="stats-grid">
                                    <div class="stat-item">
                                        <span class="stat-label">Sessions</span>
                                        <span class="stat-value">12</span>
                                    </div>
                                    <div class="stat-item">
                                        <span class="stat-label">Enemies Defeated</span>
                                        <span class="stat-value">47</span>
                                    </div>
                                    <div class="stat-item">
                                        <span class="stat-label">Gold Earned</span>
                                        <span class="stat-value">1,234</span>
                                    </div>
                                </div>
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
        showNewCharacterModal = false;
        editingCharacter = null;
    }}
    on:close={() => {
        showNewCharacterModal = false;
        editingCharacter = null;
    }}
/>

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

    .stats-grid {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
        gap: 1rem;
    }

    .stat-item {
        display: flex;
        flex-direction: column;
        align-items: center;
        text-align: center;
    }

    .stat-label {
        font-size: 0.75rem;
        color: #64748b;
        margin-bottom: 0.25rem;
    }

    .stat-value {
        font-size: 1.25rem;
        font-weight: 600;
        color: #1e293b;
    }

    .activity-placeholder {
        height: 100px;
        background: #e2e8f0;
        border-radius: 0.375rem;
        display: flex;
        align-items: center;
        justify-content: center;
        color: #64748b;
        font-size: 0.875rem;
    }

    .add-character-btn {
        background: #22c55e;
        color: white;
        padding: 0.5rem 1rem;
        border-radius: 0.375rem;
        font-weight: 500;
    }
</style> 
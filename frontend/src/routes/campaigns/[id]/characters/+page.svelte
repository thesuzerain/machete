<script lang="ts">
    import { page } from '$app/stores';
    import { onMount } from 'svelte';
    import type { Character, InsertCharacter, LibraryClass, UpdateCharacter } from '$lib/types/types';
    import LibrarySelector from '$lib/components/LibrarySelector.svelte';

    const campaignId = parseInt($page.params.id);
    let campaignCharacters: Character[] = [];
    let loading = true;
    let error: string | null = null;
    let editingCharacter: UpdateCharacter | null = null;

    let libraryClasses: Map<number, LibraryClass> = new Map();

    // Form state
    let newCharacter: InsertCharacter = {
        name: '',
        class: 0,
        level: 1,
    };

    async function loadLibraryData() {
        const response = await fetch('/api/library/classes');
        if (!response.ok) throw new Error('Failed to fetch classes');
        const classes: LibraryClass[] = await response.json();
        libraryClasses = new Map(classes.map(c => [c.id, c]));
    }

    onMount(async () => {
        try {
            const response = await fetch(`/api/campaign/${campaignId}/characters`);
            if (!response.ok) throw new Error('Failed to fetch characters');
            campaignCharacters = await response.json();
            await loadLibraryData();
        } catch (e) {
            error = e instanceof Error ? e.message : 'An error occurred';
        } finally {
            loading = false;
        }
    });

    async function addCharacter(event: SubmitEvent) {
        event.preventDefault();
        
        try {
            const response = await fetch(`/api/campaign/${campaignId}/characters`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify([newCharacter]),
            });

            if (!response.ok) throw new Error('Failed to create character');
            
            await fetchCharacters();
            
            // Reset form
            newCharacter = {
                name: '',
                class: 0,
                level: 1,
            };
        } catch (e) {
            error = e instanceof Error ? e.message : 'Failed to create character';
        }
    }

    async function updateCharacter(character: UpdateCharacter | null) {
        if (!character) return;
        try {
            const response = await fetch(`/api/campaign/${campaignId}/characters/${character.id}`, {
                method: 'PATCH',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(character),
            });

            if (!response.ok) throw new Error('Failed to update character');
            
            await fetchCharacters();
            editingCharacter = null;
        } catch (e) {
            error = e instanceof Error ? e.message : 'Failed to update character';
        }
    }

    async function fetchCharacters() {
        try {
            const response = await fetch(`/api/campaign/${campaignId}/characters`);
            if (!response.ok) throw new Error('Failed to fetch characters');
            campaignCharacters = await response.json();
        } catch (e) {
            error = e instanceof Error ? e.message : 'Failed to fetch characters';
        }
    }

    function getClassDetails(classId: number) {
        return libraryClasses.get(classId) || null;
    }
</script>

<div class="characters-page">
    <div class="header">
        <h1>Campaign Characters</h1>
        <a href="/campaigns/{campaignId}" class="back-link">← Back to Campaign</a>
    </div>

    {#if error}
        <div class="error">{error}</div>
    {/if}

    <form on:submit={addCharacter} class="character-form">
        <h2>Add New Character</h2>
        <div class="form-group">
            <label for="name">Name</label>
            <input 
                type="text" 
                id="name" 
                bind:value={newCharacter.name}
                required 
            />
        </div>
        <div class="form-group">
            <label for="class">Class</label>
            <div class="list-items">
                {#if newCharacter.class}
                    <div class="list-item">
                        <span>{getClassDetails(newCharacter.class)?.name}</span>
                        <button 
                            type="button" 
                            on:click={() => {
                                newCharacter.class = 0;
                            }}
                        >
                            Remove
                        </button>
                    </div>
                {/if}
            </div>

            <LibrarySelector
                entityType="class"
                onSelect={(id) => {
                    newCharacter.class = id;
                }}
                placeholder="Search for a class..."
            />
        </div>
        <div class="form-group">
            <label for="level">Level</label>
            <input 
                type="number" 
                id="level" 
                bind:value={newCharacter.level}
                min="1" 
                max="20" 
                required 
            />
        </div>
        <button type="submit">Add Character</button>
    </form>

    {#if loading}
        <div class="loading">Loading characters...</div>
    {:else}
        <div class="character-grid">
            {#each campaignCharacters as character}
                <div class="character-card">
                    {#if editingCharacter?.id === character.id}
                        <div class="edit-form">
                            <input 
                                type="text" 
                                bind:value={editingCharacter.name}
                                placeholder="Character name"
                            />
                            <div class="form-group">
                                <label>Class</label>
                                <div class="list-items">
                                    {#if editingCharacter.class}
                                        <div class="list-item">
                                            <span>{getClassDetails(editingCharacter.class)?.name}</span>
                                            <button 
                                                type="button" 
                                                on:click={() => {
                                                    if (editingCharacter) editingCharacter.class = 0;
                                                }}
                                            >
                                                Remove
                                            </button>
                                        </div>
                                    {/if}
                                </div>
                                <LibrarySelector
                                    entityType="class"
                                    onSelect={(id) => {
                                        if (editingCharacter) editingCharacter.class = id;
                                    }}
                                    placeholder="Search for a class..."
                                />
                            </div>
                            <input 
                                type="number" 
                                bind:value={editingCharacter.level}
                                min="1"
                                max="20"
                            />
                            <div class="edit-actions">
                                <button 
                                    class="save-button"
                                    on:click={() => updateCharacter(editingCharacter)}
                                >
                                    Save
                                </button>
                                <button 
                                    class="cancel-button"
                                    on:click={() => editingCharacter = null}
                                >
                                    Cancel
                                </button>
                            </div>
                        </div>
                    {:else}
                        <h3>{character.name}</h3>
                        <p>Level {character.level} {getClassDetails(character.class)?.name}</p>
                        <button 
                            class="edit-button"
                            on:click={() => editingCharacter = { 
                                id: character.id,
                                name: character.name,
                                class: character.class,
                                level: character.level,
                            }}
                        >
                            Edit
                        </button>
                    {/if}
                </div>
            {/each}
        </div>
    {/if}
</div>

<style>
    .characters-page {
        padding: 2rem;
        max-width: 1200px;
        margin: 0 auto;
    }

    .header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 2rem;
    }

    .back-link {
        color: #666;
        text-decoration: none;
    }

    .back-link:hover {
        text-decoration: underline;
    }

    .character-form {
        background: #f8f8f8;
        padding: 1.5rem;
        border-radius: 8px;
        margin-bottom: 2rem;
        max-width: 500px;
    }

    .form-group {
        margin-bottom: 1rem;
    }

    .form-group label {
        display: block;
        margin-bottom: 0.5rem;
        font-weight: bold;
    }

    input {
        width: 100%;
        padding: 0.5rem;
        border: 1px solid #ddd;
        border-radius: 4px;
    }

    .character-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
        gap: 1.5rem;
    }

    .character-card {
        background: white;
        padding: 1.5rem;
        border-radius: 8px;
        box-shadow: 0 2px 4px rgba(0,0,0,0.1);
    }

    .background {
        color: #666;
        margin-top: 0.5rem;
    }

    .error {
        background: #fee2e2;
        color: #ef4444;
        padding: 1rem;
        border-radius: 4px;
        margin-bottom: 1rem;
    }

    .loading {
        text-align: center;
        color: #666;
        padding: 2rem;
    }

    .edit-form {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    .edit-actions {
        display: flex;
        gap: 0.5rem;
        margin-top: 0.5rem;
    }

    .save-button {
        background: #22c55e;
        color: white;
        border: none;
        padding: 0.5rem 1rem;
        border-radius: 4px;
        cursor: pointer;
    }

    .cancel-button {
        background: #6b7280;
        color: white;
        border: none;
        padding: 0.5rem 1rem;
        border-radius: 4px;
        cursor: pointer;
    }

    .edit-button {
        background: #3b82f6;
        color: white;
        border: none;
        padding: 0.5rem 1rem;
        border-radius: 4px;
        cursor: pointer;
        margin-top: 0.5rem;
    }

    .character-card {
        background: white;
        padding: 1.5rem;
        border-radius: 8px;
        box-shadow: 0 2px 4px rgba(0,0,0,0.1);
    }

    .list-items {
        margin-bottom: 0.5rem;
    }

    .list-item {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 0.5rem;
        background: #f8f8f8;
        border-radius: 4px;
        margin-bottom: 0.5rem;
    }

    .list-item button {
        background: #ef4444;
        color: white;
        border: none;
        padding: 0.25rem 0.5rem;
        border-radius: 4px;
        cursor: pointer;
    }
</style> 
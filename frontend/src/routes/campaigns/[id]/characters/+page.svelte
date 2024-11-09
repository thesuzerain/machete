<script lang="ts">
    import { page } from '$app/stores';
    import { onMount } from 'svelte';
    import type { Character, InsertCharacter } from '$lib/types/types';

    const campaignId = parseInt($page.params.id);
    let campaignCharacters: Character[] = [];
    let loading = true;
    let error: string | null = null;

    onMount(async () => {
        try {
            const response = await fetch(`/api/campaign/${campaignId}/characters`);
            if (!response.ok) throw new Error('Failed to fetch characters');
            campaignCharacters = await response.json();
        } catch (e) {
            error = e instanceof Error ? e.message : 'An error occurred';
        } finally {
            loading = false;
        }
    });

    async function addCharacter(event: SubmitEvent) {
        event.preventDefault();
        const form = event.target as HTMLFormElement;
        const formData = new FormData(form);
        
        const newCharacter: InsertCharacter = {
            name: formData.get('name') as string,
            class: formData.get('class') as string,
            level: parseInt(formData.get('level') as string),
            race: formData.get('race') as string,
            background: formData.get('background') as string
        };

        try {
            const response = await fetch(`/api/campaign/${campaignId}/characters`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify([newCharacter]),
            });

            if (!response.ok) throw new Error('Failed to create character');
            
            // Refresh characters list
            const charactersResponse = await fetch(`/api/campaign/${campaignId}/characters`);
            if (!charactersResponse.ok) throw new Error('Failed to fetch characters');
            campaignCharacters = await charactersResponse.json();
            
            form.reset();
        } catch (e) {
            error = e instanceof Error ? e.message : 'Failed to create character';
        }
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
            <input type="text" id="name" name="name" required />
        </div>
        <div class="form-group">
            <label for="class">Class</label>
            <input type="text" id="class" name="class" required />
        </div>
        <div class="form-group">
            <label for="level">Level</label>
            <input type="number" id="level" name="level" min="1" max="20" value="1" required />
        </div>
        <div class="form-group">
            <label for="race">Race</label>
            <input type="text" id="race" name="race" required />
        </div>
        <div class="form-group">
            <label for="background">Background</label>
            <input type="text" id="background" name="background" required />
        </div>
        <button type="submit">Add Character</button>
    </form>

    {#if loading}
        <div class="loading">Loading characters...</div>
    {:else}
        <div class="character-grid">
            {#each campaignCharacters as character}
                <div class="character-card">
                    <h3>{character.name}</h3>
                    <p>Level {character.level} {character.race} {character.class}</p>
                    <p class="background">Background: {character.background}</p>
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
</style> 
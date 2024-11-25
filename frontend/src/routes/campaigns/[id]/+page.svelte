<script lang="ts">
    import { page } from '$app/stores';
    import { onMount } from 'svelte';
    import type { Campaign, Character, Event } from '$lib/types/types';
    import { API_URL } from '$lib/config';
    import { requireAuth } from '$lib/guards/auth';

    const campaignId = parseInt($page.params.id);
    let campaign: Campaign | undefined;
    let campaignCharacters: Character[] = [];
    let campaignEvents: Event[] = [];
    let loading = true;
    let error: string | null = null;

    onMount(async () => {
        requireAuth();
        try {
            // Fetch campaign details
            const campaignResponse = await fetch(`${API_URL}/campaign/${campaignId}`, {
                credentials: 'include',
            });
            if (!campaignResponse.ok) throw new Error('Failed to fetch campaign');
            campaign = await campaignResponse.json();

            // Fetch characters for this campaign
            const charactersResponse = await fetch(`${API_URL}/campaign/${campaignId}/characters`, {
                credentials: 'include',
            });
            if (!charactersResponse.ok) throw new Error('Failed to fetch characters');
            campaignCharacters = await charactersResponse.json();

            // Fetch events for this campaign
            const eventsResponse = await fetch(`${API_URL}/campaign/${campaignId}/events`, {
                credentials: 'include',
            });
            if (!eventsResponse.ok) throw new Error('Failed to fetch events');
            campaignEvents = await eventsResponse.json();
        } catch (e) {
            error = e instanceof Error ? e.message : 'An error occurred';
        } finally {
            loading = false;
        }
    });
</script>

<div class="campaign-page">
    {#if loading}
        <div class="loading">Loading campaign details...</div>
    {:else if error}
        <div class="error">{error}</div>
    {:else if campaign}
        <h1>{campaign.name}</h1>
        
        <div class="sections">
            <section class="characters">
                <h2>Characters</h2>
                <a href="/campaigns/{campaignId}/characters" class="button">Manage Characters</a>
                <div class="character-list">
                    {#each campaignCharacters as character}
                        <div class="character-card">
                            <h3>{character.name}</h3>
                            <p>Level {character.level} {character.race} {character.class}</p>
                        </div>
                    {/each}
                </div>
            </section>

            <section class="events">
                <h2>Recent Events</h2>
                <a href="/campaigns/{campaignId}/events" class="button">View All Events</a>
                <div class="event-list">
                    {#each campaignEvents.slice(0, 5) as event}
                        <div class="event-card">
                            <span class="event-type">{event.event_type}</span>
                            <p>{event.description}</p>
                        </div>
                    {/each}
                </div>
            </section>
        </div>
    {/if}
</div>

<style>
    .campaign-page {
        padding: 2rem;
        max-width: 1200px;
        margin: 0 auto;
    }

    .sections {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 2rem;
        margin-top: 2rem;
    }

    section {
        background: #f8f8f8;
        padding: 1.5rem;
        border-radius: 8px;
    }

    h2 {
        margin-bottom: 1rem;
    }

    .button {
        display: inline-block;
        padding: 0.5rem 1rem;
        background: #333;
        color: white;
        text-decoration: none;
        border-radius: 4px;
        margin-bottom: 1rem;
    }

    .button:hover {
        background: #444;
    }

    .character-list, .event-list {
        display: grid;
        gap: 1rem;
    }

    .character-card, .event-card {
        background: white;
        padding: 1rem;
        border-radius: 4px;
        box-shadow: 0 1px 3px rgba(0,0,0,0.1);
    }

    .event-type {
        font-weight: bold;
        color: #3b82f6;
    }

    .error {
        background: #fee2e2;
        color: #ef4444;
        padding: 1rem;
        border-radius: 4px;
    }

    .loading {
        text-align: center;
        color: #666;
        padding: 2rem;
    }
</style> 
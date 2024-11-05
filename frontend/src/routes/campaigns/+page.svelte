<script lang="ts">
  import { onMount } from 'svelte';
  import type { Campaign, InsertCampaign } from '$lib/types/types';

  let loading = true;
  let error: string | null = null;
  let newCampaignName = '';
  let campaigns: Campaign[] = [];

  onMount(async () => {
    try {
      const response = await fetch(`/api/campaign`);
      if (!response.ok) throw new Error('Failed to fetch campaigns');
      campaigns = await response.json();
    } catch (e) {
      error = e instanceof Error ? e.message : 'An error occurred';
    } finally {
      loading = false;
    }
  });

  async function handleSubmit() {
    if (!newCampaignName) return;

    try {
      const newCampaign: InsertCampaign = {
        name: newCampaignName
      };

      const response = await fetch(`/api/campaign`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(newCampaign),
      });

      if (!response.ok) throw new Error('Failed to create campaign');
      
      // Refresh the campaigns list
      const campaignsResponse = await fetch(`/api/campaign`);
      if (!campaignsResponse.ok) throw new Error('Failed to fetch campaigns');
      campaigns = await campaignsResponse.json();
      
      // Reset form
      newCampaignName = '';
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to create campaign';
    }
  }
</script>

<div class="campaigns">
  <h1>Campaigns</h1>

  {#if error}
    <div class="error">{error}</div>
  {/if}

  <form on:submit|preventDefault={handleSubmit} class="campaign-form">
    <h2>Create New Campaign</h2>
    <div class="form-group">
      <label for="name">Campaign Name</label>
      <input
        id="name"
        type="text"
        bind:value={newCampaignName}
        placeholder="Enter campaign name"
        required
      />
    </div>
    <button type="submit">Create Campaign</button>
  </form>

  {#if loading}
    <div class="loading">Loading campaigns...</div>
  {:else}
    <div class="campaign-grid">
      {#each campaigns as campaign}
        <div class="campaign-card">
          <h3>{campaign.name}</h3>
          <div class="actions">
            <a href="/campaigns/{campaign.id}/characters">Characters</a>
            <a href="/campaigns/{campaign.id}/events">Events</a>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .campaigns {
    padding: 1rem;
  }

  .campaign-form {
    background: #f8f8f8;
    padding: 1.5rem;
    border-radius: 8px;
    margin-bottom: 2rem;
    max-width: 600px;
  }

  .form-group {
    margin-bottom: 1rem;
  }

  .form-group label {
    display: block;
    margin-bottom: 0.5rem;
    font-weight: bold;
  }

  input, textarea {
    width: 100%;
    padding: 0.5rem;
    border: 1px solid #ddd;
    border-radius: 4px;
  }

  .campaign-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: 1.5rem;
  }

  .campaign-card {
    background: white;
    padding: 1.5rem;
    border-radius: 8px;
    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
  }

  .description {
    color: #666;
    margin: 0.5rem 0;
  }

  .date {
    color: #888;
    font-size: 0.9rem;
  }

  .status {
    display: flex;
    align-items: center;
    gap: 1rem;
    margin: 1rem 0;
  }

  .active {
    color: #22c55e;
  }

  .inactive {
    color: #ef4444;
  }

  .toggle-status {
    padding: 0.25rem 0.75rem;
    border-radius: 4px;
    border: none;
    background: #f3f4f6;
    cursor: pointer;
  }

  .toggle-status:hover {
    background: #e5e7eb;
  }

  .actions {
    display: flex;
    gap: 1rem;
    margin-top: 1rem;
  }

  .actions a {
    text-decoration: none;
    color: #3b82f6;
    font-size: 0.9rem;
  }

  .actions a:hover {
    text-decoration: underline;
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
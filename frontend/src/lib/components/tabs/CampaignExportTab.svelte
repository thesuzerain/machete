<script lang="ts">
    import { fade } from 'svelte/transition';
    import { API_URL } from '$lib/config';
  import { campaignStore } from '$lib/stores/campaigns';
    import Button from '../core/Button.svelte';

    export let campaignId: number;
    
    let exportData: string | null = null;
    let error: string | null = null;
    let copied = false;
    let loading = true;

    async function fetchExportData() {
        loading = true;
        try {
            const data = await campaignStore.exportCampaign(campaignId);
            exportData = JSON.stringify(data, null, 2);
        } catch (e) {
            error = e instanceof Error ? e.message : 'Failed to export campaign';
        } finally {
            loading = false;
        }
    }

    function copyToClipboard() {
        if (!exportData) return;
        
        navigator.clipboard.writeText(exportData).then(() => {
            copied = true;
            setTimeout(() => {
                copied = false;
            }, 2000);
        });
    }

    function downloadJson() {
        if (!exportData) return;
        
        const blob = new Blob([exportData], { type: 'application/json' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `campaign-${campaignId}-export.json`;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
    }

    // Fetch data when component mounts
    $: if (campaignId) {
        fetchExportData();
    }
</script>

<div class="export-container" transition:fade>
    <div class="export-header">
        <h3>Export Campaign</h3>
        <div class="export-actions">
            <Button colour="blue" onclick={copyToClipboard} disabled={!exportData || loading}>
                {copied ? '✓ Copied!' : 'Copy to Clipboard'}
            </Button>

            <Button colour="green" onclick={downloadJson} disabled={!exportData || loading}>
                Download JSON
            </Button>
            
        </div>
    </div>

    {#if error}
        <div class="error-message" transition:fade>
            {error}
        </div>
    {/if}

    {#if loading}
        <div class="loading-indicator">
            Loading export data...
        </div>
    {:else if exportData}
        <pre class="json-display"><code>{exportData}</code></pre>
    {/if}
</div>

<style>
    .export-container {
        background: var(--bg-color);
        border-radius: 0.5rem;
        box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
        padding: 1.5rem;
    }

    .export-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 1rem;
    }


    .export-actions {
        display: flex;
        gap: 0.5rem;
    }


    .json-display {
        background: var(--color-bg-light-raised);
        border: 1px solid var(--color-bg-light-raised-border);
        border-radius: 0.375rem;
        padding: 1rem;
        overflow-x: auto;
        font-family: monospace;
        font-size: 0.875rem;
        line-height: 1.5;
        max-height: 500px;
        overflow-y: auto;
    }

    .error-message {
        background: var(--color-bg-error);
        color: var(--color-text-error);
        padding: 1rem;
        border-radius: 0.375rem;
        margin-bottom: 1rem;
    }

    .loading-indicator {
        color: var(--color-text-secondary);
        text-align: center;
        padding: 2rem;
    }

    .json-display::-webkit-scrollbar {
        width: 8px;
        height: 8px;
    }

    .json-display::-webkit-scrollbar-track {
        background: var(--color-bg-light-raised);
        border-radius: 4px;
    }

    .json-display::-webkit-scrollbar-thumb {
        background: var(--color-bg-light-raised-border);
        border-radius: 4px;
    }

    .json-display::-webkit-scrollbar-thumb:hover {
        background: var(--color-bg-light-raised-hover);
    }
</style> 
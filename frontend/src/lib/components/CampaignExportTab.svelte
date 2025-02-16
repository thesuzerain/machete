<script lang="ts">
    import { fade } from 'svelte/transition';
    import { API_URL } from '$lib/config';
  import { campaignStore } from '$lib/stores/campaigns';

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
            <button 
                class="copy-btn" 
                on:click={copyToClipboard}
                disabled={!exportData || loading}
            >
                {copied ? '✓ Copied!' : 'Copy to Clipboard'}
            </button>
            <button 
                class="download-btn" 
                on:click={downloadJson}
                disabled={!exportData || loading}
            >
                Download JSON
            </button>
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
        background: white;
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

    .export-header h3 {
        margin: 0;
        color: #1f2937;
    }

    .export-actions {
        display: flex;
        gap: 0.5rem;
    }

    .copy-btn, .download-btn {
        padding: 0.5rem 1rem;
        border-radius: 0.375rem;
        font-size: 0.875rem;
        cursor: pointer;
        transition: all 0.2s;
        border: none;
    }

    .copy-btn {
        background: #3b82f6;
        color: white;
    }

    .download-btn {
        background: #22c55e;
        color: white;
    }

    .copy-btn:hover, .download-btn:hover {
        opacity: 0.9;
    }

    .copy-btn:disabled, .download-btn:disabled {
        background: #9ca3af;
        cursor: not-allowed;
    }

    .json-display {
        background: #f8fafc;
        border: 1px solid #e2e8f0;
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
        background: #fee2e2;
        color: #991b1b;
        padding: 1rem;
        border-radius: 0.375rem;
        margin-bottom: 1rem;
    }

    .loading-indicator {
        color: #6b7280;
        text-align: center;
        padding: 2rem;
    }

    .json-display::-webkit-scrollbar {
        width: 8px;
        height: 8px;
    }

    .json-display::-webkit-scrollbar-track {
        background: #f1f5f9;
        border-radius: 4px;
    }

    .json-display::-webkit-scrollbar-thumb {
        background: #cbd5e1;
        border-radius: 4px;
    }

    .json-display::-webkit-scrollbar-thumb:hover {
        background: #94a3b8;
    }
</style> 
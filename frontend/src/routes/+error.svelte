<script lang="ts">
    import { page } from '$app/stores';
    import { goto } from '$app/navigation';
    import { onMount } from 'svelte';

    onMount(() => {
        // If it's a 404, try to handle it client-side
        if ($page.status === 404) {
            const path = window.location.pathname;
            if (path.startsWith('/campaigns/')) {
                // Keep the URL but render the app
                return;
            }
            // Otherwise redirect to home
            goto('/');
        }
    });
</script>

{#if $page.status === 404}
    <!-- This will briefly show while the client-side routing kicks in -->
    <div>Loading...</div>
{:else}
    <h1>{$page.status}: {$page.error?.message}</h1>
{/if} 
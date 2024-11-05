<script lang="ts">
  import { goto } from '$app/navigation';
  import { onMount } from 'svelte';
  import { browser } from '$app/environment';

  onMount(() => {
    if (browser) {
      goto('/campaigns');
    }
  });
</script>

<div>Redirecting to campaigns...</div>

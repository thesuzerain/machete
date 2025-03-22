<script lang="ts">
  import { goto } from '$app/navigation';
  import { onMount } from 'svelte';
  import { browser } from '$app/environment';
  import { auth } from '$lib/stores/auth';
  import LoginModal from '$lib/components/modals/LoginModal.svelte';
  import SignupModal from '$lib/components/modals/SignupModal.svelte';
    import { API_URL } from '$lib/config';

</script>

<div class="container">
    <div>Redirecting to campaigns...</div>
</div>

<style>
  .container {
    display: flex;
    justify-content: center;
    align-items: center;
    min-height: 100vh;
  }
</style>

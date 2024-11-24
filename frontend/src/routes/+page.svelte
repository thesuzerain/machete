<script lang="ts">
  import { goto } from '$app/navigation';
  import { onMount } from 'svelte';
  import { browser } from '$app/environment';
  import { auth } from '$lib/stores/auth';
  import LoginModal from '$lib/components/LoginModal.svelte';
  import SignupModal from '$lib/components/SignupModal.svelte';
    import { API_URL } from '$lib/config';

  let showLoginModal = false;
  let showSignupModal = false;

  async function checkAuth() {
    try {
      const response = await fetch(`${API_URL}/auth/me`, {
        Wcredentials: 'include',
      });
      if (response.ok) {
        const user = await response.json();
        auth.setUser(user);
        goto('/campaigns');
      }
    } catch (error) {
      console.error('Auth check failed:', error);
    }
  }

  onMount(() => {
    if (browser) {
      checkAuth();
    }
  });

</script>

<div class="container">
  {#if $auth}
    <div>Redirecting to campaigns...</div>
  {:else}
    <div class="auth-buttons">
      <button on:click={() => showLoginModal = true}>Login</button>
      <button on:click={() => showSignupModal = true}>Sign Up</button>
    </div>
  {/if}
</div>

{#if showLoginModal}
  <LoginModal on:close={() => showLoginModal = false} />
{/if}

{#if showSignupModal}
  <SignupModal 
    on:close={() => showSignupModal = false}
  />
{/if}

<style>
  .container {
    display: flex;
    justify-content: center;
    align-items: center;
    min-height: 100vh;
  }

  .auth-buttons {
    display: flex;
    gap: 10px;
  }
</style>

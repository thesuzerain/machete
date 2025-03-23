<script lang="ts">
    import LoginModal from "$lib/components/modals/LoginModal.svelte";
    import SignupModal from "$lib/components/modals/SignupModal.svelte";
    import { auth, isLoading } from '$lib/stores/auth';
    import { onMount } from 'svelte';
    import "../app.css";
  import CampaignSelector from "$lib/components/selectors/CampaignSelector.svelte";
    import Button from "$lib/components/core/Button.svelte";

  let error: string | null = null;
    let showLoginModal = false;
    let showSignupModal = false;

    onMount(() => {
        auth.init();
    });

    async function handleLogout() {
        await auth.logout();
    }
</script>

{#if $isLoading}
    <div class="loading">Loading...</div>
{:else}
    <div class="container">
        <nav>
            <div class="nav-content">
                <ul class="nav-links">
                    <li><a href="/">Home</a></li>
                    {#if $auth.user}
                        <li><a href="/campaigns">Campaigns</a></li>
                        <li><a href="/encounters">Encounters</a></li>
                    {/if}
                    <li><a href="/library">Libraries</a></li>
                    <li><a href="/etc">(TODO...)</a></li>
                </ul>

                <div class="auth-section">
                    {#if $auth.user}
                        <div class="user-menu">
                            <span class="username">{$auth.user?.username}</span>
                            <div class="dropdown-content">
                                <a href="/settings">Settings</a>
                                <button on:click={handleLogout}>Logout</button>
                            </div>
                        </div>
                    {:else}
                        <Button colour="black" onclick={() => showLoginModal = true}>Login</Button>
                        <Button colour="blue" onclick={() => showLoginModal = true}>Sign up</Button>
                    {/if}
                </div>
            </div>
        </nav>

        <CampaignSelector bind:error />

        <main>
            <slot />
        </main>

        <LoginModal bind:show={showLoginModal} />
        <SignupModal bind:show={showSignupModal} />

    </div>
{/if}

<style>
    .container {
        max-width: 1800px;
        margin: 0 auto;
        padding: 2rem;
    }

    nav {
        background-color: #f4f4f4;
        padding: 1rem;
        margin-bottom: 2rem;
        border-radius: 8px;
    }

    .nav-content {
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    .nav-links {
        list-style: none;
        padding: 0;
        margin: 0;
        display: flex;
        gap: 2rem;
    }

    .auth-section {
        display: flex;
        gap: 1rem;
        align-items: center;
    }

    .user-menu {
        position: relative;
        cursor: pointer;
    }

    .username {
        padding: 0.5rem 1rem;
        background: var(--color-bg-raised);
        border-radius: 4px;
        font-weight: 500;
    }

    .dropdown-content {
        display: none;
        position: absolute;
        right: 0;
        top: 100%;
        background: var(--color-bg);
        border: 1px solid var(--color-bg-border);
        border-radius: 4px;
        padding: 0.5rem;
        box-shadow: var(--shadow);
    }

    .user-menu:hover .dropdown-content {
        display: block;
    }

    .dropdown-content a,
    .dropdown-content button {
        display: block;
        width: 100%;
        padding: 0.5rem 1rem;
        text-align: left;
        background: none;
        border: none;
        cursor: pointer;
        color: var(--color-text);
        text-decoration: none;
    }

    .dropdown-content a:hover,
    .dropdown-content button:hover {
        background: var(--color-bg-hover);
    }

    a {
        text-decoration: none;
        color: var(--color-text);
        font-weight: bold;
        padding: 0.5rem 1rem;
        border-radius: 4px;
        transition: background-color 0.2s;
    }

    a:hover {
        background-color: var(--color-bg-hover);
    }

    .loading {
        display: flex;
        justify-content: center;
        align-items: center;
        min-height: 100vh;
        font-size: 1.2rem;
        color: var(--color-text);
    }
</style> 
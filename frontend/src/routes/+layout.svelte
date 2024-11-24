<script lang="ts">
    import LoginModal from "$lib/components/LoginModal.svelte";
    import SignupModal from "$lib/components/SignupModal.svelte";
    import { auth, isLoading } from '$lib/stores/auth';
    import { API_URL } from '$lib/config';
    import { onMount } from 'svelte';
    import "../app.css";

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
                    <li><a href="/campaigns">Campaigns</a></li>
                    <li><a href="/encounters">Encounters</a></li>
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
                        <button class="login-btn" on:click={() => showLoginModal = true}>Login</button>
                        <button class="signup-btn" on:click={() => showSignupModal = true}>Sign Up</button>
                    {/if}
                </div>
            </div>
        </nav>

        <main>
            <slot />
        </main>

        {#if showLoginModal}
            <LoginModal on:close={() => showLoginModal = false} />
        {/if}

        {#if showSignupModal}
            <SignupModal 
                on:close={() => showSignupModal = false}
                on:success={() => {
                    showSignupModal = false;
                    showLoginModal = true;
                }}
            />
        {/if}
    </div>
{/if}

<style>
    .container {
        max-width: 1200px;
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
        background: #e0e0e0;
        border-radius: 4px;
        font-weight: 500;
    }

    .dropdown-content {
        display: none;
        position: absolute;
        right: 0;
        top: 100%;
        background: white;
        border: 1px solid #ddd;
        border-radius: 4px;
        padding: 0.5rem;
        box-shadow: 0 2px 4px rgba(0,0,0,0.1);
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
        color: #333;
        text-decoration: none;
    }

    .dropdown-content a:hover,
    .dropdown-content button:hover {
        background: #f4f4f4;
    }

    .login-btn,
    .signup-btn {
        padding: 0.5rem 1rem;
        border: none;
        border-radius: 4px;
        cursor: pointer;
        font-weight: 500;
    }

    .login-btn {
        background: #e0e0e0;
        color: #333;
    }

    .signup-btn {
        background: #3b82f6;
        color: white;
    }

    .login-btn:hover {
        background: #d0d0d0;
    }

    .signup-btn:hover {
        background: #2563eb;
    }

    a {
        text-decoration: none;
        color: #333;
        font-weight: bold;
        padding: 0.5rem 1rem;
        border-radius: 4px;
        transition: background-color 0.2s;
    }

    a:hover {
        background-color: #e0e0e0;
    }

    .loading {
        display: flex;
        justify-content: center;
        align-items: center;
        min-height: 100vh;
        font-size: 1.2rem;
        color: #666;
    }
</style> 
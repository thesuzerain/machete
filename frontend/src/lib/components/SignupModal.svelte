<script lang="ts">
    import { API_URL } from '$lib/config';
    import { auth } from '$lib/stores/auth';
    import { createEventDispatcher } from 'svelte';

    const dispatch = createEventDispatcher();
    let username = '';
    let password = '';
    let error = '';

    function handleBackgroundClick() {
        dispatch('close');
    }

    async function handleSignup(event: SubmitEvent) {
        event.preventDefault();
        error = '';

        try {
            const response = await fetch(`${API_URL}/auth/signup`, {
                method: 'POST',
                credentials: 'include',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({ username, password }),
            });

            if (response.ok) {
                const session = await response.json();
                auth.setUser(session.user);
                dispatch('close');
                dispatch('success');
            } else {
                error = 'Signup failed';
            }
        } catch (err) {
            console.error('Signup error:', err);
            error = 'Signup failed';
        }
    }
</script>

<div class="modal" on:click={handleBackgroundClick}>
    <div class="modal-content" on:click|stopPropagation>
        <h2>Sign Up</h2>
        <form on:submit={handleSignup}>
            <div class="form-group">
                <label>Username:</label>
                <input type="text" bind:value={username} required>
            </div>
            <div class="form-group">
                <label>Password:</label>
                <input type="password" bind:value={password} required>
            </div>
            {#if error}
                <div class="error">{error}</div>
            {/if}
            <div class="buttons">
                <button type="submit">Sign Up</button>
                <button type="button" on:click={() => dispatch('close')}>Cancel</button>
            </div>
        </form>
    </div>
</div>

<style>
    .modal {
        position: fixed;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        background-color: rgba(0,0,0,0.5);
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .modal-content {
        background-color: white;
        padding: 20px;
        width: 300px;
        border-radius: 5px;
    }

    .form-group {
        margin-bottom: 15px;
    }

    .form-group input {
        width: 100%;
        padding: 8px;
        margin-top: 5px;
    }

    .error {
        color: red;
        margin-bottom: 10px;
    }

    .buttons {
        display: flex;
        gap: 10px;
        justify-content: flex-end;
    }
</style> 
<script lang="ts">
    import { API_URL } from '$lib/config';
    import { auth } from '$lib/stores/auth';
    import { createEventDispatcher } from 'svelte';
    import Modal from '../core/Modal.svelte';
    import Button from '../core/Button.svelte';

    export let show = false;

    const dispatch = createEventDispatcher();
    let username = '';
    let password = '';
    let error = '';

    async function handleLogin(event: SubmitEvent) {
        event.preventDefault();
        error = '';

        try {
            const response = await fetch(`${API_URL}/auth/login`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                credentials: 'include',
                body: JSON.stringify({ username, password }),
            });

            if (response.ok) {
                const session = await response.json();
                auth.setUser(session.user);
                dispatch('close');
                show = false;
            } else {
                error = 'Invalid credentials';
            }
        } catch (err) {
            console.error('Login error:', err);
            error = 'Login failed';
        }
    }

    function handleBackgroundClick() {
        dispatch('close');
    }
</script>

<Modal bind:error bind:show closeButton>
    <div slot="header">
        <h2>Login</h2>
    </div>
    <form on:submit={handleLogin}>
        <div class="form-group">
            <label>Username:</label>
            <input type="text" bind:value={username} required>
        </div>
        <div class="form-group">
            <label>Password:</label>
            <input type="password" bind:value={password} required>
        </div>

        <div class="buttons">
            <Button submit colour='blue'>Login</Button>
            <Button onclick={() => dispatch('close')}>Cancel</Button>
        </div>
    </form>

</Modal>

<style>
    .form-group {
        margin-bottom: 15px;
    }

    .form-group input {
        width: 100%;
        padding: 8px;
        margin-top: 5px;
    }

    .buttons {
        display: flex;
        gap: 10px;
        justify-content: flex-end;
    }
</style> 
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
                show = false;
            } else {
                error = 'Signup failed';
            }
        } catch (err) {
            console.error('Signup error:', err);
            error = 'Signup failed';
        }
    }
</script>

<Modal bind:show={show} bind:error={error}>
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
        <div class="buttons">
            <Button submit colour='blue'>Sign Up</Button>
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
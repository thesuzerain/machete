<script lang="ts">
    import { onMount } from 'svelte';
    import { auth } from '$lib/stores/auth';
    import { API_URL } from '$lib/config';
    import { requireAuth } from '$lib/guards/auth';

    let currentPassword = '';
    let newPassword = '';
    let confirmPassword = '';
    let message = '';
    let error = '';

    onMount(() => {
        requireAuth();
    });

    async function handlePasswordChange(event: SubmitEvent) {
        event.preventDefault();
        message = '';
        error = '';

        if (newPassword !== confirmPassword) {
            error = 'New passwords do not match';
            return;
        }

        try {
            const response = await fetch(`${API_URL}/auth/change-password`, {
                method: 'POST',
                credentials: 'include',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    current_password: currentPassword,
                    new_password: newPassword,
                }),
            });

            if (response.ok) {
                message = 'Password updated successfully';
                currentPassword = '';
                newPassword = '';
                confirmPassword = '';
            } else {
                const data = await response.json();
                error = data.error || 'Failed to update password';
            }
        } catch (e) {
            error = 'An error occurred while updating password';
            console.error(e);
        }
    }
</script>

<div class="settings-page">
    <h1>User Settings</h1>
    <h1>TODO: Page incomplete. CSS inconsistent (Buttons need to match components, etc)</h1>

    <div class="user-info">
        <h2>Account Information</h2>
        <p>Username: {$auth?.username}</p>
    </div>

    <div class="password-section">
        <h2>Change Password</h2>
        <form on:submit={handlePasswordChange}>
            <div class="form-group">
                <label for="currentPassword">Current Password</label>
                <input
                    type="password"
                    id="currentPassword"
                    bind:value={currentPassword}
                    required
                />
            </div>

            <div class="form-group">
                <label for="newPassword">New Password</label>
                <input
                    type="password"
                    id="newPassword"
                    bind:value={newPassword}
                    required
                />
            </div>

            <div class="form-group">
                <label for="confirmPassword">Confirm New Password</label>
                <input
                    type="password"
                    id="confirmPassword"
                    bind:value={confirmPassword}
                    required
                />
            </div>

            {#if message}
                <div class="message success">{message}</div>
            {/if}

            {#if error}
                <div class="message error">{error}</div>
            {/if}

            <button type="submit">Update Password</button>
        </form>
    </div>
</div>

<style>
    .settings-page {
        max-width: 600px;
        margin: 0 auto;
        padding: 2rem;
    }

    h1 {
        margin-bottom: 2rem;
    }

    .user-info {
        background: white;
        padding: 1.5rem;
        border-radius: 8px;
        box-shadow: var(--shadow);
        margin-bottom: 2rem;
    }

    .password-section {
        background: white;
        padding: 1.5rem;
        border-radius: 8px;
        box-shadow: var(--shadow);
    }

    .form-group {
        margin-bottom: 1rem;
    }

    label {
        display: block;
        margin-bottom: 0.5rem;
        font-weight: 500;
    }

    input {
        width: 100%;
        padding: 0.5rem;
        border: 1px solid #ddd;
        border-radius: 4px;
    }

    button {
        background: #3b82f6;
        color: white;
        padding: 0.5rem 1rem;
        border: none;
        border-radius: 4px;
        cursor: pointer;
        font-weight: 500;
    }

    button:hover {
        background: #2563eb;
    }

    .message {
        padding: 1rem;
        border-radius: 4px;
        margin: 1rem 0;
    }

    .success {
        background: #dcfce7;
        color: #166534;
    }

    .error {
        background: #fee2e2;
        color: #dc2626;
    }
</style> 
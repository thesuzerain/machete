<script lang="ts">
    import { notificationStore, type Notification } from "$lib/stores/notifications";
    import { fade } from "svelte/transition";

    let notification: Notification | null = null;

    notificationStore.subscribe((n : Notification | null) => {
        if (n) {
            // Exit if timeout + timestamp is before now
            if (n.timeout && n.timestamp + n.timeout < Date.now()) {
                return;
            }

            notification = n;

            if (n.timeout) {
                setTimeout(() => {
                    notification = null;
                }, n.timeout);
            }
        }
    });

</script>

{#if notification}
<div class="notification {notification.type}" transition:fade>{notification.message}</div>
{/if}

<style>
    .notification {
        position: fixed;
        top: 1rem;
        right: 5rem;
        background: var(--color-bg-success); 
        color: var(--color-text-light); 
        padding: 0.75rem 1rem;
        border-radius: 0.5rem;
        box-shadow: var(--shadow);
        z-index: 10000000000;
    }

    .notification.error {
        background: var(--color-bg-error);
    }
    .notification.success {
        background: var(--color-bg-success);
    }

</style>
import { get, writable } from 'svelte/store';

export type Notification = {
    type: string;
    message: string;
    timeout?: number;
    timestamp: number;
};

function createNotificationStore() {
    const { subscribe, set, update } = writable<Notification | null>(null);

    return {
        subscribe,
        success: (message: string, timeout: number = 3000) => {
            console.log('success', message, timeout);
            const notification = {
                type: 'success',
                message,
                timeout,
                timestamp: Date.now()
            };
            update((notifications) => {
                return notification;
            });
        }
    };
}

export const notificationStore = createNotificationStore();

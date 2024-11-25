import { get } from 'svelte/store';
import { goto } from '$app/navigation';
import { auth, isInitialized } from '$lib/stores/auth';

export async function requireAuth() {
    // Wait for auth to be initialized if it hasn't been
    if (!get(isInitialized)) {
        await auth.init();
    }
    
    const state = get(auth);
    if (!state.user) {
        goto('/');
        return false;
    }
    return true;
} 
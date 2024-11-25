import { writable, derived } from 'svelte/store';
import type { User } from '$lib/types/types';
import { API_URL } from '$lib/config';
import { browser } from '$app/environment';
import { goto } from '$app/navigation';

interface AuthState {
    user: User | null;
    loading: boolean;
    initialized: boolean;
}

function createAuthStore() {
    const { subscribe, set, update } = writable<AuthState>({
        user: null,
        loading: false,
        initialized: false
    });

    let initPromise: Promise<void> | null = null;

    async function initializeAuth() {
        if (!browser) return;
        
        update(state => ({ ...state, loading: true }));
        
        try {
            console.log("1Current cookies: ", document.cookie);
            const response = await fetch(`${API_URL}/auth/me`, {
                credentials: 'include'
            });
            
            if (response.ok) {
                const userResponse = await response.json();
                
                auth.setSession(userResponse);
            } else {
                auth.setSession(null);
            }

            console.log("2Current cookies: ", document.cookie);
            const response2 = await fetch(`${API_URL}/auth/campaigns`, {
                credentials: 'include',
            });
            console.log("123123213Response: ", response2);

        } catch (error) {
            console.error('Auth check failed:', error);
            auth.setSession(null);
        }
    }

    return {
        subscribe,
        
        // Initialize auth state - returns a promise that resolves when done
        init: () => {
            if (!initPromise) {
                initPromise = initializeAuth();
            }
            return initPromise;
        },

        setSession: (user: User | null) => {
            update(state => ({
                ...state,
                loading: false,
                user: user,
                initialized: true
            }));

        },

        logout: async () => {
            try {
                await fetch(`${API_URL}/auth/logout`, {
                    method: 'POST',
                    credentials: 'include'
                });
            } finally {
                update(state => ({
                    ...state,
                    user: null
                }));

                goto('/');
            }
        },

        setUser: (user: User | null) => {
            update(state => ({
                ...state,
                user,
                initialized: true
            }));
        },
    };
}

export const auth = createAuthStore();

// Derived stores for convenience
export const user = derived(auth, $auth => $auth.user);
export const isAuthenticated = derived(auth, $auth => !!$auth.user);
export const isLoading = derived(auth, $auth => $auth.loading);
export const isInitialized = derived(auth, $auth => $auth.initialized); 
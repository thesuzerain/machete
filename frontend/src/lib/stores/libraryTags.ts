import { writable } from 'svelte/store';
import { API_URL } from '$lib/config';
import { auth } from './auth';
import type Library from '$lib/components/library/Library.svelte';
import type { LibraryEntityType } from '$lib/types/library';

type LibraryTagState = {
    [key in LibraryEntityType]: LibraryTagSubState;
} & {
    combined_tags: string[];
    combined_traits: string[];
}

interface LibraryTagSubState{
    tags: string[];
    traits: string[];
}


function createTagStore() {
    const initialState: LibraryTagState = {
        combined_tags: [],
        combined_traits: [],

        'creature': {
            tags: [],
            traits: [],
        },

        'item': {
            tags: [],
            traits: [],
        },

        'hazard': {
            tags: [],
            traits: [],
        },

        'spell': {
            tags: [],
            traits: [],
        },

        'class': {
            tags: [],
            traits: [],
        },
    };

    const { subscribe, set, update } = writable(initialState);

    return {
        subscribe,
        fetch: async () => {
            try {
                const response = await fetch(`${API_URL}/library/tags`, {
                });
                if (!response.ok) throw new Error('Failed to fetch tags and traits');
                const ret = await response.json();
                update(_store => ({
                    ...ret
                    
                }));
                return ret;
            } catch (e) {
                console.error('Error fetching tags and traits:', e);
                return [];
            }
        },
    };
}

export const libraryTagsStore = createTagStore();
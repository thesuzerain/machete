import { writable } from 'svelte/store';
import { API_URL } from '$lib/config';
import type { LibraryEntity } from '$lib/types/library';

interface LibraryStoreState {
    entities: Map<number, LibraryEntity>;
    loading: boolean;
    error: string | null;
}

function createLibraryStore(entityType: 'creature' | 'hazard' | 'item' | 'class') {
    const routePart = {
        'creature': 'creatures',
        'hazard': 'hazards',
        'item': 'items',
        'class': 'classes'
    };

    const initialState: LibraryStoreState = {
        entities: new Map(),
        loading: false,
        error: null
    };

    const { subscribe, set, update } = writable(initialState);

    async function fetchEntities(params: Record<string, string>) {
        const endpoint = routePart[entityType];
        const queryString = new URLSearchParams(params).toString();
        
        update(state => ({ ...state, loading: true, error: null }));
        
        try {
            const response = await fetch(`${API_URL}/library/${endpoint}?${queryString}`);
            if (!response.ok) throw new Error(`Failed to fetch ${entityType}s`);
            const data = await response.json();
            
            update(state => {
                const newEntities = new Map(state.entities);
                data.forEach((entity: LibraryEntity) => {
                    newEntities.set(entity.id, entity);
                });
                return {
                    entities: newEntities,
                    loading: false,
                    error: null
                };
            });

            return data.length === 100; // Returns whether there might be more results
        } catch (e) {
            const errorMessage = e instanceof Error ? e.message : `Failed to load ${entityType}s`;
            update(state => ({ ...state, loading: false, error: errorMessage }));
            return false;
        }
    }

    async function getEntity(id: number) {
        update(state => ({ ...state, loading: true, error: null }));
        
        try {
            const response = await fetch(`${API_URL}/library/${routePart[entityType]}/${id}`);
            if (!response.ok) throw new Error('Failed to load entity');
            const entity = await response.json();
            
            update(state => {
                const newEntities = new Map(state.entities);
                newEntities.set(entity.id, entity);
                return {
                    entities: newEntities,
                    loading: false,
                    error: null
                };
            });
        } catch (e) {
            const errorMessage = e instanceof Error ? e.message : 'Failed to load entity';
            update(state => ({ ...state, loading: false, error: errorMessage }));
        }
    }

    function reset() {
        set(initialState);
    }

    return {
        subscribe,
        fetchEntities,
        getEntity,
        reset
    };
}

export const creatureStore = createLibraryStore('creature');
export const hazardStore = createLibraryStore('hazard');
export const itemStore = createLibraryStore('item');
export const classStore = createLibraryStore('class'); 
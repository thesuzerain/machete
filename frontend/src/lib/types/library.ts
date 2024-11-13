export interface LibraryEntity {
    id: number;
    name: string;
    description?: string;
    level?: number;
    rarity?: 'common' | 'uncommon' | 'rare' | 'unique';
    source?: string;
    traits?: string[];
}

export interface LibraryClass extends LibraryEntity {
    hit_points: number;
    key_ability: string;
    skills: string[];
}

export interface LibrarySpell extends LibraryEntity {
    traditions: string[];
    casting_time: string;
    components: string[];
    range?: string;
    duration?: string;
    saving_throw?: string;
}

export interface LibraryCreature extends LibraryEntity {
    family?: string;
    size: string;
    alignment?: string;
    type: string;
}

export interface LibraryHazard extends LibraryEntity {
    complexity: string;
    stealth?: string;
    disable?: string;
}

export interface LibraryItem extends LibraryEntity {
    category: string;
    price?: {
        gold?: number;
        silver?: number;
        copper?: number;
    };
    bulk?: number;
    hands?: number;
}

export type LibraryEntityType = 'class' | 'spell' | 'creature' | 'hazard' | 'item';

export interface TableColumn {
    key: string;
    label: string;
    sortable?: boolean;
    formatter?: (value: any) => string;
}

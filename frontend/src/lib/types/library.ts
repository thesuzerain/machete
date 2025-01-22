export interface LibraryEntity {
    id: number;
    name: string;
    description?: string;
    level?: number;
    url?: string;
    rarity?: 'common' | 'uncommon' | 'rare' | 'unique';
    source?: string;
    traits?: string[];
}

export function getFullUrl(url : string) {
    const aon = "https://2e.aonprd.com";
    return aon + url;
}

export function getFullUrlWithAdjustment(url : string, adjustment : number) {
    const aon = "https://2e.aonprd.com";
    if (adjustment === -1) {
        return aon + url + '&Weak=true';
    }
    if (adjustment === 1) {
        return aon + url + '&Elite=true';
    }

    return aon + url;
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
    price?: number;
    bulk?: number;
    hands?: number;
}

export function formatCurrency(currency: number): string {
    const parts: string[] = [];

    const gold = Math.floor(currency);
    const silver = Math.floor((currency - gold) * 10);
    const copper = Math.round((currency - gold - silver / 10) * 100);

    if (gold > 0) {
        parts.push(`${gold}g`);
    }
    if (silver > 0) {
        parts.push(`${silver}s`);
    }
    if (copper > 0) {
        parts.push(`${copper}c`);
    }
    if (parts.length === 0) {
        return '0g';
    }

    return parts.join(' ');
}

export type LibraryEntityType = 'class' | 'spell' | 'creature' | 'hazard' | 'item';

export interface TableColumn {
    key: string;
    label: string;
    sortable?: boolean;
    formatter?: (value: any) => string;
}

export interface LibraryResponse<T extends LibraryEntity> {
    items: T[];
    total: number;
    hasMore: boolean;
}

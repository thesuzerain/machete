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
    price?: Currency;
    bulk?: number;
    hands?: number;
}

export interface Currency {
    gold?: number;
    silver?: number;
    copper?: number;
}

export function formatCurrency(currency: Currency): string {
    const parts: string[] = [];
    if (currency.gold) {
        parts.push(`${currency.gold}g`);
    }
    if (currency.silver) {
        parts.push(`${currency.silver}s`);
    }
    if (currency.copper) {
        parts.push(`${currency.copper}c`);
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

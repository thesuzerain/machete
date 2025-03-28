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
    legacy?: boolean;
    traits?: string[];
}

export interface LibraryCreature extends LibraryEntity {
    family?: string;
    size: string;
    alignment?: 'LG' | 'NG' | 'CG' | 'LN' | 'NN' | 'CN' | 'LE' | 'NE' | 'CE';
    type: string;
    legacy?: boolean;
    traits?: string[];
}

export interface LibraryHazard extends LibraryEntity {
    complex: boolean;
    haunt?: boolean;
    legacy?: boolean;
}

export interface LibraryItem extends LibraryEntity {
    price?: number;

    item_categories?: string[];
    traits?: string[];
    consumable?: boolean;
    magical?: boolean;
    legacy?: boolean;

    item_type?: string;
    skill_boosts?: string[];
    runes?: Rune[];
    apex_stat?: string;
}

export interface Rune {
    type: string;
    potency: number;
    property?: string;
}

export type LibrarySearchRequest<T extends LibraryEntity> = Record<string, T[]>;

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

export function formatAlignment(alignment: string): string {
    switch (alignment) {
        case 'LG':
            return 'Lawful Good';
        case 'NG':
            return 'Neutral Good';
        case 'CG':
            return 'Chaotic Good';
        case 'LN':
            return 'Lawful Neutral';
        case 'NN':
            return 'Neutral Neutral';
        case 'CN':
            return 'Chaotic Neutral';
        case 'LE':
            return 'Lawful Evil';
        case 'NE':
            return 'Neutral Evil';
        case 'CE':
            return 'Chaotic Evil';
        default:
            return alignment;
    }
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

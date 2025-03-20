import type { PageLoad } from './$types';

export const load: PageLoad = async ({ url }) => {
    return {
        // TODO: NOTE THIS: Way better way of getting search params
        activeEncounterId: parseInt(url.searchParams.get('activeEncounterId') || '') || null,
        startTab: url.searchParams.get('tab')
    };
}; 
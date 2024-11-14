import type { PageLoad } from './$types';

export const load: PageLoad = async ({ url }) => {
    return {
        activeEncounter: url.searchParams.get('encounter'),
        startTab: url.searchParams.get('tab')
    };
}; 
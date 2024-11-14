import type { PageLoad } from './$types';

export const load: PageLoad = async ({ url }) => {
    console.log('url.searchParams', JSON.stringify(url.searchParams));
    return {
        activeEncounter: url.searchParams.get('encounter'),
        startTab: url.searchParams.get('tab')
    };
}; 
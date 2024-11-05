import { API_URL } from './config';

export const api = {
    campaigns: {
        list: () => 
            fetch(`/api/campaign`),
        create: (data: any) => 
            fetch(`/api/campaign`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(data)
            }),
        characters: {
            list: (campaignId: number) => 
                fetch(`/api/campaign/${campaignId}/characters`),
            create: (campaignId: number, data: any) => 
                fetch(`/api/campaign/${campaignId}/characters`, {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify(data)
                })
        },
        events: {
            list: (campaignId: number) => 
                fetch(`/api/campaign/${campaignId}/events`),
            create: (campaignId: number, data: any) => 
                fetch(`/api/campaign/${campaignId}/events`, {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify(data)
                })
        }
    }
}; 
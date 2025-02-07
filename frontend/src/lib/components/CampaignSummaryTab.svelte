<script lang="ts">
    import { campaignSessionStore } from '$lib/stores/campaignSessions';
  import { characterStore } from '$lib/stores/characters';
  import LineGraph from './LineGraph.svelte';

    interface Props {
        selectedCampaignId: number;
    }
    let { 
        selectedCampaignId = $bindable(),

     } : Props = $props();

    let campaignSessions = $derived($campaignSessionStore.get(selectedCampaignId) || []);
    let characters = $derived($characterStore.get(selectedCampaignId) || []);

    let sessionTreasureTimeSeriesCum = $derived.by(
        () => {
            let ary = [];
            let sum = 0;
            for (let i = 0; i < campaignSessions.length; i++) {
                sum += campaignSessions[i].total_treasure_value;
                ary.push({ x: i, y: sum });
            }
            return [{
                id: "Treasure",
                data: ary
            }]
        }
    );

    let sessionExperienceTimeSeriesCum = $derived.by(
        () => {
            let ary = [];
            let sum = 0;
            for (let i = 0; i < campaignSessions.length; i++) {
                sum += campaignSessions[i].total_experience;
                ary.push({ x: i, y: sum });
            }
            return [{
                id: "Experience",
                data: ary
            }]
        }
    );

       // TODO: Move this to a better statistics component for characters- its own route maybe?
       let goldTimeSeriesPerCharacterCum = $derived.by(
        () => {
            let ary = [];
            let sum = 0;
            for (let i = 0; i < characters.length; i++) {
                let character = characters[i];
                let ary2 = [];
                for (let j = 0; j < campaignSessions.length; j++) {
                    sum += campaignSessions[j].compiled_rewards[character.id]?.gold || 0;
                    ary2.push({ x: j, y: sum });
                }
                ary.push({
                    id: character.name,
                    data: ary2
                });
            }
            return ary;
        }
    );
</script>

<div>

    <LineGraph data={sessionTreasureTimeSeriesCum} xLabel="Sessions" yLabel="Treasure" />
    <LineGraph data={sessionExperienceTimeSeriesCum} xLabel="Sessions" yLabel="Experience" />
    <LineGraph
    data={goldTimeSeriesPerCharacterCum}
    xLabel="Sessions"
    yLabel="Gold"
/>

    
</div>

<style>

</style> 
<script lang="ts">
    import { campaignSessionStore } from '$lib/stores/campaignSessions';
  import LineGraph from './LineGraph.svelte';

    interface Props {
        selectedCampaignId: number;
    }
    let { 
        selectedCampaignId = $bindable(),

     } : Props = $props();

    let campaignSessions = $derived($campaignSessionStore.get(selectedCampaignId) || []);
    console.log("abc");
    console.log("campaignSessions", $state.snapshot(campaignSessions));

     $effect(() => {
        console.log(". campaignSessions", $state.snapshot(campaignSessions));  
     })

    let sessionTreasureTimeSeriesCum = $derived.by(
        () => {
            let ary = [];
            let sum = 0;
            for (let i = 0; i < campaignSessions.length; i++) {
                sum += campaignSessions[i].total_treasure_value;
                ary.push({ x: i, y: sum });
            }
            return [{
                id: "treasure",
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
                id: "experience",
                data: ary
            }]
        }
    );
</script>

<div>

    <LineGraph data={sessionTreasureTimeSeriesCum} xLabel="Sessions" yLabel="Treasure" />
    <LineGraph data={sessionExperienceTimeSeriesCum} xLabel="Sessions" yLabel="Experience" />

</div>

<style>

</style> 
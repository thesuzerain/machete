<script lang="ts">
    import { statsStore } from '$lib/stores/stats';
    import { characterStore } from '$lib/stores/characters';
    import LineGraph from './LineGraph.svelte';
    import { onMount } from 'svelte';
    import { campaignSessionStore } from '$lib/stores/campaignSessions';

    export let selectedCampaignId: number;
    
    $: characters = $characterStore.get(selectedCampaignId);
    $: stats = $statsStore.get(selectedCampaignId);
    $: campaignSessions = $campaignSessionStore.get(selectedCampaignId);

    onMount(async () => {
        await statsStore.fetchStats(selectedCampaignId);
    });
    
    function getTreasureFraction(totalTreasure : number | undefined, expectedTreasure : number | undefined) {
        let expectedTreasureSanity = expectedTreasure || 0;
        let fraction = (totalTreasure || 0) / expectedTreasureSanity;
        if (isNaN(fraction)) {
            fraction = 1;
        }
        return fraction;
    }

    $: totalCombinedTreasure = stats?.total_combined_treasure || 0;
    $: treasureThisLevelFraction = getTreasureFraction(totalCombinedTreasure, stats?.total_expected_combined_treasure);
    $: treasureStartOfLevelFraction = getTreasureFraction(totalCombinedTreasure, stats?.total_expected_combined_treasure_start_of_level);
    $: treasureEndOfLevelFraction = getTreasureFraction(totalCombinedTreasure, stats?.total_expected_combined_treasure_end_of_level);

    function getTextForFraction(totalTreasure : number, expectedTreasure : number | undefined) {
        let expectedTreasureSanity = expectedTreasure || 0;
        let fraction = totalTreasure / expectedTreasureSanity;
        if (isNaN(fraction)) {
            fraction = 1;
        }
        let difference = -(expectedTreasureSanity - totalTreasure);
        if (fraction < 0.8) {
            return `Deficit (${difference.toFixed(1)})`;
        } else if (fraction < 1.2) {
            return `On track (${difference.toFixed(1)})`;
        } else {
            return `Surplus (${difference.toFixed(1)})`;    
        }
    }

    function getClassColorForFraction(fraction: number) {
        if (fraction < 0.8) {
            return 'large-deficit-colour';
        } else if (fraction < 0.9) {
            return 'small-deficit-colour';
        } else if (fraction < 1.1) {
            return 'no-deficit-colour';
        } else if (fraction < 1.2) {
            return 'small-surplus-colour';
        } else {
            return 'large-surplus-colour';
        }
    }

    // Cumulative series calculations
    $: totalXp = stats?.total_xp || 0;
    $: treasureByLevelCumulative = stats?.encounters.reverse().reduce((acc, e) => {
        const level = acc.length === 0 ? (totalXp/1000)+1 : acc[acc.length-1].level - (e.accumulated_xp / 1000);

        // Add to every level below (above)
        acc.forEach((data) => {
            if (data.level >= level) {
                data.actual += e.accumulated_items_treasure + e.accumulated_gold_treasure;
                data.expected += e.calculated_expected_total_treasure;
            }
        });

        // Add a new one
        acc.push({
            level: level,
            actual: e.accumulated_items_treasure + e.accumulated_gold_treasure,
            expected: e.calculated_expected_total_treasure
        });
        
        return acc;
    }, [] as Array<{level: number, actual: number, expected: number}>).reverse();

    $: treasureByLevelCumulativeSeries = treasureByLevelCumulative.map(((data) => ({
        x: data.level,
        y: data.actual
    })));

    $: expectedTreasureByLevelCumulativeSeries = treasureByLevelCumulative.map(((data) => ({
        x: data.level,
        y: data.expected
    })));

    $: expectedTreasureGrowthSeries = stats?.encounters.reduce((acc, e, i) => {
        const prev = i > 0 ? acc[i-1].y : 0;
        acc.push({
            x: i,
            y: prev + e.calculated_expected_total_treasure
        });
        return acc;
    }, [] as {x: number, y: number}[]) || [];

    $: xpGrowthEncountersSeries = stats?.encounters.reduce((acc, e, i) => {
        const prev = i > 0 ? acc[i-1].y : 0;
        acc.push({
            x: i,
            y: prev + e.accumulated_xp
        });
        return acc;
    }, [] as {x: number, y: number}[]) || [];

    $: xpGrowthSessionSeries = campaignSessions?.reduce((acc, s, i) => {
        const prev = i > 0 ? acc[i-1].y : 0;
        acc.push({
            x: i,
            y: s.level_at_end + s.experience_at_end/1000
        });
        return acc;
    }, [] as {x: number, y: number}[]) || [];

    // Session-based XP series
    $: sessionXPSeries = stats?.encounters.reduce((acc, e) => {
        const sessionId = e.session_id;
        if (!acc[sessionId]) {
            acc[sessionId] = { x: e.session_ix, y: 0 };
        }
        acc[sessionId].y += e.accumulated_xp;
        return acc;
    }, {} as Record<number, {x: number, y: number}>);

    $: sessionXPArray = sessionXPSeries ? Object.values(sessionXPSeries).sort((a, b) => a.x - b.x) : [];

    // Calculate item distribution stats
    $: itemDistributionCombinedKeys = Array.from(new Set([
        ...Object.keys(stats?.expected_permanent_items_by_end_of_level || {}),
        ...Object.keys(stats?.total_permanent_items_by_level || {})
    ])).sort((a, b) => Number(a) - Number(b));
    $: itemDistributionByLevel = itemDistributionCombinedKeys.map(level => ({
        level: Number(level),
        actual: stats?.total_permanent_items_by_level[Number(level)] || 0,
        expected: stats?.expected_permanent_items_by_end_of_level[Number(level)] || 0
    }));

    $: consumableDistributionCombinedKeys = Array.from(new Set([
        ...Object.keys(stats?.expected_consumable_items_by_end_of_level || {}),
        ...Object.keys(stats?.total_consumable_items_by_level || {})
    ])).sort((a, b) => Number(a) - Number(b));
    $: consumableDistributionByLevel = consumableDistributionCombinedKeys.map(level => ({
        level: Number(level),
        actual: stats?.total_consumable_items_by_level[Number(level)] || 0,
        expected: stats?.expected_consumable_items_by_end_of_level[Number(level)] || 0
    }));

    $: allIndividualGold = Object.values(stats?.character_stats || {}).map(c => c.total_combined_treasure).reduce((acc, val) => acc + val, 0);

    // Character equity analysis
    $: characterEquity = characters ? Object.entries(stats?.character_stats || {}).map(([id, charStats]) => {
        const character = characters.find(c => c.id === Number(id));
        const expectedGoldDivided = allIndividualGold / (characters?.length || 1);
        return {
            name: character?.name || 'Unknown',
            goldShare: charStats.total_combined_treasure,
            expectedGoldShare: expectedGoldDivided,
            itemCount: charStats.total_permanent_items.length,
            expectedItemCount: Object.values(charStats.expected_boosts || {}).length,
            boostCount: charStats.available_boosts.length,
            expectedBoostCount: charStats.expected_boosts.length,
        };
    }) : [];
</script>

<div class="summary-container">
    <div class="stats-overview">
        <div class="stat-card">
            <h3>Campaign Level</h3>
            <div class="value">{stats?.level || 0}</div>
            <div class="progress-bar">
                <div class="progress" style="width: {((stats?.experience_this_level || 0) / 1000) * 100}%"></div>
                <div class="subtext">Experience: {stats?.experience_this_level || 0}</div>
            </div>
            <div class="subtext">{stats?.experience_this_level || 0}/1000 XP</div>
        </div>

        <div class="stat-card">
            <h3>Sessions</h3>
            <div class="value">{stats?.num_sessions || 0}</div>
            <div class="subtext">Total encounters: {stats?.num_combat_encounters || 0}</div>
        </div>

        <div class="stat-card">
            <h3>Total Treasure</h3>
            <div class="stat-line">
                <div class="value">{stats?.total_combined_treasure || 0}</div>
                <div class="subtext">Expected by end of level: {stats?.total_expected_combined_treasure_end_of_level?.toFixed(1) || 0}</div>
            </div>
            <div class="progress-bar" style="--color: {stats?.total_combined_treasure >= (stats?.total_expected_combined_treasure || 0) ? '#22c55e' : '#ef4444'}">
                <div class="progress" style="width: {Math.min(treasureThisLevelFraction * 100, 100)}%"></div>
            </div>
            <div>
                <div>
                    <h3>Variance to expected treasure</h3>
                    <div class="stat-line">
                        <div class="value {getClassColorForFraction(treasureThisLevelFraction)}">{-((stats?.total_expected_combined_treasure||0) - totalCombinedTreasure)}</div>
                        <div class="subtext">Approximate expected: {stats?.total_expected_combined_treasure?.toFixed(1) || 0}</div>

                    </div>
            </div>
                    <div>
                        <h3>Variance to start of level</h3>
                        <div class="stat-line">

                        <div class="value {getClassColorForFraction(treasureStartOfLevelFraction)}">{-((stats?.total_expected_combined_treasure_start_of_level || 0) - totalCombinedTreasure)}</div>
                        <div class="subtext">Expected by start of level: {stats?.total_expected_combined_treasure_start_of_level?.toFixed(1) || 0}</div>

                    </div>
                    <div>
                        <h3>Variance to end of level</h3>
                        <div class="stat-line">

                            <div class="value {getClassColorForFraction(treasureEndOfLevelFraction)}">{-((stats?.total_expected_combined_treasure_end_of_level || 0) - totalCombinedTreasure)}</div>
                            <div class="subtext">Expected by end of level: {stats?.total_expected_combined_treasure_end_of_level?.toFixed(1) || 0}</div>
                        </div>
                </div>
                </div>
            </div>
        </div>
    </div>

    <div class="graphs-container">
        <div class="graph-card">
            <h3>Cumulative Treasure by Level</h3>
            <LineGraph 
                data={[
                    { id: 'Actual Treasure', data: treasureByLevelCumulativeSeries },
                    { id: 'Expected Treasure', data: expectedTreasureByLevelCumulativeSeries }
                ]} 
                xLabel="Level" 
                yLabel="Treasure" 
            />
        </div>

        <div class="graph-card">
            <h3>Experience Growth by Encounter</h3>
            <LineGraph 
                data={[{ id: 'XP', data: xpGrowthEncountersSeries }]} 
                xLabel="Encounters" 
                yLabel="Experience" 
            />
        </div>

        <div class="graph-card">
            <h3>XP per Session</h3>
            <LineGraph 
                data={[{ id: 'Session XP', data: sessionXPArray }]} 
                xLabel="Session" 
                yLabel="Experience" 
            />
        </div>

        <div class="graph-card">
            <h3>Experience Growth by Session</h3>
            <LineGraph 
                data={[{ id: 'XP', data: xpGrowthSessionSeries }]} 
                xLabel="Sessions" 
                yLabel="Level" 
            />
        </div>
    </div>

    <div class="distribution-section">
        <h3>Item Distribution Analysis</h3>
        <div class="distribution-grid">
            <div class="distribution-card">
                <h4>Permanent Items by Level</h4>
                <table>
                    <thead>
                        <tr>
                            <th>Level</th>
                            <th>Actual</th>
                            <th>Expected</th>
                            <th>Difference</th>
                        </tr>
                    </thead>
                    <tbody>
                        {#each itemDistributionByLevel as {level, actual, expected}}
                            <tr class={actual >= expected ? 'positive' : 'negative'}>
                                <td>{level}</td>
                                <td>{actual}</td>
                                <td>{expected}</td>
                                <td>{actual - expected}</td>
                            </tr>
                        {/each}
                    </tbody>
                </table>
            </div>

            <div class="distribution-card">
                <h4>Consumable Items by Level</h4>
                <table>
                    <thead>
                        <tr>
                            <th>Level</th>
                            <th>Actual</th>
                            <th>Expected</th>
                            <th>Difference</th>
                        </tr>
                    </thead>
                    <tbody>
                        {#each consumableDistributionByLevel as {level, actual, expected}}
                            <tr class={actual >= expected ? 'positive' : 'negative'}>
                                <td>{level}</td>
                                <td>{actual}</td>
                                <td>{expected}</td>
                                <td>{actual - expected}</td>
                            </tr>
                        {/each}
                    </tbody>
                </table>
            </div>
        </div>
    </div>

    <div class="equity-section">
        <h3>Character Equity Analysis</h3>
        <div class="equity-grid">
            {#each characterEquity as char}
                <div class="equity-card">
                    <h4>{char.name}</h4>
                    <div class="equity-stats">
                        <div class="equity-stat" class:deficit={char.goldShare < char.expectedGoldShare}
                                              class:surplus={char.goldShare >= char.expectedGoldShare}>
                            <span class="label">Gold Share</span>
                            <span class="value">{char.goldShare.toFixed(1)}</span>
                            <span class="subtext">of {char.expectedGoldShare.toFixed(1)}</span>
                            <span class="value">{((char.goldShare / char.expectedGoldShare || 0) * 100).toFixed(1)}%</span>
                            <span class="subtext">of fair share</span>
                        </div>
                        <div class="equity-stat" class:deficit={char.itemCount < char.expectedItemCount}
                                              class:surplus={char.itemCount >= char.expectedItemCount}>
                            <span class="label">Permanent Items</span>
                            <span class="value">{char.itemCount}/{char.expectedItemCount}</span>
                        </div>
                        <div class="equity-stat" class:deficit={char.boostCount < char.expectedBoostCount}
                                              class:surplus={char.boostCount >= char.expectedBoostCount}>
                            <span class="label">Available Boosts</span>
                            <span class="value">{char.boostCount}/{char.expectedBoostCount}</span>
                        </div>
                    </div>
                </div>
            {/each}
        </div>
    </div>
</div>

<style>
    .summary-container {
        display: flex;
        flex-direction: column;
        gap: 2rem;
    }

    .stats-overview {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
        gap: 1.5rem;
    }

    .stat-card {
        background: white;
        padding: 1.5rem;
        border-radius: 0.5rem;
        box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
    }

    .stat-card h3 {
        margin: 0;
        color: #64748b;
        font-size: 0.875rem;
        text-transform: uppercase;
        letter-spacing: 0.05em;
    }

    .stat-line {
        display: flex;
        gap: 0.5rem;
        justify-content: space-between;
    }
    .stat-line div {
        align-self: flex-end;
    }

    .value {
        font-size: 2rem;
        font-weight: 600;
        color: #1e293b;
        margin: 0.1rem 0;
    }

    .value-subtype {
        font-size: 1.2rem;
        font-weight: 400;
        color: #1e293b;
        margin: 0.3rem 0;
    }

    .large-deficit-colour {
        color: #ef4444;
    }
    .small-deficit-colour {
        color: rgb(250, 107, 107);
    }


    .no-deficit-colour {
        color: rgb(99, 192, 133);
    }

    .small-surplus-colour {
        color: #ca9a22;
    }

    .large-surplus-colour {
        color: #f0de0d;
    }

    .subtext {
        color: #64748b;
        font-size: 0.875rem;
    }

    .progress-bar {
        width: 100%;
        height: 0.5rem;
        background: #e2e8f0;
        border-radius: 9999px;
        margin: 0.5rem 0;
        overflow: hidden;
        --color: #3b82f6;
    }

    .progress {
        height: 100%;
        background: var(--color);
        transition: width 0.3s ease;
    }

    .graphs-container {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
        gap: 1.5rem;
    }

    .graph-card {
        background: white;
        padding: 1.5rem;
        border-radius: 0.5rem;
        box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
    }

    .graph-card h3 {
        margin: 0 0 1rem 0;
        color: #64748b;
        font-size: 0.875rem;
        text-transform: uppercase;
        letter-spacing: 0.05em;
    }

    .distribution-section {
        background: white;
        padding: 1.5rem;
        border-radius: 0.5rem;
        box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
    }

    .distribution-section h3 {
        margin: 0 0 1.5rem 0;
        color: #64748b;
        font-size: 0.875rem;
        text-transform: uppercase;
        letter-spacing: 0.05em;
    }

    .distribution-grid {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
        gap: 1.5rem;
    }

    .distribution-card {
        background: #f8fafc;
        padding: 1rem;
        border-radius: 0.375rem;
    }

    .distribution-card h4 {
        margin: 0 0 1rem 0;
        color: #475569;
        font-size: 0.875rem;
    }

    table {
        width: 100%;
        border-collapse: collapse;
    }

    th, td {
        padding: 0.5rem;
        text-align: left;
        border-bottom: 1px solid #e2e8f0;
    }

    th {
        color: #64748b;
        font-size: 0.75rem;
        text-transform: uppercase;
        letter-spacing: 0.05em;
    }

    td {
        font-size: 0.875rem;
    }

    tr.positive td {
        color: #22c55e;
    }

    tr.negative td {
        color: #ef4444;
    }

    .equity-section {
        background: white;
        padding: 1.5rem;
        border-radius: 0.5rem;
        box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
    }

    .equity-grid {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
        gap: 1.5rem;
    }

    .equity-card {
        background: #f8fafc;
        padding: 1rem;
        border-radius: 0.375rem;
    }

    .equity-card h4 {
        margin: 0 0 1rem 0;
        color: #1e293b;
        font-size: 1rem;
    }

    .equity-stats {
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
    }

    .equity-stat {
        display: flex;
        flex-direction: column;
        padding: 0.5rem;
        background: white;
        border-radius: 0.25rem;
    }

    .equity-stat .label {
        font-size: 0.75rem;
        color: #64748b;
        text-transform: uppercase;
        letter-spacing: 0.05em;
    }

    .equity-stat .value {
        font-size: 1.25rem;
        font-weight: 600;
        margin: 0.25rem 0;
    }

    .equity-stat .subtext {
        font-size: 0.75rem;
        color: #64748b;
    }

    .deficit .value {
        color: #ef4444;
    }

    .surplus .value {
        color: #22c55e;
    }
</style> 
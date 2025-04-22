<script lang="ts">
    import { EncounterDifficulty, getEncounterDifficultyThresholds, getEncounterExperienceBoundaries } from "$lib/utils/encounter";
    import Card from "../core/Card.svelte";
    import Modal from "../core/Modal.svelte";
    import { onMount } from "svelte";
    import EncounterDifficultyBar from "../encounter/EncounterDifficultyBar.svelte";

    interface Props {
        show: boolean;
    }
    let { show = $bindable() }: Props = $props();


    let boundaryRows = [];
    for (let i = 2; i < 7; i++) {
        let boundaryColumns = [];
        const thresh = getEncounterDifficultyThresholds(i);
        const boundaries = getEncounterExperienceBoundaries(i);

        boundaryColumns.push(`${thresh.get(EncounterDifficulty.Trivial)} (${boundaries.get(EncounterDifficulty.Trivial)!.join("-")})`);
        boundaryColumns.push(`${thresh.get(EncounterDifficulty.Low)} (${boundaries.get(EncounterDifficulty.Low)!.join("-")})`);
        boundaryColumns.push(`${thresh.get(EncounterDifficulty.Moderate)} (${boundaries.get(EncounterDifficulty.Moderate)!.join("-")})`);
        boundaryColumns.push(`${thresh.get(EncounterDifficulty.Severe)} (${boundaries.get(EncounterDifficulty.Severe)!.join("-")})`);
        boundaryColumns.push(`${thresh.get(EncounterDifficulty.Extreme)} (${boundaries.get(EncounterDifficulty.Extreme)!.join("-")})`);
        boundaryRows.push({
            id: i,
            boundaries: boundaryColumns
        });
    }
</script>

<Modal bind:show closeButton>
    <div slot="header">
        <h2>XP and Difficulty Calculation</h2>
    </div>
    <Card>
        <h3>XP Calculation</h3>
        <p>
        The final XP for a combat encounter follows this formula:
    </p>        <code>
            total XP = (XP of all enemies + XP of all hazards) + (extra players
            difference) + (any extra encounter experience)
        </code>
        <p>
            <span class="experience-part-header">extra players difference:</span>
            <span>Any adjustment needed because of the party size (if not a party of 4).</span>
        </p>
        <p>
            <span class="experience-part-header">extra encounter experience:</span>
            <span>Any extra experience that is not part of the combat, such as a
            reward, experience from a subsystem, or manual adjustment to get the number you want.</span>
        </p>
        <p>
        We use the total XP to keep track of statistics and actual experience gained. However, when calculting the difficulty of the encounter, we don't use 'extra experience'.
    </p><p>
        The players gain experience together, as a whole, rather than level up individually.
    </p>
        <h3>Difficulty Calculation</h3>
        <p>
        Encounters in Pathfinder 2e are designed 'top-down', where you pick a difficulty,
        which provides an XP budget for the encounter to match as closely as possible.
        <a href="https://2e.aonprd.com/Rules.aspx?ID=2715" target="_blank">[Read more here]</a>
        </p>
        <p>
        In this calculator, we aim to build bottom-up, where you can select any number
        of enemies and hazards, and we will calculate the XP and difficulty of the
        encounter. This gives more flexibility and customizability in the encounter
        design. So rather then have a fixed value to be 'close to', we define boundaries for each difficulty.
        </p>
        <div class="difficulty-bar">
        <EncounterDifficultyBar
            experience={92}
            partySize={4}
            />
        </div>
        <p>
            As described above, we do not consider 'extra experience' when calculating the difficulty of the encounter.
        </p>
        <p>
            These boundaries change as the number of players changes according to the table below (you can exceed 6 players in the calculator). We apply an adjustment to the final XP based on the number of players. So while a 5th player expands the budget of a severe encounter by 20 XP, that extra 20xp is removed from the final XP rewarded to the players (so you don't get extra XP for having more players).
        </p>

        <table>
            <thead>
                <tr><th scope='col'></th>
                <th scope='col'>Trivial</th>
                <th scope='col'>Low</th>
                <th scope='col'>Moderate</th>
                <th scope='col'>Severe</th>
                <th scope='col'>Extreme</th></tr>
            </thead>
            <tbody>
                {#each boundaryRows as row}
                    <tr>
                        <th scope='row'>{row.id} Player {row.id == 4 ? "/ Finalized" : ""}</th>
                        {#each row.boundaries as boundary}
                            <td>{boundary}</td>
                        {/each}
                    </tr>
                {/each}
                
            </tbody>
        </table>

        <p>
        Generally, it's better to stick near the middle of the boundaries, as the boundary between encounters is a bit blurry- should a 140 XP
        be 'severe' (120 XP) or 'extreme' (160 XP)?
    </p>
    <p>
        Additionally, being near the boundries lead to some weird niche cases for non-4 player encounters that
        approach a boundary, with strange jumps in 'final' XP as it changes between
        difficulty class- or even a decrease in XP when the encounter is made harder.
    </p>    
    <Card collapsed>
        <div slot="header">
            <h3>
                Example of a niche case
            </h3>
        </div>

        <p>Consider the following case, of 5 players at level 7.</p>
        <p>We have 110 experience so far, making our encounter <b>moderate</b>. Because we have 5 players, we apply a -20 adjustment to the experience (because the budget for moderate 20 points higher).</p>
        <p>Our artificial boundary is set at 125 XP. We want to add one more small enemy. But look what happens depending on what we add:</p>
        <p>- 3rd level: We add another 10 XP, putting at us at 120xp, 5 away from our boundary. Because we're still moderate, we subtract 20, and get 100xp</p>
        <p>- 4th level: We add another 15 XP, putting at us at 125xp, reaching our boundary, moving us to <b>severe</b>. We now subtract 30 and get 95xp.</p>
        <p> Obviously, this is quite a niche case! But it shows how the math breaks down at the boundaries between difficulties, and why Paizo encourages you to stay as close to the budget as possible (and why we do too!)  </p>
    </Card>

</Card>
</Modal>

<style>

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
        color: var(--color-text-secondary); ;
        font-size: 0.75rem;
        text-transform: uppercase;
        letter-spacing: 0.05em;
    }

    td {
        font-size: 0.875rem;
    }

.experience-part-header {
    color: var(--color-text-secondary);
    font-weight: bold;
    margin-top: 1rem;
    margin-bottom: 0.5rem;
}

.difficulty-bar {

    align-self: center;
    min-width: 50%
    
}
</style>

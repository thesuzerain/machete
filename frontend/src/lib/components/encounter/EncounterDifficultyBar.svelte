<script lang="ts">
    import { EncounterDifficulty, getEncounterDifficultyThresholds, getEncounterExperienceBoundaries } from "$lib/utils/encounter";

        interface Props {
        experience: number;
        partySize: number;
    }
    let { experience, partySize = 4 }: Props = $props();

    function restrictMaxXp(exp: number): number {
        return Math.min(exp, experience + 10);
    }


    // TODO: modularize, along with css classes
    function getClassForDifficulty(difficulty: EncounterDifficulty): string {
        switch (difficulty) {
            case "Trivial":
                return "difficulty-trivial";
            case "Low":
                return "difficulty-low";
            case "Moderate":
                return "difficulty-moderate";
            case "Severe":
                return "difficulty-severe";
            case "Extreme":
                return "difficulty-extreme";
            default:
                return "difficulty-unknown";
        }
    }

    let experienceBoundaries = $derived(getEncounterExperienceBoundaries(partySize));
    let boundaryTuples : [EncounterDifficulty, number, number][] = $derived.by(() => {
        let a : [EncounterDifficulty, number, number][] = []
        a.push([EncounterDifficulty.Trivial, experienceBoundaries.get(EncounterDifficulty.Trivial)![0], experienceBoundaries.get(EncounterDifficulty.Trivial)![1]])
        a.push([EncounterDifficulty.Low, experienceBoundaries.get(EncounterDifficulty.Low)![0], experienceBoundaries.get(EncounterDifficulty.Low)![1]])
        a.push([EncounterDifficulty.Moderate, experienceBoundaries.get(EncounterDifficulty.Moderate)![0], experienceBoundaries.get(EncounterDifficulty.Moderate)![1]])
        a.push([EncounterDifficulty.Severe, experienceBoundaries.get(EncounterDifficulty.Severe)![0], experienceBoundaries.get(EncounterDifficulty.Severe)![1]])
        a.push([EncounterDifficulty.Extreme, experienceBoundaries.get(EncounterDifficulty.Extreme)![0], restrictMaxXp(experienceBoundaries.get(EncounterDifficulty.Extreme)![1])])
        return a;
    });
    
    let [leftDifficulty, middleDifficulty, rightDifficulty] = $derived.by(() => {
        let left = EncounterDifficulty.Unknown;
        let middle = EncounterDifficulty.Unknown;
        let right = EncounterDifficulty.Unknown;
        for (let i = 0; i < boundaryTuples.length; i++) {
            if (experience >= boundaryTuples[i][1] && experience <= boundaryTuples[i][2]) {
                middle = boundaryTuples[i][0];
                if (i > 0) {
                    left = boundaryTuples[i - 1][0];
                }
                if (i < boundaryTuples.length - 1) {
                    right = boundaryTuples[i + 1][0];
                }
            }
        }
        return [left, middle, right];
    });

    let difficulties :EncounterDifficulty[] = $derived.by(() => {
        let difficulties: EncounterDifficulty[] = [];
        // Start with the trivial difficulty
        difficulties.push(EncounterDifficulty.Unknown);

        let i = 0;
        while (i < boundaryTuples.length) {
            if (experience >= boundaryTuples[i][1]) {
                difficulties.push(boundaryTuples[i][0]);
                i++;
            } else {
                break;
            }
        }

        // Add the next one (to ensure 3 difficulties, if no more add Unknown)
        if (i < boundaryTuples.length) {
            difficulties.push(boundaryTuples[i][0]);
        } else {
            difficulties.push(EncounterDifficulty.Unknown);
        }
        // Take last 3 difficulties
        return difficulties.slice(-3);
    })

    type DifficultyBarBoundary = {
        difficulty: EncounterDifficulty;
        leftPercentage: number;
        rightPercentage: number;
        leftXp: number;
        rightXp: number;
    }

    let [difficultBarPercentageBoundaries, experiencePercentage] : [DifficultyBarBoundary[], number] = $derived.by(() => {
        let boundaries: DifficultyBarBoundary[] = [];
        let difficultiesWithoutUnknown = difficulties.filter(difficulty => difficulty !== EncounterDifficulty.Unknown);

        let experiencePercentage = 50;

        // If the left one is unknown, condense with middle
        if (difficulties[0] === EncounterDifficulty.Unknown) {
            let [leftXp, rightXp] = experienceBoundaries.get(difficulties[1])!;
            boundaries.push({
                difficulty: difficulties[1],
                leftPercentage: 0,
                rightPercentage: 75,
                leftXp: leftXp,
                rightXp: rightXp
            });
            experiencePercentage = (75 * (experience - leftXp) / (rightXp - leftXp));
        } else {
            let [leftXp, rightXp] = experienceBoundaries.get(difficulties[0])!;
            boundaries.push({
                difficulty: difficulties[0], 
                leftPercentage: 0, 
                rightPercentage: 25,
                leftXp: leftXp,
                rightXp: rightXp
            });
        }

        // If neither are unknown, add the middle
        if (difficulties[0] !== EncounterDifficulty.Unknown && difficulties[2] !== EncounterDifficulty.Unknown) {
            let [leftXp, rightXp] = experienceBoundaries.get(difficulties[1])!;
            boundaries.push({
                difficulty: difficulties[1], 
                leftPercentage: 25, 
                rightPercentage: 75,
                leftXp: leftXp,
                rightXp: rightXp
            });
            experiencePercentage = 25 + (50 * (experience - leftXp) / (rightXp - leftXp));
        }

        // If the right one is unknown, condense with middle
        if (difficulties[2] === EncounterDifficulty.Unknown) {
            let [leftXp, rightXp] = experienceBoundaries.get(difficulties[1])!;
            rightXp = restrictMaxXp(rightXp);
            boundaries.push({
                difficulty: difficulties[1], 
                leftPercentage: 25, 
                rightPercentage: 100,
                leftXp: leftXp,
                rightXp: rightXp
            });
            experiencePercentage = 25 + (75 * (experience - leftXp) / (rightXp - leftXp));
        } else {
            let [leftXp, rightXp] = experienceBoundaries.get(difficulties[2])!;
            rightXp = restrictMaxXp(rightXp);
            boundaries.push({
                difficulty: difficulties[2], 
                leftPercentage: 75, 
                rightPercentage: 100,
                leftXp: leftXp,
                rightXp: rightXp
            });
        }
        
    
        return [boundaries, experiencePercentage];
    })

</script>

<div class="container">
    <div class="progress-bar">
        {#each difficultBarPercentageBoundaries as boundary, i}
            <div class="progress {getClassForDifficulty(boundary.difficulty)}" 
             style="width: {boundary.rightPercentage-boundary.leftPercentage}%">
            <div class="progress-text">{boundary.difficulty}</div></div>
             
             {#if i > 0}
             <div class="interruption-line" 
             style="left: {boundary.leftPercentage}%">
                <div class="xp-label">{boundary.leftXp}</div>
            </div>
             {/if}
        {/each}

        <div class="exp-line" 
             style="left: {experiencePercentage}%">
            <div class="xp-label xp-label-background current">{experience}</div>
        </div>
    </div>

</div>
  



<style>
    .container {
        position: relative;
    }

    .progress-bar {
        width: 100%;
        height: 1rem;
        border-radius: 9999px;
        margin: 0.5rem 0;
        overflow: visible;
        display: flex;
        align-items: center;
        position: relative;
    }

    .interruption-line {
        position: absolute;
        width: 0.1rem;
        height: 100%;
        background: var(--color-text-dark-secondary);
        z-index: 2;
    }

    .exp-line {
        position: absolute;
        width: 0.4rem;
        height: 100%;
        background: var(--color-text);
        z-index: 2;
        transition: left 0.3s ease;
    }

    .progress {
        height: 60%;
        transition: width 0.3s ease;
        position: relative;
        cursor: pointer;

        font: inherit;
        font-size: 0.875rem;
        color: var(--color-text-secondary);
        text-align: center;
        line-height: 1.5rem;
        justify-content: center;
    }

    .progress-text {
        position: absolute;
        top: 50%;
        left: 50%;
        transform: translate(-50%, -100%);
        font-size: 0.875rem;
        color: var(--color-text-dark-secondary);
        white-space: nowrap;
    }


    .difficulty-trivial {
        background: var(--color-difficulty-trivial);
    }

    .difficulty-low {
        background: var(--color-difficulty-low);
    }

    .difficulty-moderate {
        background: var(--color-difficulty-moderate);
    }

    .difficulty-severe {
        background: var(--color-difficulty-severe);
    }

    .difficulty-extreme {
        background: var(--color-difficulty-extreme);
    }

    .xp-label {
        position: absolute;
        top: -1.1rem;
        transform: translateX(-50%);
        left: 50%;
        font-size: 0.7rem;
        color: var(--color-text-dark-secondary);
        white-space: nowrap;
    }

    .xp-label-background {
        top: -1.8rem;
        border-radius: 3px;
        background: var(--color-bg-raised);
        padding: 0.2rem 0.4rem;
        box-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
    }

</style>
<script lang="ts">
    import { fade } from 'svelte/transition';
    import { API_URL } from '$lib/config';
    import { creatureStore, hazardStore, itemStore } from '$lib/stores/libraryStore';
  import type { LibraryCreature, LibraryHazard, LibraryItem, LibrarySpell } from '$lib/types/library';
    interface Props {
        enemies: number[];
        hazards: number[];
        treasures: number[];
    }
    let { 
        enemies = $bindable(),
        hazards = $bindable(),
        treasures = $bindable(),

     } : Props = $props();
    

    let description = $state('');
    let html = $state('');

        type LibraryObjectWithType = LibraryCreature & { type: string } | LibraryItem & { type: string } | LibraryHazard & { type: string } | LibrarySpell & { type: string };
        type AugmentedNoun = {
            span: [number, number];
            text: string;
            augmented: string;
            augment: LibraryObjectWithType;
        }

    let currentMatches : AugmentedNoun[] = $state([]);

    async function handleDescriptionChange() {
        console.log(description);

        const response = await fetch(`${API_URL}/nlp/augmented-nlp`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                text: description,
            }),
        })

        const data : AugmentedNoun[] = await response.json();
        currentMatches = data;

        // filter out spells for now
        // TODO: add spells back in
        currentMatches = currentMatches.filter(match => match.augment.type.toLowerCase() !== 'spell');
        currentMatches = currentMatches.filter(match => match.augment.type.toLowerCase() !== 'none');

        console.log(data);

        enemies = [];
        treasures = [];

        for (const entity of data) {
            console.log("entity", entity);
            if (entity.augment.type.toLowerCase() === 'creature') {
                console.log('creature', entity);
                creatureStore.insertEntity(entity.augment as LibraryCreature);
                console.log("inserted");
                enemies = [...enemies, entity.augment.id];
                console.log("pushed, ", enemies);
            } else if (entity.augment.type.toLowerCase() === 'hazard') {
                console.log('hazard', entity);
                hazardStore.insertEntity(entity.augment as LibraryHazard);
                hazards.push(entity.augment.id);
            } else if (entity.augment.type.toLowerCase() === 'item') {
                console.log('item', entity);
                itemStore.insertEntity(entity.augment as LibraryItem);
                treasures.push(entity.augment.id);
            }
        }

        enemies = enemies;

        console.log("Updated enemies", enemies);

        html = applyHighlights(description, currentMatches);
    }

    function applyHighlights(text : string, matches : AugmentedNoun[]) {
        text = text
        .replace(/\n$/g, '\n\n');
        for (const match of matches) {
            text = text.replace(match.text, '<mark>' + match.text + '</mark>');
        }
        console.log(text);
        return text;
        }


</script>

<div class="import-section" transition:fade>
    <div class="upper-section">
    <div class="container">
    <div class ="backdrop">
        <div class="highlights" bind:innerHTML={html} contenteditable="false">
        </div>
    </div>
            <textarea
            name="description"
            id="description"
            bind:value={description}
            on:keyup={handleDescriptionChange}
            required
            placeholder="Describe what happened..."
        ></textarea>
    </div>
</div>
</div>

<style>
@import url(https://fonts.googleapis.com/css?family=Open+Sans);

.log-section {
        margin-bottom: 1.5rem;
        padding: 1rem;
        background: #f9fafb;
        border-radius: 4px;
        
}

.metadata-section {
    padding: 1rem;
    background: #f9fafb;
    border-radius: 4px;
}


    .upper-section {
    padding-bottom: 1rem;
}

.container, .backdrop, textarea {
  width: 100%;
  height: 180px;
}

.highlights, textarea {
  padding: 10px;
  font: 1.0rem/1.0rem 'Open Sans', sans-serif;
  letter-spacing: 1px;
}

.container {
  display: block;
  margin: 0 auto;
  transform: translateZ(0);
  -webkit-text-size-adjust: none;
}

.backdrop {
  position: absolute;
  z-index: 1;
  border: 2px solid #685972;
  background-color: #fff;
  overflow: auto;
  pointer-events: none;
  transition: transform 1s;
}

.highlights {
  white-space: pre-wrap;
  word-wrap: break-word;
  color: transparent;
}

textarea {
  display: block;
  position: absolute;
  z-index: 2;
  margin: 0;
  border: 2px solid #74637f;
  border-radius: 0;
  color: #444;
  background-color: transparent;
  overflow: auto;
  resize: none;
  transition: transform 1s;
}

mark {
  border-radius: 3px;
  color: transparent;
  background-color: #b1d5e5;
}

textarea:focus, button:focus {
  outline: none;
  box-shadow: 0 0 0 2px #c6aada;
}


</style> 

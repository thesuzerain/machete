<script lang="ts">
    // TODO: Change from name to some styling name (for css swaps)
    type Colour = 'red' | 'green' | 'black' | 'blue' | 'white' | 'grey';
    interface Props {
        colour? : Colour;
        large? : boolean;
        tight? : boolean;
        onclick? : () => void;
        selected? : boolean;
        disabled? : boolean;
        submit? : boolean;
        selectedColour? : Colour;
    }

    let { 
        colour = 'black',
        large = false,
        selected = false,
        tight = false,
        submit = false,
        selectedColour,
        disabled = false,
        onclick
    } : Props = $props();

    let classLargeness = $derived(large ? 'button-large' : 'button-small');
    let classColour = $derived(`color-${colour}`);
    
    let classSelectedColour = $derived(selectedColour ? `color-${selectedColour}` : classColour);
    let shouldSelectedColourVersion = $derived(!selectedColour && selected);
</script>

<button type={submit ? "submit":"button"} {onclick} disabled={disabled ?? false} class="{classLargeness} {selected ? classSelectedColour : classColour}" class:selected={shouldSelectedColourVersion}
    class:disabled={disabled} class:tight={tight}
>
        <slot />
    </button>

<style>



.button-small {
    padding: 0.5rem 1rem;
    border-radius: 0.375rem;
    font-weight: 500;
    white-space: nowrap;
}

.button-large {
    font-weight: 500;
    font-size: 1.2rem;
    padding: 1rem 3rem;
    margin: 0.5rem 0;
}

.color-red {
    background-color: #f44336;
    color: white;
}

.color-green {
    background-color: #4CAF50;
    color: white;
}

.color-green.selected {
    background-color: #2d6a4f;
}

.color-black {
    background: #4b5563;
        color: white;
}
.color-black.selected {
    background-color: #000;
}

.color-blue {
    background: #3b82f6;
    color: white;
}
.color-blue.selected {
    background: #1e40af;
}

.color-white {
    background: white;
    color: rgb(63, 63, 63);
    border: 1px solid #e6e6e6;
}
.color-white.selected {
    background: #f3f4f6;
}

.color-grey {
    background: #999999;
    color: white;
}
.color-grey.selected {
    background: #666666;
}

.disabled {
    cursor: not-allowed;
    background: #9ca3af;
    color: white;
}

.tight {
    padding: 0.25rem 0.5rem;
}

</style> 
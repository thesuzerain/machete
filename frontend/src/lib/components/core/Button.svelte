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

    function onClick(e: MouseEvent) {
        if (disabled) return;
        e.stopPropagation();
        if (onclick) onclick();
    }
</script>

<button type={submit ? "submit":"button"} onclick={(e) => onClick(e)} disabled={disabled ?? false} class="{classLargeness} {selected ? classSelectedColour : classColour}" class:selected={shouldSelectedColourVersion}
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
    background-color: var(--color-red);
    color: var(--color-text-light);
}

.color-green {
    background-color: var(--color-green);
    color: var(--color-text-light);
}

.color-green.selected {
    background-color: var(--color-green-selected);
}

.color-black {
    background:var(--color-dark-grey);
}
.color-black.selected {
    background-color: var(--color-dark-grey-selected);
}

.color-blue {
    background-color: var(--color-blue);
    color: var(--color-text-light);
}
.color-blue.selected {
    background-color: var(--color-blue-selected);
}

.color-white {
    background-color: var(--color-white);
    color: var(--color-text-dark);
}
.color-white.selected {
    background: var(--color-white-selected);
}

.color-grey {
    background: var(--color-grey);
    color: var(--color-text-light);
}
.color-grey.selected {
    background: var(--color-grey-selected);
}

.disabled {
    cursor: not-allowed;
    background-color: #9ca3af;
    color: white;
}

.tight {
    padding: 0.25rem 0.5rem;
}

</style> 
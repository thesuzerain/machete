<script lang="ts">
    // TODO: Change from name to some styling name (for css swaps)
    type Colour = 'red' | 'green' | 'black' | 'blue' | 'white' | 'grey';
    interface Props {
        colour? : Colour;
        large? : boolean;
        tight? : boolean;
        left? : boolean;
        outlined?: boolean;
        onclick? : () => void;
        selected? : boolean;
        disabled? : boolean;
        submit? : boolean;
        selectedColour? : Colour;
    }

    let { 
        colour = 'black',
        large = false,
        left = false,
        selected = false,
        tight = false,
        submit = false,
        outlined = false,
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

<button type={submit ? "submit":"button"} onclick={(e) => onClick(e)} disabled={disabled ?? false} class:outlined={outlined ?? false} class="{classLargeness} {selected ? classSelectedColour : classColour}" class:selected={shouldSelectedColourVersion}
    class:disabled={disabled} class:tight={tight} 
>
    <div class:left={left}>
        <slot />
        <div>
    </button>

<style>



.button-small {
    padding: 0.5rem 1rem;
    border-radius: 0.375rem;
    font-weight: 500;
    white-space: nowrap;
    min-height: 100%;
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
.color-red.selected {
    background-color: var(--color-red-selected);
}
.color-red:hover {
    background-color: var(--color-red-hover);
}

.color-green {
    background-color: var(--color-green);
    color: var(--color-text-light);
}

.color-green.selected {
    background-color: var(--color-green-selected);
}

.color-green:hover {
    background-color: var(--color-green-hover);
}

.color-black {
    background:var(--color-off-black);
}
.color-black.selected {
    background-color: var(--color-off-black-selected);
}
.color-black:hover {
    background-color: var(--color-off-black-hover);
}

.color-blue {
    background-color: var(--color-blue);
    color: var(--color-text-light);
}
.color-blue.selected {
    background-color: var(--color-blue-selected);
}
.color-blue:hover {
    background-color: var(--color-blue-hover);
}

.color-white {
    background-color: var(--color-white);
    color: var(--color-text-dark);
}
.color-white.selected {
    background: var(--color-white-selected);
}
.color-white:hover {
    background: var(--color-white-hover);
}
.color-grey {
    background: var(--color-grey);
    color: var(--color-text-dark);
}
.color-grey.selected {
    background: var(--color-grey-selected);
}
.color-grey:hover {
    background: var(--color-grey-hover);
}
.color-grey.outlined {
    border: 1px solid var(--color-grey-border);
}

.disabled {
    cursor: not-allowed;
    background-color: var(--color-bg-disabled);
    color: var(--color-disabled-text);
}

.disabled:hover {
    background-color: var(--color-bg-disabled-hover);
}

.tight {
    height: fit-content;
}

.left {
    margin-left: 0;
    align-items: right;
    display: flex;
    justify-content: flex-start;
    margin-right: auto;
}

</style> 
<script lang="ts">
    import Button from './Button.svelte';
    import { clickOutside } from '$lib/actions/clickOutside';
    import { FontAwesomeIcon } from "@fortawesome/svelte-fontawesome";
    import { faCaretDown } from '@fortawesome/free-solid-svg-icons';

    type Colour = 'red' | 'green' | 'black' | 'blue' | 'white' | 'grey';
    
    interface Props {
        colour?: Colour;
        large?: boolean;
        tight?: boolean;
        disabled?: boolean;
        label: string;
    }

    let {
        colour = 'black',
        large = false,
        tight = false,
        disabled = false,
        label
    }: Props = $props();

    let isOpen = $state(false);

    function toggleDropdown() {
        if (!disabled) {
            isOpen = !isOpen;
        }
    }

    function closeDropdown() {
        isOpen = false;
    }
</script>

<div class="dropdown"
use:clickOutside={{ handler: closeDropdown }}>
    <Button 
        {colour}
        {large}
        {tight}
        {disabled}
        onclick={toggleDropdown}
    >
        <div class="button-content">
            {label}
            <FontAwesomeIcon
            icon={faCaretDown}
            />
    </div>
    </Button>

    {#if isOpen}
        <div class="dropdown-content color-{colour}" class:large>
            <slot />
        </div>
    {/if}
</div>

<style>
    .dropdown {
        position: relative;
        display: inline-block;
        height: auto;
    }

    .button-content {
        gap: 1rem;
        display: flex;
        align-items: center;
    }

    .dropdown-content {
        position: absolute;
        min-width: 100%;
        top: 100%;
        left: 0;
        background: var(--color-white);
        border-radius: 0.375rem;
        box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
        z-index: 50;
        display: flex;
        flex-direction: column;
    }

    .dropdown-content.large {
        gap: 0.5rem;
        padding: 0.75rem;
    }



    .color-red {
background-color: var(--color-red);
color: var(--color-text-light);
}
.color-green {
    background-color: var(--color-green);
    color: var(--color-text-light);
}

.color-black {
    background:var(--color-dark-grey);
}

.color-blue {
    background-color: var(--color-blue);
    color: var(--color-text-light);
}

.color-white {
    background-color: var(--color-white);
    color: var(--color-text-dark);
}

.color-grey {
    background: var(--color-grey);
    color: var(--color-text-light);
}
</style>

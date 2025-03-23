<script lang="ts">
// TODO: Add shadow

    //TODO: one/default slot for same header if same
    import { fade } from "svelte/transition";

    type BackgroundColour = 'light' | 'grey' | 'dark';

    interface Props {
        collapsed?: boolean | undefined;
        background?: BackgroundColour | undefined;
        outlined?: boolean | undefined;
        shadowed?: boolean | undefined;
        tight?: boolean | undefined;

        // TODO: Not sure if I like this solution, as it still uses global variables and messes with inner components
        softHeaders?: boolean | undefined;
    }

    let { 
        collapsed = $bindable(),
        background = 'grey',
        tight = false,
        outlined = false,
        shadowed = true,
        softHeaders = false
    } : Props = $props();

    let tightness = tight ? 'tight' : 'loose';

</script>

<div class="card-{tightness} colour-{background} class:soft-headers={softHeaders}" class:outlined={outlined} class:shadowed={shadowed}>
    {#if collapsed !== undefined}
        <div class="collapse-header" on:click={() => collapsed = !collapsed}>
            {#if collapsed}
            <slot name="collapsed-header" />
            {:else}
            <slot name="header" />
            {/if}
            <span class="toggle-icon">{collapsed ? '▶' : '▼'}</span>
        </div>
        {#if !collapsed }
            <div class="collapsible-content" transition:fade>
            <slot />
            </div>
        {/if}
    {:else}
        <slot />    
    {/if}
</div>

<style>

.collapse-header {
        cursor: pointer;
        user-select: none;
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    .section-content {
        padding-top: 1rem;
    }

.card-tight {
    padding: 0.5rem;
    border-radius: 0.375rem;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
}

.card-loose {
    /* TODO: check margin: 1.5rem 0; */
    display: flex;
    flex-direction: column;
    padding: 1.5rem;
    border-radius: 1rem;
    gap: 1rem;
}

.colour-dark {
    background: #64748b;
}
.colour-dark.outlined {
    border: 1px solid #a7bcd8;
}

.colour-light {
    background: white;
}

.colour-light.outlined {
    border: 1px solid #e6e6e6;
}

.colour-grey {
    background: #f8fafc;    
}

.colour-grey.outlined {
    border: 1px solid #e5e7eb;
}




.toggle-icon {
        font-size: 0.8em;
        color: #666;
    }

    .shadowed {
        box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
    }


/* 
TODO: Double check if global is best approch here. May be bad practice if ew
 */
.soft-headers :global(h4), .soft-headers :global(h3), .soft-headers :global(h2), .soft-headers :global(h1) {
    color: #64748b;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: #64748b;
}

/* .card :global(h4) {
    font-size: 0.75rem;
    font-weight: 500;
}

.card :global(h3) {
    font-size: 1rem;
    font-weight: 500;
}

.card :global(h2) {
    font-size: 1.5rem;
    font-weight: 500;
}

.card :global(h1) {
    font-size: 2rem;
    font-weight: 500;
} */
</style> 
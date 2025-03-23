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
    }

    let { 
        collapsed = $bindable(),
        background = 'grey',
        tight = false,
        outlined = false,
        shadowed = true,
    } : Props = $props();

    let tightness = tight ? 'tight' : 'loose';

</script>

<div class="card-{tightness} colour-{background}" class:outlined={outlined} class:shadowed={shadowed}>
    {#if collapsed !== undefined}
        <div class="collapse-header" on:click={() => collapsed = !collapsed}>
            {#if collapsed}
            <slot name="collapsed-header"><slot name="header" /></slot>
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
    border: 1px solid var(--color-bg-raised-border);
}

.colour-light {
    background: var(--color-bg);
}

.colour-light.outlined {
    border: 1px solid var(--color-bg-border);
}

.colour-grey {
    background: var(--color-bg-light-raised);    
}

.colour-grey.outlined {
    border: 1px solid var(--color-bg-light-raised-border);
}

.toggle-icon {
        font-size: 0.8em;
    }

    .shadowed {
        box-shadow: var(--shadow);
    }
</style> 
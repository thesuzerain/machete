<script lang="ts">
    import { createEventDispatcher } from 'svelte';
    import { fade, slide } from 'svelte/transition';

    export let options: string[] = [];
    export let placeholder = "Search for tags and traits...";
    export let selected: string[] = [];
    export let restrictChoice = true;

    const dispatch = createEventDispatcher();

    let searchTerm = '';
    let showDropdown = false;
    let selectedIndex = 0;
    let inputElement: HTMLInputElement;

    $: sortedOptions = options.sort((a, b) => {
        if (a.toLowerCase().startsWith(searchTerm.toLowerCase()) && 
            b.toLowerCase().startsWith(searchTerm.toLowerCase())) {
            return a.localeCompare(b);
        }
        if (a.toLowerCase().startsWith(searchTerm.toLowerCase())) {
            return -1;
        }
        if (b.toLowerCase().startsWith(searchTerm.toLowerCase())) {
            return 1;
        }
        return a.localeCompare(b)
    });

    $: filteredOptions = sortedOptions
        .filter(option => 
            !selected.includes(option) && 
            option.toLowerCase().includes(searchTerm.toLowerCase())
        );

    $: showCustomOption = !restrictChoice && 
        searchTerm && 
        !options.includes(searchTerm) && 
        !selected.includes(searchTerm);

    $: allOptions = showCustomOption 
        ? [...filteredOptions, searchTerm] 
        : filteredOptions;

    $: if (allOptions.length === 0) {
        showDropdown = false;
    }

    function isCustomOption(option: string): boolean {
        return !options.includes(option);
    }

    function handleKeydown(event: KeyboardEvent) {
        switch (event.key) {
            case 'ArrowDown':
                if (!showDropdown || allOptions.length === 0) return;
                event.preventDefault();
                selectedIndex = (selectedIndex + 1) % allOptions.length;
                break;
            case 'ArrowUp':
                if (!showDropdown || allOptions.length === 0) return;
                event.preventDefault();
                selectedIndex = (selectedIndex - 1 + allOptions.length) % allOptions.length;
                break;
            case 'Enter':
                if (!showDropdown || allOptions.length === 0) return;
                event.preventDefault();
                if (allOptions[selectedIndex]) {
                    handleSelect(allOptions[selectedIndex]);
                }
                break;
            case 'Escape':
                if (!showDropdown || allOptions.length === 0) return;
                event.preventDefault();
                showDropdown = false;
                break;
            case 'Backspace':
            case 'Delete':
                if (searchTerm === '' && selected.length > 0) {
                    removeItem(selected[selected.length - 1]);
                }
                break;
        }
    }

    function handleSelect(option: string) {
        selected = [...selected, option];
        searchTerm = '';
        showDropdown = false;
        dispatch('change', { selected });
        inputElement.focus();
    }

    function removeItem(item: string) {
        selected = selected.filter(i => i !== item);
        dispatch('change', { selected });
    }

    function handleFocus() {
        if (searchTerm && allOptions.length > 0) {
            showDropdown = true;
        }
    }

    $: if (searchTerm) {
        showDropdown = allOptions.length > 0;
        selectedIndex = 0;
    }
</script>

<div class="multi-select">
    <div class="input-container" 
         on:click={() => inputElement.focus()}>
        {#each selected as item}
            <div class="tag" class:custom={isCustomOption(item)} transition:fade>
                {item}
                <button class="remove-button" on:click|stopPropagation={() => removeItem(item)}>×</button>
            </div>
        {/each}
        <input
            bind:this={inputElement}
            type="text"
            {placeholder}
            bind:value={searchTerm}
            on:focus={handleFocus}
            on:keydown={handleKeydown}
            class="search-input"
        />
    </div>

    {#if showDropdown}
        <div class="dropdown" transition:slide>
            {#each allOptions as option, i}
                <button
                    class="dropdown-item"
                    class:selected={selectedIndex === i}
                    class:custom={isCustomOption(option)}
                    on:click={() => handleSelect(option)}
                >
                    {option}
                    {#if isCustomOption(option)}
                        <span class="custom-indicator">(custom)</span>
                    {/if}
                </button>
            {/each}
        </div>
    {/if}
</div>

<style>
    .multi-select {
        position: relative;
        width: 100%;
    }

    .input-container {
        display: flex;
        flex-wrap: wrap;
        gap: 0.5rem;
        padding: 0.25rem;
        min-height: 1rem;
        border: 1px solid var(--color-bg-light-raised-border);
        border-radius: 4px;
        background: var(--color-bg);
        cursor: text;
        align-items: center;
    }

    .input-container:focus-within {
        box-shadow: 0 0 0 1px var(--color-primary);
    }

    .search-input {
        border: none;
        outline: none;
        padding: 0.25rem;
        flex: 1;
        min-width: 120px;
        background: transparent;
        color: var(--color-text);
    }

    .tag {
        display: flex;
        align-items: center;
        gap: 0.25rem;
        padding: 0.25rem 0.5rem;
        background: var(--color-bg-raised);
        border-radius: 4px;
        font-size: 0.875rem;
        color: var(--color-text);
    }

    .tag.custom {
        background: var(--color-bg-light-raised);
        border: 1px dashed var(--color-bg-border);
    }

    .remove-button {
        border: none;
        background: none;
        padding: 0;
        margin-left: 0.25rem;
        cursor: pointer;
        font-size: 1.25rem;
        line-height: 1;
        color: var(--color-text-secondary);
    }

    .remove-button:hover {
        color: var(--color-text);
    }

    .dropdown {
        position: absolute;
        top: 100%;
        left: 0;
        right: 0;
        max-height: 15rem;
        overflow-y: auto;
        background: var(--color-bg);
        border: 1px solid var(--color-bg-border);
        border-radius: 4px;
        box-shadow: var(--shadow);
        z-index: 100;
        margin-top: 0.25rem;
    }

    .dropdown-item {
        display: flex;
        align-items: center;
        width: 100%;
        padding: 0.5rem;
        border: none;
        background: none;
        text-align: left;
        cursor: pointer;
        color: var(--color-text);
        justify-content: space-between;
    }

    .dropdown-item.custom {
        background: var(--color-bg-light-raised);
        font-style: italic;
    }

    .dropdown-item:hover,
    .dropdown-item.selected {
        background: var(--color-bg-raised);
    }

    .dropdown-item.custom:hover,
    .dropdown-item.custom.selected {
        background: var(--color-bg-light-raised-hover);
    }

    .custom-indicator {
        font-size: 0.75rem;
        color: var(--color-text-secondary);
        margin-left: 0.5rem;
    }
</style> 
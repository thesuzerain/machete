<script lang="ts">
    type Colour = 'red' | 'green' | 'black';
    interface Props {
        colour? : Colour;
        large? : boolean;
        onclick? : () => void;
    }

    let { 
        colour = 'black',
        large = false,
        onclick
    } : Props = $props();

    let classLargeness = large ? 'button-large' : '';
    let classColour = `color-${colour}`;
</script>

<div>
    <button {onclick} class="button {classLargeness} {classColour}" >
        <slot />
    </button>
</div>

<style>

.button {
    cursor: pointer;
}

.button {
    background-color: #f44336;
        color: white;
        padding: 14px 20px;
        margin: 8px 0;
}

.button-large {
    background-color: #f44336;
    font-weight: 400;
    font-size: 1.2rem;
        color: white;
        padding: 1rem 3rem;
        margin: 1rem 0;
}

.color-red {
    background-color: #f44336;
    color: white;
}

.color-green {
    background-color: #4CAF50;
    color: white;
}

.color-black {
    background-color: #333;
    color: white;
}
</style> 
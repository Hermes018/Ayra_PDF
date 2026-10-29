<script lang="ts" generics="T">
    import type { Snippet } from 'svelte';

    let { items, children }: { items: T[]; children: Snippet<[T, number]> } = $props();
</script>

{#each items as item, index}
    {@render children(item, index)}
{/each}

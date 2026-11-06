<script lang="ts">
    import '../app.css';
    import { type Snippet } from 'svelte';
    import { theme } from '$lib/core/themeContract.svelte';

    // Token values come only from the theme contract; write them before first paint.
    if (typeof document !== 'undefined') theme.applyToRoot(document.documentElement);

    let { children } = $props<{ children: Snippet }>();
</script>

{@render children()}

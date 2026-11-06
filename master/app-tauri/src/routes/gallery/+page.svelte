<script lang="ts">
    // Internal component gallery. Compiled in only when the build sets VITE_LPDF_GALLERY=1 (E2E/debug builds);
    // otherwise the constant folds to false, the dynamic import is dropped, and nothing of the gallery ships.
    const loader = import.meta.env.VITE_LPDF_GALLERY === '1' ? import('$lib/gallery/Gallery.svelte') : null;
</script>

{#if loader}
    {#await loader then m}
        <m.default />
    {/await}
{:else}
    <p>This page is not available in this build.</p>
{/if}

<script lang="ts">
  import { tools } from '$lib/core/toolEngine.svelte';
  import StudioIcon, { type StudioIconName } from './StudioIcon.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import IconButton from '$lib/components/ui/IconButton.svelte';
  import { labsVisible, selectToolIfAllowed, studioToolLabel } from './labs.svelte';

  let { onPersistPrefs }: { onPersistPrefs?: () => void } = $props();

  let favoritePresets = $derived(
    tools.favorites
      .map((fid) => tools.presets[fid])
      .filter((p): p is NonNullable<typeof p> => Boolean(p))
      // A Labs tool pinned earlier stays saved but is not shown while Labs is off.
      .filter((p) => labsVisible(p.tool))
  );

  let isCurrentPinned = $derived(
    tools.activePreset ? tools.favorites.includes(tools.activePreset.id) : false
  );

  function selectFavorite(presetId: string, toolId: string) {
    selectToolIfAllowed(toolId);
  }

  function toggleCurrentPin() {
    if (!tools.activePreset) return;
    tools.toggleFavorite(tools.activePreset.id);
    onPersistPrefs?.();
  }

  function resolveIcon(tool: string): StudioIconName {
    if (tool.startsWith('pen.b')) return 'pen.b';
    if (tool.startsWith('pen')) return 'pen.a';
    if (tool.startsWith('highlighter') || tool === 'review.highlight') return 'highlighter';
    if (tool.startsWith('eraser.whole')) return 'eraser.whole';
    if (tool.startsWith('eraser')) return 'eraser.partial';
    if (tool === 'laser') return 'laser';
    if (tool.startsWith('shape') || tool.startsWith('hatch')) return 'shape';
    if (tool.startsWith('measure')) return 'measure';
    if (tool.startsWith('review') || tool.startsWith('insert') || tool.startsWith('callout')) return 'text.review';
    if (tool.startsWith('select')) return 'select';
    return 'pan';
  }
</script>

<div
  class="favorites-bar glass-panel"
  role="toolbar"
  tabindex={0}
  aria-label="Pinned tool presets"
  onpointerdown={(e) => e.stopPropagation()}
>
  <span class="fav-title">Pins</span>
  <div class="fav-items">
    {#each favoritePresets as preset (preset.id)}
      {@const active = tools.currentToolId === preset.tool}
      <Button
        variant="plain"
        size="sm"
        class="fav-btn"
        pressed={active}
        title="{studioToolLabel(preset.tool, preset.label)} ({preset.stroke.widthPt} pt)"
        onclick={() => selectFavorite(preset.id, preset.tool)}
      >
        <StudioIcon name={resolveIcon(preset.tool)} size={14} />
        <span
          class="fav-swatch"
          style:background-color={preset.stroke.colorHex}
          style:opacity={preset.stroke.opacity}
        ></span>
      </Button>
    {/each}
  </div>

  {#if tools.activePreset}
    <IconButton
      icon={isCurrentPinned ? 'star.filled' : 'star'}
      size="sm"
      ariaLabel={isCurrentPinned ? 'Unpin active preset' : 'Pin active preset'}
      onclick={toggleCurrentPin}
    />
  {/if}
</div>

<style>
  .favorites-bar {
    position: absolute;
    bottom: var(--space-4);
    left: var(--space-4);
    z-index: 150;
    pointer-events: auto;
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-1) var(--space-3);
    border-radius: var(--radius-pill);
    background: var(--glass-bg);
    backdrop-filter: blur(var(--glass-blur));
    -webkit-backdrop-filter: blur(var(--glass-blur));
    border: 1px solid var(--glass-border);
    box-shadow: var(--glass-shadow);
    color: var(--glass-text);
  }

  .fav-title {
    font-size: var(--font-size-caption);
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    opacity: 0.65;
    padding-right: var(--space-1);
  }

  .fav-items {
    display: flex;
    align-items: center;
    gap: var(--space-1);
  }

  :global(.fav-btn) {
    position: relative;
  }

  .fav-swatch {
    position: absolute;
    bottom: var(--space-1);
    right: var(--space-1);
    width: var(--space-2);
    height: var(--space-2);
    border-radius: var(--radius-pill);
    border: 1px solid var(--glass-border);
  }
</style>

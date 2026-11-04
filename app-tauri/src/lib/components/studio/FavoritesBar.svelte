<script lang="ts">
  import { tools } from '$lib/core/toolEngine.svelte';
  import StudioIcon, { type StudioIconName } from './StudioIcon.svelte';

  let { onPersistPrefs }: { onPersistPrefs?: () => void } = $props();

  let favoritePresets = $derived(
    tools.favorites
      .map((fid) => tools.presets[fid])
      .filter((p): p is NonNullable<typeof p> => Boolean(p))
  );

  let isCurrentPinned = $derived(
    tools.activePreset ? tools.favorites.includes(tools.activePreset.id) : false
  );

  function selectFavorite(presetId: string, toolId: string) {
    tools.selectTool(toolId);
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
  aria-label="Pinned Tool Presets"
  onpointerdown={(e) => e.stopPropagation()}
>
  <span class="fav-title">Pins</span>
  <div class="fav-items">
    {#each favoritePresets as preset (preset.id)}
      {@const active = tools.currentToolId === preset.tool}
      <button
        type="button"
        class="fav-btn"
        class:active
        aria-pressed={active}
        onclick={() => selectFavorite(preset.id, preset.tool)}
        title="{preset.label} ({preset.stroke.widthPt} pt)"
      >
        <StudioIcon name={resolveIcon(preset.tool)} size={14} />
        <span
          class="fav-swatch"
          style:background-color={preset.stroke.colorHex}
          style:opacity={preset.stroke.opacity}
        ></span>
      </button>
    {/each}
  </div>

  {#if tools.activePreset}
    <button
      type="button"
      class="fav-pin-toggle"
      class:pinned={isCurrentPinned}
      onclick={toggleCurrentPin}
      title={isCurrentPinned ? 'Unpin active preset from Favorites' : 'Pin active preset to Favorites'}
      aria-label={isCurrentPinned ? 'Unpin active preset' : 'Pin active preset'}
    >
      <StudioIcon name="pin" size={12} />
    </button>
  {/if}
</div>

<style>
  .favorites-bar {
    position: absolute;
    bottom: 16px;
    left: 16px;
    z-index: 150;
    pointer-events: auto;
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 5px 10px;
    border-radius: 999px;
    background: var(--glass-bg);
    backdrop-filter: blur(var(--glass-blur));
    -webkit-backdrop-filter: blur(var(--glass-blur));
    border: 1px solid var(--glass-border);
    box-shadow: var(--glass-shadow);
    color: var(--glass-text);
  }

  .fav-title {
    font-size: 10px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    opacity: 0.65;
    padding-right: 4px;
  }

  .fav-items {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .fav-btn {
    position: relative;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 30px;
    height: 30px;
    border-radius: 8px;
    border: 1px solid transparent;
    background: transparent;
    color: var(--glass-text);
    cursor: pointer;
  }

  .fav-btn:hover {
    background: var(--canvas-grid-minor);
  }

  .fav-btn.active {
    background: var(--accent);
    color: var(--laser-core);
  }

  .fav-swatch {
    position: absolute;
    bottom: 3px;
    right: 3px;
    width: 6px;
    height: 6px;
    border-radius: 50%;
    border: 1px solid var(--glass-border);
  }

  .fav-pin-toggle {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border-radius: 50%;
    border: 1px solid var(--glass-border);
    background: transparent;
    color: var(--glass-text);
    font-size: 12px;
    cursor: pointer;
    opacity: 0.75;
  }

  .fav-pin-toggle.pinned {
    color: var(--accent);
    opacity: 1;
  }
</style>

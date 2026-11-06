<script lang="ts">
  import SpatialCanvas from './SpatialCanvas.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import DocumentTabs from '$lib/components/studio/DocumentTabs.svelte';
  import { SPLIT_VIEW_ID, SPLIT_VIEW_LABS, LABS_BADGE, labsVisible } from '$lib/components/studio/labs.svelte';

  export type SplitMode = 'single' | 'horizontal' | 'vertical';

  let { mode = $bindable<SplitMode>('single') } = $props<{ mode?: SplitMode }>();
  // Review r4: multi-pane split view is Labs-only (off by default). Without Labs the Studio always
  // has exactly one canvas; a second document opens as a tab.
  let effectiveMode = $derived<SplitMode>(labsVisible(SPLIT_VIEW_ID) ? mode : 'single');
  let splitRatio = $state(0.5);
  let containerEl: HTMLDivElement | null = $state(null);
  let dragging = $state(false);

  function onDividerDown(e: PointerEvent) {
    if (e.button !== 0) return;
    dragging = true;
    try {
      (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    } catch {
      // Ignored if unsupported
    }
  }

  function onDividerMove(e: PointerEvent) {
    if (!dragging || !containerEl) return;
    const rect = containerEl.getBoundingClientRect();
    if (mode === 'horizontal' && rect.width > 0) {
      const ratio = (e.clientX - rect.left) / rect.width;
      splitRatio = Math.min(0.85, Math.max(0.15, ratio));
    } else if (mode === 'vertical' && rect.height > 0) {
      const ratio = (e.clientY - rect.top) / rect.height;
      splitRatio = Math.min(0.85, Math.max(0.15, ratio));
    }
  }

  function onDividerUp(e: PointerEvent) {
    if (!dragging) return;
    dragging = false;
    try {
      (e.currentTarget as HTMLElement).releasePointerCapture(e.pointerId);
    } catch {
      // Ignored
    }
  }
</script>

<div class="studio-shell">
<DocumentTabs />
{#if labsVisible(SPLIT_VIEW_ID)}
  <div class="split-toggle">
    <Button
      variant="secondary"
      size="sm"
      data-testid="split-view-toggle"
      title="{SPLIT_VIEW_LABS.label}: {SPLIT_VIEW_LABS.note}"
      pressed={effectiveMode !== 'single'}
      onclick={() => (mode = mode === 'single' ? 'horizontal' : 'single')}
    >
      {SPLIT_VIEW_LABS.name} <span class="labs-badge">{LABS_BADGE}</span>
    </Button>
  </div>
{/if}
<div class="split-layout mode-{effectiveMode}" bind:this={containerEl}>
  {#if effectiveMode === 'single'}
    <div class="pane primary">
      <SpatialCanvas viewId="main-canvas" />
    </div>
  {:else if effectiveMode === 'horizontal'}
    <div class="pane left" style:flex="{splitRatio}">
      <SpatialCanvas viewId="main-canvas" />
    </div>
    <div
      class="split-handle vertical"
      role="separator"
      aria-label="Split divider"
      onpointerdown={onDividerDown}
      onpointermove={onDividerMove}
      onpointerup={onDividerUp}
      onpointercancel={onDividerUp}
    ></div>
    <div class="pane right" style:flex="{1 - splitRatio}">
      <SpatialCanvas viewId="pane-right" follower />
    </div>
  {:else if effectiveMode === 'vertical'}
    <div class="pane top" style:flex="{splitRatio}">
      <SpatialCanvas viewId="main-canvas" />
    </div>
    <div
      class="split-handle horizontal"
      role="separator"
      aria-label="Split divider"
      onpointerdown={onDividerDown}
      onpointermove={onDividerMove}
      onpointerup={onDividerUp}
      onpointercancel={onDividerUp}
    ></div>
    <div class="pane bottom" style:flex="{1 - splitRatio}">
      <SpatialCanvas viewId="pane-bottom" follower />
    </div>
  {/if}
</div>
</div>

<style>
  .studio-shell {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    overflow: hidden;
  }

  .split-toggle {
    align-self: flex-end;
    margin: var(--space-1) var(--space-2);
  }

  .labs-badge {
    font-size: var(--font-size-caption);
    font-weight: 700;
    text-transform: uppercase;
    padding: 0 var(--space-1);
    border-radius: var(--radius-pill);
    border: 1px solid var(--accent);
    color: var(--accent);
  }

  .split-layout {
    display: flex;
    width: 100%;
    flex: 1;
    min-height: 0;
    overflow: hidden;
  }

  .split-layout.mode-single {
    flex-direction: column;
  }

  .split-layout.mode-horizontal {
    flex-direction: row;
  }

  .split-layout.mode-vertical {
    flex-direction: column;
  }

  .pane {
    position: relative;
    flex: 1;
    overflow: hidden;
    min-width: 150px;
    min-height: 150px;
  }

  .split-handle {
    background: var(--glass-border);
    transition: background 0.15s ease;
    user-select: none;
    touch-action: none;
    z-index: 50;
  }

  .split-handle.vertical {
    width: 6px;
    cursor: col-resize;
  }

  .split-handle.horizontal {
    height: 6px;
    cursor: row-resize;
  }

  .split-handle:hover {
    background: var(--accent);
  }
</style>

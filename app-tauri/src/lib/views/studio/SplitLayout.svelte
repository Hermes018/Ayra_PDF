<script lang="ts">
  import SpatialCanvas from './SpatialCanvas.svelte';

  export type SplitMode = 'single' | 'horizontal' | 'vertical';

  let { mode = $bindable<SplitMode>('single') } = $props<{ mode?: SplitMode }>();
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

<div class="split-layout mode-{mode}" bind:this={containerEl}>
  {#if mode === 'single'}
    <div class="pane primary">
      <SpatialCanvas viewId="main-canvas" />
    </div>
  {:else if mode === 'horizontal'}
    <div class="pane left" style:flex="{splitRatio}">
      <SpatialCanvas viewId="pane-left" />
    </div>
    <div
      class="split-handle vertical"
      role="separator"
      aria-label="Split Divider"
      onpointerdown={onDividerDown}
      onpointermove={onDividerMove}
      onpointerup={onDividerUp}
      onpointercancel={onDividerUp}
    ></div>
    <div class="pane right" style:flex="{1 - splitRatio}">
      <SpatialCanvas viewId="pane-right" />
    </div>
  {:else if mode === 'vertical'}
    <div class="pane top" style:flex="{splitRatio}">
      <SpatialCanvas viewId="pane-top" />
    </div>
    <div
      class="split-handle horizontal"
      role="separator"
      aria-label="Split Divider"
      onpointerdown={onDividerDown}
      onpointermove={onDividerMove}
      onpointerup={onDividerUp}
      onpointercancel={onDividerUp}
    ></div>
    <div class="pane bottom" style:flex="{1 - splitRatio}">
      <SpatialCanvas viewId="pane-bottom" />
    </div>
  {/if}
</div>

<style>
  .split-layout {
    display: flex;
    width: 100%;
    height: 100%;
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

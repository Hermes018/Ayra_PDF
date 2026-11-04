<script lang="ts">
  import SpatialCanvas from './SpatialCanvas.svelte';

  export type SplitMode = 'single' | 'horizontal' | 'vertical';

  let { mode = 'single' } = $props<{ mode?: SplitMode }>();
  let splitRatio = $state(0.5);
</script>

<div class="split-layout mode-{mode}">
  {#if mode === 'single'}
    <div class="pane primary">
      <SpatialCanvas viewId="pane-primary" />
    </div>
  {:else if mode === 'horizontal'}
    <div class="pane left" style:flex="{splitRatio}">
      <SpatialCanvas viewId="pane-left" />
    </div>
    <div class="split-handle vertical" role="separator" aria-label="Split Divider"></div>
    <div class="pane right" style:flex="{1 - splitRatio}">
      <SpatialCanvas viewId="pane-right" />
    </div>
  {:else if mode === 'vertical'}
    <div class="pane top" style:flex="{splitRatio}">
      <SpatialCanvas viewId="pane-top" />
    </div>
    <div class="split-handle horizontal" role="separator" aria-label="Split Divider"></div>
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
    overflow: hidden;
    min-width: 150px;
    min-height: 150px;
  }

  .split-handle {
    background: var(--glass-border);
    transition: background 0.15s ease;
    user-select: none;
    z-index: 50;
  }

  .split-handle.vertical {
    width: 4px;
    cursor: col-resize;
  }

  .split-handle.horizontal {
    height: 4px;
    cursor: row-resize;
  }

  .split-handle:hover {
    background: var(--accent);
  }
</style>

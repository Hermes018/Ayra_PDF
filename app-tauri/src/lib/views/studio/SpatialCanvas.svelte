<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { viewport, ZOOM_MIN, ZOOM_MAX, type Pt } from '$lib/core/viewportEngine.svelte';
  import { tools, type ToolId } from '$lib/core/toolEngine.svelte';
  import { commands } from '$lib/core/commandEngine.svelte';
  import { theme } from '$lib/core/themeContract.svelte';
  import defaultPresets from '$lib/core/defaultPresets.json';
  import DockableToolbar from '$lib/components/studio/DockableToolbar.svelte';
  import PropertyPill from '$lib/components/studio/PropertyPill.svelte';
  import RadialMenu from '$lib/components/studio/RadialMenu.svelte';
  import ShortcutSettings from '$lib/components/studio/ShortcutSettings.svelte';

  let { viewId = 'main-canvas' } = $props<{ viewId?: string }>();

  let canvasContainer: HTMLDivElement | null = $state(null);
  let isPanning = $state(false);
  let panStart: Pt = { x: 0, y: 0 };
  let spaceHeld = $state(false);
  let editMode = $state<'markup' | 'structural'>('markup');

  // Radial Menu & Shortcuts modal state
  let radialVisible = $state(false);
  let radialPos = $state<Pt>({ x: 300, y: 300 });
  let shortcutsVisible = $state(false);

  function openRadialAt(x: number, y: number) {
    radialPos = { x, y };
    radialVisible = true;
  }

  // Active view state
  let currentView = $derived(viewport.views[viewId] ?? {
    id: viewId,
    zoom: 1.0,
    panX: 0,
    panY: 0,
    linkGroup: null,
    dpr: typeof window !== 'undefined' ? window.devicePixelRatio || 1 : 1
  });

  let worldTransform = $derived(
    viewport.worldToScreen(viewId).toDOMMatrix().toString()
  );

  onMount(() => {
    // Register view
    viewport.upsertView({
      id: viewId,
      zoom: 1.0,
      panX: 40,
      panY: 40,
      linkGroup: null,
      dpr: window.devicePixelRatio || 1
    });

    // Initialize tool presets if empty
    if (tools.presets.length === 0) {
      tools.load(defaultPresets as any, ['preset.pen.a', 'preset.highlighter', 'preset.eraser.partial']);
    }

    // Default sample page if no pages loaded yet
    if (viewport.pages.length === 0) {
      viewport.layoutPages([
        { widthPt: 595.28, heightPt: 841.89 }, // A4 page 1
        { widthPt: 595.28, heightPt: 841.89 }  // A4 page 2
      ]);
    }

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.code === 'Space' && !spaceHeld && !(e.target as HTMLElement)?.matches('input, textarea')) {
        spaceHeld = true;
        tools.springLoad('preset.pan');
      }
    };

    const handleKeyUp = (e: KeyboardEvent) => {
      if (e.code === 'Space') {
        spaceHeld = false;
        tools.springLoad(null);
      }
    };

    commands.onRadialToggle = () => {
      if (typeof window !== 'undefined') {
        openRadialAt(window.innerWidth / 2, window.innerHeight / 2);
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    window.addEventListener('keyup', handleKeyUp);

    return () => {
      commands.onRadialToggle = null;
      window.removeEventListener('keydown', handleKeyDown);
      window.removeEventListener('keyup', handleKeyUp);
      viewport.closeView(viewId);
    };
  });

  function handleContextMenu(e: MouseEvent) {
    e.preventDefault();
    openRadialAt(e.clientX, e.clientY);
  }

  function handleWheel(e: WheelEvent) {
    e.preventDefault();
    if (!canvasContainer) return;

    const rect = canvasContainer.getBoundingClientRect();
    const anchor: Pt = {
      x: e.clientX - rect.left,
      y: e.clientY - rect.top
    };

    if (e.ctrlKey || e.metaKey) {
      // Zoom
      const factor = e.deltaY < 0 ? 1.15 : 0.85;
      viewport.zoomAt(viewId, factor, anchor);
    } else {
      // Pan
      viewport.panBy(viewId, -e.deltaX, -e.deltaY);
    }
  }

  function handlePointerDown(e: PointerEvent) {
    if (!canvasContainer) return;

    const isPanAction = e.button === 1 || spaceHeld || tools.activeTool === 'pan';

    if (isPanAction) {
      isPanning = true;
      panStart = { x: e.clientX, y: e.clientY };
      canvasContainer.setPointerCapture(e.pointerId);
      return;
    }

    // Dispatch pointer to control bus
    const rect = canvasContainer.getBoundingClientRect();
    commands.dispatch({
      kind: 'pointer',
      phase: 'down',
      x: e.clientX - rect.left,
      y: e.clientY - rect.top,
      pressure: e.pressure || 0.5,
      tiltX: e.tiltX || 0,
      tiltY: e.tiltY || 0,
      twist: e.twist || 0,
      pointerType: (e.pointerType === 'pen' || e.pointerType === 'touch') ? e.pointerType : 'mouse',
      buttons: e.buttons,
      viewId,
      tMs: performance.now()
    });
  }

  function handlePointerMove(e: PointerEvent) {
    if (isPanning) {
      const dx = e.clientX - panStart.x;
      const dy = e.clientY - panStart.y;
      panStart = { x: e.clientX, y: e.clientY };
      viewport.panBy(viewId, dx, dy);
      return;
    }

    if (!canvasContainer) return;
    const rect = canvasContainer.getBoundingClientRect();

    commands.dispatch({
      kind: 'pointer',
      phase: e.buttons > 0 ? 'move' : 'hover',
      x: e.clientX - rect.left,
      y: e.clientY - rect.top,
      pressure: e.pressure || 0.5,
      tiltX: e.tiltX || 0,
      tiltY: e.tiltY || 0,
      twist: e.twist || 0,
      pointerType: (e.pointerType === 'pen' || e.pointerType === 'touch') ? e.pointerType : 'mouse',
      buttons: e.buttons,
      viewId,
      tMs: performance.now()
    });
  }

  function handlePointerUp(e: PointerEvent) {
    if (isPanning) {
      isPanning = false;
      if (canvasContainer && canvasContainer.hasPointerCapture(e.pointerId)) {
        canvasContainer.releasePointerCapture(e.pointerId);
      }
      return;
    }

    if (!canvasContainer) return;
    const rect = canvasContainer.getBoundingClientRect();

    commands.dispatch({
      kind: 'pointer',
      phase: 'up',
      x: e.clientX - rect.left,
      y: e.clientY - rect.top,
      pressure: 0,
      tiltX: e.tiltX || 0,
      tiltY: e.tiltY || 0,
      twist: 0,
      pointerType: (e.pointerType === 'pen' || e.pointerType === 'touch') ? e.pointerType : 'mouse',
      buttons: e.buttons,
      viewId,
      tMs: performance.now()
    });
  }
</script>

<div
  class="spatial-viewport"
  bind:this={canvasContainer}
  onwheel={handleWheel}
  onpointerdown={handlePointerDown}
  onpointermove={handlePointerMove}
  onpointerup={handlePointerUp}
  onpointercancel={handlePointerUp}
  oncontextmenu={handleContextMenu}
  role="region"
  aria-label="Drawboard Spatial Studio Canvas"
  tabindex="-1"
>
  <!-- HUD / Spatial Controls Pill -->
  <div class="spatial-hud glass-panel">
    <div class="hud-mode-toggle">
      <button
        type="button"
        class="hud-btn"
        class:active={editMode === 'markup'}
        onclick={() => { editMode = 'markup'; }}
        title="Markup Mode (Fast 60-120fps Annotations & Measurements)"
      >
        ✏️ Markup
      </button>
      <button
        type="button"
        class="hud-btn"
        class:active={editMode === 'structural'}
        onclick={() => { editMode = 'structural'; }}
        title="Structural PDF Edit Mode (True Text, Font & Object Manipulation)"
      >
        📝 Structural Edit
      </button>
    </div>

    <div class="hud-divider"></div>

    <div class="hud-zoom">
      <button
        type="button"
        class="hud-icon-btn"
        onclick={() => viewport.zoomAt(viewId, 0.8, { x: 300, y: 300 })}
        title="Zoom Out"
      >
        −
      </button>
      <span class="zoom-readout">{Math.round(currentView.zoom * 100)}%</span>
      <button
        type="button"
        class="hud-icon-btn"
        onclick={() => viewport.zoomAt(viewId, 1.25, { x: 300, y: 300 })}
        title="Zoom In"
      >
        +
      </button>
      <button
        type="button"
        class="hud-icon-btn"
        onclick={() => viewport.upsertView({ ...currentView, zoom: 1.0, panX: 40, panY: 40 })}
        title="Reset 100%"
      >
        ⟲
      </button>
    </div>

    <div class="hud-divider"></div>

    <div class="hud-tool-indicator">
      <span class="hud-tool-label">Tool:</span>
      <span class="hud-tool-name">{tools.active?.label ?? tools.activeTool}</span>
    </div>

    <div class="hud-undo-redo">
      <button
        type="button"
        class="hud-icon-btn"
        disabled={!commands.canUndo}
        onclick={() => commands.undo()}
        title="Undo ({commands.undoLabel ?? 'None'})"
      >
        ↺
      </button>
      <button
        type="button"
        class="hud-icon-btn"
        disabled={!commands.canRedo}
        onclick={() => commands.redo()}
        title="Redo"
      >
        ↻
      </button>
    </div>
  </div>

  <!-- Hardware-Accelerated World Transform Layer -->
  <div class="world" style:transform={worldTransform}>
    {#each viewport.pages as page, index}
      <div
        class="spatial-page"
        style:left="{page.worldX}px"
        style:top="{page.worldY}px"
        style:width="{page.widthPt * (96 / 72)}px"
        style:height="{page.heightPt * (96 / 72)}px"
      >
        <!-- Page Header / Number Badge -->
        <div class="page-badge">
          Page {index + 1} ({Math.round(page.widthPt)} × {Math.round(page.heightPt)} pt)
        </div>

        <!-- Render Layer (PDF Raster Tile Surface) -->
        <div class="page-render-surface">
          <div class="page-grid-watermark">
            <span class="grid-subtext">Spatial Studio PDF Canvas • Page {index + 1}</span>
          </div>
        </div>

        <!-- Mode A: Non-Destructive Vector Annotation Overlay -->
        <canvas
          class="page-overlay-canvas"
          width="{page.widthPt * (96 / 72)}"
          height="{page.heightPt * (96 / 72)}"
        ></canvas>

        <!-- Mode B / Live Ink: Desynchronized Wet Canvas for Zero-Latency Stylus Strokes -->
        <canvas
          class="page-wet-canvas"
          width="{page.widthPt * (96 / 72)}"
          height="{page.heightPt * (96 / 72)}"
        ></canvas>
      </div>
    {/each}
  </div>

  <!-- Dockable Markup Toolbar (Top, Left, Right, Floating) -->
  <DockableToolbar
    onOpenRadial={() => {
      if (typeof window !== 'undefined') openRadialAt(window.innerWidth / 2, window.innerHeight / 2);
    }}
    onOpenShortcuts={() => {
      shortcutsVisible = true;
    }}
  />

  <!-- Active Tool Reactive Property Pill -->
  <PropertyPill />

  <!-- 8-Sector Radial Menu (Right-click / Pen Barrel / UI Trigger / '/') -->
  <RadialMenu
    bind:visible={radialVisible}
    x={radialPos.x}
    y={radialPos.y}
  />

  <!-- Custom Keyboard Shortcuts Settings Modal -->
  <ShortcutSettings
    bind:visible={shortcutsVisible}
  />
</div>

<style>
  .spatial-viewport {
    position: relative;
    width: 100%;
    height: 100%;
    min-height: calc(100vh - 120px);
    overflow: hidden;
    background-color: var(--canvas-bg);
    background-image: 
      radial-gradient(circle, var(--canvas-grid-minor) 1px, transparent 1px),
      radial-gradient(circle, var(--canvas-grid-major) 1.5px, transparent 1.5px);
    background-size: 16px 16px, 96px 96px;
    background-position: 0 0, 0 0;
    user-select: none;
    touch-action: none;
    cursor: default;
    outline: none;
  }

  .world {
    position: absolute;
    top: 0;
    left: 0;
    transform-origin: 0 0;
    will-change: transform;
    pointer-events: none;
  }

  .spatial-page {
    position: absolute;
    background-color: #ffffff;
    box-shadow: 0 10px 30px var(--canvas-page-shadow);
    border-radius: 4px;
    overflow: hidden;
    pointer-events: auto;
  }

  .page-badge {
    position: absolute;
    top: -24px;
    left: 0;
    font-size: 11px;
    font-weight: 600;
    color: var(--glass-text);
    opacity: 0.7;
    white-space: nowrap;
  }

  .page-render-surface {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: #ffffff;
  }

  .page-grid-watermark {
    opacity: 0.15;
    text-align: center;
    font-weight: 500;
    color: #1d1d1f;
  }

  .page-overlay-canvas,
  .page-wet-canvas {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    pointer-events: none;
  }

  .page-wet-canvas {
    z-index: 10;
  }

  /* Frosted Glass HUD Controls */
  .spatial-hud {
    position: absolute;
    top: 16px;
    left: 50%;
    transform: translateX(-50%);
    z-index: 100;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 14px;
    background: var(--glass-bg);
    backdrop-filter: blur(var(--glass-blur));
    -webkit-backdrop-filter: blur(var(--glass-blur));
    border: 1px solid var(--glass-border);
    box-shadow: var(--glass-shadow);
    border-radius: 9999px;
    color: var(--glass-text);
    pointer-events: auto;
  }

  .hud-mode-toggle {
    display: flex;
    background: rgba(0, 0, 0, 0.05);
    border-radius: 9999px;
    padding: 2px;
  }

  .hud-btn {
    border: none;
    background: transparent;
    color: var(--glass-text);
    padding: 4px 10px;
    border-radius: 9999px;
    font-size: 12px;
    font-weight: 500;
    cursor: pointer;
    transition: background var(--motion-duration) ease, color var(--motion-duration) ease;
  }

  .hud-btn.active {
    background: var(--accent);
    color: #ffffff;
  }

  .hud-divider {
    width: 1px;
    height: 18px;
    background: var(--glass-border);
  }

  .hud-zoom {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .zoom-readout {
    font-size: 12px;
    font-weight: 600;
    min-width: 42px;
    text-align: center;
    font-variant-numeric: tabular-nums;
  }

  .hud-icon-btn {
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    border: none;
    background: transparent;
    color: var(--glass-text);
    border-radius: 4px;
    cursor: pointer;
    font-size: 14px;
    font-weight: 600;
    transition: background 0.15s ease;
  }

  .hud-icon-btn:hover:not(:disabled) {
    background: rgba(0, 0, 0, 0.08);
  }

  .hud-icon-btn:disabled {
    opacity: 0.35;
    cursor: not-allowed;
  }

  .hud-tool-indicator {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
  }

  .hud-tool-label {
    opacity: 0.6;
  }

  .hud-tool-name {
    font-weight: 600;
  }

  .hud-undo-redo {
    display: flex;
    gap: 2px;
  }
</style>

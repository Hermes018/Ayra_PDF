<script lang="ts">
  import { onMount, onDestroy, tick } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { appState } from '$lib/state.svelte';
  import { viewport, type Pt } from '$lib/core/viewportEngine.svelte';
  import { tools } from '$lib/core/toolEngine.svelte';
  import { commands } from '$lib/core/commandEngine.svelte';
  import { theme } from '$lib/core/themeContract.svelte';
  import { ink } from '$lib/core/inkEngine.svelte';
  import defaultPresets from '$lib/core/defaultPresets.json';
  import DockableToolbar from '$lib/components/studio/DockableToolbar.svelte';
  import PropertyPill from '$lib/components/studio/PropertyPill.svelte';
  import RadialMenu from '$lib/components/studio/RadialMenu.svelte';
  import ShortcutSettings from '$lib/components/studio/ShortcutSettings.svelte';
  import FavoritesBar from '$lib/components/studio/FavoritesBar.svelte';
  import StudioIcon from '$lib/components/studio/StudioIcon.svelte';

  let { viewId = 'main-canvas' } = $props<{ viewId?: string }>();

  let canvasContainer: HTMLDivElement | null = $state(null);
  let isPanning = $state(false);
  let panStart: Pt = { x: 0, y: 0 };
  let editMode = $state<'markup' | 'structural'>('markup');

  // Radial Menu & Shortcuts modal state
  let radialVisible = $state(false);
  let radialPos = $state<Pt>({ x: 300, y: 300 });
  let shortcutsVisible = $state(false);

  // PDF rendering state
  let pageCanvases: Record<number, HTMLCanvasElement | null> = $state({});
  let currentPdfDoc: any = null;
  let loadedFilePath = $state<string | null>(null);

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

  async function loadPdfIntoCanvas(path: string) {
    if (!path || typeof window === 'undefined') return;
    try {
      const pdfjs = (window as any)['pdfjs-dist/build/pdf'] || (window as any).pdfjsLib;
      if (!pdfjs) return;
      pdfjs.GlobalWorkerOptions.workerSrc = '/pdf.worker.min.js';

      const bytes: number[] = await invoke('read_file_bytes', { path });
      const uint8Array = new Uint8Array(bytes);

      if (currentPdfDoc) {
        await currentPdfDoc.destroy();
      }
      const loadingTask = pdfjs.getDocument({ data: uint8Array });
      const pdf = await loadingTask.promise;
      currentPdfDoc = pdf;
      loadedFilePath = path;

      const pageSizes: Array<{ widthPt: number; heightPt: number }> = [];
      for (let i = 1; i <= pdf.numPages; i++) {
        const page = await pdf.getPage(i);
        const vp = page.getViewport({ scale: 1.0 });
        pageSizes.push({ widthPt: vp.width, heightPt: vp.height });
      }
      viewport.layoutPages(pageSizes);
      await tick();

      // Render pages at 96/72 CSS pixel scale * devicePixelRatio
      const baseScale = (96 / 72) * (window.devicePixelRatio || 1);
      for (let i = 1; i <= pdf.numPages; i++) {
        const page = await pdf.getPage(i);
        const canvasEl = pageCanvases[i - 1];
        if (!canvasEl) continue;
        const vp = page.getViewport({ scale: baseScale });
        canvasEl.width = vp.width;
        canvasEl.height = vp.height;
        const ctx = canvasEl.getContext('2d');
        if (ctx) {
          await page.render({ canvasContext: ctx, viewport: vp }).promise;
        }
      }
    } catch (err) {
      console.error('Failed to render PDF in SpatialCanvas:', err);
    }
  }

  $effect(() => {
    const file = appState.currentFile;
    if (file && file !== loadedFilePath) {
      loadPdfIntoCanvas(file);
    }
  });

  async function loadStudioPrefs() {
    try {
      const prefs = await invoke<{
        dockEdge: string;
        floatX: number;
        floatY: number;
        shortcuts: Record<string, string>;
        favorites: string[];
        historyLimit: number;
      }>('prefs_load');
      if (prefs) {
        if (prefs.shortcuts && Object.keys(prefs.shortcuts).length > 0) {
          commands.shortcuts = { ...commands.shortcuts, ...prefs.shortcuts };
        }
        if (Array.isArray(prefs.favorites) && prefs.favorites.length > 0) {
          tools.favorites = [...prefs.favorites];
        }
        if (typeof prefs.historyLimit === 'number' && prefs.historyLimit > 0) {
          commands.historyLimit = prefs.historyLimit;
        }
      }
    } catch {
      // Safe fallback in browser/unit-test environments without Tauri IPC
    }
  }

  async function persistStudioPrefs() {
    try {
      await invoke('prefs_save', {
        prefs: {
          dockEdge: 'left',
          floatX: 16,
          floatY: 80,
          shortcuts: commands.shortcuts,
          favorites: tools.favorites,
          historyLimit: commands.historyLimit
        }
      });
    } catch {
      // Safe fallback in browser/unit-test environments
    }
  }

  onMount(() => {
    viewport.upsertView({
      id: viewId,
      zoom: 1.0,
      panX: 40,
      panY: 40,
      linkGroup: null,
      dpr: window.devicePixelRatio || 1
    });

    if (Object.keys(tools.presets).length === 0) {
      tools.load(defaultPresets as any, ['preset.pen.a', 'preset.highlighter', 'preset.eraser.partial']);
    }

    if (viewport.pages.length === 0) {
      viewport.layoutPages([
        { widthPt: 595.28, heightPt: 841.89 },
        { widthPt: 595.28, heightPt: 841.89 }
      ]);
    }

    commands.onRadialToggle = () => {
      if (typeof window !== 'undefined') {
        openRadialAt(window.innerWidth / 2, window.innerHeight / 2);
      }
    };

    loadStudioPrefs();

    return () => {
      commands.onRadialToggle = null;
      viewport.closeView(viewId);
    };
  });

  onDestroy(() => {
    if (currentPdfDoc) {
      currentPdfDoc.destroy();
    }
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
      const factor = e.deltaY < 0 ? 1.15 : 0.85;
      viewport.zoomAt(viewId, factor, anchor);
    } else {
      viewport.panBy(viewId, -e.deltaX, -e.deltaY);
    }
  }

  function getWorldPoint(clientX: number, clientY: number): Pt {
    if (!canvasContainer) return { x: clientX, y: clientY };
    const rect = canvasContainer.getBoundingClientRect();
    const screenX = clientX - rect.left;
    const screenY = clientY - rect.top;
    const inv = viewport.worldToScreen(viewId).invert();
    return inv ? inv.apply({ x: screenX, y: screenY }) : { x: screenX, y: screenY };
  }

  function handlePointerDown(e: PointerEvent) {
    if (!canvasContainer) return;

    // Failsafe: Never intercept clicks on UI overlays
    const target = e.target as HTMLElement | null;
    if (target?.closest('.spatial-hud, .dockable-toolbar, .property-pill, .favorites-bar, .modal-backdrop, .radial-menu, button, select, input')) {
      return;
    }

    const isPanAction = e.button === 1 || commands.spaceSpringActive || tools.activeTool === 'pan';

    if (isPanAction) {
      isPanning = true;
      panStart = { x: e.clientX, y: e.clientY };
      try {
        (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
      } catch {}
      return;
    }

    try {
      (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    } catch {}

    const worldPt = getWorldPoint(e.clientX, e.clientY);
    const pressure = e.pointerType === 'pen' ? (e.pressure || 0.5) : 0.5;

    if (tools.activeToolId === 'laser') {
      ink.addLaserPoint(worldPt.x, worldPt.y);
      return;
    }

    if (tools.activeToolId === 'eraser.whole') {
      ink.eraseWhole(worldPt, 18);
      return;
    }

    if (tools.activeToolId === 'eraser.partial') {
      ink.erasePartial(worldPt, 12);
      return;
    }

    // Drawing / Inking
    ink.startStroke(worldPt.x, worldPt.y, pressure, tools.currentToolId);

    const rect = canvasContainer.getBoundingClientRect();
    commands.dispatch({
      kind: 'pointer',
      phase: 'down',
      x: e.clientX - rect.left,
      y: e.clientY - rect.top,
      pressure,
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

    // Process all hardware coalesced 120Hz/240Hz samples
    const events = typeof e.getCoalescedEvents === 'function' ? e.getCoalescedEvents() : [e];
    for (const ev of events) {
      const worldPt = getWorldPoint(ev.clientX, ev.clientY);
      const pressure = ev.pointerType === 'pen' ? (ev.pressure || 0.5) : 0.5;

      if (tools.activeToolId === 'laser') {
        ink.addLaserPoint(worldPt.x, worldPt.y);
      } else if (tools.activeToolId === 'eraser.whole') {
        if (ev.buttons > 0) ink.eraseWhole(worldPt, 18);
      } else if (tools.activeToolId === 'eraser.partial') {
        if (ev.buttons > 0) ink.erasePartial(worldPt, 12);
      } else if (ink.isDrawing) {
        ink.addPoint(worldPt.x, worldPt.y, pressure);
      }
    }

    const rect = canvasContainer.getBoundingClientRect();
    commands.dispatch({
      kind: 'pointer',
      phase: e.buttons > 0 ? 'move' : 'hover',
      x: e.clientX - rect.left,
      y: e.clientY - rect.top,
      pressure: e.pointerType === 'pen' ? (e.pressure || 0.5) : 0.5,
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
      try {
        if ((e.currentTarget as HTMLElement).hasPointerCapture(e.pointerId)) {
          (e.currentTarget as HTMLElement).releasePointerCapture(e.pointerId);
        }
      } catch {}
      return;
    }

    if (ink.isDrawing) {
      const committed = ink.endStroke();
      if (committed) {
        commands.dispatch({
          kind: 'execute',
          command: 'ink.commit',
          args: { id: committed.id, tool: committed.tool, pointCount: committed.points.length }
        });
      }
    }

    try {
      if ((e.currentTarget as HTMLElement).hasPointerCapture(e.pointerId)) {
        (e.currentTarget as HTMLElement).releasePointerCapture(e.pointerId);
      }
    } catch {}

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

  function handlePointerCancel(e: PointerEvent) {
    if (isPanning) {
      isPanning = false;
    }
    if (ink.isDrawing) {
      ink.cancelStroke();
    }
    try {
      if ((e.currentTarget as HTMLElement).hasPointerCapture(e.pointerId)) {
        (e.currentTarget as HTMLElement).releasePointerCapture(e.pointerId);
      }
    } catch {}
  }
</script>

<div
  class="spatial-viewport"
  bind:this={canvasContainer}
  onwheel={handleWheel}
  role="region"
  aria-label="Drawboard Spatial Studio Canvas"
  tabindex="-1"
>
  <!-- HUD / Spatial Controls Pill -->
  <div
    class="spatial-hud glass-panel"
    role="toolbar"
    tabindex={0}
    aria-label="Spatial Canvas HUD"
    onpointerdown={(e) => e.stopPropagation()}
  >
    <div class="hud-mode-toggle">
      <button
        type="button"
        class="hud-btn"
        class:active={editMode === 'markup'}
        onclick={() => { editMode = 'markup'; }}
        title="Markup Mode (Fast 60-120fps Annotations & Measurements)"
      >
        <StudioIcon name="pen.a" size={13} />
        <span>Markup</span>
      </button>
      <button
        type="button"
        class="hud-btn"
        class:active={editMode === 'structural'}
        onclick={() => { editMode = 'structural'; }}
        title="Structural PDF Edit Mode (True Text, Font & Object Manipulation)"
      >
        <StudioIcon name="text.review" size={13} />
        <span>Structural Edit</span>
      </button>
    </div>

    <div class="hud-divider"></div>

    <div class="hud-zoom">
      <button
        type="button"
        class="hud-icon-btn"
        onclick={() => viewport.zoomAt(viewId, 0.8, { x: 300, y: 300 })}
        title="Zoom Out"
        aria-label="Zoom Out"
      >
        <StudioIcon name="zoom.out" size={14} />
      </button>
      <span class="zoom-readout">{Math.round(currentView.zoom * 100)}%</span>
      <button
        type="button"
        class="hud-icon-btn"
        onclick={() => viewport.zoomAt(viewId, 1.25, { x: 300, y: 300 })}
        title="Zoom In"
        aria-label="Zoom In"
      >
        <StudioIcon name="zoom.in" size={14} />
      </button>
      <button
        type="button"
        class="hud-icon-btn"
        onclick={() => viewport.upsertView({ ...currentView, zoom: 1.0, panX: 40, panY: 40 })}
        title="Reset 100%"
        aria-label="Reset Zoom"
      >
        <StudioIcon name="reset" size={14} />
      </button>
    </div>

    <div class="hud-divider"></div>

    <div class="hud-tool-indicator">
      <span class="hud-tool-label">Tool:</span>
      <span class="hud-tool-name">{tools.activeToolLabel}</span>
    </div>

    <div class="hud-undo-redo">
      <button
        type="button"
        class="hud-icon-btn"
        disabled={!commands.canUndo}
        onclick={() => commands.undo()}
        title="Undo ({commands.undoLabel ?? 'None'})"
        aria-label="Undo"
      >
        <StudioIcon name="undo" size={14} />
      </button>
      <button
        type="button"
        class="hud-icon-btn"
        disabled={!commands.canRedo}
        onclick={() => commands.redo()}
        title="Redo"
        aria-label="Redo"
      >
        <StudioIcon name="redo" size={14} />
      </button>
    </div>
  </div>

  <!-- LAYER A: #pdf-layer (Hardware-accelerated PDF.js raster surface) -->
  <div id="pdf-layer" class="pdf-layer world" style:transform={worldTransform}>
    {#each viewport.pages as page, index}
      <div
        class="spatial-page"
        style:left="{page.worldX}px"
        style:top="{page.worldY}px"
        style:width="{page.widthPt * (96 / 72)}px"
        style:height="{page.heightPt * (96 / 72)}px"
      >
        <div class="page-badge">
          Page {index + 1} ({Math.round(page.widthPt)} × {Math.round(page.heightPt)} pt)
        </div>

        <div class="page-render-surface">
          <canvas
            class="page-pdf-canvas"
            bind:this={pageCanvases[index]}
          ></canvas>
        </div>
      </div>
    {/each}
  </div>

  <!-- LAYER B: #vector-layer (Rendered completed ink strokes via SVG paths) -->
  <svg id="vector-layer" class="vector-layer" role="presentation">
    <g style:transform={worldTransform}>
      {#each ink.completedStrokes as stroke (stroke.id)}
        <path
          d={ink.getSvgPath(stroke)}
          fill={stroke.color}
          opacity={stroke.opacity}
          style={stroke.isHighlighter || stroke.tool === 'highlighter' ? 'mix-blend-mode: multiply;' : ''}
        />
      {/each}
    </g>
  </svg>

  <!-- LAYER C: #wet-layer (Active stroke and laser pointer decay for zero-latency feedback) -->
  <svg
    id="wet-layer"
    class="wet-layer"
    role="presentation"
    class:cursor-pan={tools.activeToolId === 'pan'}
    class:cursor-pen={tools.activeToolId.startsWith('pen') || tools.activeToolId === 'highlighter'}
    class:cursor-eraser={tools.activeToolId.startsWith('eraser')}
    class:cursor-laser={tools.activeToolId === 'laser'}
    onpointerdown={handlePointerDown}
    onpointermove={handlePointerMove}
    onpointerup={handlePointerUp}
    onpointercancel={handlePointerCancel}
    oncontextmenu={handleContextMenu}
  >
    <g style:transform={worldTransform}>
      {#if ink.currentStroke}
        <path
          d={ink.getSvgPath(ink.currentStroke)}
          fill={ink.currentStroke.color}
          opacity={ink.currentStroke.opacity}
          style={ink.currentStroke.isHighlighter || ink.currentStroke.tool === 'highlighter' ? 'mix-blend-mode: multiply;' : ''}
        />
      {/if}
      {#if ink.laserPoints.length > 1}
        <path
          d={ink.getLaserSvgPath()}
          fill="var(--laser-glow)"
          opacity="0.85"
        />
        <path
          d={ink.getLaserCoreSvgPath()}
          fill="var(--laser-core)"
          opacity="1.0"
        />
      {/if}
    </g>
  </svg>

  <!-- Dockable Markup Toolbar (Top, Left, Right, Floating) -->
  <DockableToolbar
    onOpenRadial={() => {
      if (typeof window !== 'undefined') openRadialAt(window.innerWidth / 2, window.innerHeight / 2);
    }}
    onOpenShortcuts={() => {
      shortcutsVisible = true;
    }}
  />

  <!-- Pinned Favorites Bar -->
  <FavoritesBar onPersistPrefs={persistStudioPrefs} />

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
    onClose={persistStudioPrefs}
  />
</div>

<style>
  .spatial-viewport {
    position: relative;
    width: 100%;
    height: 100%;
    min-height: calc(100vh - 28px);
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

  .pdf-layer {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    transform-origin: 0 0;
    will-change: transform;
    pointer-events: none;
    z-index: 1;
  }

  .vector-layer {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    pointer-events: none;
    overflow: visible;
    z-index: 5;
  }

  .wet-layer {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    pointer-events: auto;
    touch-action: none;
    overflow: visible;
    z-index: 10;
  }

  .vector-layer g,
  .wet-layer g {
    transform-origin: 0 0;
    will-change: transform;
  }

  .cursor-pan {
    cursor: grab;
  }
  .cursor-pan:active {
    cursor: grabbing;
  }
  .cursor-pen {
    cursor: crosshair;
  }
  .cursor-eraser {
    cursor: cell;
  }
  .cursor-laser {
    cursor: crosshair;
  }

  .spatial-page {
    position: absolute;
    background-color: var(--laser-core);
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
    background: var(--laser-core);
  }

  .page-pdf-canvas {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    pointer-events: none;
  }

  /* Frosted Glass HUD Controls */
  .spatial-hud {
    position: absolute;
    top: 16px;
    left: 50%;
    transform: translateX(-50%);
    z-index: 160;
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
    background: var(--canvas-selection);
    border-radius: 9999px;
    padding: 2px;
  }

  .hud-btn {
    display: flex;
    align-items: center;
    gap: 4px;
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
    color: var(--laser-core);
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
    background: var(--canvas-selection);
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

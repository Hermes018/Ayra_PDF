<script module lang="ts">
  export type DockEdge = 'left' | 'right' | 'top' | 'floating';

  export function resolveDock(
    x: number,
    y: number,
    w: number,
    _h: number,
    pw: number,
    _ph: number,
    threshold = 40
  ): 'left' | 'right' | 'top' | null {
    if (y <= threshold) return 'top';
    if (x <= threshold) return 'left';
    if (pw - (x + w) <= threshold) return 'right';
    return null;
  }
</script>

<script lang="ts">
  import { tools } from '$lib/core/toolEngine.svelte';
  import StudioIcon from './StudioIcon.svelte';

  let {
    onOpenRadial,
    onOpenShortcuts
  }: {
    onOpenRadial?: () => void;
    onOpenShortcuts?: () => void;
  } = $props();

  let dockEdge = $state<DockEdge>('left');
  let floatPos = $state<{ x: number; y: number }>({ x: 16, y: 80 });
  let dragPos = $state<{ x: number; y: number }>({ x: 16, y: 80 });
  let snapPreview = $state<'left' | 'right' | 'top' | null>(null);
  let isDragging = $state(false);
  let grabOffset = { x: 0, y: 0 };
  let toolbarEl: HTMLElement | null = $state(null);

  function handleGripDown(e: PointerEvent) {
    if (e.button !== 0 || !toolbarEl) return;
    try {
      (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    } catch {
      // Ignored if capture unsupported
    }

    const parent = toolbarEl.offsetParent as HTMLElement | null;
    const parentRect = parent
      ? parent.getBoundingClientRect()
      : { left: 0, top: 0, width: window.innerWidth, height: window.innerHeight };
    const selfRect = toolbarEl.getBoundingClientRect();

    grabOffset = {
      x: e.clientX - selfRect.left,
      y: e.clientY - selfRect.top
    };

    if (dockEdge !== 'floating') {
      floatPos = {
        x: selfRect.left - parentRect.left,
        y: selfRect.top - parentRect.top
      };
      dragPos = { ...floatPos };
    } else {
      dragPos = { ...floatPos };
    }

    isDragging = true;
    snapPreview = null;
  }

  function handleGripMove(e: PointerEvent) {
    if (!isDragging || !toolbarEl) return;
    const parent = toolbarEl.offsetParent as HTMLElement | null;
    const parentRect = parent
      ? parent.getBoundingClientRect()
      : { left: 0, top: 0, width: window.innerWidth, height: window.innerHeight };
    const selfRect = toolbarEl.getBoundingClientRect();

    const parentW = parentRect.width;
    const parentH = parentRect.height;
    const selfW = selfRect.width;
    const selfH = selfRect.height;

    const minX = 8;
    const maxX = Math.max(minX, parentW - selfW - 8);
    const minY = 8;
    const maxY = Math.max(minY, parentH - selfH - 8);

    const rawX = e.clientX - parentRect.left - grabOffset.x;
    const rawY = e.clientY - parentRect.top - grabOffset.y;

    const nextX = Math.min(Math.max(minX, rawX), maxX);
    const nextY = Math.min(Math.max(minY, rawY), maxY);

    dragPos = { x: nextX, y: nextY };
    snapPreview = resolveDock(dragPos.x, dragPos.y, selfW, selfH, parentW, parentH, 40);
  }

  function handleGripUp(e: PointerEvent) {
    if (!isDragging) return;
    try {
      (e.currentTarget as HTMLElement)?.releasePointerCapture(e.pointerId);
    } catch {
      // Ignored
    }

    if (snapPreview !== null) {
      dockEdge = snapPreview;
      snapPreview = null;
    } else {
      dockEdge = 'floating';
      floatPos = { ...dragPos };
    }
    isDragging = false;
  }
</script>

<aside
  bind:this={toolbarEl}
  class="dockable-toolbar glass-panel dock-{dockEdge}"
  class:is-dragging={isDragging}
  class:snap-left={snapPreview === 'left'}
  class:snap-right={snapPreview === 'right'}
  class:snap-top={snapPreview === 'top'}
  style={isDragging || dockEdge === 'floating'
    ? `left: 0; top: 0; transform: translate3d(${isDragging ? dragPos.x : floatPos.x}px, ${isDragging ? dragPos.y : floatPos.y}px, 0); transition: none;`
    : undefined}
  aria-label="Drawboard Studio Markup Toolbar"
>
  <!-- Drag Handle -->
  <div
    class="toolbar-drag-handle"
    onpointerdown={handleGripDown}
    onpointermove={handleGripMove}
    onpointerup={handleGripUp}
    onpointercancel={handleGripUp}
    title="Drag to dock (Top, Left, Right) or float"
    role="button"
    tabindex="0"
    aria-label="Drag handle to position toolbar"
  >
    <StudioIcon name="grip" size={14} />
  </div>

  <div
    class="toolbar-tools"
    role="toolbar"
    tabindex={0}
    aria-label="Drawing Tools"
    onpointerdown={(e) => e.stopPropagation()}
  >
    <!-- Pan / Hand -->
    <button
      type="button"
      class="tool-btn"
      class:active={tools.activeToolId === 'pan'}
      aria-pressed={tools.activeToolId === 'pan'}
      onclick={() => tools.selectTool('pan')}
      title="Pan Canvas (H / Space)"
      aria-label="Pan Tool"
    >
      <StudioIcon name="pan" size={16} />
    </button>

    <!-- Selection / Marquee -->
    <button
      type="button"
      class="tool-btn"
      class:active={tools.activeToolId === 'select'}
      aria-pressed={tools.activeToolId === 'select'}
      onclick={() => tools.selectTool('select')}
      title="Select Marquee (V)"
      aria-label="Select Tool"
    >
      <StudioIcon name="select" size={16} />
    </button>

    <div class="toolbar-separator"></div>

    <!-- Fine Black Pen (Preset A) -->
    <button
      type="button"
      class="tool-btn"
      class:active={tools.activeToolId === 'pen.a'}
      aria-pressed={tools.activeToolId === 'pen.a'}
      onclick={() => tools.selectTool('pen.a')}
      title="Fine Pen (P)"
      aria-label="Fine Black Pen"
    >
      <StudioIcon name="pen.a" size={16} />
    </button>

    <!-- Medium Red Pen (Preset B) -->
    <button
      type="button"
      class="tool-btn"
      class:active={tools.activeToolId === 'pen.b'}
      aria-pressed={tools.activeToolId === 'pen.b'}
      onclick={() => tools.selectTool('pen.b')}
      title="Red Pen (Shift+P)"
      aria-label="Medium Red Pen"
    >
      <StudioIcon name="pen.b" size={16} />
    </button>

    <!-- Highlighter -->
    <button
      type="button"
      class="tool-btn"
      class:active={tools.activeToolId === 'highlighter'}
      aria-pressed={tools.activeToolId === 'highlighter'}
      onclick={() => tools.selectTool('highlighter')}
      title="Highlighter (I)"
      aria-label="Highlighter Tool"
    >
      <StudioIcon name="highlighter" size={16} />
    </button>

    <!-- Partial Eraser -->
    <button
      type="button"
      class="tool-btn"
      class:active={tools.supportsEraserMode}
      aria-pressed={tools.supportsEraserMode}
      onclick={() => tools.selectTool('eraser.partial')}
      title="Eraser (E)"
      aria-label="Eraser Tool"
    >
      <StudioIcon name="eraser" size={16} />
    </button>

    <!-- Laser Pointer -->
    <button
      type="button"
      class="tool-btn"
      class:active={tools.activeToolId === 'laser'}
      aria-pressed={tools.activeToolId === 'laser'}
      onclick={() => tools.selectTool('laser')}
      title="Laser Pointer (L)"
      aria-label="Laser Pointer"
    >
      <StudioIcon name="laser" size={16} />
    </button>

    <div class="toolbar-separator"></div>

    <!-- Radial Menu Summon Trigger -->
    <button
      type="button"
      class="tool-btn radial-trigger-btn"
      onclick={() => onOpenRadial?.()}
      title="8-Sector Radial Menu (/)"
      aria-label="Open Radial Menu"
    >
      <StudioIcon name="radial" size={16} />
    </button>

    <!-- Keyboard Shortcuts Settings -->
    <button
      type="button"
      class="tool-btn"
      onclick={() => onOpenShortcuts?.()}
      title="Custom Shortcuts Editor"
      aria-label="Shortcuts Settings"
    >
      <StudioIcon name="shortcuts" size={16} />
    </button>
  </div>
</aside>

<style>
  .dockable-toolbar {
    position: absolute;
    z-index: 150;
    pointer-events: auto;
    display: flex;
    background: var(--glass-bg);
    backdrop-filter: blur(var(--glass-blur));
    -webkit-backdrop-filter: blur(var(--glass-blur));
    border: 1px solid var(--glass-border);
    box-shadow: var(--glass-shadow);
    border-radius: 12px;
    padding: 6px;
    gap: 4px;
    color: var(--glass-text);
    touch-action: none;
    user-select: none;
    transition: box-shadow var(--motion-duration) ease;
  }

  .dock-floating {
    cursor: default;
  }

  .dock-left {
    left: 12px;
    top: 50%;
    transform: translateY(-50%);
    flex-direction: column;
  }

  .dock-right {
    right: 12px;
    top: 50%;
    transform: translateY(-50%);
    flex-direction: column;
  }

  .dock-top {
    top: 12px;
    left: 50%;
    transform: translateX(-50%);
    flex-direction: row;
  }

  .is-dragging {
    opacity: 0.95;
    box-shadow: 0 16px 36px var(--glass-shadow);
    cursor: grabbing;
  }

  .snap-left,
  .snap-right,
  .snap-top {
    outline: 2px dashed var(--accent);
    outline-offset: 4px;
  }

  .toolbar-drag-handle {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 6px;
    cursor: grab;
    border-radius: 6px;
    opacity: 0.6;
    transition: opacity var(--motion-duration) ease, background var(--motion-duration) ease;
  }

  .toolbar-drag-handle:hover,
  .toolbar-drag-handle:active {
    opacity: 1;
    background: var(--canvas-selection);
  }

  .toolbar-drag-handle:active {
    cursor: grabbing;
  }

  .toolbar-tools {
    display: flex;
    gap: 4px;
  }

  .dock-left .toolbar-tools,
  .dock-right .toolbar-tools {
    flex-direction: column;
  }

  .dock-top .toolbar-tools {
    flex-direction: row;
  }

  .dock-floating .toolbar-tools {
    flex-direction: column;
  }

  .toolbar-separator {
    background: var(--glass-border);
    border-radius: 1px;
  }

  .dock-left .toolbar-separator,
  .dock-right .toolbar-separator,
  .dock-floating .toolbar-separator {
    height: 1px;
    width: 100%;
    margin: 2px 0;
  }

  .dock-top .toolbar-separator {
    width: 1px;
    height: 100%;
    margin: 0 2px;
  }

  .tool-btn {
    width: 36px;
    height: 36px;
    display: flex;
    align-items: center;
    justify-content: center;
    border: none;
    background: transparent;
    border-radius: 8px;
    cursor: pointer;
    color: var(--glass-text);
    transition: background var(--motion-duration) ease, transform 0.1s ease;
  }

  .tool-btn:hover {
    background: var(--canvas-selection);
  }

  .tool-btn:active {
    transform: scale(0.94);
  }

  .tool-btn.active {
    background: var(--accent);
    color: var(--laser-core);
    box-shadow: 0 2px 8px var(--glass-shadow);
  }

  .radial-trigger-btn {
    border: 1px dashed var(--glass-border);
  }
</style>

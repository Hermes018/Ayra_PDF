<script lang="ts">
  import { tools, type ToolId } from '$lib/core/toolEngine.svelte';
  import { commands } from '$lib/core/commandEngine.svelte';

  export type DockPosition = 'float' | 'top' | 'left' | 'right';

  let {
    onOpenRadial,
    onOpenShortcuts
  } = $props<{
    onOpenRadial?: () => void;
    onOpenShortcuts?: () => void;
  }>();

  let dock = $state<DockPosition>('left');
  let posX = $state(16);
  let posY = $state(80);
  let isDragging = $state(false);
  let dragOffset = { x: 0, y: 0 };
  let toolbarElement: HTMLElement | null = $state(null);

  const DOCK_THRESHOLD_PX = 36;

  function resolveDock(x: number, y: number, w: number, vw: number, threshold: number): DockPosition {
    if (y <= threshold) return 'top';
    if (x <= threshold) return 'left';
    if (vw - (x + w) <= threshold) return 'right';
    return 'float';
  }

  function handleDragStart(e: PointerEvent) {
    if (e.button !== 0) return;
    isDragging = true;
    (e.target as HTMLElement)?.setPointerCapture(e.pointerId);

    if (toolbarElement) {
      const rect = toolbarElement.getBoundingClientRect();
      dragOffset = {
        x: e.clientX - rect.left,
        y: e.clientY - rect.top
      };
      if (dock !== 'float') {
        posX = rect.left;
        posY = rect.top;
        dock = 'float';
      }
    }
  }

  function handleDragMove(e: PointerEvent) {
    if (!isDragging) return;
    posX = Math.max(8, e.clientX - dragOffset.x);
    posY = Math.max(8, e.clientY - dragOffset.y);
  }

  function handleDragEnd(e: PointerEvent) {
    if (!isDragging) return;
    isDragging = false;
    try {
      (e.target as HTMLElement)?.releasePointerCapture(e.pointerId);
    } catch {
      // Ignored if pointer capture lost
    }

    if (toolbarElement && typeof window !== 'undefined') {
      const rect = toolbarElement.getBoundingClientRect();
      dock = resolveDock(rect.left, rect.top, rect.width, window.innerWidth, DOCK_THRESHOLD_PX);
    }
  }

  function selectTool(presetId: string) {
    tools.select(presetId);
  }

  function activateToolId(toolId: ToolId) {
    const preset = tools.presets.find(p => p.tool === toolId);
    if (preset) {
      tools.select(preset.id);
    } else {
      // Create or select fallback
      const found = tools.presets.find(p => p.tool.startsWith(toolId.split('.')[0] || ''));
      if (found) tools.select(found.id);
    }
  }
</script>

<aside
  bind:this={toolbarElement}
  class="dockable-toolbar glass-panel dock-{dock}"
  style:left={dock === 'float' ? `${posX}px` : undefined}
  style:top={dock === 'float' ? `${posY}px` : undefined}
  aria-label="Drawboard Studio Markup Toolbar"
>
  <!-- Drag Handle -->
  <div
    class="toolbar-drag-handle"
    onpointerdown={handleDragStart}
    onpointermove={handleDragMove}
    onpointerup={handleDragEnd}
    onpointercancel={handleDragEnd}
    title="Drag to dock (Top, Left, Right) or float"
    role="button"
    tabindex="0"
    aria-label="Drag handle to position toolbar"
  >
    <div class="drag-grip-dots"></div>
  </div>

  <div class="toolbar-tools">
    <!-- Pan / Hand -->
    <button
      type="button"
      class="tool-btn"
      class:active={tools.activeTool === 'pan'}
      onclick={() => selectTool('preset.pan')}
      title="Pan Canvas (H / Space)"
      aria-label="Pan Tool"
    >
      <span class="tool-icon">✋</span>
    </button>

    <!-- Selection / Marquee -->
    <button
      type="button"
      class="tool-btn"
      class:active={tools.activeTool === 'select.marquee' || tools.activeTool === 'select.lasso'}
      onclick={() => activateToolId('select.marquee')}
      title="Select Marquee (V)"
      aria-label="Select Tool"
    >
      <span class="tool-icon">↖️</span>
    </button>

    <div class="toolbar-separator"></div>

    <!-- Fine Black Pen (Preset A) -->
    <button
      type="button"
      class="tool-btn"
      class:active={tools.active?.id === 'preset.pen.a'}
      onclick={() => selectTool('preset.pen.a')}
      title="Fine Pen (P)"
      aria-label="Fine Black Pen"
    >
      <span class="tool-icon">🖊️</span>
    </button>

    <!-- Medium Red Pen (Preset B) -->
    <button
      type="button"
      class="tool-btn"
      class:active={tools.active?.id === 'preset.pen.b'}
      onclick={() => selectTool('preset.pen.b')}
      title="Red Pen (Shift+P)"
      aria-label="Medium Red Pen"
    >
      <span class="tool-icon">🖍️</span>
    </button>

    <!-- Highlighter -->
    <button
      type="button"
      class="tool-btn"
      class:active={tools.activeTool === 'highlighter'}
      onclick={() => selectTool('preset.highlighter')}
      title="Highlighter (I)"
      aria-label="Highlighter Tool"
    >
      <span class="tool-icon">🖍</span>
    </button>

    <!-- Partial Eraser -->
    <button
      type="button"
      class="tool-btn"
      class:active={tools.activeTool === 'eraser.partial'}
      onclick={() => selectTool('preset.eraser.partial')}
      title="Partial Eraser (E)"
      aria-label="Partial Eraser"
    >
      <span class="tool-icon">🧹</span>
    </button>

    <!-- Laser Pointer -->
    <button
      type="button"
      class="tool-btn"
      class:active={tools.activeTool === 'laser'}
      onclick={() => activateToolId('laser')}
      title="Laser Pointer (L)"
      aria-label="Laser Pointer"
    >
      <span class="tool-icon">🔦</span>
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
      <span class="tool-icon">🎯</span>
    </button>

    <!-- Keyboard Shortcuts Settings -->
    <button
      type="button"
      class="tool-btn"
      onclick={() => onOpenShortcuts?.()}
      title="Custom Shortcuts Editor"
      aria-label="Shortcuts Settings"
    >
      <span class="tool-icon">⌨️</span>
    </button>
  </div>
</aside>

<style>
  .dockable-toolbar {
    position: absolute;
    z-index: 120;
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
    transition: box-shadow var(--motion-duration) ease;
  }

  .dock-float {
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

  .drag-grip-dots {
    width: 14px;
    height: 14px;
    background-image: radial-gradient(circle, var(--glass-text) 1.5px, transparent 1.5px);
    background-size: 6px 6px;
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

  .toolbar-separator {
    background: var(--glass-border);
    border-radius: 1px;
  }

  .dock-left .toolbar-separator,
  .dock-right .toolbar-separator {
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

  .tool-icon {
    font-size: 16px;
    line-height: 1;
  }

  .radial-trigger-btn {
    border: 1px dashed var(--glass-border);
  }
</style>

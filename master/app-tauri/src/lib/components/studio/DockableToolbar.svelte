<script module lang="ts">
  import type { ToolId } from '$lib/core/toolEngine.svelte';
  import type { StudioIconName } from './StudioIcon.svelte';
  import { MATH_TEXT_LABS } from './labs.svelte';

  export type DockEdge = 'left' | 'right' | 'top' | 'floating';

  export interface ToolItem {
    id: ToolId;
    label: string;
    icon: StudioIconName;
  }

  export interface CategoryGroup {
    id: string;
    name: string;
    defaultTool: ToolId;
    tools: ToolItem[];
  }

  export const CATEGORIES: CategoryGroup[] = [
    {
      id: 'nav',
      name: 'Pan & Select',
      defaultTool: 'pan',
      tools: [
        { id: 'pan', label: 'Pan Canvas', icon: 'pan' },
        { id: 'select.marquee', label: 'Marquee Select', icon: 'select' },
        { id: 'select.lasso', label: 'Freeform Lasso', icon: 'lasso' },
        { id: 'select.text', label: 'Select Text', icon: 'text.review' },
        { id: 'select.snapshot', label: 'Area Snapshot Stamp', icon: 'shape' }
      ]
    },
    {
      id: 'ink',
      name: 'Inking & Laser',
      defaultTool: 'pen.a',
      tools: [
        { id: 'pen.a', label: 'Technical Pen A', icon: 'pen.a' },
        { id: 'pen.b', label: 'Felt Pen B', icon: 'pen.b' },
        { id: 'highlighter', label: 'Highlighter', icon: 'highlighter' },
        { id: 'laser', label: 'Neon Laser Pointer', icon: 'laser' }
      ]
    },
    {
      id: 'erase',
      name: 'Erasers',
      defaultTool: 'eraser.partial',
      tools: [
        { id: 'eraser.whole', label: 'Whole Stroke Eraser', icon: 'eraser.whole' },
        { id: 'eraser.partial', label: 'Boolean Partial Eraser', icon: 'eraser.partial' }
      ]
    },
    {
      id: 'shapes',
      name: 'Shapes & Hatch',
      defaultTool: 'shape.rect',
      tools: [
        { id: 'shape.line', label: 'Line', icon: 'line' },
        { id: 'shape.arrow', label: 'Arrow', icon: 'arrow' },
        { id: 'shape.polyline', label: 'Polyline', icon: 'shape' },
        { id: 'shape.rect', label: 'Rectangle', icon: 'rect' },
        { id: 'shape.ellipse', label: 'Ellipse / Circle', icon: 'ellipse' },
        { id: 'shape.polygon', label: 'Polygon', icon: 'shape' },
        { id: 'shape.cloud', label: 'Revision Cloud', icon: 'cloud' },
        { id: 'hatch.area', label: 'Procedural Hatch Area', icon: 'hatch' },
        { id: 'hatch.pick', label: 'Pick-Point Hatch Fill', icon: 'hatch' }
      ]
    },
    {
      id: 'review',
      name: 'Text & Review Markup',
      defaultTool: 'insert.text',
      tools: [
        { id: 'insert.text', label: 'Rich Text Box', icon: 'text.review' },
        { id: 'review.highlight', label: 'Highlight Text', icon: 'highlighter' },
        { id: 'review.underline', label: 'Underline', icon: 'text.review' },
        { id: 'review.squiggle', label: 'Squiggly Underline', icon: 'text.review' },
        { id: 'review.strikethrough', label: 'Strikethrough', icon: 'text.review' },
        { id: 'review.redact', label: 'Cryptographic Redaction', icon: 'text.review' }
      ]
    },
    {
      id: 'measure',
      name: 'CAD Measure & Calibrate',
      defaultTool: 'measure.length',
      tools: [
        { id: 'measure.calibrate', label: 'Calibrate Page Scale', icon: 'measure' },
        { id: 'measure.length', label: 'Linear Length', icon: 'measure' },
        { id: 'measure.polylength', label: 'Polylength Chain', icon: 'measure' },
        { id: 'measure.area_rect', label: 'Rectangle Area', icon: 'rect' },
        { id: 'measure.area_polygon', label: 'Polygon Area', icon: 'shape' },
        { id: 'measure.perimeter', label: 'Perimeter', icon: 'measure' },
        { id: 'measure.protractor', label: '3-Point Protractor', icon: 'protractor' },
        { id: 'measure.ruler', label: 'Digital Ruler', icon: 'ruler' },
        { id: 'measure.tape', label: 'Transient Tape Measure', icon: 'tape' }
      ]
    },
    {
      id: 'insert',
      name: 'Insert Suite & Media',
      defaultTool: 'callout.single',
      tools: [
        { id: 'callout.single', label: 'Single Callout', icon: 'callout' },
        { id: 'callout.multi', label: 'Multi-Leader Callout', icon: 'callout' },
        { id: 'insert.note', label: 'Sticky Note', icon: 'note' },
        { id: 'insert.stamp', label: 'Engineering Stamp', icon: 'stamp' },
        { id: 'insert.signature', label: 'Signature Suite', icon: 'signature' },
        { id: 'insert.image', label: 'Insert Image', icon: 'image' },
        // Labs-only (2.16): hidden unless Settings → Labs is on; see ./labs.ts.
        { id: 'insert.latex', label: MATH_TEXT_LABS.name, icon: 'latex' },
        { id: 'insert.audio', label: 'Voice Audio Note', icon: 'audio' }
      ]
    }
  ];

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
  import { commands } from '$lib/core/commandEngine.svelte';
  import StudioIcon from './StudioIcon.svelte';
  import { LABS_BADGE, isLabsTool, labsToolNote, labsVisible, selectToolIfAllowed } from './labs.svelte';

  let {
    onToggleRadial,
    onOpenRadial,
    onOpenShortcuts,
    onToggleStructuralEdit,
    isStructuralEdit = false
  }: {
    onToggleRadial?: () => void;
    onOpenRadial?: () => void;
    onOpenShortcuts?: () => void;
    onToggleStructuralEdit?: () => void;
    isStructuralEdit?: boolean;
  } = $props();

  let dockEdge = $state<DockEdge>('right');
  let floatPos = $state<{ x: number; y: number }>({ x: 16, y: 80 });
  let dragPos = $state<{ x: number; y: number }>({ x: 16, y: 80 });
  let snapPreview = $state<'left' | 'right' | 'top' | null>(null);
  let isDragging = $state(false);
  let grabOffset = { x: 0, y: 0 };
  let toolbarEl: HTMLElement | null = $state(null);

  let activeCategoryPopover = $state<string | null>(null);
  let categoryActiveTools = $state<Record<string, ToolId>>({
    nav: 'pan',
    ink: 'pen.a',
    erase: 'eraser.partial',
    shapes: 'shape.rect',
    review: 'insert.text',
    measure: 'measure.length',
    insert: 'callout.single',
  });

  $effect(() => {
    const current = tools.currentToolId;
    for (const cat of CATEGORIES) {
      if (cat.tools.some((t) => t.id === current)) {
        categoryActiveTools[cat.id] = current;
        break;
      }
    }
  });

  /** Tools the user may see in this drawer (Labs tools only when Labs is on). */
  function visibleTools(cat: CategoryGroup): ToolItem[] {
    return cat.tools.filter((t) => labsVisible(t.id));
  }

  function getCategoryActiveTool(cat: CategoryGroup): ToolItem {
    const shown = visibleTools(cat);
    const activeId = categoryActiveTools[cat.id] || cat.defaultTool;
    return shown.find((t) => t.id === activeId) ?? shown[0] ?? cat.tools[0];
  }

  function isCategoryActive(cat: CategoryGroup): boolean {
    return cat.tools.some((t) => t.id === tools.currentToolId);
  }

  function handleCategoryClick(cat: CategoryGroup) {
    const activeTool = getCategoryActiveTool(cat);
    selectToolIfAllowed(activeTool.id);
    if (activeCategoryPopover === cat.id) {
      activeCategoryPopover = null;
    } else {
      activeCategoryPopover = cat.id;
    }
  }

  function handleSelectToolFromPopover(toolId: ToolId, catId: string) {
    if (!selectToolIfAllowed(toolId)) return;
    categoryActiveTools[catId] = toolId;
    activeCategoryPopover = null;
  }

  function handlePinFavorite(e: MouseEvent, toolId: string) {
    e.stopPropagation();
    if (!labsVisible(toolId)) return;
    tools.pinFavorite(toolId);
  }

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
    {#each CATEGORIES as cat (cat.id)}
      <div class="category-wrapper" class:popover-open={activeCategoryPopover === cat.id}>
        <button
          type="button"
          class="tool-btn category-btn"
          class:active={isCategoryActive(cat)}
          aria-pressed={isCategoryActive(cat)}
          onclick={() => handleCategoryClick(cat)}
          title="{cat.name} — Click to select, click again to open drawer"
          aria-label="{cat.name}"
          data-testid="category-btn-{cat.id}"
        >
          <StudioIcon name={getCategoryActiveTool(cat).icon} size={16} />
          <span class="category-caret">▾</span>
        </button>

        {#if activeCategoryPopover === cat.id}
          <div
            class="category-popover glass-panel"
            data-testid="toolbar-category-popover"
            role="menu"
            tabindex="-1"
            aria-label="{cat.name} Tools"
            onpointerdown={(e) => e.stopPropagation()}
          >
            <div class="popover-header">
              <span class="popover-title">{cat.name}</span>
              <button
                type="button"
                class="popover-close-btn"
                onclick={() => (activeCategoryPopover = null)}
                title="Close drawer"
                aria-label="Close drawer"
              >
                <StudioIcon name="close" size={12} />
              </button>
            </div>
            <div class="popover-items-list">
              {#each visibleTools(cat) as tool (tool.id)}
                <div
                  class="popover-item-row"
                  class:active={tools.currentToolId === tool.id}
                  role="menuitem"
                  tabindex="-1"
                  title={labsToolNote(tool.id)}
                >
                  <!-- svelte-ignore a11y_click_events_have_key_events -->
                  <div
                    class="item-left"
                    role="button"
                    tabindex="0"
                    data-testid="toolbar-tool-{tool.id}"
                    onclick={() => handleSelectToolFromPopover(tool.id, cat.id)}
                  >
                    <StudioIcon name={tool.icon} size={15} />
                    <span class="item-label">{tool.label}</span>
                    {#if isLabsTool(tool.id)}
                      <span class="labs-badge" data-testid="labs-badge">{LABS_BADGE}</span>
                    {/if}
                  </div>
                  <div class="item-right">
                    {#if commands.shortcutFor(tool.id)}
                      <kbd class="item-kbd">{commands.shortcutFor(tool.id)}</kbd>
                    {/if}
                    <button
                      type="button"
                      class="pin-btn"
                      class:pinned={tools.isFavorite(tool.id)}
                      onclick={(e) => handlePinFavorite(e, tool.id)}
                      title={tools.isFavorite(tool.id) ? 'Unpin favorite' : 'Pin to favorites bar'}
                      aria-label={tools.isFavorite(tool.id) ? 'Unpin favorite' : 'Pin to favorites bar'}
                    >
                      <StudioIcon name={tools.isFavorite(tool.id) ? 'star.filled' : 'star'} size={13} />
                    </button>
                  </div>
                </div>
              {/each}
            </div>
          </div>
        {/if}
      </div>
    {/each}

    <div class="toolbar-separator"></div>

    <!-- Structural PDF Text Edit Mode Toggle -->
    <button
      type="button"
      class="tool-btn structural-toggle-btn"
      class:active={isStructuralEdit}
      aria-pressed={isStructuralEdit}
      onclick={() => onToggleStructuralEdit?.()}
      title={isStructuralEdit ? 'Exit Structural PDF Text Edit' : 'Edit Native PDF Text Blocks (Structural)'}
      aria-label="Structural PDF Text Edit"
      data-testid="toolbar-structural-edit-toggle"
    >
      <StudioIcon name="text.review" size={16} />
    </button>

    <!-- Radial Menu Summon Trigger -->
    <button
      type="button"
      class="tool-btn radial-trigger-btn"
      onclick={() => {
        if (onToggleRadial) onToggleRadial();
        else onOpenRadial?.();
      }}
      title="Toggle 8-Sector Radial Menu (/)"
      aria-label="Toggle Radial Menu"
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

  .structural-toggle-btn {
    border: 1px solid var(--glass-border);
  }

  .category-wrapper {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .category-btn {
    position: relative;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 1px;
    padding-bottom: 2px;
  }

  .category-caret {
    font-size: 8px;
    line-height: 8px;
    opacity: 0.6;
    user-select: none;
    pointer-events: none;
  }

  .category-btn.active .category-caret {
    opacity: 0.9;
  }

  .category-popover {
    position: absolute;
    z-index: 200;
    width: 260px;
    max-height: 380px;
    display: flex;
    flex-direction: column;
    background: var(--glass-bg);
    backdrop-filter: blur(var(--glass-blur));
    -webkit-backdrop-filter: blur(var(--glass-blur));
    border: 1px solid var(--glass-border);
    border-radius: 12px;
    box-shadow: var(--glass-shadow);
    padding: 8px;
    color: var(--glass-text);
  }

  .dock-right .category-popover,
  .dock-floating .category-popover {
    right: calc(100% + 8px);
    top: 0;
  }

  .dock-left .category-popover {
    left: calc(100% + 8px);
    top: 0;
  }

  .dock-top .category-popover {
    top: calc(100% + 8px);
    left: 0;
  }

  .popover-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4px 6px 8px 6px;
    border-bottom: 1px solid var(--glass-border);
    margin-bottom: 6px;
  }

  .popover-title {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    opacity: 0.8;
  }

  .popover-close-btn {
    background: transparent;
    border: none;
    cursor: pointer;
    color: var(--glass-text);
    opacity: 0.6;
    padding: 2px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 4px;
    transition: opacity var(--motion-duration) ease, background var(--motion-duration) ease;
  }

  .popover-close-btn:hover {
    opacity: 1;
    background: var(--canvas-selection);
  }

  .popover-items-list {
    display: flex;
    flex-direction: column;
    gap: 2px;
    overflow-y: auto;
    padding-right: 2px;
  }

  .popover-item-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: 6px 8px;
    border-radius: 8px;
    background: transparent;
    border: none;
    color: var(--glass-text);
    cursor: pointer;
    font-size: 12px;
    text-align: left;
    transition: background var(--motion-duration) ease;
  }

  .popover-item-row:hover {
    background: var(--canvas-selection);
  }

  .popover-item-row.active {
    background: var(--accent);
    color: var(--laser-core);
  }

  .item-left {
    display: flex;
    align-items: center;
    gap: 8px;
    flex: 1;
    min-width: 0;
  }

  .item-label {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .labs-badge {
    flex-shrink: 0;
    font-size: 9px;
    font-weight: 700;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    padding: 1px 5px;
    border-radius: 999px;
    border: 1px solid var(--accent);
    color: var(--accent);
  }

  .popover-item-row.active .labs-badge {
    border-color: currentColor;
    color: inherit;
  }

  .item-right {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-shrink: 0;
  }

  .item-kbd {
    font-size: 10px;
    padding: 1px 5px;
    border-radius: 4px;
    background: var(--canvas-selection);
    border: 1px solid var(--glass-border);
    color: var(--glass-text);
    font-family: monospace;
  }

  .popover-item-row.active .item-kbd {
    background: rgba(0, 0, 0, 0.2);
    color: var(--laser-core);
    border-color: rgba(255, 255, 255, 0.3);
  }

  .pin-btn {
    background: transparent;
    border: none;
    cursor: pointer;
    color: var(--glass-text);
    opacity: 0.4;
    padding: 2px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 4px;
    transition: opacity var(--motion-duration) ease, transform 0.1s ease;
  }

  .pin-btn:hover {
    opacity: 1;
    transform: scale(1.15);
  }

  .pin-btn.pinned {
    opacity: 1;
    color: var(--focus-ring);
  }

  .popover-item-row.active .pin-btn {
    color: var(--laser-core);
  }
</style>

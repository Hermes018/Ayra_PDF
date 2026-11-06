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
  import Button from '$lib/components/ui/Button.svelte';
  import IconButton from '$lib/components/ui/IconButton.svelte';
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
  aria-label="Drawboard Studio markup toolbar"
>
  <!-- Drag Handle -->
  <div
    class="toolbar-drag-handle"
    onpointerdown={handleGripDown}
    onpointermove={handleGripMove}
    onpointerup={handleGripUp}
    onpointercancel={handleGripUp}
    title="Drag to dock (top, left, right) or float"
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
    aria-label="Drawing tools"
    onpointerdown={(e) => e.stopPropagation()}
  >
    {#each CATEGORIES as cat (cat.id)}
      <div class="category-wrapper" class:popover-open={activeCategoryPopover === cat.id}>
        <Button
          variant="plain"
          size="sm"
          class="category-btn"
          pressed={isCategoryActive(cat)}
          title="{cat.name}: click to select, click again to open the drawer"
          data-testid="category-btn-{cat.id}"
          onclick={() => handleCategoryClick(cat)}
        >
          <span class="category-glyph">
            <StudioIcon name={getCategoryActiveTool(cat).icon} size={16} />
            <span class="category-caret" aria-hidden="true">▾</span>
          </span>
          <span class="sr-only">{cat.name}</span>
        </Button>

        {#if activeCategoryPopover === cat.id}
          <div
            class="category-popover glass-panel"
            data-testid="toolbar-category-popover"
            role="menu"
            tabindex="-1"
            aria-label="{cat.name} tools"
            onpointerdown={(e) => e.stopPropagation()}
          >
            <div class="popover-header">
              <span class="popover-title">{cat.name}</span>
              <IconButton icon="close" size="sm" ariaLabel="Close drawer" onclick={() => (activeCategoryPopover = null)} />
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
                  <Button
                    variant="plain"
                    block
                    size="sm"
                    class="item-left"
                    data-testid="toolbar-tool-{tool.id}"
                    onclick={() => handleSelectToolFromPopover(tool.id, cat.id)}
                  >
                    <StudioIcon name={tool.icon} size={15} />
                    <span class="item-label">{tool.label}</span>
                    {#if isLabsTool(tool.id)}
                      <span class="labs-badge" data-testid="labs-badge">{LABS_BADGE}</span>
                    {/if}
                  </Button>
                  <div class="item-right">
                    {#if commands.shortcutFor(tool.id)}
                      <kbd class="item-kbd">{commands.shortcutFor(tool.id)}</kbd>
                    {/if}
                    <IconButton
                      icon={tools.isFavorite(tool.id) ? 'star.filled' : 'star'}
                      size="sm"
                      ariaLabel={tools.isFavorite(tool.id) ? 'Unpin favorite' : 'Pin to favorites bar'}
                      onclick={(e) => handlePinFavorite(e, tool.id)}
                    />
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
    <IconButton
      icon="text.review"
      ariaLabel={isStructuralEdit ? 'Exit structural PDF text edit' : 'Structural PDF text edit'}
      pressed={isStructuralEdit}
      data-testid="toolbar-structural-edit-toggle"
      onclick={() => onToggleStructuralEdit?.()}
    />

    <!-- Radial Menu Summon Trigger -->
    <IconButton
      icon="radial"
      ariaLabel="Toggle radial menu"
      shortcutCmd="radial.open"
      onclick={() => {
        if (onToggleRadial) onToggleRadial();
        else onOpenRadial?.();
      }}
    />

    <!-- Keyboard Shortcuts Settings -->
    <IconButton icon="shortcuts" ariaLabel="Shortcuts settings" onclick={() => onOpenShortcuts?.()} />
  </div>
</aside>

<style>
  .dockable-toolbar {
    position: absolute;
    z-index: 150;
    pointer-events: auto;
    display: flex;
    background: var(--surface-dialog);
    backdrop-filter: blur(var(--glass-blur));
    -webkit-backdrop-filter: blur(var(--glass-blur));
    border: 1px solid var(--glass-border);
    box-shadow: var(--glass-shadow);
    border-radius: 12px;
    padding: var(--space-2);
    gap: var(--space-1);
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
    padding: var(--space-2);
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
    gap: var(--space-1);
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
    margin: var(--space-1) 0;
  }

  .dock-top .toolbar-separator {
    width: 1px;
    height: 100%;
    margin: 0 var(--space-1);
  }

  /* An open drawer must stack above the property pill (same z-index, later in the DOM). */
  .dockable-toolbar:has(.popover-open) {
    z-index: 210;
  }

  .category-wrapper {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  :global(.category-btn) {
    position: relative;
  }

  .category-glyph {
    position: relative;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 1px;
    padding-bottom: var(--space-1);
  }

  .category-caret {
    font-size: var(--font-size-caption);
    line-height: 1;
    opacity: 0.6;
    user-select: none;
    pointer-events: none;
  }

  .category-popover {
    position: absolute;
    z-index: 200;
    width: 260px;
    max-height: 380px;
    display: flex;
    flex-direction: column;
    background: var(--surface-dialog);
    backdrop-filter: blur(var(--glass-blur));
    -webkit-backdrop-filter: blur(var(--glass-blur));
    border: 1px solid var(--glass-border);
    border-radius: 12px;
    box-shadow: var(--glass-shadow);
    padding: var(--space-2);
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
    padding: var(--space-1) var(--space-2) var(--space-2) var(--space-2);
    border-bottom: 1px solid var(--glass-border);
    margin-bottom: var(--space-2);
  }

  .popover-title {
    font-size: var(--font-size-caption);
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    opacity: 0.8;
  }

  .popover-items-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    overflow-y: auto;
    padding-right: var(--space-1);
  }

  .popover-item-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: var(--space-2) var(--space-2);
    border-radius: 8px;
    background: transparent;
    border: none;
    color: var(--glass-text);
    cursor: pointer;
    font-size: var(--font-size-footnote);
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

  :global(.item-left) {
    display: flex;
    align-items: center;
    gap: var(--space-2);
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
    font-size: var(--font-size-caption);
    font-weight: 700;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    padding: 1px var(--space-1);
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
    gap: var(--space-2);
    flex-shrink: 0;
  }

  .item-kbd {
    font-size: var(--font-size-caption);
    padding: 1px var(--space-1);
    border-radius: 4px;
    background: var(--canvas-selection);
    border: 1px solid var(--glass-border);
    color: var(--glass-text);
    font-family: monospace;
  }

  .popover-item-row.active .item-kbd {
    background: var(--glass-bg-active);
    color: var(--laser-core);
    border-color: var(--glass-border-subtle);
  }

  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip: rect(0 0 0 0);
    white-space: nowrap;
  }
</style>

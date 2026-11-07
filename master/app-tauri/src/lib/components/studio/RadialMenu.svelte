<script lang="ts">
  import { tools, type ToolId } from '$lib/core/toolEngine.svelte';
  import { commands } from '$lib/core/commandEngine.svelte';
  import StudioIcon, { type StudioIconName } from './StudioIcon.svelte';
  import {
    RADIAL,
    hitTestRadial,
    clampCenter,
    sectorAngles,
    flyoutItemAngles,
    polarToCartesian,
    describeArc,
    type RadialHit
  } from '$lib/core/radialGeometry';
  import radialLayout from '$lib/core/radialLayout.json';

  interface FlyoutItem {
    id: string;
    label: string;
    tool: string;
    icon: string;
  }

  interface WedgeItem {
    sector: number;
    id: string;
    label: string;
    icon: string;
    tool: string;
    presetId: string | null;
    flyout: FlyoutItem[];
  }

  const WEDGES: WedgeItem[] = radialLayout as WedgeItem[];
  const flyoutCounts = WEDGES.map(w => w.flyout.length);

  let {
    x = 200,
    y = 200,
    visible = $bindable(false),
    onClose
  }: {
    x: number;
    y: number;
    visible: boolean;
    onClose?: () => void;
  } = $props();

  let activeSector = $state<number | null>(null);
  let activeFlyout = $state<number | null>(null);
  let menuContainer: HTMLDivElement | null = $state(null);
  let svgEl: SVGSVGElement | null = $state(null);

  // Capture exact viewport coordinates when summoned
  let menuCenter = $derived.by(() => {
    if (typeof window !== 'undefined') {
      return clampCenter(x, y, window.innerWidth, window.innerHeight);
    }
    return { x, y };
  });

  const SVG_SIZE = 440;
  const CX = 220;
  const CY = 220;

  function closeMenu() {
    visible = false;
    activeSector = null;
    activeFlyout = null;
    onClose?.();
  }

  function getSvgRelativePointer(e: PointerEvent): { dx: number; dy: number } {
    if (svgEl) {
      const rect = svgEl.getBoundingClientRect();
      const cx = rect.left + rect.width / 2;
      const cy = rect.top + rect.height / 2;
      const scaleX = 440 / (rect.width || 440);
      const scaleY = 440 / (rect.height || 440);
      return {
        dx: (e.clientX - cx) * scaleX,
        dy: (e.clientY - cy) * scaleY,
      };
    }
    return {
      dx: e.clientX - menuCenter.x,
      dy: e.clientY - menuCenter.y,
    };
  }

  function handlePointerMove(e: PointerEvent) {
    if (!visible) return;
    const { dx, dy } = getSvgRelativePointer(e);
    const hit: RadialHit = hitTestRadial(dx, dy, flyoutCounts, activeSector);

    if (hit.zone === 'wedge') {
      activeSector = hit.sector;
      activeFlyout = null;
    } else if (hit.zone === 'flyout') {
      activeSector = hit.sector;
      activeFlyout = hit.item;
    } else if (hit.zone === 'center' || hit.zone === 'cancel' || hit.zone === 'outside') {
      activeFlyout = null;
    }
  }

  function handlePointerUp(e: PointerEvent) {
    if (!visible) return;
    const { dx, dy } = getSvgRelativePointer(e);
    const hit: RadialHit = hitTestRadial(dx, dy, flyoutCounts, activeSector);

    if (hit.zone === 'wedge') {
      const wedge = WEDGES[hit.sector];
      if (wedge) {
        tools.selectTool(wedge.tool as ToolId);
      }
      closeMenu();
    } else if (hit.zone === 'flyout') {
      const wedge = WEDGES[hit.sector];
      const item = wedge?.flyout[hit.item];
      if (item) {
        tools.selectTool(item.tool as ToolId);
      }
      closeMenu();
    } else if (hit.zone === 'center' || hit.zone === 'cancel') {
      closeMenu();
    }
  }

  $effect(() => {
    if (!visible) {
      if (commands.onRadialKey) commands.onRadialKey = null;
      return;
    }
    commands.onRadialKey = (key: string): boolean => {
      if (key === 'Escape') {
        closeMenu();
        return true;
      }
      const num = parseInt(key, 10);
      if (num >= 1 && num <= 8) {
        const sector = num - 1;
        const wedge = WEDGES[sector];
        if (wedge) {
          tools.selectTool(wedge.tool as ToolId);
        }
        closeMenu();
        return true;
      }
      return false;
    };
    return () => {
      commands.onRadialKey = null;
    };
  });
</script>

<svelte:window
  onpointermove={handlePointerMove}
  onpointerup={handlePointerUp}
/>

{#if visible}
  <div
    class="radial-backdrop"
    bind:this={menuContainer}
    role="presentation"
    onpointermove={handlePointerMove}
    onpointerdown={(e) => {
      if (e.target === menuContainer || (e.target as HTMLElement)?.classList.contains('radial-backdrop')) {
        closeMenu();
      }
    }}
  >
    <div
      class="radial-menu"
      style:left="{menuCenter.x - CX}px"
      style:top="{menuCenter.y - CY}px"
      style:width="{SVG_SIZE}px"
      style:height="{SVG_SIZE}px"
      role="menu"
      aria-label="8-Sector Radial Markup Menu"
    >
      <svg
        bind:this={svgEl}
        width={SVG_SIZE}
        height={SVG_SIZE}
        viewBox="-220 -220 440 440"
        class="radial-svg"
      >
        <!-- Center Cancel / Hub Circle -->
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <g
          class="center-hub-g"
          role="button"
          tabindex={0}
          aria-label="Close Radial Menu"
          onclick={closeMenu}
          onpointerdown={(e) => { e.stopPropagation(); closeMenu(); }}
          style="cursor: pointer;"
        >
          <circle
            cx={0}
            cy={0}
            r={RADIAL.rEsc}
            class="center-hub"
          />
          <g transform="translate(-8, -14)" class="center-hub-icon-g">
            <StudioIcon name="close" size={16} />
          </g>
          <text
            x={0}
            y={12}
            text-anchor="middle"
            dominant-baseline="central"
            class="center-hub-label"
          >
            Esc
          </text>
        </g>

        <!-- 8 Primary Wedges -->
        {#each WEDGES as wedge (wedge.sector)}
          {@const angles = sectorAngles(wedge.sector)}
          {@const pathD = describeArc(0, 0, RADIAL.rInnerIn, RADIAL.rInnerOut, angles.startDeg, angles.endDeg)}
          {@const iconR = (RADIAL.rInnerIn + RADIAL.rInnerOut) / 2}
          {@const pt = polarToCartesian(iconR, angles.midDeg)}
          {@const isHovered = activeSector === wedge.sector}
          {@const isCurrentTool = tools.activeToolId === wedge.tool || (wedge.presetId && tools.activePreset?.id === wedge.presetId)}

          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <g
            class="wedge-group"
            class:hovered={isHovered}
            class:active-tool={isCurrentTool}
            role="menuitem"
            tabindex={0}
            aria-label={wedge.label}
            onpointerenter={() => { activeSector = wedge.sector; activeFlyout = null; }}
            onpointerdown={(e) => { e.stopPropagation(); tools.selectTool(wedge.tool as ToolId); closeMenu(); }}
            onclick={() => { tools.selectTool(wedge.tool as ToolId); closeMenu(); }}
            style="cursor: pointer;"
          >
            <path
              d={pathD}
              class="wedge-path"
            />
            <!-- Wedge Icon (Vector SVG) -->
            <g transform="translate({pt.x - 9}, {pt.y - 14})" class="wedge-icon-g">
              <StudioIcon name={wedge.icon as StudioIconName} size={18} />
            </g>
            <!-- Wedge Label -->
            <text
              x={pt.x}
              y={pt.y + 12}
              text-anchor="middle"
              dominant-baseline="central"
              class="wedge-label"
            >
              {wedge.label}
            </text>
          </g>
        {/each}

        <!-- Outer Flyout Ring for Active Hover Wedge -->
        {#if activeSector !== null && WEDGES[activeSector]?.flyout.length}
          {@const activeWedge = WEDGES[activeSector]!}
          {@const count = activeWedge.flyout.length}

          {#each activeWedge.flyout as item, idx}
            {@const { itemStart, itemEnd, itemMid } = flyoutItemAngles(activeSector, count, idx)}
            {@const flyoutPath = describeArc(0, 0, RADIAL.rSubIn, RADIAL.rSubOut, itemStart, itemEnd)}
            {@const flyoutR = (RADIAL.rSubIn + RADIAL.rSubOut) / 2}
            {@const pt = polarToCartesian(flyoutR, itemMid)}
            {@const isFlyoutHovered = activeFlyout === idx}

            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <g
              class="flyout-group"
              class:hovered={isFlyoutHovered}
              role="menuitem"
              tabindex={0}
              aria-label={item.label}
              onpointerenter={() => { activeFlyout = idx; }}
              onpointerdown={(e) => { e.stopPropagation(); tools.selectTool(item.tool as ToolId); closeMenu(); }}
              onclick={() => { tools.selectTool(item.tool as ToolId); closeMenu(); }}
              style="cursor: pointer;"
            >
              <path
                d={flyoutPath}
                class="flyout-path"
              />
              <g transform="translate({pt.x - 7}, {pt.y - 12})" class="flyout-icon-g">
                <StudioIcon name={item.icon as StudioIconName} size={14} />
              </g>
              <text
                x={pt.x}
                y={pt.y + 12}
                text-anchor="middle"
                dominant-baseline="central"
                class="flyout-label"
              >
                {item.label}
              </text>
            </g>
          {/each}
        {/if}
      </svg>
    </div>
  </div>
{/if}

<style>
  .radial-backdrop {
    position: fixed;
    inset: 0;
    z-index: var(--z-dialog, 300);
    background: transparent;
    user-select: none;
    touch-action: none;
  }

  .radial-menu {
    position: absolute;
    pointer-events: auto;
    filter: drop-shadow(0 12px 32px var(--glass-shadow));
  }

  .radial-svg {
    display: block;
    overflow: visible;
  }

  .center-hub {
    fill: var(--surface-dialog);
    stroke: var(--glass-border);
    stroke-width: 1.5;
    cursor: pointer;
    transition: none !important;
  }

  .center-hub:hover {
    fill: var(--danger);
    transition: none !important;
  }

  .center-hub-icon-g {
    pointer-events: none;
    color: var(--glass-text);
  }

  .center-hub-label {
    font-size: var(--font-size-caption);
    font-weight: 600;
    fill: var(--glass-text);
    pointer-events: none;
    user-select: none;
  }

  .wedge-group {
    cursor: pointer;
  }

  /* Zero CSS transitions for 120Hz deterministic tracking */
  .wedge-path {
    fill: var(--surface-dialog);
    stroke: var(--glass-border);
    stroke-width: 1.5;
    transition: none !important;
  }

  .wedge-group.hovered .wedge-path {
    fill: var(--accent);
    stroke: var(--focus-ring);
    transition: none !important;
  }

  .wedge-group.active-tool .wedge-path {
    stroke: var(--accent);
    stroke-width: 2.5;
    transition: none !important;
  }

  .wedge-icon-g {
    pointer-events: none;
    color: var(--glass-text);
  }

  .wedge-group.hovered .wedge-icon-g {
    color: var(--laser-core);
  }

  .wedge-label {
    fill: var(--glass-text);
    font-size: var(--font-size-caption);
    font-weight: 600;
    pointer-events: none;
  }

  .wedge-group.hovered .wedge-label {
    fill: var(--laser-core);
  }

  /* Flyout Outer Ring */
  .flyout-group {
    cursor: pointer;
  }

  .flyout-path {
    fill: var(--surface-dialog);
    stroke: var(--glass-border);
    stroke-width: 1.5;
    transition: none !important;
  }

  .flyout-group.hovered .flyout-path {
    fill: var(--focus-ring);
    stroke: var(--laser-core);
    transition: none !important;
  }

  .flyout-icon-g {
    pointer-events: none;
    color: var(--glass-text);
  }

  .flyout-group.hovered .flyout-icon-g {
    color: var(--laser-core);
  }

  .flyout-label {
    fill: var(--glass-text);
    font-size: var(--font-size-caption);
    font-weight: 600;
    pointer-events: none;
  }

  .flyout-group.hovered .flyout-label {
    fill: var(--laser-core);
  }
</style>

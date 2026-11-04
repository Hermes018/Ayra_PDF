<script lang="ts">
  import { onMount } from 'svelte';
  import { tools, type ToolId } from '$lib/core/toolEngine.svelte';
  import {
    RADIAL,
    radialHit,
    clampCenter,
    sectorAngles,
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
  } = $props<{
    x: number;
    y: number;
    visible: boolean;
    onClose?: () => void;
  }>();

  let activeSector = $state<number | null>(null);
  let activeFlyout = $state<number | null>(null);
  let menuContainer: HTMLDivElement | null = $state(null);

  // Clamped center
  let center = $derived.by(() => {
    if (typeof window === 'undefined') return { x, y };
    return clampCenter(x, y, window.innerWidth, window.innerHeight);
  });

  const SVG_SIZE = RADIAL.r2 * 2 + 16;
  const CX = SVG_SIZE / 2;
  const CY = SVG_SIZE / 2;

  function closeMenu() {
    visible = false;
    activeSector = null;
    activeFlyout = null;
    onClose?.();
  }

  function handlePointerMove(e: PointerEvent) {
    const dx = e.clientX - center.x;
    const dy = e.clientY - center.y;
    const hit: RadialHit = radialHit(dx, dy, flyoutCounts);

    if (hit.zone === 'wedge') {
      activeSector = hit.sector;
      activeFlyout = null;
    } else if (hit.zone === 'flyout') {
      activeSector = hit.sector;
      activeFlyout = hit.item;
    } else if (hit.zone === 'cancel' || hit.zone === 'outside') {
      activeFlyout = null;
    }
  }

  function handlePointerUp(e: PointerEvent) {
    const dx = e.clientX - center.x;
    const dy = e.clientY - center.y;
    const hit: RadialHit = radialHit(dx, dy, flyoutCounts);

    if (hit.zone === 'wedge') {
      const wedge = WEDGES[hit.sector];
      if (wedge) {
        if (wedge.presetId) {
          tools.select(wedge.presetId);
        } else {
          activateTool(wedge.tool as ToolId);
        }
      }
      closeMenu();
    } else if (hit.zone === 'flyout') {
      const wedge = WEDGES[hit.sector];
      const item = wedge?.flyout[hit.item];
      if (item) {
        if (item.id.startsWith('preset.')) {
          tools.select(item.id);
        } else {
          activateTool(item.tool as ToolId);
        }
      }
      closeMenu();
    } else if (hit.zone === 'cancel') {
      closeMenu();
    }
  }

  function activateTool(toolId: ToolId) {
    const preset = tools.presets.find(p => p.tool === toolId);
    if (preset) {
      tools.select(preset.id);
    } else {
      const found = tools.presets.find(p => p.tool.startsWith(toolId.split('.')[0] || ''));
      if (found) tools.select(found.id);
    }
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (!visible) return;
    if (e.key === 'Escape') {
      closeMenu();
      return;
    }
    // Number keys 1-8 select sectors
    const num = parseInt(e.key, 10);
    if (num >= 1 && num <= 8) {
      const sector = num - 1;
      const wedge = WEDGES[sector];
      if (wedge) {
        if (wedge.presetId) tools.select(wedge.presetId);
        else activateTool(wedge.tool as ToolId);
      }
      closeMenu();
    }
  }
</script>

<svelte:window onkeydown={handleKeyDown} />

{#if visible}
  <div
    class="radial-backdrop"
    bind:this={menuContainer}
    onpointermove={handlePointerMove}
    onpointerup={handlePointerUp}
    role="presentation"
  >
    <div
      class="radial-menu"
      style:left="{center.x - CX}px"
      style:top="{center.y - CY}px"
      style:width="{SVG_SIZE}px"
      style:height="{SVG_SIZE}px"
      role="menu"
      aria-label="8-Sector Radial Markup Menu"
    >
      <svg
        width={SVG_SIZE}
        height={SVG_SIZE}
        viewBox="0 0 {SVG_SIZE} {SVG_SIZE}"
        class="radial-svg"
      >
        <!-- Center Cancel / Hub Circle -->
        <circle
          cx={CX}
          cy={CY}
          r={RADIAL.r0}
          class="center-hub"
          role="menuitem"
          aria-label="Cancel"
        />
        <text
          x={CX}
          y={CY + 5}
          text-anchor="middle"
          class="center-hub-icon"
        >
          ✕
        </text>

        <!-- 8 Primary Wedges -->
        {#each WEDGES as wedge (wedge.sector)}
          {@const angles = sectorAngles(wedge.sector)}
          {@const pathD = describeArc(CX, CY, RADIAL.r0, RADIAL.r1, angles.startDeg, angles.endDeg)}
          {@const rad = angles.midDeg * (Math.PI / 180)}
          {@const iconR = (RADIAL.r0 + RADIAL.r1) / 2}
          {@const iconX = CX + iconR * Math.cos(rad)}
          {@const iconY = CY + iconR * Math.sin(rad)}
          {@const isHovered = activeSector === wedge.sector}
          {@const isCurrentTool = tools.activeTool === wedge.tool || tools.active?.id === wedge.presetId}

          <g class="wedge-group" class:hovered={isHovered} class:active-tool={isCurrentTool}>
            <path
              d={pathD}
              class="wedge-path"
              role="menuitem"
              aria-label={wedge.label}
            />
            <!-- Wedge Icon -->
            <text
              x={iconX}
              y={iconY - 4}
              text-anchor="middle"
              class="wedge-icon"
            >
              {wedge.icon}
            </text>
            <!-- Wedge Label -->
            <text
              x={iconX}
              y={iconY + 12}
              text-anchor="middle"
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
          {@const angles = sectorAngles(activeSector)}
          {@const wedgeSpan = angles.endDeg - angles.startDeg}
          {@const itemSpan = wedgeSpan / count}

          {#each activeWedge.flyout as item, idx}
            {@const itemStart = angles.startDeg + idx * itemSpan}
            {@const itemEnd = itemStart + itemSpan}
            {@const itemMid = (itemStart + itemEnd) / 2}
            {@const flyoutPath = describeArc(CX, CY, RADIAL.r1 + 2, RADIAL.r2, itemStart, itemEnd)}
            {@const flyoutRad = itemMid * (Math.PI / 180)}
            {@const flyoutR = (RADIAL.r1 + RADIAL.r2) / 2}
            {@const fx = CX + flyoutR * Math.cos(flyoutRad)}
            {@const fy = CY + flyoutR * Math.sin(flyoutRad)}
            {@const isFlyoutHovered = activeFlyout === idx}

            <g class="flyout-group" class:hovered={isFlyoutHovered}>
              <path
                d={flyoutPath}
                class="flyout-path"
                role="menuitem"
                aria-label={item.label}
              />
              <text
                x={fx}
                y={fy - 2}
                text-anchor="middle"
                class="flyout-icon"
              >
                {item.icon}
              </text>
              <text
                x={fx}
                y={fy + 11}
                text-anchor="middle"
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
    z-index: 300;
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
    fill: var(--glass-bg);
    stroke: var(--glass-border);
    stroke-width: 1.5;
    cursor: pointer;
    transition: fill var(--motion-duration) ease;
  }

  .center-hub:hover {
    fill: var(--danger);
  }

  .center-hub-icon {
    fill: var(--glass-text);
    font-size: 14px;
    font-weight: bold;
    pointer-events: none;
  }

  .wedge-group {
    cursor: pointer;
    transition: transform var(--motion-duration) ease;
  }

  .wedge-path {
    fill: var(--glass-bg);
    stroke: var(--glass-border);
    stroke-width: 1.2;
    transition: fill var(--motion-duration) ease, stroke var(--motion-duration) ease;
  }

  .wedge-group.hovered .wedge-path {
    fill: var(--accent);
    stroke: var(--focus-ring);
  }

  .wedge-group.active-tool .wedge-path {
    stroke: var(--accent);
    stroke-width: 2;
  }

  .wedge-icon {
    font-size: 16px;
    pointer-events: none;
  }

  .wedge-label {
    fill: var(--glass-text);
    font-size: 10px;
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
    fill: var(--glass-bg);
    stroke: var(--glass-border);
    stroke-width: 1;
    transition: fill var(--motion-duration) ease;
  }

  .flyout-group.hovered .flyout-path {
    fill: var(--focus-ring);
    stroke: var(--laser-core);
  }

  .flyout-icon {
    font-size: 13px;
    pointer-events: none;
  }

  .flyout-label {
    fill: var(--glass-text);
    font-size: 9px;
    font-weight: 600;
    pointer-events: none;
  }

  .flyout-group.hovered .flyout-label {
    fill: var(--laser-core);
  }
</style>

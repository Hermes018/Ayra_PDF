<script lang="ts">
  import {
    measureEngine,
    unitsPerPt,
    type MeasurementRecord,
  } from '$lib/core/measureEngine.svelte';
  import { snapEngine } from '$lib/core/snapEngine.svelte';
  import { generatePolylineSvgPath } from '$lib/core/shapeAndHatchEngine.svelte';

  let {
    activePageIndex = 0,
  }: {
    activePageIndex?: number;
  } = $props();

  let rulerDragMode = $state<'none' | 'move' | 'rotate'>('none');
  let rulerDragOffset = { x: 0, y: 0 };

  let allRenderedMeasurements = $derived<MeasurementRecord[]>(
    measureEngine.previewRecord
      ? [...measureEngine.measurements, measureEngine.previewRecord]
      : measureEngine.measurements
  );

  function getLabelAnchor(m: MeasurementRecord): { x: number; y: number } {
    if (m.points.length === 0) return { x: 0, y: 0 };
    if (m.kind === 'protractor' && m.points.length >= 2) {
      return { x: m.points[1].x, y: m.points[1].y - 18 };
    }
    if (m.points.length === 2) {
      return {
        x: (m.points[0].x + m.points[1].x) / 2,
        y: (m.points[0].y + m.points[1].y) / 2 - 10,
      };
    }
    let sx = 0;
    let sy = 0;
    for (const p of m.points) {
      sx += p.x;
      sy += p.y;
    }
    return { x: sx / m.points.length, y: sy / m.points.length };
  }

  function generateEndpointTickPath(
    pt: { x: number; y: number },
    adj: { x: number; y: number },
    style: MeasurementRecord['endpointStyle']
  ): string {
    const dx = adj.x - pt.x;
    const dy = adj.y - pt.y;
    const angle = Math.atan2(dy, dx);
    const size = 6;

    if (style === 'open-arrow' || style === 'closed-arrow') {
      const a1 = angle + Math.PI / 6;
      const a2 = angle - Math.PI / 6;
      const x1 = pt.x + size * 1.4 * Math.cos(a1);
      const y1 = pt.y + size * 1.4 * Math.sin(a1);
      const x2 = pt.x + size * 1.4 * Math.cos(a2);
      const y2 = pt.y + size * 1.4 * Math.sin(a2);
      return style === 'closed-arrow'
        ? `M ${x1} ${y1} L ${pt.x} ${pt.y} L ${x2} ${y2} Z`
        : `M ${x1} ${y1} L ${pt.x} ${pt.y} L ${x2} ${y2}`;
    }

    // Architectural 45° tick + perpendicular witness extension line
    const perp = angle + Math.PI / 2;
    const slash = angle + Math.PI / 4;
    const wx1 = pt.x + size * Math.cos(perp);
    const wy1 = pt.y + size * Math.sin(perp);
    const wx2 = pt.x - size * Math.cos(perp);
    const wy2 = pt.y - size * Math.sin(perp);
    const sx1 = pt.x + size * 0.9 * Math.cos(slash);
    const sy1 = pt.y + size * 0.9 * Math.sin(slash);
    const sx2 = pt.x - size * 0.9 * Math.cos(slash);
    const sy2 = pt.y - size * 0.9 * Math.sin(slash);
    return `M ${wx1} ${wy1} L ${wx2} ${wy2} M ${sx1} ${sy1} L ${sx2} ${sy2}`;
  }

  function generateProtractorArcPath(pts: { x: number; y: number }[]): string {
    if (pts.length < 3) return '';
    const [p1, v, p2] = pts;
    const r = 26;
    const a1 = Math.atan2(p1.y - v.y, p1.x - v.x);
    const a2 = Math.atan2(p2.y - v.y, p2.x - v.x);
    const sx = v.x + r * Math.cos(a1);
    const sy = v.y + r * Math.sin(a1);
    const ex = v.x + r * Math.cos(a2);
    const ey = v.y + r * Math.sin(a2);
    let diff = a2 - a1;
    while (diff <= -Math.PI) diff += 2 * Math.PI;
    while (diff > Math.PI) diff -= 2 * Math.PI;
    const sweep = diff >= 0 ? 1 : 0;
    return `M ${sx} ${sy} A ${r} ${r} 0 0 ${sweep} ${ex} ${ey}`;
  }

  let rulerTicks = $derived.by(() => {
    const len = measureEngine.ruler.lengthPt;
    const scale = measureEngine.getScaleForPage(activePageIndex);
    const ratio = unitsPerPt(scale);
    const stepPt = 18;
    const list: { x: number; major: boolean; label: string }[] = [];
    const totalSteps = Math.floor(len / stepPt);
    for (let i = 0; i <= totalSteps; i++) {
      const x = i * stepPt;
      const major = i % 4 === 0;
      const calVal = x * ratio;
      list.push({
        x,
        major,
        label: major ? `${calVal.toFixed(scale.precision === 0 ? 0 : 1)}` : '',
      });
    }
    return list;
  });

  function startRulerMove(e: PointerEvent) {
    e.stopPropagation();
    rulerDragMode = 'move';
    rulerDragOffset = {
      x: e.clientX - measureEngine.ruler.xPt,
      y: e.clientY - measureEngine.ruler.yPt,
    };
    try {
      (e.currentTarget as Element).setPointerCapture(e.pointerId);
    } catch {}
  }

  function startRulerRotate(e: PointerEvent) {
    e.stopPropagation();
    rulerDragMode = 'rotate';
    try {
      (e.currentTarget as Element).setPointerCapture(e.pointerId);
    } catch {}
  }

  function onRulerPointerMove(e: PointerEvent) {
    if (rulerDragMode === 'none') return;
    e.stopPropagation();
    if (rulerDragMode === 'move') {
      measureEngine.ruler = {
        ...measureEngine.ruler,
        xPt: e.clientX - rulerDragOffset.x,
        yPt: e.clientY - rulerDragOffset.y,
      };
    } else if (rulerDragMode === 'rotate') {
      const dx = e.clientX - measureEngine.ruler.xPt;
      const dy = e.clientY - measureEngine.ruler.yPt;
      let deg = (Math.atan2(dy, dx) * 180) / Math.PI;
      if (e.shiftKey) {
        deg = Math.round(deg / 15) * 15;
      }
      measureEngine.ruler = {
        ...measureEngine.ruler,
        angleDeg: Number(deg.toFixed(1)),
      };
    }
  }

  function onRulerPointerUp(e: PointerEvent) {
    if (rulerDragMode === 'none') return;
    e.stopPropagation();
    rulerDragMode = 'none';
    try {
      (e.currentTarget as Element).releasePointerCapture(e.pointerId);
    } catch {}
  }
</script>

<!-- Measurement Dimension Lines, Area Polygons, Protractor Arcs & Labels -->
<g class="measure-overlay-group">
  {#each allRenderedMeasurements as m (m.id)}
    {@const isClosedArea = m.kind === 'area.rect' || m.kind === 'area.polygon'}
    {@const anchor = getLabelAnchor(m)}
    {@const labelW = Math.max(64, m.formattedLabel.length * 6.8 + 14)}
    <g class="measurement-item" data-kind={m.kind}>
      <!-- Dimension / Polyline / Area Path -->
      <path
        d={generatePolylineSvgPath(m.points, isClosedArea || (m.kind === 'perimeter' && m.points.length >= 3))}
        fill={isClosedArea ? 'url(\x23hatch-diagonal-steel)' : 'none'}
        fill-opacity={isClosedArea ? 0.28 : 0}
        stroke="var(--measure-line)"
        stroke-width={m.strokeWidthPt}
        stroke-dasharray={m.kind === 'tape' ? '5 3' : undefined}
        stroke-linecap="round"
        stroke-linejoin="round"
      />

      <!-- Protractor Degree Arc -->
      {#if m.kind === 'protractor' && m.points.length >= 3}
        <path
          d={generateProtractorArcPath(m.points)}
          fill="var(--canvas-selection)"
          stroke="var(--accent)"
          stroke-width="1.5"
        />
      {/if}

      <!-- Architectural Endpoint Ticks & Witness Lines -->
      {#if m.points.length >= 2 && !isClosedArea}
        {@const firstPt = m.points[0]}
        {@const secondPt = m.points[1]}
        {@const lastPt = m.points[m.points.length - 1]}
        {@const prevPt = m.points[m.points.length - 2]}
        {#if m.endpointStyle === 'circle'}
          <circle cx={firstPt.x} cy={firstPt.y} r="3.5" fill="var(--measure-line)" />
          <circle cx={lastPt.x} cy={lastPt.y} r="3.5" fill="var(--measure-line)" />
        {:else}
          <path
            d={generateEndpointTickPath(firstPt, secondPt, m.endpointStyle)}
            fill={m.endpointStyle === 'closed-arrow' ? 'var(--measure-line)' : 'none'}
            stroke="var(--measure-line)"
            stroke-width="1.4"
          />
          <path
            d={generateEndpointTickPath(lastPt, prevPt, m.endpointStyle)}
            fill={m.endpointStyle === 'closed-arrow' ? 'var(--measure-line)' : 'none'}
            stroke="var(--measure-line)"
            stroke-width="1.4"
          />
        {/if}
      {/if}

      <!-- Centered Dimension Readout Pill -->
      <g transform="translate({anchor.x}, {anchor.y})">
        <rect
          x={-labelW / 2}
          y="-11"
          width={labelW}
          height="20"
          rx="6"
          fill="var(--measure-label-bg)"
          stroke="var(--measure-line)"
          stroke-width="1"
        />
        <text
          x="0"
          y="3"
          text-anchor="middle"
          fill="var(--measure-label-text)"
          font-size="10.5px"
          font-weight="600"
          font-family="Inter, monospace"
        >
          {m.formattedLabel}
        </text>
      </g>
    </g>
  {/each}

  <!-- Active OSNAP Indicator (Endpoint Square, Midpoint Triangle, Intersection X, Perpendicular ∟) -->
  {#if snapEngine.activeSnap}
    {@const snap = snapEngine.activeSnap}
    <g
      class="osnap-indicator"
      transform="translate({snap.x}, {snap.y})"
      data-snap-kind={snap.kind}
    >
      {#if snap.kind === 'endpoint'}
        <rect
          x="-5"
          y="-5"
          width="10"
          height="10"
          fill="none"
          stroke="var({snap.tokenVar})"
          stroke-width="2"
        />
      {:else if snap.kind === 'midpoint'}
        <polygon
          points="0,-6 6,5 -6,5"
          fill="none"
          stroke="var({snap.tokenVar})"
          stroke-width="2"
        />
      {:else if snap.kind === 'intersection'}
        <path
          d="M -5 -5 L 5 5 M -5 5 L 5 -5"
          stroke="var({snap.tokenVar})"
          stroke-width="2"
        />
      {:else if snap.kind === 'perpendicular'}
        <path
          d="M -5 -6 L -5 5 L 6 5 M -5 0 L 0 0 L 0 5"
          fill="none"
          stroke="var({snap.tokenVar})"
          stroke-width="1.8"
        />
      {:else}
        <circle
          cx="0"
          cy="0"
          r="4"
          fill="none"
          stroke="var({snap.tokenVar})"
          stroke-width="1.8"
        />
      {/if}
    </g>
  {/if}

  <!-- Interactive Translucent Digital Ruler -->
  {#if measureEngine.ruler.visible}
    {@const r = measureEngine.ruler}
    <g
      class="digital-ruler-widget"
      transform="translate({r.xPt}, {r.yPt}) rotate({r.angleDeg})"
      role="toolbar"
      tabindex="0"
      aria-label="Digital Ruler Straightedge"
      onpointerdown={startRulerMove}
      onpointermove={onRulerPointerMove}
      onpointerup={onRulerPointerUp}
      onpointercancel={onRulerPointerUp}
    >
      <rect
        x="0"
        y="0"
        width={r.lengthPt}
        height="44"
        rx="6"
        fill="var(--glass-bg)"
        stroke="var(--accent)"
        stroke-width="1.5"
      />
      <!-- Straightedge Top Guide Rail -->
      <line
        x1="0"
        y1="0"
        x2={r.lengthPt}
        y2="0"
        stroke="var(--measure-line)"
        stroke-width="2.5"
      />

      <!-- Graduated Calibrated Tick Marks -->
      {#each rulerTicks as tick}
        <line
          x1={tick.x}
          y1="0"
          x2={tick.x}
          y2={tick.major ? 14 : 7}
          stroke="var(--glass-text)"
          stroke-width={tick.major ? 1.4 : 0.9}
        />
        {#if tick.label}
          <text
            x={tick.x + 2}
            y="24"
            fill="var(--glass-text)"
            font-size="8.5px"
            font-family="Inter, monospace"
          >
            {tick.label}
          </text>
        {/if}
      {/each}

      <!-- Live Angle Badge -->
      <text
        x={r.lengthPt / 2}
        y="37"
        text-anchor="middle"
        fill="var(--accent)"
        font-size="10px"
        font-weight="600"
      >
        Ruler {r.angleDeg.toFixed(1)}° (Drag edge to draw collinear lines)
      </text>

      <!-- Rotation Handle on Right End -->
      <circle
        cx={r.lengthPt - 14}
        cy="22"
        r="8"
        fill="var(--accent)"
        stroke="var(--laser-core)"
        stroke-width="1.5"
        role="button"
        tabindex="0"
        aria-label="Rotate Digital Ruler"
        onpointerdown={startRulerRotate}
      />
    </g>
  {/if}
</g>

<style>
  .digital-ruler-widget {
    cursor: grab;
    pointer-events: auto;
  }

  .digital-ruler-widget:active {
    cursor: grabbing;
  }
</style>

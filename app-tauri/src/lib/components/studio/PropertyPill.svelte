<script lang="ts">
  import {
    tools,
    type StrokeStyle,
    type FillStyle,
    type LineStyleId,
    type LineEnding,
    type HatchPatternExtendedId,
    type ToolId,
  } from '$lib/core/toolEngine.svelte';
  import { commands } from '$lib/core/commandEngine.svelte';
  import { theme, type TokenName } from '$lib/core/themeContract.svelte';
  import {
    measureEngine,
    isMeasureTool,
    type CadUnit,
  } from '$lib/core/measureEngine.svelte';
  import { snapEngine } from '$lib/core/snapEngine.svelte';

  // Swatch tokens mapped strictly to the theme contract (zero hex literals)
  const SWATCH_TOKENS: { token: TokenName; label: string }[] = [
    { token: '--glass-text', label: 'Primary' },
    { token: '--accent', label: 'Blue' },
    { token: '--danger', label: 'Red' },
    { token: '--snap-indicator-midpoint', label: 'Green' },
    { token: '--snap-indicator-intersection', label: 'Orange' },
    { token: '--snap-indicator-perpendicular', label: 'Purple' },
    { token: '--measure-line', label: 'Cyan' },
  ];

  const WIDTH_PRESETS = [0.75, 1.5, 3.0, 6.0, 12.0];
  const OPACITY_PRESETS = [0.2, 0.4, 0.7, 1.0];

  const LINE_STYLES: { id: LineStyleId; label: string }[] = [
    { id: 'solid', label: 'Solid' },
    { id: 'dashed', label: 'Dashed' },
    { id: 'dotted', label: 'Dotted' },
    { id: 'dash-dot', label: 'Dash-Dot' },
  ];

  const LINE_ENDINGS: { id: LineEnding; label: string }[] = [
    { id: 'none', label: 'None' },
    { id: 'open-arrow', label: 'Open Arrow' },
    { id: 'closed-arrow', label: 'Closed Arrow' },
    { id: 'circle', label: 'Circle' },
    { id: 'diamond', label: 'Diamond' },
    { id: 'slash', label: 'Slash' },
    { id: 'butt', label: 'Butt' },
  ];

  const HATCH_PATTERNS: { id: HatchPatternExtendedId; label: string }[] = [
    { id: 'none', label: 'No Hatch' },
    { id: 'solid', label: 'Solid Fill' },
    { id: 'diagonal-steel', label: 'Diagonal Steel' },
    { id: 'crosshatch', label: 'Crosshatch' },
    { id: 'concrete', label: 'Concrete' },
    { id: 'timber', label: 'Timber' },
    { id: 'tiling', label: 'Tiling' },
    { id: 'masonry', label: 'Masonry' },
    { id: 'earth', label: 'Earth' },
    { id: 'insulation', label: 'Insulation' },
  ];

  const HATCH_SCALES = [0.5, 1.0, 1.5, 2.0];

  let showLineStyleControls = $derived(
    tools.currentToolId === 'shape' ||
      tools.currentToolId.startsWith('shape.') ||
      tools.currentToolId.startsWith('callout')
  );

  let showLineEndingControls = $derived(
    tools.currentToolId === 'shape.line' ||
      tools.currentToolId === 'shape.arrow' ||
      tools.currentToolId === 'shape.polyline' ||
      tools.currentToolId.startsWith('callout')
  );

  let showHatchControls = $derived(
    tools.currentToolId === 'shape' ||
      tools.currentToolId === 'shape.rect' ||
      tools.currentToolId === 'shape.ellipse' ||
      tools.currentToolId === 'shape.polygon' ||
      tools.currentToolId === 'shape.cloud' ||
      tools.currentToolId.startsWith('hatch')
  );

  let showMeasureControls = $derived(isMeasureTool(tools.currentToolId));

  let activePageScale = $derived(measureEngine.getScaleForPage(0));

  let takeoffSummaryText = $derived.by(() => {
    const entries = Object.entries(measureEngine.totalsByUnit);
    if (entries.length === 0) return 'No measurements';
    return entries.map(([u, val]) => `${val.toFixed(activePageScale.precision)} ${u}`).join(' | ');
  });

  const MEASURE_TOOLS: { id: ToolId; label: string }[] = [
    { id: 'measure.calibrate', label: 'Calibrate' },
    { id: 'measure.length', label: 'Length' },
    { id: 'measure.polylength', label: 'Polylength' },
    { id: 'measure.rectArea', label: 'Rect Area' },
    { id: 'measure.polyArea', label: 'Poly Area' },
    { id: 'measure.perimeter', label: 'Perimeter' },
    { id: 'measure.protractor', label: 'Protractor' },
  ];

  const CAD_UNITS: CadUnit[] = ['mm', 'cm', 'm', 'in', 'ft', 'pt'];

  function updateStroke(patch: Partial<StrokeStyle>) {
    if (!tools.activePreset) return;
    const updated = { ...tools.activePreset.stroke, ...patch };
    tools.updateActivePreset({ stroke: updated });
    commands.dispatch({
      kind: 'execute',
      command: 'style.apply',
      args: { presetId: tools.activePreset.id, stroke: updated },
    });
  }

  function updateFill(patch: Partial<FillStyle>) {
    if (!tools.activePreset) return;
    const updated = { ...tools.activePreset.fill, ...patch };
    tools.updateActivePreset({ fill: updated });
    commands.dispatch({
      kind: 'execute',
      command: 'style.apply',
      args: { presetId: tools.activePreset.id, fill: updated },
    });
  }

  function selectColorToken(token: TokenName) {
    const resolvedColor = theme.token(token);
    updateStroke({ colorToken: token, colorHex: resolvedColor });
  }
</script>

<div
  class="property-pill glass-panel"
  role="toolbar"
  tabindex={0}
  aria-label="Tool Property Pill"
  onpointerdown={(e) => e.stopPropagation()}
>
  <!-- Active Tool Badge -->
  <div class="pill-section tool-badge-section">
    <span class="tool-label" data-testid="active-tool-label">{tools.activeToolLabel}</span>
  </div>

  {#if tools.supportsStrokeProperties && tools.activePreset}
    <div class="pill-divider"></div>

    <!-- Color Swatches (Theme Contract Driven) -->
    <div class="pill-section color-section" aria-label="Color Palette">
      {#each SWATCH_TOKENS as swatch}
        <button
          type="button"
          class="swatch-btn"
          class:active={tools.activePreset.stroke.colorToken === swatch.token}
          style:background="var({swatch.token})"
          onclick={() => selectColorToken(swatch.token)}
          title="{swatch.label} Color"
          aria-label="{swatch.label} Color"
        ></button>
      {/each}
    </div>

    <div class="pill-divider"></div>

    <!-- Stroke Width Presets -->
    <div class="pill-section width-section" aria-label="Stroke Thickness">
      {#each WIDTH_PRESETS as w}
        <button
          type="button"
          class="width-btn"
          class:active={Math.abs(tools.activePreset.stroke.widthPt - w) < 0.1}
          onclick={() => updateStroke({ widthPt: w })}
          title="{w} pt stroke"
          aria-label="{w} pt stroke"
        >
          <span
            class="width-dot"
            style:width="{Math.min(18, Math.max(3, w * 1.5))}px"
            style:height="{Math.min(18, Math.max(3, w * 1.5))}px"
          ></span>
        </button>
      {/each}
    </div>

    <div class="pill-divider"></div>

    <!-- Opacity Selector -->
    <div class="pill-section opacity-section" aria-label="Opacity Presets">
      {#each OPACITY_PRESETS as op}
        <button
          type="button"
          class="opacity-btn"
          class:active={Math.abs(tools.activePreset.stroke.opacity - op) < 0.05}
          onclick={() => updateStroke({ opacity: op })}
          title="{Math.round(op * 100)}% Opacity"
          aria-label="{Math.round(op * 100)}% Opacity"
        >
          {Math.round(op * 100)}%
        </button>
      {/each}
    </div>

    <!-- Line Style Selector for Shapes & Polylines -->
    {#if showLineStyleControls}
      <div class="pill-divider"></div>
      <div class="pill-section line-style-section" aria-label="Line Style">
        <select
          class="hatch-select"
          value={tools.activePreset.stroke.lineStyle ?? 'solid'}
          onchange={(e) =>
            updateStroke({ lineStyle: (e.target as HTMLSelectElement).value as LineStyleId })}
          aria-label="Select Line Style"
        >
          {#each LINE_STYLES as ls}
            <option value={ls.id}>{ls.label}</option>
          {/each}
        </select>
      </div>
    {/if}

    <!-- Line Endings Selector for Lines, Arrows & Polylines -->
    {#if showLineEndingControls}
      <div class="pill-divider"></div>
      <div class="pill-section line-ending-section" aria-label="Line Endings">
        <select
          class="hatch-select"
          value={tools.activePreset.stroke.startEnd}
          onchange={(e) =>
            updateStroke({ startEnd: (e.target as HTMLSelectElement).value as LineEnding })}
          aria-label="Select Start Line Ending"
          title="Start Ending"
        >
          {#each LINE_ENDINGS as le}
            <option value={le.id}>Start: {le.label}</option>
          {/each}
        </select>
        <select
          class="hatch-select"
          value={tools.activePreset.stroke.endEnd}
          onchange={(e) =>
            updateStroke({ endEnd: (e.target as HTMLSelectElement).value as LineEnding })}
          aria-label="Select End Line Ending"
          title="End Ending"
        >
          {#each LINE_ENDINGS as le}
            <option value={le.id}>End: {le.label}</option>
          {/each}
        </select>
      </div>
    {/if}

    <!-- Procedural Hatch Pattern & Scale Selector for Closed Shapes / Hatch Tool -->
    {#if showHatchControls}
      <div class="pill-divider"></div>
      <div class="pill-section hatch-section" aria-label="Hatch Pattern">
        <select
          class="hatch-select"
          value={tools.activePreset.fill.hatch}
          onchange={(e) => {
            const val = (e.target as HTMLSelectElement).value as HatchPatternExtendedId;
            updateFill({
              hatch: val,
              kind: val === 'none' ? 'none' : val === 'solid' ? 'solid' : 'hatch',
            });
          }}
          aria-label="Select Hatch Pattern"
        >
          {#each HATCH_PATTERNS as pat}
            <option value={pat.id}>{pat.label}</option>
          {/each}
        </select>
        <select
          class="hatch-select"
          value={tools.activePreset.fill.hatchScale}
          onchange={(e) =>
            updateFill({ hatchScale: parseFloat((e.target as HTMLSelectElement).value) || 1.0 })}
          aria-label="Select Hatch Scale"
          title="Hatch Scale"
        >
          {#each HATCH_SCALES as sc}
            <option value={sc}>{sc}x</option>
          {/each}
        </select>
      </div>
    {/if}

    <!-- Phase D5 CAD Calibration, Measurement, OSNAP & Digital Ruler Controls -->
    {#if showMeasureControls}
      <div class="pill-divider"></div>
      <div class="pill-section measure-controls-section" aria-label="CAD Measurement Controls">
        <!-- Sub-tool selector -->
        <select
          class="hatch-select"
          value={tools.currentToolId === 'measure' ? 'measure.length' : tools.currentToolId}
          onchange={(e) => tools.selectTool((e.target as HTMLSelectElement).value as ToolId)}
          aria-label="Select Measurement Tool"
          title="Measurement Mode"
        >
          {#each MEASURE_TOOLS as mt}
            <option value={mt.id}>{mt.label}</option>
          {/each}
        </select>

        <!-- Scale Status Badge (click to open Calibration Modal) -->
        <button
          type="button"
          class="toggle-btn"
          onclick={() => measureEngine.openCalibrationModal(0, activePageScale.ptDistance)}
          title="Calibrate Page Scale"
        >
          Scale: {activePageScale.scalePresetLabel ?? `${activePageScale.realDistance} ${activePageScale.unit}`}
        </button>

        <!-- Unit Switcher -->
        <select
          class="hatch-select"
          value={activePageScale.unit}
          onchange={(e) =>
            measureEngine.updateCurrentPageScalePatch(0, {
              unit: (e.target as HTMLSelectElement).value as CadUnit,
            })}
          aria-label="Select CAD Unit"
          title="Measurement Unit"
        >
          {#each CAD_UNITS as u}
            <option value={u}>{u}</option>
          {/each}
        </select>

        <!-- Precision Selector -->
        <select
          class="hatch-select"
          value={activePageScale.precision}
          onchange={(e) =>
            measureEngine.updateCurrentPageScalePatch(0, {
              precision: Number((e.target as HTMLSelectElement).value) as 0 | 1 | 2 | 3,
            })}
          aria-label="Select Decimal Precision"
          title="Decimal Precision"
        >
          <option value={0}>0</option>
          <option value={1}>0.1</option>
          <option value={2}>0.01</option>
          <option value={3}>0.001</option>
        </select>

        <!-- Depth Input for Volume Calculation -->
        <input
          type="number"
          step="any"
          min="0"
          class="depth-input"
          value={activePageScale.depthValue}
          onchange={(e) =>
            measureEngine.updateCurrentPageScalePatch(0, {
              depthValue: Math.max(0, parseFloat((e.target as HTMLInputElement).value) || 0),
            })}
          aria-label="Volume Depth"
          title="Volume Depth ({activePageScale.unit})"
        />

        <!-- Endpoint Tick Style -->
        <select
          class="hatch-select"
          value={measureEngine.endpointStyle}
          onchange={(e) => {
            measureEngine.endpointStyle = (e.target as HTMLSelectElement)
              .value as typeof measureEngine.endpointStyle;
            measureEngine.recalculateAllMeasurements();
          }}
          aria-label="Select Dimension Tick Style"
          title="Dimension Tick Style"
        >
          <option value="tick">45° Tick</option>
          <option value="open-arrow">Open Arrow</option>
          <option value="closed-arrow">Closed Arrow</option>
          <option value="circle">Dot</option>
        </select>

        <!-- Tape Mode Toggle -->
        <button
          type="button"
          class="toggle-btn"
          class:active={measureEngine.tapeMode}
          onclick={() => {
            measureEngine.tapeMode = !measureEngine.tapeMode;
          }}
          title="Transient Tape Measure (Does not persist on release)"
        >
          Tape
        </button>

        <!-- OSNAP Toggle -->
        <button
          type="button"
          class="toggle-btn"
          class:active={snapEngine.enabled}
          onclick={() => {
            snapEngine.enabled = !snapEngine.enabled;
            if (!snapEngine.enabled) snapEngine.clearSnap();
          }}
          title="Toggle CAD Vector OSNAP (Endpoint, Midpoint, Intersection, Perpendicular)"
        >
          OSNAP
        </button>

        <!-- Digital Ruler Toggle -->
        <button
          type="button"
          class="toggle-btn"
          class:active={measureEngine.ruler.visible}
          onclick={() => measureEngine.toggleRuler()}
          title="Toggle Interactive Digital Ruler"
        >
          Ruler
        </button>

        <!-- Live Takeoff Summary Pill -->
        <span class="takeoff-summary-badge" title="Quantity Takeoff Totals">
          Σ {takeoffSummaryText}
        </span>
      </div>
    {/if}
  {:else if tools.supportsEraserMode}
    <div class="pill-divider"></div>
    <div class="pill-section eraser-mode-section" aria-label="Eraser Mode Toggle">
      <button
        type="button"
        class="toggle-btn"
        class:active={tools.activeToolId === 'eraser.partial'}
        onclick={() => tools.selectTool('eraser.partial')}
      >
        Partial
      </button>
      <button
        type="button"
        class="toggle-btn"
        class:active={tools.activeToolId === 'eraser.whole'}
        onclick={() => tools.selectTool('eraser.whole')}
      >
        Whole
      </button>
    </div>
  {:else if tools.activeToolId === 'pan'}
    <div class="pill-divider"></div>
    <div class="pill-section status-hint-section">
      <span class="status-hint">Drag canvas or hold Space to pan</span>
    </div>
  {:else if tools.activeToolId.startsWith('select')}
    <div class="pill-divider"></div>
    <div class="pill-section eraser-mode-section" aria-label="Selection Mode Toggle">
      <button
        type="button"
        class="toggle-btn"
        class:active={tools.activeToolId === 'select' || tools.activeToolId === 'select.marquee'}
        onclick={() => tools.selectTool('select.marquee')}
      >
        Marquee
      </button>
      <button
        type="button"
        class="toggle-btn"
        class:active={tools.activeToolId === 'select.lasso'}
        onclick={() => tools.selectTool('select.lasso')}
      >
        Lasso
      </button>
      <button
        type="button"
        class="toggle-btn"
        class:active={tools.activeToolId === 'select.snapshot'}
        onclick={() => tools.selectTool('select.snapshot')}
      >
        Snapshot
      </button>
    </div>
  {:else if tools.activeToolId === 'laser'}
    <div class="pill-divider"></div>
    <div class="pill-section status-hint-section">
      <span class="status-hint">Temporary laser beam pointer</span>
    </div>
  {/if}
</div>

<style>
  .property-pill {
    position: absolute;
    bottom: 24px;
    left: 50%;
    transform: translateX(-50%);
    z-index: 150;
    pointer-events: auto;
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
    font-size: 12px;
  }

  .pill-section {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .tool-label {
    font-weight: 600;
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    opacity: 0.8;
  }

  .status-hint {
    font-size: 11px;
    opacity: 0.7;
    white-space: nowrap;
  }

  .pill-divider {
    width: 1px;
    height: 18px;
    background: var(--glass-border);
  }

  /* Color Swatches */
  .swatch-btn {
    width: 18px;
    height: 18px;
    border-radius: 50%;
    border: 2px solid transparent;
    cursor: pointer;
    box-shadow: 0 1px 3px var(--glass-border);
    transition: transform 0.1s ease, border-color 0.1s ease;
  }

  .swatch-btn:hover {
    transform: scale(1.15);
  }

  .swatch-btn.active {
    border-color: var(--focus-ring);
    transform: scale(1.25);
    box-shadow: 0 0 0 2px var(--glass-bg), 0 0 0 4px var(--accent);
  }

  /* Width Dots */
  .width-btn {
    width: 22px;
    height: 22px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    transition: background 0.1s ease;
  }

  .width-btn:hover {
    background: var(--canvas-selection);
  }

  .width-btn.active {
    background: var(--accent);
  }

  .width-dot {
    background: var(--glass-text);
    border-radius: 50%;
    transition: background 0.1s ease;
  }

  .width-btn.active .width-dot {
    background: var(--laser-core);
  }

  /* Opacity Buttons */
  .opacity-btn {
    border: none;
    background: transparent;
    color: var(--glass-text);
    padding: 2px 6px;
    border-radius: 4px;
    font-size: 11px;
    font-weight: 500;
    cursor: pointer;
    transition: background 0.1s ease;
  }

  .opacity-btn:hover {
    background: var(--canvas-selection);
  }

  .opacity-btn.active {
    background: var(--accent);
    color: var(--laser-core);
    font-weight: 600;
  }

  /* Eraser Toggle Buttons */
  .toggle-btn {
    border: none;
    background: transparent;
    color: var(--glass-text);
    padding: 3px 8px;
    border-radius: 6px;
    font-size: 11px;
    font-weight: 500;
    cursor: pointer;
    transition: background 0.1s ease;
  }

  .toggle-btn:hover {
    background: var(--canvas-selection);
  }

  .toggle-btn.active {
    background: var(--accent);
    color: var(--laser-core);
    font-weight: 600;
  }

  /* Hatch Select */
  .hatch-select {
    background: var(--canvas-selection);
    color: var(--glass-text);
    border: 1px solid var(--glass-border);
    border-radius: 6px;
    padding: 2px 6px;
    font-size: 11px;
    cursor: pointer;
    outline: none;
  }

  .depth-input {
    width: 48px;
    background: var(--canvas-selection);
    color: var(--glass-text);
    border: 1px solid var(--glass-border);
    border-radius: 6px;
    padding: 2px 6px;
    font-size: 11px;
    outline: none;
  }

  .takeoff-summary-badge {
    padding: 2px 8px;
    background: var(--canvas-selection);
    border: 1px solid var(--glass-border);
    border-radius: 9999px;
    font-size: 10.5px;
    font-weight: 600;
    white-space: nowrap;
  }
</style>

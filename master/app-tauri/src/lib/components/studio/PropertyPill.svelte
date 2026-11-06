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
  import Button from '$lib/components/ui/Button.svelte';
  import ColorSwatch from '$lib/components/ui/ColorSwatch.svelte';
  import Select from '$lib/components/ui/Select.svelte';
  import Input from '$lib/components/ui/Input.svelte';
  import { MATH_TEXT_LABS, labsVisible, selectToolIfAllowed, studioToolLabel } from './labs.svelte';

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
    { id: 'dash-dot', label: 'Dash-dot' },
  ];

  const LINE_ENDINGS: { id: LineEnding; label: string }[] = [
    { id: 'none', label: 'None' },
    { id: 'open-arrow', label: 'Open arrow' },
    { id: 'closed-arrow', label: 'Closed arrow' },
    { id: 'circle', label: 'Circle' },
    { id: 'diamond', label: 'Diamond' },
    { id: 'slash', label: 'Slash' },
    { id: 'butt', label: 'Butt' },
  ];

  const HATCH_PATTERNS: { id: HatchPatternExtendedId; label: string }[] = [
    { id: 'none', label: 'No hatch' },
    { id: 'solid', label: 'Solid fill' },
    { id: 'diagonal-steel', label: 'Diagonal steel' },
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

  let showInsertControls = $derived(
    tools.currentToolId.startsWith('insert.') ||
      tools.currentToolId.startsWith('callout.') ||
      tools.currentToolId === 'text.review'
  );

  const INSERT_TOOLS: { id: ToolId; label: string }[] = [
    { id: 'insert.textBox', label: 'Text box' },
    { id: 'insert.note', label: 'Sticky note' },
    { id: 'callout.single', label: 'Callout' },
    { id: 'callout.multi', label: 'Multi-callout' },
    { id: 'insert.latex', label: MATH_TEXT_LABS.label },
    { id: 'insert.signature', label: 'Signature' },
    { id: 'insert.audio', label: 'Audio note' },
    { id: 'insert.stamp', label: 'Stamp' },
    { id: 'insert.image', label: 'Image' },
  ];

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
    { id: 'measure.rectArea', label: 'Rect area' },
    { id: 'measure.polyArea', label: 'Poly area' },
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

  // ui/Select works with strings; these option lists are built once from the tables above.
  const toOptions = <T extends { id: string; label: string }>(list: T[]) => list.map((x) => ({ value: x.id, label: x.label }));
  const LINE_STYLE_OPTIONS = toOptions(LINE_STYLES);
  const START_ENDING_OPTIONS = LINE_ENDINGS.map((l) => ({ value: l.id, label: `Start: ${l.label}` }));
  const END_ENDING_OPTIONS = LINE_ENDINGS.map((l) => ({ value: l.id, label: `End: ${l.label}` }));
  const HATCH_OPTIONS = toOptions(HATCH_PATTERNS);
  const HATCH_SCALE_OPTIONS = HATCH_SCALES.map((v) => ({ value: String(v), label: `${v}x` }));
  const MEASURE_TOOL_OPTIONS = toOptions(MEASURE_TOOLS);
  const UNIT_OPTIONS = CAD_UNITS.map((u) => ({ value: u, label: u }));
  const PRECISION_OPTIONS = [
    { value: '0', label: '0' },
    { value: '1', label: '0.1' },
    { value: '2', label: '0.01' },
    { value: '3', label: '0.001' },
  ];
  const TICK_OPTIONS = [
    { value: 'tick', label: '45° tick' },
    { value: 'open-arrow', label: 'Open arrow' },
    { value: 'closed-arrow', label: 'Closed arrow' },
    { value: 'circle', label: 'Dot' },
  ];
  let insertOptions = $derived(toOptions(INSERT_TOOLS.filter((it) => labsVisible(it.id))));

  function selectColorToken(token: TokenName) {
    const resolvedColor = theme.token(token);
    updateStroke({ colorToken: token, colorHex: resolvedColor });
  }
</script>

<div
  class="property-pill glass-panel"
  role="toolbar"
  tabindex={0}
  aria-label="Tool property pill"
  onpointerdown={(e) => e.stopPropagation()}
>
  <!-- Active Tool Badge -->
  <div class="pill-section tool-badge-section">
    <span class="tool-label" data-testid="active-tool-label">{studioToolLabel(tools.currentToolId, tools.activeToolLabel)}</span>
  </div>

  {#if tools.supportsStrokeProperties && tools.activePreset}
    <div class="pill-divider"></div>

    <!-- Color Swatches (Theme Contract Driven) -->
    <div class="pill-section color-section" role="group" aria-label="Color palette">
      {#each SWATCH_TOKENS as swatch}
        <ColorSwatch
          color="var({swatch.token})"
          label="{swatch.label} color"
          pressed={tools.activePreset.stroke.colorToken === swatch.token}
          onclick={() => selectColorToken(swatch.token)}
        />
      {/each}
    </div>

    <div class="pill-divider"></div>

    <!-- Stroke Width Presets -->
    <div class="pill-section width-section" role="group" aria-label="Stroke thickness">
      {#each WIDTH_PRESETS as w}
        <Button
          variant="plain"
          size="sm"
          pressed={Math.abs(tools.activePreset.stroke.widthPt - w) < 0.1}
          ariaLabel="{w} pt stroke"
          title="{w} pt stroke"
          onclick={() => updateStroke({ widthPt: w })}
        >
          <span
            class="width-dot"
            style:width="{Math.min(18, Math.max(3, w * 1.5))}px"
            style:height="{Math.min(18, Math.max(3, w * 1.5))}px"
          ></span>
        </Button>
      {/each}
    </div>

    <div class="pill-divider"></div>

    <!-- Opacity Selector -->
    <div class="pill-section opacity-section" role="group" aria-label="Opacity presets">
      {#each OPACITY_PRESETS as op}
        <Button
          variant="plain"
          size="sm"
          pressed={Math.abs(tools.activePreset.stroke.opacity - op) < 0.05}
          ariaLabel="{Math.round(op * 100)}% opacity"
          title="{Math.round(op * 100)}% opacity"
          onclick={() => updateStroke({ opacity: op })}
        >
          {Math.round(op * 100)}%
        </Button>
      {/each}
    </div>

    <!-- Line Style Selector for Shapes & Polylines -->
    {#if showLineStyleControls}
      <div class="pill-divider"></div>
      <div class="pill-section line-style-section" role="group" aria-label="Line style">
        <Select
          compact
          hideLabel
          label="Line style"
          options={LINE_STYLE_OPTIONS}
          bind:value={() => tools.activePreset?.stroke.lineStyle ?? 'solid', (v) => updateStroke({ lineStyle: v as LineStyleId })}
        />
      </div>
    {/if}

    <!-- Line Endings Selector for Lines, Arrows & Polylines -->
    {#if showLineEndingControls}
      <div class="pill-divider"></div>
      <div class="pill-section line-ending-section" role="group" aria-label="Line endings">
        <Select
          compact
          hideLabel
          label="Start line ending"
          options={START_ENDING_OPTIONS}
          bind:value={() => tools.activePreset?.stroke.startEnd ?? 'none', (v) => updateStroke({ startEnd: v as LineEnding })}
        />
        <Select
          compact
          hideLabel
          label="End line ending"
          options={END_ENDING_OPTIONS}
          bind:value={() => tools.activePreset?.stroke.endEnd ?? 'none', (v) => updateStroke({ endEnd: v as LineEnding })}
        />
      </div>
    {/if}

    <!-- Procedural Hatch Pattern & Scale Selector for Closed Shapes / Hatch Tool -->
    {#if showHatchControls}
      <div class="pill-divider"></div>
      <div class="pill-section hatch-section" role="group" aria-label="Hatch pattern">
        <Select
          compact
          hideLabel
          label="Hatch pattern"
          data-testid="hatch-pattern"
          options={HATCH_OPTIONS}
          bind:value={() => tools.activePreset?.fill.hatch ?? 'none', (val) =>
            updateFill({
              hatch: val as HatchPatternExtendedId,
              kind: val === 'none' ? 'none' : val === 'solid' ? 'solid' : 'hatch',
            })}
        />
        <Select
          compact
          hideLabel
          label="Hatch scale"
          options={HATCH_SCALE_OPTIONS}
          bind:value={() => String(tools.activePreset?.fill.hatchScale ?? 1), (v) => updateFill({ hatchScale: parseFloat(v) || 1.0 })}
        />
      </div>
    {/if}

    <!-- Phase D5 CAD Calibration, Measurement, OSNAP & Digital Ruler Controls -->
    {#if showMeasureControls}
      <div class="pill-divider"></div>
      <div class="pill-section measure-controls-section" role="group" aria-label="CAD measurement controls">
        <Select
          compact
          hideLabel
          label="Measurement tool"
          options={MEASURE_TOOL_OPTIONS}
          bind:value={() => (tools.currentToolId === 'measure' ? 'measure.length' : tools.currentToolId), (v) => tools.selectTool(v as ToolId)}
        />

        <!-- Scale status (click to open the calibration dialog) -->
        <Button
          variant="secondary"
          size="sm"
          title="Calibrate page scale"
          onclick={() => measureEngine.openCalibrationModal(0, activePageScale.ptDistance)}
        >
          Scale: {activePageScale.scalePresetLabel ?? `${activePageScale.realDistance} ${activePageScale.unit}`}
        </Button>

        <Select
          compact
          hideLabel
          label="Measurement unit"
          options={UNIT_OPTIONS}
          bind:value={() => activePageScale.unit, (v) => measureEngine.updateCurrentPageScalePatch(0, { unit: v as CadUnit })}
        />

        <Select
          compact
          hideLabel
          label="Decimal precision"
          options={PRECISION_OPTIONS}
          bind:value={() => String(activePageScale.precision), (v) =>
            measureEngine.updateCurrentPageScalePatch(0, { precision: Number(v) as 0 | 1 | 2 | 3 })}
        />

        <!-- Depth for volume calculation -->
        <Input
          compact
          hideLabel
          type="number"
          label="Volume depth ({activePageScale.unit})"
          class="depth-field"
          bind:value={() => String(activePageScale.depthValue), (v) =>
            measureEngine.updateCurrentPageScalePatch(0, { depthValue: Math.max(0, parseFloat(v) || 0) })}
        />

        <Select
          compact
          hideLabel
          label="Dimension tick style"
          options={TICK_OPTIONS}
          bind:value={() => measureEngine.endpointStyle, (v) => {
            measureEngine.endpointStyle = v as typeof measureEngine.endpointStyle;
            measureEngine.recalculateAllMeasurements();
          }}
        />

        <Button
          variant="plain"
          size="sm"
          pressed={measureEngine.tapeMode}
          title="Transient tape measure (does not persist on release)"
          onclick={() => {
            measureEngine.tapeMode = !measureEngine.tapeMode;
          }}
        >
          Tape
        </Button>

        <Button
          variant="plain"
          size="sm"
          pressed={snapEngine.enabled}
          title="Toggle vector snapping (endpoint, midpoint, intersection, perpendicular)"
          onclick={() => {
            snapEngine.enabled = !snapEngine.enabled;
            if (!snapEngine.enabled) snapEngine.clearSnap();
          }}
        >
          OSNAP
        </Button>

        <Button
          variant="plain"
          size="sm"
          pressed={measureEngine.ruler.visible}
          title="Toggle the interactive digital ruler"
          onclick={() => measureEngine.toggleRuler()}
        >
          Ruler
        </Button>

        <!-- Live takeoff summary -->
        <span class="takeoff-summary-badge" title="Quantity takeoff totals">
          Σ {takeoffSummaryText}
        </span>
      </div>
    {/if}

    <!-- Phase D6 Insert Suite Sub-Tool Switcher -->
    {#if showInsertControls}
      <div class="pill-divider"></div>
      <div class="pill-section insert-controls-section" role="group" aria-label="Insert and callout suite">
        <Select
          compact
          hideLabel
          label="Insert or callout tool"
          options={insertOptions}
          bind:value={() => tools.currentToolId, (v) => selectToolIfAllowed(v)}
        />
      </div>
    {/if}
  {:else if tools.supportsEraserMode}
    <div class="pill-divider"></div>
    <div class="pill-section eraser-mode-section" role="group" aria-label="Eraser mode">
      <Button variant="plain" size="sm" pressed={tools.activeToolId === 'eraser.partial'} onclick={() => tools.selectTool('eraser.partial')}>Partial</Button>
      <Button variant="plain" size="sm" pressed={tools.activeToolId === 'eraser.whole'} onclick={() => tools.selectTool('eraser.whole')}>Whole</Button>
    </div>
  {:else if tools.activeToolId === 'pan'}
    <div class="pill-divider"></div>
    <div class="pill-section status-hint-section">
      <span class="status-hint">Drag canvas or hold Space to pan</span>
    </div>
  {:else if tools.activeToolId.startsWith('select')}
    <div class="pill-divider"></div>
    <div class="pill-section eraser-mode-section" role="group" aria-label="Selection mode">
      <Button variant="plain" size="sm" pressed={tools.activeToolId === 'select' || tools.activeToolId === 'select.marquee'} onclick={() => tools.selectTool('select.marquee')}>Marquee</Button>
      <Button variant="plain" size="sm" pressed={tools.activeToolId === 'select.lasso'} onclick={() => tools.selectTool('select.lasso')}>Lasso</Button>
      <Button variant="plain" size="sm" pressed={tools.activeToolId === 'select.snapshot'} onclick={() => tools.selectTool('select.snapshot')}>Snapshot</Button>
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
    bottom: var(--space-6);
    left: 50%;
    transform: translateX(-50%);
    z-index: 150;
    pointer-events: auto;
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2) var(--space-3);
    background: var(--surface-dialog);
    border: 1px solid var(--glass-border);
    box-shadow: var(--glass-shadow);
    border-radius: var(--radius-pill);
    color: var(--text-primary);
    font-size: var(--font-size-footnote);
  }

  .pill-section {
    display: flex;
    align-items: center;
    gap: var(--space-1);
  }

  .tool-label {
    font-weight: 600;
    font-size: var(--font-size-caption);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .status-hint {
    font-size: var(--font-size-caption);
    color: var(--text-secondary);
    white-space: nowrap;
  }

  .pill-divider {
    width: 1px;
    height: 18px;
    background: var(--glass-border);
  }

  /* Pressed preset buttons read as primary text on the tint (the accent blue is under 4.5:1 on dark). */
  .property-pill :global(.lp-btn.lp-btn--plain) {
    color: var(--text-primary);
  }

  .width-dot {
    background: var(--text-primary);
    border-radius: var(--radius-pill);
  }

  :global(.depth-field) {
    width: 5.5em;
  }

  .takeoff-summary-badge {
    padding: var(--space-1) var(--space-2);
    background: var(--glass-bg-input);
    border: 1px solid var(--glass-border);
    border-radius: var(--radius-pill);
    font-size: var(--font-size-caption);
    font-weight: 600;
    white-space: nowrap;
  }
</style>

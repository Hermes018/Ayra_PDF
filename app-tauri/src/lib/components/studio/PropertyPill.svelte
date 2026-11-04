<script lang="ts">
  import { tools, type StrokeStyle, type FillStyle } from '$lib/core/toolEngine.svelte';
  import { commands } from '$lib/core/commandEngine.svelte';
  import { theme, type TokenName } from '$lib/core/themeContract.svelte';
  import type { HatchPatternId } from '$lib/bindings/HatchPatternId';

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

  const HATCH_PATTERNS: { id: HatchPatternId; label: string }[] = [
    { id: 'concrete', label: 'Concrete' },
    { id: 'steel', label: 'Steel' },
    { id: 'tiling', label: 'Tiling' },
    { id: 'masonry', label: 'Masonry' },
    { id: 'timber', label: 'Timber' },
  ];

  function updateStroke(patch: Partial<StrokeStyle>) {
    if (!tools.activePreset) return;
    const updated = { ...tools.activePreset.stroke, ...patch };
    tools.updateActivePreset({ stroke: updated });
    commands.dispatch({
      kind: 'execute',
      command: 'style.apply',
      args: { presetId: tools.activePreset.id, stroke: updated }
    });
  }

  function updateFill(patch: Partial<FillStyle>) {
    if (!tools.activePreset) return;
    const updated = { ...tools.activePreset.fill, ...patch };
    tools.updateActivePreset({ fill: updated });
    commands.dispatch({
      kind: 'execute',
      command: 'style.apply',
      args: { presetId: tools.activePreset.id, fill: updated }
    });
  }

  function selectColorToken(token: TokenName) {
    const resolvedColor = theme.token(token);
    updateStroke({ colorToken: token, colorHex: resolvedColor });
  }
</script>

<div class="property-pill glass-panel" role="toolbar" aria-label="Tool Property Pill">
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

    <!-- Optional Hatch Selector for Shapes/Hatch Tool -->
    {#if tools.activeToolId === 'shape' || tools.activeToolId.startsWith('shape') || tools.activeToolId.startsWith('hatch')}
      <div class="pill-divider"></div>
      <div class="pill-section hatch-section" aria-label="Hatch Pattern">
        <select
          class="hatch-select"
          value={tools.activePreset.fill.hatch}
          onchange={(e) => updateFill({ hatch: (e.target as HTMLSelectElement).value as HatchPatternId, kind: 'hatch' })}
          aria-label="Select Hatch Pattern"
        >
          <option value="solid">Solid Fill</option>
          {#each HATCH_PATTERNS as pat}
            <option value={pat.id}>{pat.label}</option>
          {/each}
        </select>
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
  {:else if tools.activeToolId === 'select'}
    <div class="pill-divider"></div>
    <div class="pill-section status-hint-section">
      <span class="status-hint">Drag marquee or click to select annotations</span>
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
    z-index: 110;
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
</style>

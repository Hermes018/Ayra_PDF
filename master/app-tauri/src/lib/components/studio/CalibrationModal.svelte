<script lang="ts">
  import {
    measureEngine,
    ARCHITECTURAL_SCALE_PRESETS,
    type CadUnit,
    type PageScaleConfig,
  } from '$lib/core/measureEngine.svelte';
  import { viewport } from '$lib/core/viewportEngine.svelte';
  import StudioIcon from './StudioIcon.svelte';

  let {
    onSaveScales,
  }: {
    onSaveScales?: (scales: PageScaleConfig[]) => void;
  } = $props();

  const UNITS: { id: CadUnit; label: string }[] = [
    { id: 'mm', label: 'Millimeters (mm)' },
    { id: 'cm', label: 'Centimeters (cm)' },
    { id: 'm', label: 'Meters (m)' },
    { id: 'in', label: 'Inches (in)' },
    { id: 'ft', label: 'Feet (ft)' },
    { id: 'pt', label: 'PDF Points (pt)' },
  ];

  const PRECISIONS: (0 | 1 | 2 | 3)[] = [0, 1, 2, 3];

  let realDistance = $state<number>(1.0);
  let unit = $state<CadUnit>('m');
  let precision = $state<0 | 1 | 2 | 3>(2);
  let depthValue = $state<number>(0);
  let selectedPresetId = $state<string>('custom');
  let applyScope = $state<'page' | 'all'>('page');

  $effect(() => {
    const modal = measureEngine.calibrationModal;
    if (modal?.open) {
      const existing = measureEngine.getScaleForPage(modal.pageIndex);
      unit = existing.unit;
      precision = existing.precision;
      depthValue = existing.depthValue;
      const ratio = existing.realDistance / Math.max(existing.ptDistance, 1e-6);
      realDistance = Number((modal.measuredPt * ratio).toFixed(3));
      selectedPresetId = 'custom';
    }
  });

  function applyPreset(presetId: string) {
    selectedPresetId = presetId;
    const found = ARCHITECTURAL_SCALE_PRESETS.find((p) => p.id === presetId);
    if (!found || !measureEngine.calibrationModal) return;
    unit = found.unit;
    const ratio = found.realDistance / Math.max(found.ptDistance, 1e-6);
    realDistance = Number((measureEngine.calibrationModal.measuredPt * ratio).toFixed(3));
  }

  function commitCalibration() {
    const modal = measureEngine.calibrationModal;
    if (!modal) return;

    const ptDist = Math.max(1, modal.measuredPt);
    const cleanReal = Math.max(0.0001, Number(realDistance) || 1);
    const presetObj = ARCHITECTURAL_SCALE_PRESETS.find((p) => p.id === selectedPresetId);

    const config: PageScaleConfig = {
      pageIndex: modal.pageIndex,
      ptDistance: ptDist,
      realDistance: cleanReal,
      unit,
      precision,
      scalePresetLabel: presetObj
        ? presetObj.label
        : `${ptDist.toFixed(1)} pt = ${cleanReal} ${unit}`,
      depthValue: Math.max(0, Number(depthValue) || 0),
    };

    const totalPages = Math.max(1, viewport.pages.length);
    measureEngine.setPageScale(config, applyScope === 'all', totalPages);
    measureEngine.closeCalibrationModal();
    onSaveScales?.(Object.values(measureEngine.scales));
  }
</script>

{#if measureEngine.calibrationModal?.open}
  <div
    class="modal-backdrop"
    role="dialog"
    tabindex="-1"
    aria-label="Page Scale Calibration Dialog"
    onpointerdown={(e) => e.stopPropagation()}
  >
    <div class="calibration-card glass-panel">
      <div class="card-header">
        <div class="header-title">
          <StudioIcon name="measure" size={16} />
          <span>Calibrate Page Scale (Page {measureEngine.calibrationModal.pageIndex + 1})</span>
        </div>
        <button
          type="button"
          class="close-btn"
          onclick={() => measureEngine.closeCalibrationModal()}
          aria-label="Close Calibration Dialog"
        >
          <StudioIcon name="close" size={14} />
        </button>
      </div>

      <div class="measured-summary">
        <span>Measured Reference Span:</span>
        <strong>{measureEngine.calibrationModal.measuredPt.toFixed(2)} pt</strong>
      </div>

      <label class="field-row">
        <span>Architectural Scale Preset:</span>
        <select
          class="modal-select"
          value={selectedPresetId}
          onchange={(e) => applyPreset((e.target as HTMLSelectElement).value)}
        >
          <option value="custom">Custom Two-Point Calibration</option>
          {#each ARCHITECTURAL_SCALE_PRESETS as preset}
            <option value={preset.id}>{preset.label}</option>
          {/each}
        </select>
      </label>

      <div class="field-grid">
        <label class="field-row">
          <span>Known Real Distance:</span>
          <input
            type="number"
            step="any"
            min="0.001"
            class="modal-input"
            bind:value={realDistance}
          />
        </label>

        <label class="field-row">
          <span>Measurement Unit:</span>
          <select class="modal-select" bind:value={unit}>
            {#each UNITS as u}
              <option value={u.id}>{u.label}</option>
            {/each}
          </select>
        </label>
      </div>

      <div class="field-grid">
        <label class="field-row">
          <span>Decimal Precision:</span>
          <select
            class="modal-select"
            value={precision}
            onchange={(e) => {
              precision = Number((e.target as HTMLSelectElement).value) as 0 | 1 | 2 | 3;
            }}
          >
            {#each PRECISIONS as p}
              <option value={p}>{p === 0 ? '0 (Integer)' : `0.${'0'.repeat(p)}`}</option>
            {/each}
          </select>
        </label>

        <label class="field-row">
          <span>Default Volume Depth ({unit}):</span>
          <input
            type="number"
            step="any"
            min="0"
            class="modal-input"
            bind:value={depthValue}
          />
        </label>
      </div>

      <div class="scope-row" role="radiogroup" aria-label="Scale Application Scope">
        <button
          type="button"
          class="scope-btn"
          class:active={applyScope === 'page'}
          onclick={() => {
            applyScope = 'page';
          }}
        >
          Apply to Current Page
        </button>
        <button
          type="button"
          class="scope-btn"
          class:active={applyScope === 'all'}
          onclick={() => {
            applyScope = 'all';
          }}
        >
          Apply to All Pages
        </button>
      </div>

      <div class="card-actions">
        <button
          type="button"
          class="action-btn"
          onclick={() => measureEngine.closeCalibrationModal()}
        >
          Cancel
        </button>
        <button type="button" class="action-btn primary" onclick={commitCalibration}>
          Save Calibrated Scale
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: absolute;
    inset: 0;
    z-index: 220;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--canvas-page-shadow);
    pointer-events: auto;
  }

  .calibration-card {
    width: 400px;
    padding: 16px;
    background: var(--glass-bg);
    backdrop-filter: blur(var(--glass-blur));
    -webkit-backdrop-filter: blur(var(--glass-blur));
    border: 1px solid var(--glass-border);
    box-shadow: var(--glass-shadow);
    border-radius: 14px;
    color: var(--glass-text);
    display: flex;
    flex-direction: column;
    gap: 12px;
    font-size: 12px;
  }

  .card-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .header-title {
    display: flex;
    align-items: center;
    gap: 6px;
    font-weight: 600;
    font-size: 13px;
  }

  .close-btn {
    border: none;
    background: transparent;
    color: var(--glass-text);
    cursor: pointer;
    padding: 4px;
    border-radius: 6px;
  }

  .close-btn:hover {
    background: var(--canvas-selection);
  }

  .measured-summary {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 10px;
    background: var(--canvas-selection);
    border: 1px solid var(--glass-border);
    border-radius: 8px;
  }

  .field-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 10px;
  }

  .field-row {
    display: flex;
    flex-direction: column;
    gap: 4px;
    font-size: 11px;
  }

  .modal-input,
  .modal-select {
    background: var(--canvas-bg);
    color: var(--glass-text);
    border: 1px solid var(--glass-border);
    border-radius: 6px;
    padding: 6px 8px;
    font-size: 12px;
    outline: none;
  }

  .scope-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 6px;
    background: var(--canvas-bg);
    padding: 3px;
    border-radius: 8px;
    border: 1px solid var(--glass-border);
  }

  .scope-btn {
    border: none;
    background: transparent;
    color: var(--glass-text);
    padding: 6px;
    border-radius: 6px;
    font-size: 11px;
    font-weight: 600;
    cursor: pointer;
  }

  .scope-btn.active {
    background: var(--accent);
    color: var(--laser-core);
  }

  .card-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 4px;
  }

  .action-btn {
    border: 1px solid var(--glass-border);
    background: var(--canvas-selection);
    color: var(--glass-text);
    padding: 6px 12px;
    border-radius: 8px;
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
  }

  .action-btn.primary {
    background: var(--accent);
    color: var(--laser-core);
    border-color: var(--accent);
  }
</style>

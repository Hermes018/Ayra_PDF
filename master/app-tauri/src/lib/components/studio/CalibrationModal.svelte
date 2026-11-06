<script lang="ts">
  import { untrack } from 'svelte';
  import {
    measureEngine,
    ARCHITECTURAL_SCALE_PRESETS,
    type CadUnit,
    type PageScaleConfig,
  } from '$lib/core/measureEngine.svelte';
  import { viewport } from '$lib/core/viewportEngine.svelte';
  import StudioIcon from './StudioIcon.svelte';
  import Dialog from '$lib/components/ui/Dialog.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import Input from '$lib/components/ui/Input.svelte';
  import Select from '$lib/components/ui/Select.svelte';

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
    { id: 'pt', label: 'PDF points (pt)' },
  ];

  const PRECISIONS: (0 | 1 | 2 | 3)[] = [0, 1, 2, 3];

  // ui/Input and ui/Select work with strings; converted back when the scale is saved.
  let realDistance = $state<string>('1');
  let unit = $state<string>('m');
  let precision = $state<string>('2');
  let depthValue = $state<string>('0');
  const UNIT_OPTIONS = UNITS.map((u) => ({ value: u.id, label: u.label }));
  const PRECISION_OPTIONS = PRECISIONS.map((p) => ({ value: String(p), label: p === 0 ? '0 (integer)' : `0.${'0'.repeat(p)}` }));
  let presetOptions = $derived([
    { value: 'custom', label: 'Custom two-point calibration' },
    ...ARCHITECTURAL_SCALE_PRESETS.map((p) => ({ value: p.id, label: p.label })),
  ]);
  let dialogOpen = $state(false);
  $effect(() => {
    dialogOpen = !!measureEngine.calibrationModal?.open;
  });
  // Picking a preset fills unit and distance from the measured span.
  $effect(() => {
    const id = selectedPresetId;
    untrack(() => applyPreset(id));
  });
  let selectedPresetId = $state<string>('custom');
  let applyScope = $state<'page' | 'all'>('page');

  $effect(() => {
    const modal = measureEngine.calibrationModal;
    if (modal?.open) {
      const existing = measureEngine.getScaleForPage(modal.pageIndex);
      unit = existing.unit;
      precision = String(existing.precision);
      depthValue = String(existing.depthValue);
      const ratio = existing.realDistance / Math.max(existing.ptDistance, 1e-6);
      realDistance = String(Number((modal.measuredPt * ratio).toFixed(3)));
      selectedPresetId = 'custom';
    }
  });

  function applyPreset(presetId: string) {
    const found = ARCHITECTURAL_SCALE_PRESETS.find((p) => p.id === presetId);
    if (!found || !measureEngine.calibrationModal) return;
    unit = found.unit;
    const ratio = found.realDistance / Math.max(found.ptDistance, 1e-6);
    realDistance = String(Number((measureEngine.calibrationModal.measuredPt * ratio).toFixed(3)));
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
      unit: unit as CadUnit,
      precision: Number(precision) as 0 | 1 | 2 | 3,
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

<!-- Keeps canvas pointer gestures from starting under the dialog. -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="calibration-host" onpointerdown={(e) => e.stopPropagation()}>
  <Dialog
    bind:open={dialogOpen}
    title="Calibrate page scale (page {(measureEngine.calibrationModal?.pageIndex ?? 0) + 1})"
    onclose={() => measureEngine.closeCalibrationModal()}
    data-testid="calibration-dialog"
  >
    {#if measureEngine.calibrationModal}
      <div class="calibration-body">
        <p class="measured-summary">
          <StudioIcon name="measure" size={14} />
          Measured reference span: <strong>{measureEngine.calibrationModal.measuredPt.toFixed(2)} pt</strong>
        </p>

        <Select label="Scale preset" bind:value={selectedPresetId} options={presetOptions} data-testid="calibration-preset" />

        <div class="field-grid">
          <Input type="number" label="Known real distance" bind:value={realDistance} data-testid="calibration-distance" />
          <Select label="Measurement unit" bind:value={unit} options={UNIT_OPTIONS} data-testid="calibration-unit" />
        </div>

        <div class="field-grid">
          <Select label="Decimal precision" bind:value={precision} options={PRECISION_OPTIONS} data-testid="calibration-precision" />
          <Input type="number" label="Default volume depth ({unit})" bind:value={depthValue} data-testid="calibration-depth" />
        </div>

        <div class="scope-row" role="group" aria-label="Where to apply the scale">
          <Button variant="secondary" size="sm" pressed={applyScope === 'page'} onclick={() => (applyScope = 'page')}>
            Apply to current page
          </Button>
          <Button variant="secondary" size="sm" pressed={applyScope === 'all'} onclick={() => (applyScope = 'all')}>
            Apply to all pages
          </Button>
        </div>
      </div>
    {/if}
    {#snippet actions()}
      <Button variant="secondary" onclick={() => measureEngine.closeCalibrationModal()}>Cancel</Button>
      <Button variant="primary" data-testid="calibration-save" onclick={commitCalibration}>Save calibrated scale</Button>
    {/snippet}
  </Dialog>
</div>

<style>
  .calibration-body {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .measured-summary {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    margin: 0;
    font-size: var(--font-size-footnote);
    color: var(--text-secondary);
  }

  .field-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--space-3);
  }

  .scope-row {
    display: flex;
    gap: var(--space-2);
  }
</style>

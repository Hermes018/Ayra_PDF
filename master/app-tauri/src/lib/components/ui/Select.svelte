<script lang="ts">
  import Tooltip from './Tooltip.svelte';
  import { warnIfBlank, type CommonProps, type DisabledProps } from './types';

  type Props = CommonProps &
    DisabledProps & {
      value: string;
      label: string;
      options: { value: string; label: string }[];
      /** Visually hide the label (compact toolbars); it still names the control for assistive tech. */
      hideLabel?: boolean;
    };

  let {
    value = $bindable(),
    label,
    options,
    hideLabel = false,
    disabled = false,
    disabledReason,
    class: klass = '',
    'data-testid': testid,
  }: Props = $props();

  const uid = $props.id();
  const selectId = `${uid}-select`;
  const tipId = `${uid}-reason`;
  $effect(() => warnIfBlank('Select', 'disabledReason', disabledReason, disabled));
</script>

<div class="lp-field lp-tipwrap {klass}">
  <label class="lp-label" class:lp-sr-only={hideLabel} for={selectId}>{label}</label>
  <select
    id={selectId}
    class="lp-select"
    {disabled}
    bind:value
    aria-describedby={disabled && disabledReason ? tipId : undefined}
    data-testid={testid}
  >
    {#each options as o (o.value)}
      <option value={o.value}>{o.label}</option>
    {/each}
  </select>
  {#if disabled && disabledReason}<Tooltip id={tipId} text={disabledReason} />{/if}
</div>

<style>
  .lp-field { position: relative; display: flex; flex-direction: column; gap: var(--space-1); }
  .lp-sr-only { position: absolute; width: 1px; height: 1px; overflow: hidden; clip: rect(0 0 0 0); white-space: nowrap; }
  .lp-label { font-size: var(--font-size-footnote); font-weight: 600; color: var(--text-secondary); }
  .lp-select {
    font: inherit;
    font-size: var(--font-size-body);
    min-height: var(--control-height-md);
    padding: 0 var(--space-3);
    border: 1px solid var(--glass-border-subtle);
    border-radius: var(--radius-md);
    background: var(--glass-bg-input);
    color: var(--text-primary);
    transition: border-color var(--dur-fast), box-shadow var(--dur-fast);
  }
  .lp-select option { background: var(--option-bg); color: var(--option-fg); }
  .lp-select:hover:not(:disabled) { border-color: var(--glass-border-specular); }
  .lp-select:focus-visible { outline: none; border-color: var(--accent-primary); box-shadow: 0 0 0 3px var(--accent-ring); }
  .lp-select:disabled { opacity: 0.5; cursor: not-allowed; }
</style>

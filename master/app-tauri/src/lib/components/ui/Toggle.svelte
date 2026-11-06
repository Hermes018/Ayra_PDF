<script lang="ts">
  import Tooltip from './Tooltip.svelte';
  import { warnIfBlank, type CommonProps, type DisabledProps } from './types';

  type Props = CommonProps & DisabledProps & { checked: boolean; label: string };

  let {
    checked = $bindable(),
    label,
    disabled = false,
    disabledReason,
    class: klass = '',
    'data-testid': testid,
  }: Props = $props();

  const uid = $props.id();
  const labelId = `${uid}-label`;
  const tipId = `${uid}-reason`;
  $effect(() => warnIfBlank('Toggle', 'disabledReason', disabledReason, disabled));

  function flip(): void {
    if (!disabled) checked = !checked;
  }
</script>

<span class="lp-tipwrap lp-togglewrap {klass}">
  <button
    type="button"
    role="switch"
    class="lp-toggle"
    class:is-on={checked}
    class:is-disabled={disabled}
    aria-checked={checked}
    aria-labelledby={labelId}
    aria-disabled={disabled || undefined}
    aria-describedby={disabled ? tipId : undefined}
    data-testid={testid}
    onclick={flip}
  >
    <span class="lp-track" aria-hidden="true"><span class="lp-thumb"></span></span>
    <span id={labelId} class="lp-toggle-label">{label}</span>
  </button>
  {#if disabled && disabledReason}<Tooltip id={tipId} text={disabledReason} />{/if}
</span>

<style>
  .lp-togglewrap { position: relative; display: inline-flex; }
  .lp-toggle {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    min-height: var(--control-height-md);
    padding: 0;
    border: 0;
    background: transparent;
    color: var(--text-primary);
    font: inherit;
    font-size: var(--font-size-body);
    cursor: pointer;
  }
  .lp-track {
    position: relative;
    display: inline-block;
    width: var(--control-height-lg);
    height: var(--space-5);
    border-radius: var(--radius-pill);
    background: var(--glass-bg-input);
    border: 1px solid var(--glass-border-subtle);
    transition: background-color var(--motion-duration) var(--ease-out);
  }
  .lp-thumb {
    position: absolute;
    top: 50%;
    left: var(--space-1);
    width: var(--space-4);
    height: var(--space-4);
    transform: translateY(-50%);
    border-radius: var(--radius-pill);
    background: var(--text-on-accent);
    box-shadow: var(--glass-shadow-sm);
    transition: left var(--motion-duration) var(--ease-out);
  }
  .is-on .lp-track { background: var(--accent-primary); border-color: var(--accent-primary); }
  .is-on .lp-thumb { left: calc(100% - var(--space-4) - var(--space-1)); }
  .lp-toggle:focus-visible { outline: 2px solid var(--focus-ring); outline-offset: 2px; border-radius: var(--radius-sm); }
  .lp-toggle.is-disabled { cursor: not-allowed; opacity: 0.5; }
</style>

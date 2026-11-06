<script lang="ts">
  import Tooltip from './Tooltip.svelte';
  import { warnIfBlank, type CommonProps, type DisabledProps } from './types';

  type Props = CommonProps &
    DisabledProps & {
      value: string;
      label: string;
      placeholder?: string;
      error?: string;
      hint?: string;
      type?: 'text' | 'number' | 'password' | 'search' | 'email' | 'range';
      /** Native autocomplete hint, e.g. "off" for secrets. */
      autocomplete?: 'off' | 'on' | 'new-password' | 'current-password';
      /** Visually hide the label (compact toolbars); it still names the control for assistive tech. */
      hideLabel?: boolean;
      /** For type="range". */
      min?: number;
      max?: number;
      /** Native change event (fires on commit / release). */
      onchange?: () => void;
    };

  let {
    value = $bindable(),
    label,
    placeholder,
    error,
    hint,
    type = 'text',
    hideLabel = false,
    autocomplete,
    min,
    max,
    onchange,
    disabled = false,
    disabledReason,
    class: klass = '',
    'data-testid': testid,
  }: Props = $props();

  const uid = $props.id();
  const inputId = `${uid}-input`;
  const noteId = `${uid}-note`;
  const tipId = `${uid}-reason`;
  $effect(() => warnIfBlank('Input', 'disabledReason', disabledReason, disabled));
  const describedBy = $derived(
    [disabled && disabledReason ? tipId : null, error || hint ? noteId : null].filter(Boolean).join(' ') || undefined,
  );
</script>

<div class="lp-field lp-tipwrap {klass}">
  <label class="lp-label" class:lp-sr-only={hideLabel} for={inputId}>{label}</label>
  <input
    id={inputId}
    class="lp-input"
    class:has-error={!!error}
    {type}
    {placeholder}
    {autocomplete}
    {min}
    {max}
    {onchange}
    {disabled}
    bind:value
    aria-invalid={error ? true : undefined}
    aria-describedby={describedBy}
    data-testid={testid}
  />
  {#if error}
    <span id={noteId} class="lp-note lp-note--error" role="alert">{error}</span>
  {:else if hint}
    <span id={noteId} class="lp-note">{hint}</span>
  {/if}
  {#if disabled && disabledReason}<Tooltip id={tipId} text={disabledReason} />{/if}
</div>

<style>
  .lp-field { position: relative; display: flex; flex-direction: column; gap: var(--space-1); }
  .lp-sr-only { position: absolute; width: 1px; height: 1px; overflow: hidden; clip: rect(0 0 0 0); white-space: nowrap; }
  .lp-label { font-size: var(--font-size-footnote); font-weight: 600; color: var(--text-secondary); }
  .lp-input {
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
  .lp-input:hover:not(:disabled) { border-color: var(--glass-border-specular); }
  .lp-input:focus-visible { outline: none; border-color: var(--accent-primary); box-shadow: 0 0 0 3px var(--accent-ring); }
  .lp-input.has-error { border-color: var(--semantic-danger); }
  .lp-input:disabled { opacity: 0.5; cursor: not-allowed; }
  .lp-note { font-size: var(--font-size-caption); color: var(--text-secondary); }
  .lp-note--error { color: var(--semantic-danger); }
</style>

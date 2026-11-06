<script lang="ts">
  import Button from './Button.svelte';
  import type { CommonProps } from './types';

  /** value 0..100, or null while the amount of work is unknown. */
  let {
    value,
    label,
    oncancel,
    class: klass = '',
    'data-testid': testid,
  }: CommonProps & { value: number | null; label: string; oncancel?: () => void } = $props();

  const pct = $derived(value === null ? null : Math.max(0, Math.min(100, value)));
</script>

<div class="lp-progress {klass}" data-testid={testid}>
  <div class="lp-progress-head">
    <span class="lp-progress-label">{label}</span>
    {#if pct !== null}<span class="lp-progress-pct">{Math.round(pct)}%</span>{/if}
    {#if oncancel}<Button variant="plain" size="sm" onclick={() => oncancel()}>Cancel</Button>{/if}
  </div>
  <div
    class="lp-bar"
    class:is-indeterminate={pct === null}
    role="progressbar"
    aria-label={label}
    aria-valuemin="0"
    aria-valuemax="100"
    aria-valuenow={pct === null ? undefined : Math.round(pct)}
  >
    <span class="lp-fill" style:width={pct === null ? undefined : `${pct}%`}></span>
  </div>
</div>

<style>
  .lp-progress { display: flex; flex-direction: column; gap: var(--space-2); min-width: 16em; }
  .lp-progress-head { display: flex; align-items: center; gap: var(--space-2); font-size: var(--font-size-body); }
  .lp-progress-label { flex: 1; color: var(--text-primary); }
  .lp-progress-pct { color: var(--text-secondary); font-variant-numeric: tabular-nums; }
  .lp-bar {
    position: relative;
    height: var(--space-2);
    overflow: hidden;
    border-radius: var(--radius-pill);
    background: var(--glass-bg-input);
  }
  .lp-fill {
    display: block;
    height: 100%;
    border-radius: var(--radius-pill);
    background: var(--accent-primary);
    transition: width var(--motion-duration) var(--ease-out);
  }
  .is-indeterminate .lp-fill { width: 40%; animation: lp-slide 1.2s var(--ease-out) infinite; }
  @keyframes lp-slide { from { margin-left: -40%; } to { margin-left: 100%; } }
  @media (prefers-reduced-motion: reduce) { .is-indeterminate .lp-fill { animation: none; margin-left: 30%; } }
</style>

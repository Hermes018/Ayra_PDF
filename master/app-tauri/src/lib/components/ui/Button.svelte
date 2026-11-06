<script lang="ts">
  import type { Snippet } from 'svelte';
  import Tooltip from './Tooltip.svelte';
  import { warnIfBlank, type CommonProps, type DisabledProps } from './types';

  type Props = CommonProps &
    DisabledProps & {
      variant: 'primary' | 'secondary' | 'destructive' | 'plain';
      size?: 'sm' | 'md' | 'lg';
      onclick: (e: MouseEvent) => void;
      children: Snippet;
      /** Loading: shows progress dots, announces aria-busy and ignores clicks. */
      busy?: boolean;
      type?: 'button' | 'submit';
      /** Navigation item: marks the current page (aria-current) and highlights it. */
      current?: boolean;
      /** Full width, label aligned to the start (sidebar items). */
      block?: boolean;
      /** Disclosure (accordion header): exposes aria-expanded. */
      expanded?: boolean;
    };

  let {
    variant,
    size = 'md',
    onclick,
    children,
    disabled = false,
    disabledReason,
    busy = false,
    type = 'button',
    current = false,
    block = false,
    expanded,
    class: klass = '',
    'data-testid': testid,
  }: Props = $props();

  const uid = $props.id();
  const tipId = `${uid}-reason`;
  $effect(() => warnIfBlank('Button', 'disabledReason', disabledReason, disabled));

  // aria-disabled (not `disabled`) keeps the control focusable, so keyboard users can reach the reason.
  function activate(e: MouseEvent): void {
    if (disabled || busy) {
      e.preventDefault();
      return;
    }
    onclick(e);
  }
</script>

<span class="lp-tipwrap lp-btnwrap {klass}" class:is-block={block}>
  <button
    {type}
    class="lp-btn lp-btn--{variant} lp-btn--{size}"
    class:is-disabled={disabled}
    class:is-busy={busy}
    class:is-block={block}
    class:is-current={current}
    aria-current={current ? 'page' : undefined}
    aria-expanded={expanded}
    aria-disabled={disabled || undefined}
    aria-busy={busy || undefined}
    aria-describedby={disabled ? tipId : undefined}
    data-testid={testid}
    onclick={activate}
  >
    {#if busy}
      <span class="lp-dots" aria-hidden="true"><i></i><i></i><i></i></span>
    {/if}
    {@render children()}
  </button>
  {#if disabled && disabledReason}<Tooltip id={tipId} text={disabledReason} />{/if}
</span>

<style>
  .lp-btnwrap {
    position: relative;
    display: inline-flex;
  }
  .lp-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
    font: inherit;
    font-size: var(--font-size-body);
    font-weight: 600;
    min-height: var(--control-height-md);
    padding: 0 var(--space-3);
    border: 1px solid transparent;
    border-radius: var(--radius-md);
    cursor: pointer;
    color: var(--text-primary);
    transition:
      background-color var(--dur-fast) var(--ease-out),
      box-shadow var(--dur-fast) var(--ease-out),
      opacity var(--dur-fast);
  }
  .lp-btn--sm { min-height: var(--control-height-sm); padding: 0 var(--space-2); font-size: var(--font-size-footnote); }
  .lp-btn--lg { min-height: var(--control-height-lg); padding: 0 var(--space-4); }

  .lp-btn--primary { background: var(--accent-primary); color: var(--text-on-accent); }
  .lp-btn--primary:hover:not(.is-disabled) { background: var(--accent-primary-hover); }
  .lp-btn--secondary { background: var(--glass-bg-input); border-color: var(--glass-border-subtle); }
  .lp-btn--secondary:hover:not(.is-disabled) { background: var(--glass-bg-hover); }
  .lp-btn--destructive { background: var(--semantic-danger); color: var(--text-on-accent); }
  .lp-btn--destructive:hover:not(.is-disabled) { box-shadow: var(--glass-shadow-sm); opacity: 0.9; }
  .lp-btn--plain { background: transparent; color: var(--accent-primary); }
  .lp-btn--plain:hover:not(.is-disabled) { background: var(--glass-bg-hover); }

  .lp-btn:focus-visible { outline: 2px solid var(--focus-ring); outline-offset: 2px; }
  .lp-btn:active:not(.is-disabled) { box-shadow: inset 0 0 0 999px var(--glass-bg-hover); }
  .lp-btn.is-disabled { cursor: not-allowed; opacity: 0.5; }
  .lp-btn.is-busy { cursor: progress; }
  .lp-btnwrap.is-block { display: flex; width: 100%; }
  .lp-btn.is-block { width: 100%; justify-content: flex-start; font-weight: 400; }
  .lp-btn--plain.is-block { color: var(--text-primary); }
  .lp-btn.is-current { background: var(--glass-bg-active); color: var(--accent-primary); font-weight: 600; }

  .lp-dots { display: inline-flex; gap: var(--space-1); }
  .lp-dots i {
    width: var(--space-1);
    height: var(--space-1);
    border-radius: var(--radius-pill);
    background: currentColor;
    animation: lp-pulse 900ms ease-in-out infinite;
  }
  .lp-dots i:nth-child(2) { animation-delay: 150ms; }
  .lp-dots i:nth-child(3) { animation-delay: 300ms; }
  @keyframes lp-pulse { 0%, 100% { opacity: 0.25; } 50% { opacity: 1; } }
  @media (prefers-reduced-motion: reduce) { .lp-dots i { animation: none; } }
</style>

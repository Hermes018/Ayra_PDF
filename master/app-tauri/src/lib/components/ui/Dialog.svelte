<script lang="ts">
  import type { Snippet } from 'svelte';
  import { tick } from 'svelte';
  import { commands } from '$lib/core/commandEngine.svelte';
  import type { CommonProps } from './types';

  type Props = CommonProps & {
    open: boolean;
    title: string;
    onclose: () => void;
    children: Snippet;
    actions?: Snippet;
    /** 'sheet' docks to the bottom edge; 'dialog' is centred. */
    variant?: 'dialog' | 'sheet';
  };

  let {
    open = $bindable(),
    title,
    onclose,
    children,
    actions,
    variant = 'dialog',
    class: klass = '',
    'data-testid': testid,
  }: Props = $props();

  const uid = $props.id();
  const titleId = `${uid}-title`;
  let card = $state<HTMLElement | null>(null);
  let returnTo: HTMLElement | null = null;

  function close(): void {
    open = false;
    onclose();
  }

  // Escape goes through the single keyboard entry point (commandEngine), registered only while open.
  $effect(() => {
    if (!open) return;
    return commands.registerEscape(() => {
      if (!open) return false;
      close();
      return true;
    });
  });

  function tabbables(): HTMLElement[] {
    if (!card) return [];
    return [
      ...card.querySelectorAll<HTMLElement>(
        'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])',
      ),
    ].filter((el) => !el.classList.contains('lp-sentinel'));
  }

  // On open: remember the opener, move focus inside ([data-autofocus] wins, e.g. Cancel on a destructive confirm).
  $effect(() => {
    if (!open || !card) return;
    returnTo = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    const target = card;
    void tick().then(() => {
      const el = target.querySelector<HTMLElement>('[data-autofocus]') ?? tabbables()[0] ?? target;
      el.focus();
    });
    return () => {
      returnTo?.focus?.();
      returnTo = null;
    };
  });

  // Focus trap without key listeners: two sentinels at the ends of the card wrap focus around.
  function wrapToLast(): void {
    const t = tabbables();
    (t.at(-1) ?? card)?.focus();
  }
  function wrapToFirst(): void {
    const t = tabbables();
    (t[0] ?? card)?.focus();
  }
  // Focus that escapes the card by mouse or script is pulled back in.
  function pullBack(e: FocusEvent): void {
    if (open && card && e.target instanceof Node && !card.contains(e.target)) wrapToFirst();
  }
  $effect(() => {
    if (!open) return;
    document.addEventListener('focusin', pullBack);
    return () => document.removeEventListener('focusin', pullBack);
  });
</script>

{#if open}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div class="lp-backdrop lp-backdrop--{variant}" role="presentation" onclick={(e) => e.target === e.currentTarget && close()}>
    <div
      bind:this={card}
      class="lp-dialog lp-dialog--{variant} {klass}"
      role="dialog"
      aria-modal="true"
      aria-labelledby={titleId}
      tabindex="-1"
      data-testid={testid}
    >
      <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
      <span class="lp-sentinel" tabindex="0" aria-hidden="true" onfocus={wrapToLast}></span>
      <h2 id={titleId} class="lp-dialog-title">{title}</h2>
      <div class="lp-dialog-body">{@render children()}</div>
      {#if actions}<div class="lp-dialog-actions">{@render actions()}</div>{/if}
      <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
      <span class="lp-sentinel" tabindex="0" aria-hidden="true" onfocus={wrapToFirst}></span>
    </div>
  </div>
{/if}

<style>
  .lp-backdrop {
    position: fixed;
    inset: 0;
    z-index: var(--z-dialog);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--space-6);
    background: var(--scrim);
    -webkit-backdrop-filter: var(--glass-blur);
    backdrop-filter: var(--glass-blur);
  }
  .lp-backdrop--sheet { align-items: flex-end; }
  .lp-dialog {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    min-width: 20em;
    max-width: 100%;
    max-height: 100%;
    overflow: auto;
    padding: var(--space-5);
    border: 1px solid var(--glass-border-subtle);
    border-radius: var(--radius-xl);
    background: var(--surface-dialog);
    box-shadow: var(--glass-shadow-lg);
    color: var(--text-primary);
    animation: lp-rise var(--dur-med) var(--ease-out);
  }
  .lp-dialog--sheet { width: 100%; border-bottom-left-radius: 0; border-bottom-right-radius: 0; }
  .lp-dialog:focus { outline: none; }
  .lp-dialog-title { margin: 0; font-size: var(--font-size-title2); font-weight: 600; }
  .lp-dialog-body { font-size: var(--font-size-body); color: var(--text-primary); }
  .lp-dialog-actions { display: flex; justify-content: flex-end; gap: var(--space-2); }
  .lp-sentinel { position: absolute; width: 0; height: 0; overflow: hidden; }
  @keyframes lp-rise { from { opacity: 0; transform: translateY(var(--space-2)); } to { opacity: 1; transform: none; } }
  @media (prefers-reduced-motion: reduce) { .lp-dialog { animation: none; } }
</style>

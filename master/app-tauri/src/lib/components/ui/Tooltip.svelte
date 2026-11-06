<script lang="ts">
  import { placeTooltip } from './tooltipPlacement';

  // Text bubble shown while the parent `.lp-tipwrap` is hovered or contains focus. The element is always in the
  // DOM so `aria-describedby` can point at it. It is fixed-positioned and placed on show: clamped to the
  // viewport, flipped above the anchor when there is no room below.
  let { id, text, class: klass = '' }: { id: string; text: string; class?: string } = $props();
  let el = $state<HTMLElement | null>(null);

  function place(): void {
    const anchor = el?.parentElement;
    if (!el || !anchor) return;
    const gap = parseFloat(getComputedStyle(el).marginTop) || 4;
    const t = el.getBoundingClientRect();
    const p = placeTooltip(anchor.getBoundingClientRect(), { width: t.width, height: t.height }, { width: window.innerWidth, height: window.innerHeight }, gap);
    el.style.left = `${p.left}px`;
    el.style.top = `${p.top}px`;
  }

  $effect(() => {
    const anchor = el?.parentElement;
    if (!anchor) return;
    anchor.addEventListener('pointerenter', place);
    anchor.addEventListener('focusin', place);
    return () => {
      anchor.removeEventListener('pointerenter', place);
      anchor.removeEventListener('focusin', place);
    };
  });
</script>

<span bind:this={el} {id} role="tooltip" class="lp-tooltip {klass}">{text}</span>

<style>
  .lp-tooltip {
    position: fixed;
    left: 0;
    top: 0;
    z-index: var(--z-dialog);
    max-width: min(24em, 90vw);
    width: max-content;
    margin: var(--space-1);
    padding: var(--space-1) var(--space-2);
    border-radius: var(--radius-xs);
    border: 1px solid var(--glass-border-subtle);
    background: var(--surface-dialog);
    box-shadow: var(--glass-shadow-sm);
    color: var(--text-primary);
    font-size: var(--font-size-footnote);
    font-weight: 400;
    line-height: 1.35;
    text-align: left;
    white-space: normal;
    pointer-events: none;
    opacity: 0;
    visibility: hidden;
    transition: opacity var(--dur-fast) var(--ease-out), visibility var(--dur-fast);
  }
  :global(.lp-tipwrap:hover) > .lp-tooltip,
  :global(.lp-tipwrap:focus-within) > .lp-tooltip {
    opacity: 1;
    visibility: visible;
  }
</style>

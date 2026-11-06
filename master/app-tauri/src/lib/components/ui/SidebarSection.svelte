<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { CommonProps } from './types';

  // Extracted from Sidebar.svelte's nav group (title + list of items). Layout-free: width comes from the shell.
  let {
    title,
    children,
    collapsible = false,
    class: klass = '',
    'data-testid': testid,
  }: CommonProps & { title: string; children: Snippet; collapsible?: boolean } = $props();

  const uid = $props.id();
  const bodyId = `${uid}-body`;
  let expanded = $state(true);
</script>

<section class="lp-sbsec {klass}" aria-labelledby="{uid}-title" data-testid={testid}>
  {#if collapsible}
    <button
      type="button"
      id="{uid}-title"
      class="lp-sbsec-head lp-sbsec-toggle"
      aria-expanded={expanded}
      aria-controls={bodyId}
      onclick={() => (expanded = !expanded)}
    >
      {title}
    </button>
  {:else}
    <h3 id="{uid}-title" class="lp-sbsec-head">{title}</h3>
  {/if}
  {#if expanded}
    <div id={bodyId} class="lp-sbsec-body">{@render children()}</div>
  {/if}
</section>

<style>
  .lp-sbsec { display: flex; flex-direction: column; gap: var(--space-1); padding: var(--space-2) 0; }
  .lp-sbsec-head {
    margin: 0;
    padding: 0 var(--space-3);
    font-size: var(--font-size-caption);
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--text-tertiary);
    text-align: left;
  }
  .lp-sbsec-toggle {
    min-height: var(--control-height-sm);
    border: 0;
    background: transparent;
    font-family: inherit;
    cursor: pointer;
  }
  .lp-sbsec-toggle:hover { color: var(--text-secondary); }
  .lp-sbsec-toggle:focus-visible { outline: 2px solid var(--focus-ring); outline-offset: 2px; border-radius: var(--radius-sm); }
  .lp-sbsec-body { display: flex; flex-direction: column; gap: var(--space-1); }
</style>

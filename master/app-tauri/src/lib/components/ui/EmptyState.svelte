<script lang="ts">
  import Button from './Button.svelte';
  import type { CommonProps } from './types';

  let {
    title,
    body,
    action,
    class: klass = '',
    'data-testid': testid,
  }: CommonProps & { title: string; body: string; action?: { label: string; onclick: () => void } } = $props();
</script>

<section class="lp-empty {klass}" data-testid={testid}>
  <h3 class="lp-empty-title">{title}</h3>
  <p class="lp-empty-body">{body}</p>
  {#if action}
    <Button variant="primary" onclick={() => action.onclick()}>{action.label}</Button>
  {/if}
</section>

<style>
  .lp-empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-8);
    text-align: center;
  }
  .lp-empty-title { margin: 0; font-size: var(--font-size-title2); font-weight: 600; color: var(--text-primary); }
  .lp-empty-body { margin: 0; max-width: 32em; font-size: var(--font-size-body); color: var(--text-secondary); }
</style>

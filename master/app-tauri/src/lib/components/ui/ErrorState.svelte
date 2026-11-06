<script lang="ts">
  import Button from './Button.svelte';
  import type { CommonProps } from './types';

  /** `what` = what happened, `next` = what the user can do about it. Plain words, no codes or paths. */
  let {
    what,
    next,
    retry,
    class: klass = '',
    'data-testid': testid,
  }: CommonProps & { what: string; next: string; retry?: () => void } = $props();
</script>

<section class="lp-error {klass}" role="alert" data-testid={testid}>
  <h3 class="lp-error-what">{what}</h3>
  <p class="lp-error-next">{next}</p>
  {#if retry}
    <Button variant="secondary" onclick={() => retry()}>Retry</Button>
  {/if}
</section>

<style>
  .lp-error {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: var(--space-2);
    padding: var(--space-4);
    border: 1px solid var(--semantic-danger);
    border-radius: var(--radius-lg);
    background: var(--glass-bg-surface);
  }
  .lp-error-what { margin: 0; font-size: var(--font-size-title3); font-weight: 600; color: var(--semantic-danger); }
  .lp-error-next { margin: 0; font-size: var(--font-size-body); color: var(--text-secondary); }
</style>

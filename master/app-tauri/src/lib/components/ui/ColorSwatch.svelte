<script lang="ts">
  import Tooltip from './Tooltip.svelte';
  import { warnIfBlank, type CommonProps } from './types';

  // A colour choice: a round swatch button. The label names the colour ("Blue colour") for assistive tech and is its
  // tooltip; `pressed` marks the selected colour (aria-pressed + a ring that does not rely on colour alone).
  let {
    color,
    label,
    pressed = false,
    onclick,
    class: klass = '',
    'data-testid': testid,
  }: CommonProps & { color: string; label: string; pressed?: boolean; onclick: (e: MouseEvent) => void } = $props();

  const uid = $props.id();
  const tipId = `${uid}-tip`;
  $effect(() => warnIfBlank('ColorSwatch', 'label', label));
</script>

<span class="lp-tipwrap lp-swatchwrap {klass}">
  <button
    type="button"
    class="lp-swatch"
    class:is-pressed={pressed}
    style:background={color}
    aria-label={label}
    aria-pressed={pressed}
    aria-describedby={tipId}
    data-testid={testid}
    {onclick}
  ></button>
  <Tooltip id={tipId} text={label} />
</span>

<style>
  .lp-swatchwrap { position: relative; display: inline-flex; }
  .lp-swatch {
    width: var(--space-5);
    height: var(--space-5);
    padding: 0;
    border-radius: var(--radius-pill);
    border: 2px solid transparent;
    cursor: pointer;
    box-shadow: 0 0 0 1px var(--glass-border-specular);
    transition: transform var(--dur-fast) var(--ease-out), box-shadow var(--dur-fast);
  }
  .lp-swatch:hover { transform: scale(1.12); }
  .lp-swatch.is-pressed {
    transform: scale(1.2);
    box-shadow: 0 0 0 2px var(--surface-dialog), 0 0 0 4px var(--accent-primary);
  }
  .lp-swatch:focus-visible { outline: 2px solid var(--focus-ring); outline-offset: 2px; }
</style>

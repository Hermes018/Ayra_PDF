<script lang="ts">
  import StudioIcon, { type StudioIconName } from '$lib/components/studio/StudioIcon.svelte';
  import Tooltip from './Tooltip.svelte';
  import { commands } from '$lib/core/commandEngine.svelte';
  import { warnIfBlank, type CommonProps, type DisabledProps } from './types';

  type Props = CommonProps &
    DisabledProps & {
      icon: StudioIconName;
      /** Required: names the control for screen readers and is also the tooltip. */
      ariaLabel: string;
      onclick: (e: MouseEvent) => void;
      pressed?: boolean;
      /** Command id; the bound chord (commands.shortcutFor) is appended to the tooltip. */
      shortcutCmd?: string;
      size?: 'sm' | 'md' | 'lg';
    };

  let {
    icon,
    ariaLabel,
    onclick,
    pressed,
    disabled = false,
    disabledReason,
    shortcutCmd,
    size = 'md',
    class: klass = '',
    'data-testid': testid,
  }: Props = $props();

  const uid = $props.id();
  const tipId = `${uid}-tip`;
  $effect(() => warnIfBlank('IconButton', 'ariaLabel', ariaLabel));
  $effect(() => warnIfBlank('IconButton', 'disabledReason', disabledReason, disabled));

  const chord = $derived(shortcutCmd ? commands.shortcutFor(shortcutCmd) : null);
  const tipText = $derived(disabled && disabledReason ? `${ariaLabel}: ${disabledReason}` : chord ? `${ariaLabel} (${chord})` : ariaLabel);

  function activate(e: MouseEvent): void {
    if (disabled) {
      e.preventDefault();
      return;
    }
    onclick(e);
  }
</script>

<span class="lp-tipwrap lp-iconwrap {klass}">
  <button
    type="button"
    class="lp-icon lp-icon--{size}"
    class:is-pressed={pressed}
    class:is-disabled={disabled}
    aria-label={ariaLabel}
    aria-pressed={pressed === undefined ? undefined : pressed}
    aria-disabled={disabled || undefined}
    aria-describedby={tipId}
    data-testid={testid}
    onclick={activate}
  >
    <StudioIcon name={icon} size={16} />
  </button>
  <Tooltip id={tipId} text={tipText} />
</span>

<style>
  .lp-iconwrap { position: relative; display: inline-flex; }
  .lp-icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: var(--control-height-md);
    height: var(--control-height-md);
    padding: 0;
    border: 1px solid transparent;
    border-radius: var(--radius-md);
    background: transparent;
    color: var(--text-secondary);
    cursor: pointer;
    transition: background-color var(--dur-fast) var(--ease-out), color var(--dur-fast);
  }
  .lp-icon--sm { width: var(--control-height-sm); height: var(--control-height-sm); }
  .lp-icon--lg { width: var(--control-height-lg); height: var(--control-height-lg); }
  .lp-icon:hover:not(.is-disabled) { background: var(--glass-bg-hover); color: var(--text-primary); }
  .lp-icon.is-pressed { background: var(--glass-bg-active); color: var(--accent-primary); }
  .lp-icon:focus-visible { outline: 2px solid var(--focus-ring); outline-offset: 2px; }
  .lp-icon.is-disabled { cursor: not-allowed; opacity: 0.5; }
</style>

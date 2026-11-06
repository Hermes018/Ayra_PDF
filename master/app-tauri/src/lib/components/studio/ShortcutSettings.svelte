<script lang="ts">
  import { commands, type Chord } from '$lib/core/commandEngine.svelte';
  import Dialog from '$lib/components/ui/Dialog.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import { labsVisible } from './labs.svelte';

  let {
    visible = $bindable(false),
    onClose
  } = $props<{
    visible: boolean;
    onClose?: () => void;
  }>();

  let recordingCommand = $state<string | null>(null);
  let conflictNotice = $state<string | null>(null);

  let shortcutList = $derived(
    Object.entries(commands.shortcuts)
      // A shortcut bound to a Labs tool is not listed while Labs is off.
      .filter(([, cmd]) => labsVisible(cmd))
      .map(([chord, cmd]) => ({
        chord,
        command: cmd,
        label: formatCommandLabel(cmd)
      }))
  );

  function formatCommandLabel(cmd: string): string {
    return cmd
      .replace(/^tool\./, 'Tool: ')
      .replace(/^history\./, 'History: ')
      .replace(/^radial\./, 'Radial: ')
      .replace(/\./g, ' › ');
  }

  async function startRecording(cmd: string) {
    recordingCommand = cmd;
    conflictNotice = null;

    try {
      const newChord = await commands.captureNextChord();
      if (newChord === null || !recordingCommand) return; // cancelled (Escape or the dialog closed)

      // Find if old chord was bound to this command
      const oldEntry = Object.entries(commands.shortcuts).find(([_, c]) => c === recordingCommand);
      if (oldEntry) {
        commands.unbind(oldEntry[0]);
      }

      const res = commands.bind(newChord, recordingCommand);
      if (res.conflict) {
        conflictNotice = `Shortcut ${newChord} replaced previously assigned "${res.conflict}"`;
      }
    } finally {
      recordingCommand = null;
    }
  }

  function handleReset() {
    commands.resetShortcuts();
    conflictNotice = 'Shortcuts reset to defaults.';
  }

  function handleClose() {
    commands.cancelCaptureChord();
    recordingCommand = null;
    conflictNotice = null;
    visible = false;
    onClose?.();
  }
</script>

<Dialog bind:open={visible} title="Custom shortcuts" onclose={handleClose} class="shortcuts-dialog">
  {#if conflictNotice}
    <div class="conflict-alert" role="status">
      {conflictNotice}
    </div>
  {/if}

  <div class="shortcuts-table-wrapper">
    <table class="shortcuts-table">
      <thead>
        <tr>
          <th>Action</th>
          <th>Shortcut</th>
          <th>Change</th>
        </tr>
      </thead>
      <tbody>
        {#each shortcutList as item}
          <tr>
            <td class="cmd-cell">{item.label}</td>
            <td>
              <kbd class="chord-badge">{item.chord}</kbd>
            </td>
            <td>
              <Button
                variant="secondary"
                size="sm"
                pressed={recordingCommand === item.command}
                onclick={() => startRecording(item.command)}
              >
                {recordingCommand === item.command ? 'Press a key chord…' : 'Record'}
              </Button>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>

  {#snippet actions()}
    <Button variant="secondary" onclick={handleReset}>Reset to defaults</Button>
    <Button variant="primary" onclick={handleClose}>Done</Button>
  {/snippet}
</Dialog>

<style>
  :global(.lp-dialog.shortcuts-dialog) {
    width: min(580px, 100%);
    max-height: 80vh;
  }

  .conflict-alert {
    padding: var(--space-2) var(--space-3);
    margin-bottom: var(--space-3);
    border: 1px solid var(--glass-border-subtle);
    border-radius: var(--radius-sm);
    background: var(--glass-bg-input);
    font-size: var(--font-size-footnote);
  }

  .shortcuts-table {
    width: 100%;
    border-collapse: collapse;
    font-size: var(--font-size-footnote);
  }

  .shortcuts-table th {
    text-align: left;
    padding: var(--space-2);
    color: var(--text-secondary);
    font-weight: 600;
    border-bottom: 1px solid var(--glass-border-subtle);
  }

  .shortcuts-table td {
    padding: var(--space-2);
    border-bottom: 1px solid var(--glass-border-subtle);
  }

  .chord-badge {
    padding: var(--space-1) var(--space-2);
    border-radius: var(--radius-xs);
    border: 1px solid var(--glass-border-subtle);
    background: var(--glass-bg-input);
    font-family: var(--font-mono);
    font-size: var(--font-size-caption);
  }
</style>

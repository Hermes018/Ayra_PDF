<script lang="ts">
  import { commands, type Chord } from '$lib/core/commandEngine.svelte';
  import StudioIcon from './StudioIcon.svelte';

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
    Object.entries(commands.shortcuts).map(([chord, cmd]) => ({
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
      if (!recordingCommand) return;

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

{#if visible}
  <div class="modal-backdrop" role="presentation">
    <div
      class="modal-window glass-panel"
      role="dialog"
      aria-modal="true"
      aria-label="Custom Shortcuts Settings"
      tabindex="-1"
    >
      <div class="modal-header">
        <div class="header-title">
          <StudioIcon name="shortcuts" size={18} />
          <h2>Custom Shortcuts</h2>
        </div>
        <button
          type="button"
          class="close-btn"
          onclick={handleClose}
          aria-label="Close settings"
        >
          <StudioIcon name="close" size={14} />
        </button>
      </div>

      {#if conflictNotice}
        <div class="conflict-alert" role="status">
          {conflictNotice}
        </div>
      {/if}

      <div class="shortcuts-table-wrapper">
        <table class="shortcuts-table">
          <thead>
            <tr>
              <th>Action / Command</th>
              <th>Current Chord</th>
              <th>Remap</th>
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
                  <button
                    type="button"
                    class="record-btn"
                    class:recording={recordingCommand === item.command}
                    onclick={() => startRecording(item.command)}
                  >
                    {recordingCommand === item.command ? 'Press Key Chord...' : 'Record'}
                  </button>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>

      <div class="modal-footer">
        <button
          type="button"
          class="reset-btn"
          onclick={handleReset}
        >
          Reset to Defaults
        </button>
        <button
          type="button"
          class="done-btn"
          onclick={handleClose}
        >
          Done
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    z-index: 400;
    background: var(--canvas-selection);
    backdrop-filter: blur(8px);
    -webkit-backdrop-filter: blur(8px);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 24px;
  }

  .modal-window {
    width: 100%;
    max-width: 580px;
    max-height: 80vh;
    background: var(--glass-bg);
    backdrop-filter: blur(var(--glass-blur));
    -webkit-backdrop-filter: blur(var(--glass-blur));
    border: 1px solid var(--glass-border);
    border-radius: 16px;
    box-shadow: var(--glass-shadow);
    color: var(--glass-text);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 20px;
    border-bottom: 1px solid var(--glass-border);
  }

  .header-title {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .header-title h2 {
    margin: 0;
    font-size: 16px;
    font-weight: 600;
  }

  .close-btn {
    background: transparent;
    border: none;
    color: var(--glass-text);
    font-size: 16px;
    cursor: pointer;
    border-radius: 4px;
    padding: 4px 8px;
    opacity: 0.7;
    transition: opacity 0.15s ease, background 0.15s ease;
  }

  .close-btn:hover {
    opacity: 1;
    background: var(--canvas-selection);
  }

  .conflict-alert {
    margin: 12px 20px 0;
    padding: 8px 12px;
    background: var(--canvas-selection);
    border-left: 3px solid var(--accent);
    border-radius: 4px;
    font-size: 12px;
  }

  .shortcuts-table-wrapper {
    flex: 1;
    overflow-y: auto;
    padding: 12px 20px;
  }

  .shortcuts-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 13px;
  }

  .shortcuts-table th {
    text-align: left;
    padding: 8px;
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    border-bottom: 1px solid var(--glass-border);
    opacity: 0.7;
  }

  .shortcuts-table td {
    padding: 8px;
    border-bottom: 1px solid var(--glass-border);
  }

  .cmd-cell {
    font-weight: 500;
  }

  .chord-badge {
    display: inline-block;
    padding: 2px 8px;
    background: var(--canvas-bg);
    border: 1px solid var(--glass-border);
    border-radius: 6px;
    font-family: monospace;
    font-size: 12px;
    font-weight: 600;
    color: var(--glass-text);
  }

  .record-btn {
    padding: 4px 10px;
    background: transparent;
    border: 1px solid var(--glass-border);
    color: var(--glass-text);
    border-radius: 6px;
    font-size: 11px;
    cursor: pointer;
    transition: background 0.15s ease, border-color 0.15s ease;
  }

  .record-btn:hover {
    background: var(--canvas-selection);
    border-color: var(--accent);
  }

  .record-btn.recording {
    background: var(--accent);
    color: var(--laser-core);
    border-color: var(--accent);
    animation: pulse 1s infinite alternate;
  }

  .modal-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 14px 20px;
    border-top: 1px solid var(--glass-border);
  }

  .reset-btn {
    background: transparent;
    border: 1px solid var(--glass-border);
    color: var(--glass-text);
    padding: 6px 14px;
    border-radius: 8px;
    font-size: 12px;
    cursor: pointer;
    transition: background 0.15s ease;
  }

  .reset-btn:hover {
    background: var(--canvas-selection);
  }

  .done-btn {
    background: var(--accent);
    color: var(--laser-core);
    border: none;
    padding: 6px 18px;
    border-radius: 8px;
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
    transition: opacity 0.15s ease;
  }

  .done-btn:hover {
    opacity: 0.9;
  }
</style>

<script lang="ts">
  // BUG-016: one tab per open document (appState.activeFiles); the selected tab is what the
  // Studio canvas shows. Switching keeps each document's view and markups (documentSessions.ts).
  import { appState } from '$lib/state.svelte';
  import {
    closeDocument,
    isDocumentDirty,
    saveDocument,
    whenDocumentLive,
  } from '$lib/views/studio/documentSessions';
  // The store's one saver (registered when this module loads); its message explains a refusal.
  import { lastSaveMessage } from '$lib/views/studio/studioSave';
  import StudioIcon from './StudioIcon.svelte';

  // Known limit (prototype): undo history is per visit; switching tabs clears it.
  const TAB_NOTE = 'Switching tabs clears Undo history.';

  /** F11 (review r2): the tab whose close is waiting for Save / Discard / Cancel. */
  let pendingClose = $state<string | null>(null);
  let saving = $state(false);
  let closeError = $state<string | null>(null);

  function fileName(path: string): string {
    return path.split(/[\\/]/).pop() || path;
  }

  function close(path: string) {
    // Through the store FIRST (review r5): the document is marked closed and, if live, the engines
    // are reset without stashing, before the canvas reacts to the tab change.
    closeDocument(path);
    appState.closeFile(path);
  }

  function requestClose(path: string) {
    if (isDocumentDirty(path)) {
      closeError = null;
      pendingClose = path;
    } else {
      close(path);
    }
  }

  async function saveThenClose() {
    const path = pendingClose;
    if (!path || saving) return;
    saving = true;
    closeError = null;
    try {
      if (appState.currentFile !== path) {
        appState.setActiveFile(path);
        if (!(await whenDocumentLive(path))) throw new Error('the document could not be opened');
      }
      if (!(await saveDocument(path))) throw new Error(lastSaveMessage || 'the save did not complete');
      pendingClose = null;
      close(path);
    } catch (e) {
      closeError = `Could not save "${fileName(path)}" (${e instanceof Error ? e.message : String(e)}). The tab stays open.`;
    } finally {
      saving = false;
    }
  }

  function discardAndClose() {
    const path = pendingClose;
    pendingClose = null;
    if (path) close(path);
  }
</script>

{#if appState.activeFiles.length > 0}
  <div class="doc-tabs" role="tablist" aria-label="Open documents" data-testid="doc-tabs">
    {#each appState.activeFiles as path (path)}
      {@const selected = path === appState.currentFile}
      <div class="doc-tab-wrap" class:selected>
        <button
          type="button"
          role="tab"
          class="doc-tab"
          aria-selected={selected}
          title="{path} — {TAB_NOTE}"
          data-testid="doc-tab"
          data-path={path}
          onclick={() => appState.setActiveFile(path)}
        >
          {fileName(path)}
        </button>
        <button
          type="button"
          class="doc-tab-close"
          aria-label="Close {fileName(path)}"
          title="Close {fileName(path)}"
          data-testid="doc-tab-close"
          onclick={() => requestClose(path)}
        >
          <StudioIcon name="close" size={10} />
        </button>
      </div>
    {/each}
  </div>
{/if}

{#if pendingClose}
  <div class="close-tab-backdrop" role="presentation">
    <div
      class="close-tab-dialog"
      role="alertdialog"
      aria-modal="true"
      aria-labelledby="close-tab-title"
      data-testid="close-tab-dialog"
    >
      <strong id="close-tab-title">Close "{fileName(pendingClose)}"?</strong>
      <span>It has unsaved markups or measurements. Save them to a new file first?</span>
      {#if closeError}
        <span class="close-tab-error" role="alert">{closeError}</span>
      {/if}
      <div class="close-tab-actions">
        <button type="button" data-testid="close-tab-cancel" disabled={saving} onclick={() => (pendingClose = null)}>
          Cancel
        </button>
        <button type="button" class="danger" data-testid="close-tab-discard" disabled={saving} onclick={discardAndClose}>
          Discard
        </button>
        <button type="button" class="primary" data-testid="close-tab-save" disabled={saving} onclick={saveThenClose}>
          {saving ? 'Saving…' : 'Save'}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .doc-tabs {
    display: flex;
    gap: 2px;
    padding: 4px 8px 0;
    overflow-x: auto;
    flex-shrink: 0;
    background: var(--glass-bg);
    border-bottom: 1px solid var(--glass-border);
  }

  .doc-tab-wrap {
    display: flex;
    align-items: center;
    border: 1px solid var(--glass-border);
    border-bottom: none;
    border-radius: 6px 6px 0 0;
    background: var(--canvas-bg);
    color: var(--glass-text);
    opacity: 0.75;
  }

  .doc-tab-wrap.selected {
    opacity: 1;
    border-color: var(--accent);
  }

  .doc-tab {
    max-width: 220px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    padding: 4px 8px;
    font-size: 12px;
    background: none;
    border: none;
    color: inherit;
    cursor: pointer;
  }

  .doc-tab-close {
    display: flex;
    align-items: center;
    padding: 4px 6px 4px 2px;
    background: none;
    border: none;
    color: inherit;
    cursor: pointer;
  }

  .close-tab-backdrop {
    position: fixed;
    inset: 0;
    z-index: 300;
    display: flex;
    align-items: flex-start;
    justify-content: center;
    padding-top: 120px;
    background: color-mix(in srgb, var(--canvas-bg) 60%, transparent);
  }

  .close-tab-dialog {
    width: 360px;
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 16px;
    border-radius: 12px;
    background: var(--glass-bg);
    border: 1px solid var(--glass-border);
    color: var(--glass-text);
  }

  .close-tab-error {
    color: var(--danger);
    font-size: 12px;
  }

  .close-tab-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }

  .close-tab-actions button {
    padding: 4px 12px;
    border-radius: 6px;
    border: 1px solid var(--glass-border);
    background: var(--canvas-bg);
    color: var(--glass-text);
    cursor: pointer;
  }

  .close-tab-actions button.primary {
    border-color: var(--accent);
  }

  .close-tab-actions button.danger {
    border-color: var(--danger);
  }

  .doc-tab:focus-visible,
  .doc-tab-close:focus-visible {
    outline: 2px solid var(--focus-ring);
  }
</style>

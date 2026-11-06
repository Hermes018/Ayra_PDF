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
  import Button from '$lib/components/ui/Button.svelte';
  import IconButton from '$lib/components/ui/IconButton.svelte';
  import Dialog from '$lib/components/ui/Dialog.svelte';

  // Known limit (prototype): undo history is per visit; switching tabs clears it.
  const TAB_NOTE = 'Switching tabs clears Undo history.';

  /** F11 (review r2): the tab whose close is waiting for Save / Discard / Cancel. */
  let pendingClose = $state<string | null>(null);
  let saving = $state(false);
  let closeError = $state<string | null>(null);
  // ui/Dialog is driven by `pendingClose`. Escape / backdrop cancel the close, except while a save runs.
  let dialogOpen = $state(false);
  $effect(() => {
    dialogOpen = pendingClose !== null;
  });
  function onDialogClose() {
    if (saving) dialogOpen = true;
    else pendingClose = null;
  }

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
        <Button
          variant="plain"
          size="sm"
          role="tab"
          {selected}
          title="{path} — {TAB_NOTE}"
          data-testid="doc-tab"
          attrs={{ 'data-path': path }}
          onclick={() => appState.setActiveFile(path)}
        >
          <span class="doc-tab-name">{fileName(path)}</span>
        </Button>
        <IconButton
          icon="close"
          size="sm"
          ariaLabel="Close {fileName(path)}"
          data-testid="doc-tab-close"
          onclick={() => requestClose(path)}
        />
      </div>
    {/each}
  </div>
{/if}

{#if pendingClose}
  <Dialog
    bind:open={dialogOpen}
    variant="alert"
    title={`Close "${fileName(pendingClose)}"?`}
    onclose={onDialogClose}
    data-testid="close-tab-dialog"
  >
    <p class="close-tab-body">It has unsaved markups or measurements. Save them to a new file first?</p>
    {#if closeError}
      <p class="close-tab-error" role="alert">{closeError}</p>
    {/if}
    {#snippet actions()}
      <Button
        variant="secondary"
        data-testid="close-tab-cancel"
        disabled={saving}
        disabledReason="Saving. Wait for it to finish."
        onclick={() => (pendingClose = null)}
      >
        Cancel
      </Button>
      <Button
        variant="destructive"
        data-testid="close-tab-discard"
        disabled={saving}
        disabledReason="Saving. Wait for it to finish."
        onclick={discardAndClose}
      >
        Discard changes
      </Button>
      <Button variant="primary" data-testid="close-tab-save" busy={saving} onclick={saveThenClose}>
        {saving ? 'Saving…' : 'Save a copy'}
      </Button>
    {/snippet}
  </Dialog>
{/if}

<style>
  .doc-tabs {
    display: flex;
    gap: var(--space-1);
    padding: var(--space-1) var(--space-2) 0;
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
    border-radius: var(--radius-xs) var(--radius-xs) 0 0;
    background: var(--canvas-bg);
    color: var(--glass-text);
    opacity: 0.75;
  }

  .doc-tab-wrap.selected {
    opacity: 1;
    border-color: var(--accent);
  }

  .doc-tab-name {
    max-width: 220px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .close-tab-body,
  .close-tab-error {
    margin: 0;
  }

  .close-tab-error {
    color: var(--danger);
    font-size: var(--font-size-footnote);
  }
</style>

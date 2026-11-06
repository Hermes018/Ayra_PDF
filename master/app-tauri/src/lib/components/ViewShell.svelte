<script lang="ts">
  import type { Snippet } from 'svelte';
  import { appState } from '$lib/state.svelte';
  import { pickFilesDialog } from '$lib/ipc';
  import Button from '$lib/components/ui/Button.svelte';
  import ErrorState from '$lib/components/ui/ErrorState.svelte';
  import Progress from '$lib/components/ui/Progress.svelte';

  // Shared frame for the tool views (Optimize, Edit, Forms, Security, ...): title, file bar with "Browse files",
  // a tools column, a main area, and the screen's job feedback (Progress with Cancel, ErrorState with Retry).
  let {
    title,
    showFileBar = true,
    tools,
    main,
    busyLabel = null,
    oncancel,
    error = null,
  }: {
    title: string;
    showFileBar?: boolean;
    tools: Snippet;
    main: Snippet;
    busyLabel?: string | null;
    oncancel?: () => void;
    error?: { what: string; next: string; retry: () => void } | null;
  } = $props();

  let currentPath = $derived(appState.currentFile);
  let pickerFailed = $state(false);

  async function browse() {
    pickerFailed = false;
    try {
      const files = await pickFilesDialog(true, ['pdf']);
      if (files && files.length > 0) appState.addFiles(files);
    } catch (e) {
      console.error('Failed to open file picker:', e);
      pickerFailed = true;
    }
  }
</script>

<div class="view-container">
  <div class="header">
    <h2>{title}</h2>
    {#if showFileBar}
      <div class="controls">
        <Button variant="primary" onclick={browse}>Browse files</Button>
        <span class="active-file-path">{currentPath || 'No file selected. Drag and drop a PDF.'}</span>
      </div>
    {/if}
  </div>

  {#if pickerFailed}
    <ErrorState what="Couldn't open the file picker." next="Try again, or drag a PDF into the window." retry={browse} />
  {/if}
  {#if error}
    <ErrorState what={error.what} next={error.next} retry={error.retry} data-testid="view-op-error" />
  {/if}
  {#if busyLabel}
    <Progress value={null} label={busyLabel} oncancel={oncancel} data-testid="view-op-progress" />
  {/if}

  <div class="content">
    <div class="sidebar">{@render tools()}</div>
    <div class="viewer-main">{@render main()}</div>
  </div>
</div>

<style>
  .view-container {
    display: flex;
    flex-direction: column;
    height: 100%;
    padding: var(--space-4);
    gap: var(--space-4);
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    background-color: var(--glass-bg-surface);
    -webkit-backdrop-filter: var(--glass-blur);
    backdrop-filter: var(--glass-blur);
    padding: var(--space-4) var(--space-6);
    border-radius: var(--radius-lg);
    border: 1px solid var(--glass-border-subtle);
    box-shadow: inset 0 1px 0 0 var(--glass-border-specular), var(--glass-shadow-sm);
  }

  h2 {
    margin: 0;
    font-size: var(--font-size-title1);
    color: var(--accent-primary);
  }

  .controls {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    flex-wrap: wrap;
  }

  .active-file-path {
    background-color: var(--glass-bg-input);
    padding: var(--space-2) var(--space-4);
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    font-family: var(--font-mono);
    font-size: var(--font-size-footnote);
    border: 1px solid var(--glass-border-subtle);
    word-break: break-all;
  }

  .content {
    display: flex;
    flex: 1;
    gap: var(--space-4);
    min-height: 0;
  }

  .sidebar {
    width: 300px;
    background-color: var(--glass-bg-surface);
    border-radius: var(--radius-lg);
    border: 1px solid var(--glass-border-subtle);
    padding: var(--space-4);
    display: flex;
    flex-direction: column;
    gap: var(--space-6);
    overflow-y: auto;
  }

  .viewer-main {
    flex: 1;
    background-color: var(--glass-bg-surface);
    border-radius: var(--radius-lg);
    border: 1px solid var(--glass-border-subtle);
    display: flex;
    align-items: center;
    justify-content: center;
    min-width: 0;
  }
</style>

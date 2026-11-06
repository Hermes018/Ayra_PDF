<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import { getCurrentWebview } from '@tauri-apps/api/webview';
    import { appState } from '$lib/state.svelte';
    import { pickFilesDialog } from '$lib/ipc';
    import Button from '$lib/components/ui/Button.svelte';
    import ErrorState from '$lib/components/ui/ErrorState.svelte';

    let {
        onFilesSelected,
        mode = 'overlay',
        extensions = ['pdf'],
        multiple = true
    }: {
        onFilesSelected?: (paths: string[]) => void;
        mode?: 'overlay' | 'inline';
        extensions?: string[];
        multiple?: boolean;
    } = $props();
    
    let isDragging = $state(false);
    let pickerFailed = $state(false);
    let unlisten: (() => void) | undefined;

    async function handleBrowse() {
        pickerFailed = false;
        try {
            const paths = await pickFilesDialog(multiple, extensions);
            if (paths && paths.length > 0) {
                if (onFilesSelected) {
                    onFilesSelected(paths);
                } else {
                    appState.addFiles(paths);
                }
            }
        } catch (e) {
            console.error('Failed to open file picker:', e);
            pickerFailed = true;
        }
    }

    function handleDropPaths(paths: string[]) {
        if (Array.isArray(paths) && paths.length > 0) {
            if (onFilesSelected) {
                onFilesSelected(paths);
            } else {
                appState.addFiles(paths);
            }
        }
    }

    onMount(async () => {
        unlisten = await getCurrentWebview().onDragDropEvent((event) => {
            if (event.payload.type === 'enter' || event.payload.type === 'over') {
                isDragging = true;
            } else if (event.payload.type === 'leave') {
                isDragging = false;
            } else if (event.payload.type === 'drop') {
                isDragging = false;
                handleDropPaths(event.payload.paths);
            }
        });
    });

    onDestroy(() => {
        if (unlisten) unlisten();
    });
</script>

{#if mode === 'inline'}
    <div class="inline-dropzone">
        <span class="icon" aria-hidden="true">📄</span>
        <div class="drop-text">
            <h3>Drag and drop PDFs here</h3>
            <p>or choose files from this computer</p>
        </div>
        <Button variant="secondary" onclick={handleBrowse}>Browse files</Button>
        {#if pickerFailed}
            <ErrorState what="Couldn't open the file picker." next="Try again, or drag the PDFs into the window." retry={handleBrowse} />
        {/if}
    </div>
{:else if isDragging}
    <div class="dropzone-overlay" role="presentation">
        <div class="drop-message">
            <span class="icon" aria-hidden="true">📄</span>
            <h2>Drop PDFs to open them</h2>
            <Button variant="secondary" onclick={handleBrowse}>Browse files</Button>
            {#if pickerFailed}
                <ErrorState what="Couldn't open the file picker." next="Try again, or drop the PDFs here." retry={handleBrowse} />
            {/if}
        </div>
    </div>
{/if}

<style>
    .dropzone-overlay {
        position: fixed;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        background-color: var(--glass-bg-active);
        -webkit-backdrop-filter: var(--glass-blur);
        backdrop-filter: var(--glass-blur);
        z-index: 9999;
        display: flex;
        align-items: center;
        justify-content: center;
        border: 4px dashed var(--accent-primary);
    }

    .drop-message {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: var(--space-4);
        background-color: var(--glass-bg-elevated);
        -webkit-backdrop-filter: var(--glass-blur);
        backdrop-filter: var(--glass-blur);
        padding: var(--space-8) calc(var(--space-8) * 2);
        border-radius: var(--radius-xl, 20px);
        box-shadow: var(--glass-shadow-lg);
        border: 1px solid var(--glass-border-subtle);
    }

    .inline-dropzone {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        gap: var(--space-4);
        padding: var(--space-8) var(--space-6);
        border: 1.5px dashed var(--glass-border-subtle);
        border-radius: var(--radius-lg, 14px);
        background: var(--glass-bg-surface);
        -webkit-backdrop-filter: var(--glass-blur);
        backdrop-filter: var(--glass-blur);
        transition: border-color var(--dur-fast, 120ms) ease, background-color var(--dur-fast, 120ms) ease;
        text-align: center;
        margin: var(--space-4) 0;
    }

    .inline-dropzone:hover {
        border-color: var(--accent-primary);
        background: var(--glass-bg-hover);
    }

    .drop-text h3 {
        margin: 0 0 var(--space-1) 0;
        color: var(--text-primary);
        font-size: var(--font-size-title2);
    }

    .drop-text p {
        margin: 0;
        color: var(--text-secondary);
        font-size: var(--font-size-body);
    }

    .icon {
        font-size: calc(var(--font-size-title1) * 2.5);
    }

    h2 {
        margin: 0;
        color: var(--text-primary);
    }
</style>

<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import { getCurrentWebview } from '@tauri-apps/api/webview';
    import { appState } from '$lib/state.svelte';
    import { pickFilesDialog } from '$lib/ipc';

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
    let unlisten: (() => void) | undefined;

    async function handleBrowse() {
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
    <div
        class="inline-dropzone"
        role="button"
        tabindex="0"
        onclick={handleBrowse}
        onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); handleBrowse(); } }}
    >
        <span class="icon" aria-hidden="true">📄</span>
        <div class="drop-text">
            <h3>Drag & Drop PDFs here</h3>
            <p>or click to browse from local storage</p>
        </div>
        <button
            type="button"
            class="browse-btn"
            onclick={(e) => { e.stopPropagation(); handleBrowse(); }}
        >
            📁 Browse Files...
        </button>
    </div>
{:else if isDragging}
    <div
        class="dropzone-overlay"
        role="button"
        tabindex="0"
        onclick={handleBrowse}
        onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); handleBrowse(); } }}
    >
        <div class="drop-message">
            <span class="icon" aria-hidden="true">📄</span>
            <h2>Drop PDFs here</h2>
            <button
                type="button"
                class="browse-btn"
                onclick={(e) => { e.stopPropagation(); handleBrowse(); }}
            >
                📁 Browse Files...
            </button>
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
        background-color: rgba(137, 180, 250, 0.25);
        backdrop-filter: blur(4px);
        z-index: 9999;
        display: flex;
        align-items: center;
        justify-content: center;
        border: 4px dashed #89b4fa;
        cursor: pointer;
    }

    .drop-message {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 1rem;
        background-color: #1e1e2e;
        padding: 3rem 4rem;
        border-radius: 1rem;
        box-shadow: 0 10px 25px rgba(0,0,0,0.5);
        border: 1px solid #45475a;
    }

    .inline-dropzone {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        gap: 1rem;
        padding: 2.5rem 1.5rem;
        border: 2px dashed #45475a;
        border-radius: 12px;
        background: #181825;
        cursor: pointer;
        transition: all 0.2s ease;
        text-align: center;
        margin: 1rem 0;
    }

    .inline-dropzone:hover {
        border-color: #89b4fa;
        background: rgba(137, 180, 250, 0.05);
    }

    .drop-text h3 {
        margin: 0 0 0.25rem 0;
        color: #cdd6f4;
        font-size: 1.15rem;
    }

    .drop-text p {
        margin: 0;
        color: #a6adc8;
        font-size: 0.9rem;
    }

    .browse-btn {
        display: inline-flex;
        align-items: center;
        gap: 0.5rem;
        padding: 0.6rem 1.25rem;
        background: #89b4fa;
        color: #11111b;
        font-weight: 600;
        font-size: 0.9rem;
        border: none;
        border-radius: 8px;
        cursor: pointer;
        transition: background 0.15s ease, transform 0.1s ease;
    }

    .browse-btn:hover {
        background: #b4befe;
        transform: translateY(-1px);
    }

    .browse-btn:active {
        transform: translateY(0);
    }

    .icon {
        font-size: 3.5rem;
    }

    h2 {
        margin: 0;
        color: #cdd6f4;
    }
</style>

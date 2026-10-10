<script lang="ts">
    import PdfViewer from '$lib/components/PdfViewer.svelte';
    import { appState } from '$lib/state.svelte';
    import { IpcClient } from '$lib/ipc';
    import type { JobRequest } from '$lib/bindings/JobRequest';

    // The viewer reactive path comes directly from the global active files
    let currentPath = $derived(appState.currentFile);
    let activeFiles = $derived(appState.activeFiles);

    async function handleMerge() {
        if (activeFiles.length < 2) return;
        
        const outputPath = activeFiles[0].replace(/\.pdf$/i, '_merged.pdf');
        const req: JobRequest = {
            op: "Merge",
            input_paths: activeFiles,
            output_path: outputPath
        };

        try {
            const jobId = await IpcClient.submitJob(req);
            console.log("Merge job submitted with ID:", jobId);
        } catch (e) {
            console.error("Failed to submit merge job:", e);
        }
    }
</script>

<div class="view-container">
    <div class="header">
        <h2>Organize PDFs</h2>
        <div class="controls">
            <span class="active-file-path">
                {currentPath || 'No file selected. Drag and drop a PDF.'}
            </span>
        </div>
    </div>

    <div class="viewer-layout">
        <div class="tools-sidebar">
            <h3>Tools</h3>
            <p>Select pages to extract, delete, or reorder.</p>

            <div class="merge-section">
                <h4>Files to Merge</h4>
                {#if activeFiles.length > 0}
                    <ul class="file-list">
                        {#each activeFiles as file}
                            <li>{file.split(/[\/\\]/).pop()}</li>
                        {/each}
                    </ul>
                    <button class="merge-button" onclick={handleMerge} disabled={activeFiles.length < 2}>
                        Merge All
                    </button>
                {:else}
                    <p class="empty-text">Drag files here to merge.</p>
                {/if}
            </div>
        </div>
        
        <!-- Main PDF Viewer Area -->
        <div class="viewer-main">
            <PdfViewer filePath={currentPath} />
        </div>
    </div>
</div>

<style>
    .view-container {
        display: flex;
        flex-direction: column;
        height: 100%;
        padding: 1rem;
        gap: 1rem;
    }

    .header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        background-color: #1e1e2e;
        padding: 1rem 1.5rem;
        border-radius: 8px;
        border: 1px solid #313244;
    }

    h2 {
        margin: 0;
        font-size: 1.5rem;
        color: #cba6f7;
    }

    .controls {
        display: flex;
        gap: 0.5rem;
    }

    .active-file-path {
        background-color: #11111b;
        padding: 0.5rem 1rem;
        border-radius: 6px;
        color: #a6adc8;
        font-family: monospace;
        font-size: 0.9rem;
        border: 1px solid #45475a;
    }

    .viewer-layout {
        display: flex;
        flex: 1;
        gap: 1rem;
        min-height: 0; /* Important for flex child scrolling */
    }

    .tools-sidebar {
        width: 250px;
        background-color: #1e1e2e;
        border: 1px solid #313244;
        border-radius: 8px;
        padding: 1rem;
    }

    .tools-sidebar h3 {
        margin-top: 0;
        color: #f38ba8;
    }

    .viewer-main {
        flex: 1;
        border: 1px solid #313244;
        border-radius: 8px;
        overflow: hidden;
    }

    .merge-section {
        margin-top: 2rem;
        padding-top: 1rem;
        border-top: 1px solid #313244;
    }

    .merge-section h4 {
        margin-top: 0;
        margin-bottom: 0.5rem;
        color: #cba6f7;
    }

    .file-list {
        list-style: none;
        padding: 0;
        margin: 0 0 1rem 0;
        max-height: 150px;
        overflow-y: auto;
    }

    .file-list li {
        padding: 0.25rem 0.5rem;
        background-color: #11111b;
        margin-bottom: 0.25rem;
        border-radius: 4px;
        font-size: 0.85rem;
        color: #a6adc8;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .merge-button {
        width: 100%;
        padding: 0.75rem;
        background-color: #89b4fa;
        color: #11111b;
        border: none;
        border-radius: 6px;
        font-weight: bold;
        cursor: pointer;
        transition: opacity 0.2s;
    }

    .merge-button:disabled {
        background-color: #45475a;
        color: #7f849c;
        cursor: not-allowed;
    }

    .merge-button:not(:disabled):hover {
        opacity: 0.9;
    }

    .empty-text {
        font-size: 0.85rem;
        color: #6c7086;
        font-style: italic;
    }
</style>

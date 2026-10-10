<script lang="ts">
    import PdfViewer from '$lib/components/PdfViewer.svelte';
    import { appState } from '$lib/state.svelte';
    import { IpcClient } from '$lib/ipc';
    import type { JobRequest } from '$lib/bindings/JobRequest';

    // The viewer reactive path comes directly from the global active files
    let currentPath = $derived(appState.currentFile);
    let activeFiles = $derived(appState.activeFiles);

    let extractPagesInput = $state("");
    let deletePagesInput = $state("");
    let rotatePagesInput = $state("");
    let rotateDegrees = $state(90);

    function parsePages(input: string): number[] {
        const pages = new Set<number>();
        const parts = input.split(',');
        for (let part of parts) {
            part = part.trim();
            if (!part) continue;
            if (part.includes('-')) {
                const [startStr, endStr] = part.split('-');
                const start = parseInt(startStr, 10);
                const end = parseInt(endStr, 10);
                if (!isNaN(start) && !isNaN(end) && start <= end) {
                    for (let i = start; i <= end; i++) pages.add(i);
                }
            } else {
                const page = parseInt(part, 10);
                if (!isNaN(page)) pages.add(page);
            }
        }
        return Array.from(pages).sort((a, b) => a - b);
    }

    async function handleExtract() {
        if (!currentPath) return;
        const pages = parsePages(extractPagesInput);
        if (pages.length === 0) {
            alert("Please enter valid page numbers (e.g. 1, 3, 5-7).");
            return;
        }

        const outputPath = currentPath.replace(/\.pdf$/i, '_extracted.pdf');
        const req: JobRequest = {
            op: "ExtractPages",
            input_path: currentPath,
            output_path: outputPath,
            pages: pages
        };

        try {
            const jobId = await IpcClient.submitJob(req);
            console.log("Extract job submitted with ID:", jobId);
        } catch (e) {
            console.error("Failed to submit extract job:", e);
        }
    }

    async function handleDelete() {
        if (!currentPath) return;
        const pages = parsePages(deletePagesInput);
        if (pages.length === 0) {
            alert("Please enter valid page numbers to delete.");
            return;
        }

        const outputPath = currentPath.replace(/\.pdf$/i, '_deleted.pdf');
        const req: JobRequest = {
            op: "DeletePages",
            input_path: currentPath,
            output_path: outputPath,
            pages: pages
        };

        try {
            const jobId = await IpcClient.submitJob(req);
            console.log("Delete job submitted with ID:", jobId);
        } catch (e) {
            console.error("Failed to submit delete job:", e);
        }
    }

    async function handleRotate() {
        if (!currentPath) return;
        const pages = parsePages(rotatePagesInput);
        if (pages.length === 0) {
            alert("Please enter valid page numbers to rotate.");
            return;
        }

        const outputPath = currentPath.replace(/\.pdf$/i, '_rotated.pdf');
        const req: JobRequest = {
            op: "RotatePages",
            input_path: currentPath,
            output_path: outputPath,
            pages: pages,
            degrees: rotateDegrees
        };

        try {
            const jobId = await IpcClient.submitJob(req);
            console.log("Rotate job submitted with ID:", jobId);
        } catch (e) {
            console.error("Failed to submit rotate job:", e);
        }
    }

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

            <div class="tool-section">
                <h4>Extract Pages</h4>
                {#if currentPath}
                    <input 
                        type="text" 
                        class="pages-input" 
                        bind:value={extractPagesInput} 
                        placeholder="e.g. 1, 3, 5-10" 
                    />
                    <button class="action-button" onclick={handleExtract} disabled={!extractPagesInput.trim()}>
                        Extract
                    </button>
                {:else}
                    <p class="empty-text">Select a file to extract pages.</p>
                {/if}
            </div>

            <div class="tool-section">
                <h4>Delete Pages</h4>
                {#if currentPath}
                    <input 
                        type="text" 
                        class="pages-input" 
                        bind:value={deletePagesInput} 
                        placeholder="e.g. 1, 3, 5-10" 
                    />
                    <button class="action-button action-danger" onclick={handleDelete} disabled={!deletePagesInput.trim()}>
                        Delete
                    </button>
                {:else}
                    <p class="empty-text">Select a file to delete pages.</p>
                {/if}
            </div>

            <div class="tool-section">
                <h4>Rotate Pages</h4>
                {#if currentPath}
                    <input 
                        type="text" 
                        class="pages-input" 
                        bind:value={rotatePagesInput} 
                        placeholder="e.g. 1, 3, 5-10" 
                    />
                    <select class="pages-input" bind:value={rotateDegrees}>
                        <option value={90}>90° Clockwise</option>
                        <option value={180}>180°</option>
                        <option value={-90}>90° Counter-Clockwise</option>
                    </select>
                    <button class="action-button" onclick={handleRotate} disabled={!rotatePagesInput.trim()}>
                        Rotate
                    </button>
                {:else}
                    <p class="empty-text">Select a file to rotate pages.</p>
                {/if}
            </div>

            <div class="tool-section">
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

    .tool-section {
        margin-top: 2rem;
        padding-top: 1rem;
        border-top: 1px solid #313244;
    }

    .tool-section h4 {
        margin-top: 0;
        margin-bottom: 0.5rem;
        color: #cba6f7;
    }

    .pages-input {
        width: 100%;
        padding: 0.5rem;
        background-color: #11111b;
        border: 1px solid #45475a;
        color: #cdd6f4;
        border-radius: 4px;
        margin-bottom: 0.75rem;
        box-sizing: border-box;
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

    .action-button, .merge-button {
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

    .action-button:disabled, .merge-button:disabled {
        background-color: #45475a;
        color: #7f849c;
        cursor: not-allowed;
    }

    .action-button:not(:disabled):hover, .merge-button:not(:disabled):hover {
        opacity: 0.9;
    }

    .action-danger {
        background-color: #f38ba8;
        color: #11111b;
    }


    .empty-text {
        font-size: 0.85rem;
        color: #6c7086;
        font-style: italic;
    }
</style>

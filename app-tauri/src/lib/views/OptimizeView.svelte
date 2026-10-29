<script lang="ts">
    import { appState } from '$lib/state.svelte';
    import { IpcClient, pickFilesDialog } from '$lib/ipc';
    import type { JobRequest } from '$lib/bindings/JobRequest';
    import type { OptimizeProfile } from '$lib/bindings/OptimizeProfile';

    let currentPath = $derived(appState.currentFile);
    let activeFiles = $derived(appState.activeFiles);

    let selectedProfile = $state<OptimizeProfile>("Medium");

    async function handleCompress() {
        if (!currentPath) return;

        const outputPath = currentPath.replace(/\.pdf$/i, `_compressed_${selectedProfile}.pdf`);
        const req: JobRequest = {
            op: "Compress",
            input_path: currentPath,
            output_path: outputPath,
            profile: selectedProfile
        };

        try {
            const jobId = await IpcClient.submitJob(req);
            console.log("Compress job submitted with ID:", jobId);
        } catch (e) {
            console.error("Failed to submit compress job:", e);
        }
    }
    async function handleRepair() {
        if (!currentPath) return;

        const outputPath = currentPath.replace(/\.pdf$/i, `_repaired.pdf`);
        const req: JobRequest = {
            op: "Repair",
            input_path: currentPath,
            output_path: outputPath
        };

        try {
            const jobId = await IpcClient.submitJob(req);
            console.log("Repair job submitted with ID:", jobId);
        } catch (e) {
            console.error("Failed to submit repair job:", e);
        }
    }

    async function handleOcr() {
        if (!currentPath) return;

        const outputPath = currentPath.replace(/\.pdf$/i, `_ocr.pdf`);
        const req: JobRequest = {
            op: "Ocr",
            input_path: currentPath,
            output_path: outputPath,
            lang: "en" // Only Latin supported right now natively
        };

        try {
            const jobId = await IpcClient.submitJob(req);
            console.log("OCR job submitted with ID:", jobId);
        } catch (e) {
            console.error("Failed to submit OCR job:", e);
        }
    }

    async function handleBrowseFiles() {
        try {
            const files = await pickFilesDialog(true, ['pdf']);
            if (files && files.length > 0) {
                appState.addFiles(files);
            }
        } catch (e) {
            console.error("Failed to open file picker:", e);
        }
    }
</script>

<div class="view-container">
    <div class="header">
        <h2>Optimize PDF</h2>
        <div class="controls">
            <button type="button" class="browse-button" onclick={handleBrowseFiles} aria-label="Browse PDF files">
                📁 Browse Files
            </button>
            <span class="active-file-path">
                {currentPath || 'No file selected. Drag and drop a PDF.'}
            </span>
        </div>
    </div>

    <div class="content">
        <div class="sidebar">
            <div class="tool-section">
                <h4>Compress Document</h4>
                {#if currentPath}
                    <p class="file-label">Active: {currentPath.split(/[\/\\]/).pop()}</p>
                    
                    <label class="profile-label" for="profileSelect">Compression Profile:</label>
                    <select id="profileSelect" class="profile-select" bind:value={selectedProfile}>
                        <option value="Low">Low (High Quality, Large Size)</option>
                        <option value="Medium">Medium (Balanced)</option>
                        <option value="High">High (Low Quality, Small Size)</option>
                    </select>

                    <button class="action-button" onclick={handleCompress}>
                        Compress Now
                    </button>
                    
                    <p class="help-text">
                        The compressed file will be saved alongside the original.
                    </p>
                {:else}
                    <p class="empty-text">Select or drag a file to compress.</p>
                {/if}
            </div>

            <div class="tool-section">
                <h4>Repair Document</h4>
                {#if currentPath}
                    <p class="help-text">
                        Attempts to fix broken xref tables, recover orphaned objects, and rewrite the PDF cleanly.
                    </p>
                    <button class="action-button secondary-button" onclick={handleRepair}>
                        Repair Now
                    </button>
                {:else}
                    <p class="empty-text">Select or drag a file to repair.</p>
                {/if}
            </div>

            <div class="tool-section">
                <h4>OCR (Make Text Searchable)</h4>
                {#if currentPath}
                    <p class="help-text">
                        Uses native, local ML to recognize Latin alphabet text and overlay an invisible searchable layer.
                    </p>
                    <button class="action-button secondary-button" onclick={handleOcr}>
                        Recognize Text (OCR)
                    </button>
                {:else}
                    <p class="empty-text">Select or drag a file to OCR.</p>
                {/if}
            </div>
        </div>
        
        <div class="viewer-main">
            {#if currentPath}
                <div class="placeholder-preview">
                    <p>Compression preview & sizes will be displayed here in a future update.</p>
                </div>
            {:else}
                <div class="empty-state">
                    <p>No file selected.</p>
                </div>
            {/if}
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
        background-color: var(--glass-bg-surface);
        -webkit-backdrop-filter: var(--glass-blur);
        backdrop-filter: var(--glass-blur);
        padding: 1rem 1.5rem;
        border-radius: var(--radius-lg, 14px);
        border: 1px solid var(--glass-border-subtle);
        box-shadow: inset 0 1px 0 0 var(--glass-border-specular), var(--glass-shadow-sm);
    }

    h2 {
        margin: 0;
        font-size: 1.5rem;
        color: var(--accent-primary);
    }

    .controls {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        flex-wrap: wrap;
    }

    .browse-button {
        display: inline-flex;
        align-items: center;
        gap: 0.5rem;
        padding: 0.5rem 1rem;
        background: var(--accent-primary);
        color: var(--text-on-accent);
        font-weight: 600;
        font-size: 0.85rem;
        border: none;
        border-radius: var(--radius-md, 10px);
        box-shadow: inset 0 1px 0 0 var(--glass-border-specular), var(--glass-shadow-sm);
        cursor: pointer;
        white-space: nowrap;
        transition: opacity var(--dur-fast, 120ms) ease, transform var(--dur-fast, 120ms) ease;
    }

    .browse-button:hover {
        opacity: 0.9;
        transform: translateY(-1px);
    }

    .browse-button:active {
        transform: translateY(0);
    }

    .active-file-path {
        background-color: var(--glass-bg-input);
        padding: 0.5rem 1rem;
        border-radius: var(--radius-sm, 8px);
        color: var(--text-secondary);
        font-family: var(--font-mono, monospace);
        font-size: 0.85rem;
        border: 1px solid var(--glass-border-subtle);
        word-break: break-all;
    }

    .content {
        display: flex;
        flex: 1;
        gap: 1rem;
        min-height: 0;
    }

    .sidebar {
        width: 300px;
        background-color: var(--glass-bg-surface);
        border-radius: var(--radius-lg, 14px);
        border: 1px solid var(--glass-border-subtle);
        padding: 1rem;
        display: flex;
        flex-direction: column;
        gap: 1.5rem;
        overflow-y: auto;
    }

    .viewer-main {
        flex: 1;
        background-color: var(--glass-bg-surface);
        border-radius: var(--radius-lg, 14px);
        border: 1px solid var(--glass-border-subtle);
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .tool-section {
        background-color: var(--glass-bg-input);
        padding: 1rem;
        border-radius: var(--radius-md, 10px);
        border: 1px solid var(--glass-border-subtle);
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
    }

    .tool-section h4 {
        margin: 0 0 0.5rem 0;
        color: var(--text-primary);
        font-size: 1.1rem;
    }

    .file-label {
        margin: 0;
        color: var(--text-secondary);
        font-size: 0.9rem;
        word-break: break-all;
    }

    .profile-label {
        font-size: 0.9rem;
        color: var(--text-secondary);
    }

    .profile-select {
        width: 100%;
        background-color: var(--glass-bg-input);
        border: 1px solid var(--glass-border-subtle);
        color: var(--text-primary);
        padding: 0.5rem;
        border-radius: var(--radius-sm, 8px);
        outline: none;
    }

    .action-button {
        width: 100%;
        background-color: var(--accent-primary);
        color: var(--text-on-accent);
        border: none;
        padding: 0.65rem 1rem;
        border-radius: var(--radius-md, 10px);
        font-weight: 600;
        cursor: pointer;
        transition: opacity var(--dur-fast, 120ms) ease;
    }

    .action-button:hover {
        opacity: 0.9;
    }

    .secondary-button {
        background-color: var(--glass-bg-elevated);
        color: var(--text-primary);
    }

    .secondary-button:hover {
        background-color: var(--glass-bg-hover);
    }

    .help-text {
        font-size: 0.8rem;
        color: var(--text-secondary);
        margin-top: 0.5rem;
        line-height: 1.4;
    }

    .empty-text, .empty-state p {
        color: var(--text-tertiary);
        text-align: center;
        font-style: italic;
    }
    
    .placeholder-preview {
        color: var(--text-secondary);
        text-align: center;
    }
</style>

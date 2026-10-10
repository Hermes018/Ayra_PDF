<script lang="ts">
    import { appState } from '$lib/state.svelte';
    import { IpcClient } from '$lib/ipc';
    import type { JobRequest } from '$lib/bindings/JobRequest';

    let currentPath = $derived(appState.currentFile);

    async function handleConvert(type: 'ImageToPdf' | 'DocxToPdf' | 'HtmlToPdf') {
        if (!currentPath) return;

        const outputPath = currentPath + '.pdf';

        const req: JobRequest = {
            op: type,
            input_path: currentPath,
            output_path: outputPath
        } as JobRequest;

        try {
            const jobId = await IpcClient.submitJob(req);
            console.log(`${type} job submitted with ID:`, jobId);
        } catch (e) {
            console.error(`Failed to submit ${type} job:`, e);
        }
    }
</script>

<div class="view-container">
    <div class="header">
        <h2>Convert to PDF</h2>
    </div>

    <div class="content">
        <div class="sidebar">
            <div class="tool-section">
                <h4>Convert options</h4>
                {#if currentPath}
                    <p class="file-label">Active: {currentPath.split(/[\/\\]/).pop()}</p>
                    
                    <button class="action-button" onclick={() => handleConvert('ImageToPdf')}>
                        Image to PDF
                    </button>
                    
                    <button class="action-button" onclick={() => handleConvert('DocxToPdf')}>
                        Word (.docx) to PDF
                    </button>
                    
                    <button class="action-button" onclick={() => handleConvert('HtmlToPdf')}>
                        HTML to PDF
                    </button>
                    
                    <p class="help-text">
                        Converts the selected file into a PDF natively.
                    </p>
                {:else}
                    <p class="empty-text">Select or drag a file to convert.</p>
                {/if}
            </div>
        </div>
        
        <div class="viewer-main">
            {#if currentPath}
                <div class="placeholder-preview">
                    <p>File ready for conversion.</p>
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

    .header h2 {
        margin: 0;
        font-size: 1.5rem;
        color: var(--text-color);
    }

    .content {
        display: flex;
        flex: 1;
        gap: 1.5rem;
        min-height: 0;
    }

    .sidebar {
        width: 300px;
        display: flex;
        flex-direction: column;
        gap: 1.5rem;
        overflow-y: auto;
    }

    .tool-section {
        background: var(--surface-bg);
        border: 1px solid var(--border-color);
        border-radius: 8px;
        padding: 1rem;
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
    }

    .tool-section h4 {
        margin: 0;
        color: var(--text-color);
        font-size: 1.1rem;
    }

    .file-label {
        font-size: 0.85rem;
        color: var(--primary-color);
        word-break: break-all;
        margin: 0;
    }

    .action-button {
        background: var(--primary-color);
        color: white;
        border: none;
        border-radius: 4px;
        padding: 0.75rem;
        font-weight: 600;
        cursor: pointer;
        transition: opacity 0.2s;
        margin-top: 0.5rem;
    }
    
    .action-button:hover {
        opacity: 0.9;
    }

    .help-text {
        font-size: 0.8rem;
        color: var(--text-muted);
        margin: 0;
        line-height: 1.4;
    }

    .empty-text {
        color: var(--text-muted);
        font-style: italic;
        margin: 0;
        font-size: 0.9rem;
    }

    .viewer-main {
        flex: 1;
        background: var(--surface-bg);
        border: 1px solid var(--border-color);
        border-radius: 8px;
        display: flex;
        align-items: center;
        justify-content: center;
        overflow: hidden;
    }

    .placeholder-preview, .empty-state {
        color: var(--text-muted);
        text-align: center;
        padding: 2rem;
    }
</style>

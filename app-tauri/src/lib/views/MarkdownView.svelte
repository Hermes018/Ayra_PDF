<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import { appState } from '$lib/state.svelte';
    import { IpcClient, pickFilesDialog, pickFolderDialog, saveFileDialog } from '$lib/ipc';
    import type { JobRequest } from '$lib/bindings/JobRequest';

    type MarkdownMode = 'PdfToMarkdown' | 'MarkdownToPdf' | 'BatchFolder';

    let currentFile = $derived(appState.currentFile);
    let selectedMode = $state<MarkdownMode>('PdfToMarkdown');
    let sourcePath = $state<string>('');
    let customOutputPath = $state<string>('');
    let isSubmitting = $state(false);
    let lastJobId = $state<string | null>(null);

    let completedResult = $state<{ op: string; outputPath: string } | null>(null);
    let failedResult = $state<{ op: string; message: string } | null>(null);
    let unlistenEvents: (() => void) | undefined;

    // Batch options
    let injectFrontMatter = $state(true);
    let preserveFrontMatter = $state(true);
    let extractAssets = $state(true);

    let activeSource = $derived(sourcePath || currentFile || '');

    let computedOutputPath = $derived.by(() => {
        if (!activeSource) {
            return selectedMode === 'PdfToMarkdown'
                ? 'document.md'
                : selectedMode === 'MarkdownToPdf'
                ? 'document.pdf'
                : 'markdown_batch_output';
        }
        if (selectedMode === 'PdfToMarkdown') {
            return activeSource.replace(/\.pdf$/i, '.md');
        } else if (selectedMode === 'MarkdownToPdf') {
            return activeSource.replace(/\.md$/i, '.pdf');
        } else {
            return activeSource.replace(/\/?$/, '') + '_markdown_batch';
        }
    });

    async function handleBrowseSource() {
        try {
            if (selectedMode === 'BatchFolder') {
                const folder = await pickFolderDialog();
                if (folder) {
                    sourcePath = folder;
                    appState.addFiles([folder]);
                }
            } else if (selectedMode === 'PdfToMarkdown') {
                const files = await pickFilesDialog(false, ['pdf']);
                if (files && files.length > 0) {
                    sourcePath = files[0];
                    appState.addFiles(files);
                }
            } else {
                const files = await pickFilesDialog(false, ['md', 'markdown', 'txt']);
                if (files && files.length > 0) {
                    sourcePath = files[0];
                    appState.addFiles(files);
                }
            }
        } catch (e) {
            console.error('Failed to browse source:', e);
        }
    }

    async function handleBrowseSaveAs() {
        try {
            const defaultName = computedOutputPath.split(/[\/\\]/).pop() || (selectedMode === 'PdfToMarkdown' ? 'converted.md' : 'converted.pdf');
            const savePath = await saveFileDialog(defaultName);
            if (savePath) {
                customOutputPath = savePath;
            }
        } catch (e) {
            console.error('Failed to browse save destination:', e);
        }
    }

    async function handleBrowseOutputFolder() {
        try {
            const folder = await pickFolderDialog();
            if (folder) {
                const defaultName = computedOutputPath.split(/[\/\\]/).pop() || '';
                customOutputPath = folder + (folder.endsWith('/') ? '' : '/') + defaultName;
            }
        } catch (e) {
            console.error('Failed to browse folder destination:', e);
        }
    }

    async function handleConvert() {
        if (!activeSource || isSubmitting) return;

        isSubmitting = true;
        completedResult = null;
        failedResult = null;

        const out = customOutputPath.trim() || computedOutputPath;
        let req: JobRequest;

        if (selectedMode === 'PdfToMarkdown') {
            req = {
                op: 'PdfToMarkdown',
                input_path: activeSource,
                output_path: out
            };
        } else if (selectedMode === 'MarkdownToPdf') {
            req = {
                op: 'MarkdownToPdf',
                input_path: activeSource,
                output_path: out
            };
        } else {
            req = {
                op: 'BatchMarkdownConvert',
                input_dir: activeSource,
                output_dir: out,
                options: {
                    target: 'Markdown',
                    inject_front_matter: injectFrontMatter,
                    preserve_front_matter: preserveFrontMatter,
                    rewrite_assets: extractAssets
                }
            };
        }

        try {
            const jobId = await IpcClient.submitJob(req);
            lastJobId = jobId;
        } catch (e: any) {
            failedResult = {
                op: selectedMode === 'PdfToMarkdown' ? 'PDF to Markdown' : selectedMode === 'MarkdownToPdf' ? 'Markdown to PDF' : 'Batch Markdown',
                message: e?.message || String(e) || 'Failed to submit job'
            };
            isSubmitting = false;
        }
    }

    onMount(async () => {
        unlistenEvents = await IpcClient.onJobEvent((event) => {
            if (!lastJobId) return;
            const eventJobId = (event as any)?.data?.job_id;
            if (eventJobId !== lastJobId) return;

            if (event.kind === 'Completed') {
                const out = (event as any)?.data?.output_path || customOutputPath || computedOutputPath;
                const modeName = selectedMode === 'PdfToMarkdown' ? 'PDF to Markdown' : selectedMode === 'MarkdownToPdf' ? 'Markdown to PDF' : 'Batch Markdown';
                completedResult = {
                    op: modeName,
                    outputPath: out
                };
                failedResult = null;
                isSubmitting = false;
            } else if (event.kind === 'Failed') {
                const modeName = selectedMode === 'PdfToMarkdown' ? 'PDF to Markdown' : selectedMode === 'MarkdownToPdf' ? 'Markdown to PDF' : 'Batch Markdown';
                failedResult = {
                    op: modeName,
                    message: (event as any)?.data?.error_message || 'Markdown conversion failed.'
                };
                completedResult = null;
                isSubmitting = false;
            }
        });
    });

    onDestroy(() => {
        if (unlistenEvents) unlistenEvents();
    });

    function resetForm() {
        completedResult = null;
        failedResult = null;
        customOutputPath = '';
        lastJobId = null;
    }
</script>

<div class="markdown-view-container">
    <div class="header">
        <div class="title-wrap">
            <h2>📘 Markdown Studio</h2>
            <p class="subtitle">Convert between PDF documents and clean Markdown with asset extraction and front-matter support.</p>
        </div>
    </div>

    <div class="content-grid">
        <div class="main-panel">
            <!-- Mode Selector -->
            <div class="section-card mode-card">
                <span class="card-title">Select Operation</span>
                <div class="mode-buttons">
                    <button
                        type="button"
                        class="mode-btn"
                        class:active={selectedMode === 'PdfToMarkdown'}
                        onclick={() => { selectedMode = 'PdfToMarkdown'; resetForm(); }}
                    >
                        <span class="mode-icon">📄➔📘</span>
                        <div class="mode-info">
                            <strong>PDF to Markdown</strong>
                            <span>Extract text, tables, and images into Markdown (.md)</span>
                        </div>
                    </button>
                    <button
                        type="button"
                        class="mode-btn"
                        class:active={selectedMode === 'MarkdownToPdf'}
                        onclick={() => { selectedMode = 'MarkdownToPdf'; resetForm(); }}
                    >
                        <span class="mode-icon">📘➔📄</span>
                        <div class="mode-info">
                            <strong>Markdown to PDF</strong>
                            <span>Compile Markdown (.md) into a styled PDF document</span>
                        </div>
                    </button>
                    <button
                        type="button"
                        class="mode-btn"
                        class:active={selectedMode === 'BatchFolder'}
                        onclick={() => { selectedMode = 'BatchFolder'; resetForm(); }}
                    >
                        <span class="mode-icon">📁➔📚</span>
                        <div class="mode-info">
                            <strong>Batch Folder Conversion</strong>
                            <span>Convert an entire directory of documents into Markdown</span>
                        </div>
                    </button>
                </div>
            </div>

            <!-- Source Document/Folder -->
            <div class="section-card source-card">
                <div class="card-head-row">
                    <span class="card-title">
                        {selectedMode === 'BatchFolder' ? 'Source Directory' : 'Source Document'}
                    </span>
                    <button type="button" class="btn-picker" onclick={handleBrowseSource}>
                        📁 {selectedMode === 'BatchFolder' ? 'Select Folder...' : 'Browse File...'}
                    </button>
                </div>

                {#if activeSource}
                    <div class="file-info-box">
                        <span class="file-icon">{selectedMode === 'BatchFolder' ? '📁' : '📄'}</span>
                        <div class="file-text">
                            <span class="file-name">{activeSource.split(/[\/\\]/).pop()}</span>
                            <span class="file-path">{activeSource}</span>
                        </div>
                    </div>
                {:else}
                    <div class="empty-file-box">
                        <p>No document selected. Click browse or drop a file.</p>
                        <button type="button" class="btn-picker" onclick={handleBrowseSource} style="margin-top: 0.5rem;">
                            📁 {selectedMode === 'BatchFolder' ? 'Browse Directory...' : 'Browse File...'}
                        </button>
                    </div>
                {/if}
            </div>

            <!-- Batch Options -->
            {#if selectedMode === 'BatchFolder'}
                <div class="section-card options-card">
                    <span class="card-title">Batch Processing Options</span>
                    <div class="checkboxes-group">
                        <label class="checkbox-label">
                            <input type="checkbox" bind:checked={injectFrontMatter} />
                            <span>Inject YAML front-matter (title, source, timestamp)</span>
                        </label>
                        <label class="checkbox-label">
                            <input type="checkbox" bind:checked={preserveFrontMatter} />
                            <span>Preserve existing YAML metadata</span>
                        </label>
                        <label class="checkbox-label">
                            <input type="checkbox" bind:checked={extractAssets} />
                            <span>Extract embedded images and rewrite links to assets/</span>
                        </label>
                    </div>
                </div>
            {/if}

            <!-- Output Destination -->
            <div class="section-card output-card">
                <label class="card-title" for="md-output-path">Destination Path</label>
                <div class="output-row">
                    <input
                        id="md-output-path"
                        type="text"
                        class="text-input"
                        placeholder={computedOutputPath}
                        bind:value={customOutputPath}
                    />
                    <button type="button" class="btn-picker" onclick={handleBrowseSaveAs} title="Save As file">
                        💾 Save As...
                    </button>
                    <button type="button" class="btn-picker" onclick={handleBrowseOutputFolder} title="Select folder">
                        📁 Select Folder...
                    </button>
                </div>
                <span class="help-text">Leave blank to automatically save adjacent to the source document.</span>
            </div>

            <!-- Toast / Completion Notification Banner -->
            {#if completedResult}
                <div class="toast-banner success-toast" role="status" aria-live="polite">
                    <div class="toast-content">
                        <div class="toast-header">
                            <span class="toast-icon">✅</span>
                            <span class="toast-title">{completedResult.op} — Completed Successfully</span>
                        </div>
                        <p class="toast-path"><strong>Saved to:</strong> {completedResult.outputPath}</p>
                    </div>
                    <button type="button" class="btn-toast" onclick={resetForm}>
                        🔄 Clear / Convert Another
                    </button>
                </div>
            {:else if failedResult}
                <div class="toast-banner error-toast" role="alert" aria-live="assertive">
                    <div class="toast-content">
                        <div class="toast-header">
                            <span class="toast-icon">❌</span>
                            <span class="toast-title">{failedResult.op} — Failed</span>
                        </div>
                        <p class="toast-path">{failedResult.message}</p>
                    </div>
                    <button type="button" class="btn-toast" onclick={() => { failedResult = null; }}>
                        Dismiss
                    </button>
                </div>
            {/if}

            <div class="action-row">
                <button
                    type="button"
                    class="btn-primary convert-btn"
                    disabled={!activeSource || isSubmitting}
                    onclick={handleConvert}
                >
                    {#if isSubmitting}
                        <span class="spinner">⏳</span> Converting...
                    {:else}
                        <span>⚡ Convert Document</span>
                    {/if}
                </button>
            </div>
        </div>

        <div class="sidebar-info-panel">
            <div class="info-card">
                <h3>📌 Engine Details</h3>
                <ul class="features-list">
                    <li><strong>Offline Execution:</strong> Strictly local Pandoc / pdfcraft conversion with zero network calls.</li>
                    <li><strong>Asset Isolation:</strong> Embedded images extracted into clean `assets/` subfolders.</li>
                    <li><strong>Front-Matter:</strong> Generates standard Obsidian/Hugo compatible YAML front-matter blocks.</li>
                    <li><strong>Atomic Writes:</strong> Existing files are never overwritten unless explicitly targeted.</li>
                </ul>
            </div>
        </div>
    </div>
</div>

<style>
    .markdown-view-container {
        display: flex;
        flex-direction: column;
        height: 100%;
        background: transparent;
        color: var(--text-primary);
        overflow-y: auto;
        padding: 1.5rem;
        box-sizing: border-box;
    }

    .header {
        margin-bottom: 1.5rem;
    }

    .title-wrap h2 {
        margin: 0;
        font-size: 1.6rem;
        color: var(--text-primary);
    }

    .subtitle {
        margin: 0.35rem 0 0 0;
        color: var(--text-secondary);
        font-size: 0.9rem;
    }

    .content-grid {
        display: grid;
        grid-template-columns: minmax(0, 1fr) 300px;
        gap: 1.5rem;
        align-items: start;
    }

    @media (max-width: 1024px) {
        .content-grid {
            grid-template-columns: minmax(0, 1fr);
        }
    }

    .main-panel {
        display: flex;
        flex-direction: column;
        gap: 1.25rem;
    }

    .section-card {
        background: var(--glass-bg-surface);
        border: 1px solid var(--glass-border-subtle);
        border-radius: var(--radius-lg, 14px);
        box-shadow: inset 0 1px 0 0 var(--glass-border-specular), var(--glass-shadow-sm);
        padding: 1.25rem;
    }

    .card-title {
        display: block;
        font-weight: 600;
        font-size: 0.95rem;
        color: var(--text-primary);
        margin-bottom: 0.75rem;
    }

    .card-head-row {
        display: flex;
        align-items: center;
        justify-content: space-between;
        margin-bottom: 0.75rem;
    }

    .card-head-row .card-title {
        margin-bottom: 0;
    }

    .mode-buttons {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
        gap: 0.75rem;
    }

    .mode-btn {
        display: flex;
        align-items: center;
        gap: 0.85rem;
        padding: 1rem;
        background: var(--glass-bg-input);
        border: 1px solid var(--glass-border-subtle);
        border-radius: var(--radius-md, 10px);
        color: var(--text-primary);
        cursor: pointer;
        text-align: left;
        transition: all var(--dur-fast, 120ms) ease;
    }

    .mode-btn:hover {
        border-color: var(--accent-primary);
        background: var(--glass-bg-hover);
    }

    .mode-btn.active {
        border-color: var(--accent-primary);
        background: var(--glass-bg-active);
        box-shadow: 0 0 0 1px var(--accent-primary);
    }

    .mode-icon {
        font-size: 1.6rem;
    }

    .mode-info strong {
        display: block;
        font-size: 0.95rem;
        margin-bottom: 0.2rem;
    }

    .mode-info span {
        font-size: 0.78rem;
        color: var(--text-secondary);
    }

    .file-info-box {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        background: var(--glass-bg-input);
        border: 1px solid var(--glass-border-subtle);
        padding: 0.75rem 1rem;
        border-radius: var(--radius-md, 10px);
    }

    .file-icon {
        font-size: 1.75rem;
    }

    .file-text {
        display: flex;
        flex-direction: column;
        overflow: hidden;
    }

    .file-name {
        font-weight: 600;
        color: var(--accent-primary);
        font-size: 0.95rem;
    }

    .file-path {
        font-size: 0.75rem;
        color: var(--text-secondary);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .empty-file-box {
        background: var(--glass-bg-input);
        border: 1px dashed var(--glass-border-subtle);
        border-radius: var(--radius-md, 10px);
        padding: 1.5rem;
        text-align: center;
        color: var(--text-secondary);
        font-size: 0.9rem;
    }

    .output-row {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        flex-wrap: wrap;
    }

    .output-row .text-input {
        flex: 1;
        min-width: 220px;
        padding: 0.65rem 0.9rem;
        background: var(--glass-bg-input);
        border: 1px solid var(--glass-border-subtle);
        border-radius: var(--radius-md, 10px);
        color: var(--text-primary);
        font-size: 0.9rem;
        box-sizing: border-box;
    }

    .output-row .text-input:focus {
        outline: none;
        border-color: var(--accent-primary);
    }

    .btn-picker {
        display: inline-flex;
        align-items: center;
        gap: 0.4rem;
        padding: 0.55rem 0.95rem;
        background: var(--glass-bg-input);
        color: var(--text-primary);
        border: 1px solid var(--glass-border-subtle);
        border-radius: var(--radius-sm, 8px);
        font-size: 0.85rem;
        font-weight: 500;
        cursor: pointer;
        white-space: nowrap;
        transition: background-color var(--dur-fast, 120ms) ease;
    }

    .btn-picker:hover {
        background: var(--glass-bg-hover);
        color: var(--text-primary);
        border-color: var(--accent-primary);
    }

    .help-text {
        display: block;
        font-size: 0.8rem;
        color: var(--text-secondary);
        margin-top: 0.5rem;
    }

    .checkboxes-group {
        display: flex;
        flex-direction: column;
        gap: 0.65rem;
    }

    .checkbox-label {
        display: flex;
        align-items: center;
        gap: 0.65rem;
        font-size: 0.88rem;
        color: var(--text-primary);
        cursor: pointer;
    }

    .toast-banner {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 1rem;
        padding: 1rem 1.25rem;
        border-radius: var(--radius-md, 10px);
        animation: fadeIn 0.2s ease-in-out;
    }

    .success-toast {
        background: var(--glass-bg-surface);
        border: 1px solid var(--semantic-success);
        color: var(--semantic-success);
    }

    .error-toast {
        background: var(--glass-bg-surface);
        border: 1px solid var(--semantic-danger);
        color: var(--semantic-danger);
    }

    .toast-header {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        font-weight: 600;
        font-size: 0.95rem;
    }

    .toast-path {
        margin: 0.35rem 0 0 0;
        font-size: 0.85rem;
        color: var(--text-secondary);
        word-break: break-all;
    }

    .btn-toast {
        padding: 0.5rem 1rem;
        background: var(--accent-primary);
        color: var(--text-on-accent);
        font-weight: 600;
        font-size: 0.85rem;
        border: none;
        border-radius: var(--radius-sm, 8px);
        cursor: pointer;
        white-space: nowrap;
        transition: opacity var(--dur-fast, 120ms) ease;
    }

    .btn-toast:hover {
        opacity: 0.9;
    }

    .action-row {
        margin-top: 0.5rem;
    }

    .convert-btn {
        padding: 0.85rem 1.75rem;
        font-size: 1rem;
        font-weight: 600;
        border-radius: var(--radius-md, 10px);
        background: var(--accent-primary);
        color: var(--text-on-accent);
        border: none;
        cursor: pointer;
        transition: opacity var(--dur-fast, 120ms) ease;
    }

    .convert-btn:hover:not(:disabled) {
        opacity: 0.9;
        transform: translateY(-1px);
    }

    .convert-btn:disabled {
        opacity: 0.5;
        cursor: not-allowed;
    }

    .sidebar-info-panel .info-card {
        background: var(--glass-bg-surface);
        border: 1px solid var(--glass-border-subtle);
        border-radius: var(--radius-lg, 14px);
        padding: 1.25rem;
    }

    .info-card h3 {
        margin: 0 0 0.85rem 0;
        font-size: 1rem;
        color: var(--text-primary);
    }

    .features-list {
        margin: 0;
        padding-left: 1.2rem;
        color: var(--text-secondary);
        font-size: 0.85rem;
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
    }

    .features-list li strong {
        color: var(--text-primary);
    }
</style>

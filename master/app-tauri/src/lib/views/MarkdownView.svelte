<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import { appState } from '$lib/state.svelte';
    import { siblingOutputPath, swapExtension, resolveFreeOutput } from '$lib/views/studio/outputNames';
    import { IpcClient, pickFilesDialog, pickFolderDialog, saveFileDialog } from '$lib/ipc';
    import { friendlyJobError } from '$lib/core/jobError';
    import ErrorState from '$lib/components/ui/ErrorState.svelte';
    import Button from '$lib/components/ui/Button.svelte';
    import Input from '$lib/components/ui/Input.svelte';
    import Toggle from '$lib/components/ui/Toggle.svelte';
    import type { JobRequest } from '$lib/bindings/JobRequest';

    type MarkdownMode = 'PdfToMarkdown' | 'MarkdownToPdf' | 'BatchFolder';
    const MODES: { id: MarkdownMode; icon: string; title: string; desc: string }[] = [
        { id: 'PdfToMarkdown', icon: '📄➔📘', title: 'PDF to Markdown', desc: 'Extract text, tables, and images into Markdown (.md)' },
        { id: 'MarkdownToPdf', icon: '📘➔📄', title: 'Markdown to PDF', desc: 'Compile Markdown (.md) into a styled PDF document' },
        { id: 'BatchFolder', icon: '📁➔📚', title: 'Batch folder conversion', desc: 'Convert an entire directory of documents into Markdown' },
    ];

    let currentFile = $derived(appState.currentFile);
    let selectedMode = $state<MarkdownMode>('PdfToMarkdown');
    let sourcePath = $state<string>('');
    let customOutputPath = $state<string>('');
    let isSubmitting = $state(false);
    let pickerFailed = $state(false);
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
            return siblingOutputPath(activeSource, '.md');
        } else if (selectedMode === 'MarkdownToPdf') {
            return swapExtension(activeSource, '.pdf');
        } else {
            return activeSource.replace(/\/?$/, '') + '_markdown_batch';
        }
    });

    async function handleBrowseSource() {
        pickerFailed = false;
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
            pickerFailed = true;
        }
    }

    async function handleBrowseSaveAs() {
        pickerFailed = false;
        try {
            const defaultName = computedOutputPath.split(/[\/\\]/).pop() || (selectedMode === 'PdfToMarkdown' ? 'converted.md' : 'converted.pdf');
            const savePath = await saveFileDialog(defaultName);
            if (savePath) {
                customOutputPath = savePath;
            }
        } catch (e) {
            console.error('Failed to browse save destination:', e);
            pickerFailed = true;
        }
    }

    async function handleBrowseOutputFolder() {
        pickerFailed = false;
        try {
            const folder = await pickFolderDialog();
            if (folder) {
                const defaultName = computedOutputPath.split(/[\/\\]/).pop() || '';
                customOutputPath = folder + (folder.endsWith('/') ? '' : '/') + defaultName;
            }
        } catch (e) {
            console.error('Failed to browse folder destination:', e);
            pickerFailed = true;
        }
    }

    async function handleConvert() {
        if (!activeSource || isSubmitting) return;

        isSubmitting = true;
        completedResult = null;
        failedResult = null;

        let req: JobRequest;
        try {
        const wanted = customOutputPath.trim() || computedOutputPath;
        // Never the input, never an existing file: next free name (BUG-051); the batch folder is left as chosen.
        const out = selectedMode === 'BatchFolder' ? wanted : await resolveFreeOutput(wanted, activeSource);
        if (out !== wanted) customOutputPath = out;

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
            <h2>Markdown Studio</h2>
            <p class="subtitle">Convert between PDF documents and clean Markdown with asset extraction and front-matter support.</p>
        </div>
    </div>

    <div class="content-grid">
        <div class="main-panel">
            <!-- Mode -->
            <div class="section-card mode-card">
                <span class="card-title">Operation</span>
                <div class="mode-buttons">
                    {#each MODES as m (m.id)}
                        <Button
                            variant="secondary"
                            block
                            class="mode-btn"
                            pressed={selectedMode === m.id}
                            onclick={() => { selectedMode = m.id; resetForm(); }}
                        >
                            <span class="mode-btn-body">
                                <span class="mode-icon" aria-hidden="true">{m.icon}</span>
                                <span class="mode-info">
                                    <strong>{m.title}</strong>
                                    <span>{m.desc}</span>
                                </span>
                            </span>
                        </Button>
                    {/each}
                </div>
            </div>

            <!-- Source -->
            <div class="section-card source-card">
                <div class="card-head-row">
                    <span class="card-title">{selectedMode === 'BatchFolder' ? 'Source directory' : 'Source document'}</span>
                    <Button variant="secondary" size="sm" onclick={handleBrowseSource}>
                        {selectedMode === 'BatchFolder' ? 'Select folder…' : 'Browse file'}
                    </Button>
                </div>

                {#if pickerFailed}
                    <ErrorState what="Couldn't open the file picker." next="Try again." retry={handleBrowseSource} />
                {/if}

                {#if activeSource}
                    <div class="file-info-box">
                        <span class="file-icon" aria-hidden="true">{selectedMode === 'BatchFolder' ? '📁' : '📄'}</span>
                        <div class="file-text">
                            <span class="file-name">{activeSource.split(/[\/\\]/).pop()}</span>
                            <span class="file-path">{activeSource}</span>
                        </div>
                    </div>
                {:else}
                    <div class="empty-file-box">
                        <p>No document selected. Browse for a file or drop one here.</p>
                    </div>
                {/if}
            </div>

            <!-- Batch options -->
            {#if selectedMode === 'BatchFolder'}
                <div class="section-card options-card">
                    <span class="card-title">Batch processing options</span>
                    <Toggle label="Inject YAML front-matter (title, source, timestamp)" bind:checked={injectFrontMatter} />
                    <Toggle label="Preserve existing YAML metadata" bind:checked={preserveFrontMatter} />
                    <Toggle label="Extract embedded images and rewrite links to assets/" bind:checked={extractAssets} />
                </div>
            {/if}

            <!-- Destination -->
            <div class="section-card output-card">
                <Input label="Destination path" placeholder={computedOutputPath} bind:value={customOutputPath} data-testid="md-output-path" />
                <div class="output-row">
                    <Button variant="secondary" size="sm" title="Choose the destination file" onclick={handleBrowseSaveAs}>Save as…</Button>
                    <Button variant="secondary" size="sm" title="Choose a destination folder" onclick={handleBrowseOutputFolder}>Select folder…</Button>
                </div>
                <span class="help-text">Leave blank to save next to the source document.</span>
            </div>

            {#if completedResult}
                <div class="toast-banner success-toast" role="status" aria-live="polite">
                    <div class="toast-content">
                        <span class="toast-title">{completedResult.op}: completed</span>
                        <p class="toast-path"><strong>Saved to:</strong> {completedResult.outputPath}</p>
                    </div>
                    <Button variant="secondary" size="sm" onclick={resetForm}>Clear and convert another</Button>
                </div>
            {:else if failedResult}
                {@const f = friendlyJobError(failedResult.message, { what: `${failedResult.op} failed.`, next: 'Check the file, then try again.' })}
                <ErrorState what={f.what} next={f.next} retry={() => { failedResult = null; void handleConvert(); }} data-testid="convert-failure" />
            {/if}

            <div class="action-row">
                <Button
                    variant="primary"
                    size="lg"
                    busy={isSubmitting}
                    disabled={!activeSource}
                    disabledReason="Choose a source document first"
                    onclick={handleConvert}
                >
                    {isSubmitting ? 'Converting…' : 'Convert document'}
                </Button>
            </div>
        </div>

        <div class="sidebar-info-panel">
            <div class="info-card">
                <h3>Engine details</h3>
                <ul class="features-list">
                    <li><strong>Runs locally:</strong> Pandoc and the built-in engine convert on this computer with no network calls.</li>
                    <li><strong>Asset isolation:</strong> Embedded images are extracted into an `assets/` subfolder.</li>
                    <li><strong>Front-matter:</strong> Generates Obsidian/Hugo-compatible YAML front-matter blocks.</li>
                    <li><strong>Safe writes:</strong> Single-file conversions never overwrite an existing file; they use a new name. A folder batch writes into the folder you choose.</li>
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
        padding: var(--space-4);
        gap: var(--space-4);
        color: var(--text-primary);
    }

    .header {
        padding: var(--space-4) var(--space-6);
        background: var(--glass-bg-surface);
        border: 1px solid var(--glass-border-subtle);
        border-radius: var(--radius-lg);
    }

    h2 {
        margin: 0;
        font-size: var(--font-size-title1);
        color: var(--accent-primary);
    }

    .subtitle {
        margin: var(--space-1) 0 0;
        color: var(--text-secondary);
        font-size: var(--font-size-footnote);
    }

    .content {
        display: grid;
        grid-template-columns: minmax(0, 2fr) minmax(0, 1fr);
        gap: var(--space-4);
        flex: 1;
        min-height: 0;
        overflow-y: auto;
    }

    .controls-panel {
        display: flex;
        flex-direction: column;
        gap: var(--space-4);
        min-width: 0;
    }

    .section-card,
    .preview-panel {
        background: var(--glass-bg-surface);
        border: 1px solid var(--glass-border-subtle);
        border-radius: var(--radius-lg);
        padding: var(--space-4);
        display: flex;
        flex-direction: column;
        gap: var(--space-3);
    }

    .card-head-row {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: var(--space-3);
    }

    .card-title {
        font-weight: 600;
        font-size: var(--font-size-title3);
    }

    .file-info-box {
        display: flex;
        align-items: center;
        gap: var(--space-3);
        padding: var(--space-3);
        background: var(--glass-bg-input);
        border-radius: var(--radius-md);
    }

    .file-text {
        display: flex;
        flex-direction: column;
        min-width: 0;
    }

    .file-name {
        font-weight: 600;
    }

    .file-path {
        color: var(--text-secondary);
        font-size: var(--font-size-caption);
        word-break: break-all;
    }

    .empty-file-box p {
        margin: 0;
        color: var(--text-secondary);
        font-size: var(--font-size-footnote);
    }

    .formats-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
        gap: var(--space-3);
    }

    .formats-grid :global(.format-tile .lp-btn) {
        height: auto;
        min-height: 0;
        align-items: flex-start;
        justify-content: flex-start;
        text-align: left;
        padding: var(--space-3);
        font-weight: 400;
    }

    .tile-body {
        display: flex;
        flex-direction: column;
        gap: var(--space-1);
        align-items: flex-start;
        white-space: normal;
    }

    .tile-header {
        display: flex;
        align-items: center;
        gap: var(--space-2);
    }

    .badge {
        font-size: var(--font-size-caption);
        padding: 0 var(--space-2);
        border-radius: var(--radius-pill);
        background: var(--glass-bg-active);
    }

    .tile-title {
        font-weight: 600;
        font-size: var(--font-size-body);
    }

    .tile-desc,
    .toggle-subtitle,
    .radio-desc,
    .help-text {
        color: var(--text-secondary);
        font-size: var(--font-size-footnote);
        line-height: 1.4;
    }

    .output-row {
        display: flex;
        gap: var(--space-2);
        flex-wrap: wrap;
    }

    .toast-banner {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: var(--space-3);
        padding: var(--space-3) var(--space-4);
        border-radius: var(--radius-md);
        border: 1px solid var(--semantic-success);
        background: var(--glass-bg-input);
    }

    .toast-content {
        display: flex;
        flex-direction: column;
        gap: var(--space-1);
        min-width: 0;
    }

    .toast-title {
        font-weight: 600;
    }

    .toast-path {
        margin: 0;
        font-size: var(--font-size-footnote);
        word-break: break-all;
    }

    .status-banner {
        padding: var(--space-3) var(--space-4);
        border-radius: var(--radius-md);
        background: var(--glass-bg-input);
        border: 1px solid var(--glass-border-subtle);
        font-size: var(--font-size-footnote);
    }

    .action-row {
        display: flex;
        justify-content: flex-end;
    }

    .preview-header h3 {
        margin: 0;
        font-size: var(--font-size-title3);
    }

    .summary-details {
        display: flex;
        flex-direction: column;
        gap: var(--space-2);
    }

    .summary-item {
        display: flex;
        justify-content: space-between;
        gap: var(--space-3);
        font-size: var(--font-size-footnote);
    }

    .summary-label {
        color: var(--text-secondary);
    }

    .summary-val {
        text-align: right;
        word-break: break-all;
    }

    .summary-val.highlight {
        color: var(--accent-primary);
    }

    .offline-guarantee-box {
        display: flex;
        gap: var(--space-3);
        padding: var(--space-3);
        border-radius: var(--radius-md);
        background: var(--glass-bg-input);
        font-size: var(--font-size-footnote);
    }

    .offline-guarantee-box p {
        margin: var(--space-1) 0 0;
        color: var(--text-secondary);
    }

    .mode-buttons {
        display: flex;
        flex-direction: column;
        gap: var(--space-2);
    }

    .mode-buttons :global(.mode-btn .lp-btn) {
        height: auto;
        min-height: 0;
        justify-content: flex-start;
        text-align: left;
        padding: var(--space-3);
        font-weight: 400;
    }

    .mode-btn-body {
        display: flex;
        align-items: center;
        gap: var(--space-3);
        white-space: normal;
    }

    .mode-info {
        display: flex;
        flex-direction: column;
        gap: var(--space-1);
        font-size: var(--font-size-footnote);
        color: var(--text-secondary);
    }

    .mode-info strong {
        color: var(--text-primary);
        font-size: var(--font-size-body);
    }

    .info-card h3 {
        margin: 0 0 var(--space-2);
        font-size: var(--font-size-title3);
    }

    .features-list {
        margin: 0;
        padding-left: var(--space-5);
        display: flex;
        flex-direction: column;
        gap: var(--space-2);
        font-size: var(--font-size-footnote);
        color: var(--text-secondary);
    }

    .content-grid {
        display: grid;
        grid-template-columns: minmax(0, 2fr) minmax(0, 1fr);
        gap: var(--space-4);
        flex: 1;
        min-height: 0;
        overflow-y: auto;
    }

    .main-panel {
        display: flex;
        flex-direction: column;
        gap: var(--space-4);
        min-width: 0;
    }

    .sidebar-info-panel .info-card {
        background: var(--glass-bg-surface);
        border: 1px solid var(--glass-border-subtle);
        border-radius: var(--radius-lg);
        padding: var(--space-4);
    }
</style>

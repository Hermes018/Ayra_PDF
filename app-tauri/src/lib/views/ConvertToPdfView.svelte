<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import { appState } from '$lib/state.svelte';
    import { IpcClient, pickFilesDialog, pickFolderDialog, saveFileDialog } from '$lib/ipc';
    import type { JobRequest } from '$lib/bindings/JobRequest';
    import type { OfficeInputFormat } from '$lib/bindings/OfficeInputFormat';
    import type { MarkdownDocFormat } from '$lib/bindings/MarkdownDocFormat';
    import type { MarkdownBatchTarget } from '$lib/bindings/MarkdownBatchTarget';

    let currentPath = $derived(appState.currentFile);
    let customOutputPath = $state<string>('');
    let statusMessage = $state<string | null>(null);
    let isSubmitting = $state<boolean>(false);
    let lastJobId = $state<string | null>(null);

    let completedResult = $state<{ op: string; outputPath: string } | null>(null);
    let failedResult = $state<{ op: string; message: string } | null>(null);
    let unlistenEvents: (() => void) | undefined;

    let batchTarget = $state<MarkdownBatchTarget>('Pdf');
    let batchInjectFrontMatter = $state<boolean>(true);
    let batchPreserveFrontMatter = $state<boolean>(true);
    let batchRewriteAssets = $state<boolean>(true);

    async function handleBrowseSource() {
        try {
            if (selectedOptionId === 'BatchMarkdownConvert') {
                const folder = await pickFolderDialog();
                if (folder) appState.addFiles([folder]);
            } else {
                const exts = ['docx', 'xlsx', 'pptx', 'odt', 'ods', 'odp', 'md', 'html', 'png', 'jpg', 'jpeg'];
                const files = await pickFilesDialog(false, exts);
                if (files && files.length > 0) {
                    appState.addFiles(files);
                }
            }
        } catch (e) {
            console.error('Failed to browse source file:', e);
        }
    }

    async function handleBrowseSaveAs() {
        try {
            const defaultName = computedOutputPath.split(/[\/\\]/).pop() || 'converted.pdf';
            const savePath = await saveFileDialog(defaultName);
            if (savePath) {
                customOutputPath = savePath;
            }
        } catch (e) {
            console.error('Failed to browse save path:', e);
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
            console.error('Failed to browse output folder:', e);
        }
    }

    onMount(async () => {
        unlistenEvents = await IpcClient.onJobEvent((event) => {
            if (!lastJobId) return;
            const eventJobId = (event as any)?.data?.job_id;
            if (eventJobId !== lastJobId) return;

            if (event.kind === 'Completed') {
                const out = (event as any)?.data?.output_path || customOutputPath || computedOutputPath;
                completedResult = {
                    op: 'Convert to PDF',
                    outputPath: out
                };
                failedResult = null;
                statusMessage = null;
            } else if (event.kind === 'Failed') {
                failedResult = {
                    op: 'Convert to PDF',
                    message: (event as any)?.data?.error_message || 'Conversion failed'
                };
                completedResult = null;
                statusMessage = null;
            }
        });
    });

    onDestroy(() => {
        if (unlistenEvents) unlistenEvents();
    });

    function resetConversion() {
        completedResult = null;
        failedResult = null;
        statusMessage = null;
        customOutputPath = '';
        lastJobId = null;
    }

    interface ConvertOption {
        id: string;
        title: string;
        ext: string;
        targetExt?: string;
        icon: string;
        desc: string;
        badge: string;
        engine: string;
        kind: 'native' | 'office' | 'pandoc';
        nativeOp?: 'ImageToPdf' | 'DocxToPdf' | 'HtmlToPdf';
        officeFormat?: OfficeInputFormat;
        markdownDocFormat?: MarkdownDocFormat;
    }

    const convertOptions: ConvertOption[] = [
        {
            id: 'OfficeDocx',
            title: 'Word to PDF (High-Fidelity Layout via Sidecar)',
            ext: '.docx',
            targetExt: '.pdf',
            icon: '📝',
            desc: 'Converts Word documents (.docx/.odt) with pixel-perfect pagination and typography via headless Office sidecar.',
            badge: 'High-Fidelity',
            engine: 'Headless Office Sidecar (LibreOffice)',
            kind: 'office',
            officeFormat: 'Docx'
        },
        {
            id: 'OfficeXlsx',
            title: 'Excel to PDF (.xlsx)',
            ext: '.xlsx',
            targetExt: '.pdf',
            icon: '📊',
            desc: 'Converts spreadsheets (.xlsx/.ods) preserving multi-sheet tables, formulas, and print layouts.',
            badge: 'Spreadsheet',
            engine: 'Headless Office Sidecar (LibreOffice)',
            kind: 'office',
            officeFormat: 'Xlsx'
        },
        {
            id: 'OfficePptx',
            title: 'PowerPoint to PDF (.pptx)',
            ext: '.pptx',
            targetExt: '.pdf',
            icon: '📽️',
            desc: 'Renders slide decks (.pptx/.odp) into high-fidelity PDF slides with vector shapes and transitions.',
            badge: 'Presentation',
            engine: 'Headless Office Sidecar (LibreOffice)',
            kind: 'office',
            officeFormat: 'Pptx'
        },
        {
            id: 'MarkdownToPdf',
            title: 'Markdown to PDF (.md)',
            ext: '.md',
            targetExt: '.pdf',
            icon: '📑',
            desc: 'Compiles Markdown (.md) to PDF preserving typography, tables, and code formatting via Pandoc.',
            badge: 'Markdown',
            engine: 'Pandoc Sidecar + Native Engine',
            kind: 'pandoc'
        },
        {
            id: 'MarkdownToDocx',
            title: 'Markdown to Word (.docx)',
            ext: '.md',
            targetExt: '.docx',
            icon: '📄',
            desc: 'Translates Markdown (.md) into a styled Microsoft Word (.docx) document via Pandoc.',
            badge: 'MD -> DOCX',
            engine: 'Pandoc Sidecar',
            kind: 'pandoc',
            markdownDocFormat: 'Docx'
        },
        {
            id: 'MarkdownToHtml',
            title: 'Markdown to HTML (.html)',
            ext: '.md',
            targetExt: '.html',
            icon: '🌐',
            desc: 'Compiles Markdown (.md) into a standalone HTML5 document with inline styles via Pandoc.',
            badge: 'MD -> HTML',
            engine: 'Pandoc Sidecar',
            kind: 'pandoc',
            markdownDocFormat: 'Html'
        },
        {
            id: 'DocxToMarkdown',
            title: 'Word (.docx) to Markdown',
            ext: '.docx',
            targetExt: '.md',
            icon: '📝',
            desc: 'Converts Microsoft Word (.docx) to clean GFM Markdown, extracting embedded media to assets/ via Pandoc.',
            badge: 'DOCX -> MD',
            engine: 'Pandoc Sidecar',
            kind: 'pandoc',
            markdownDocFormat: 'Docx'
        },
        {
            id: 'HtmlToMarkdown',
            title: 'HTML to Markdown (.md)',
            ext: '.html',
            targetExt: '.md',
            icon: '📝',
            desc: 'Translates HTML web documents into clean GFM Markdown via Pandoc.',
            badge: 'HTML -> MD',
            engine: 'Pandoc Sidecar',
            kind: 'pandoc',
            markdownDocFormat: 'Html'
        },
        {
            id: 'BatchMarkdownConvert',
            title: 'Batch Markdown Folder Conversion',
            ext: 'folder',
            targetExt: '_batch_converted',
            icon: '📁',
            desc: 'Batch converts all Markdown files or documents in a folder with YAML front-matter injection and assets/ link rewriting.',
            badge: 'Batch Folder',
            engine: 'Pandoc Sidecar + Batch Pipeline',
            kind: 'pandoc'
        },
        {
            id: 'ImageToPdf',
            title: 'Image to PDF',
            ext: '.png',
            targetExt: '.pdf',
            icon: '🖼️',
            desc: 'Imports raster images (PNG, JPEG) into structured PDF pages natively.',
            badge: 'Native',
            engine: 'Pure Rust Native (`pdfcraft`)',
            kind: 'native',
            nativeOp: 'ImageToPdf'
        },
        {
            id: 'DocxToPdf',
            title: 'Word (.docx) to PDF (Structural)',
            ext: '.docx',
            targetExt: '.pdf',
            icon: '📄',
            desc: 'Fast, lightweight native conversion preserving structural flow and text without external dependencies.',
            badge: 'Native Fast',
            engine: 'Pure Rust Native (`pdfcraft`)',
            kind: 'native',
            nativeOp: 'DocxToPdf'
        },
        {
            id: 'HtmlToPdf',
            title: 'HTML to PDF',
            ext: '.html',
            targetExt: '.pdf',
            icon: '🌐',
            desc: 'Renders local HTML documents to PDF with zero network access and scripts disabled.',
            badge: 'Native Offline',
            engine: 'Pure Rust Native (`pdfcraft`)',
            kind: 'native',
            nativeOp: 'HtmlToPdf'
        }
    ];

    let selectedOptionId = $state<string>('OfficeDocx');

    let currentOption = $derived(
        convertOptions.find(opt => opt.id === selectedOptionId) || convertOptions[0]
    );

    let computedOutputPath = $derived.by(() => {
        if (!currentPath) return '';
        if (customOutputPath.trim()) return customOutputPath.trim();

        const dotIdx = currentPath.lastIndexOf('.');
        const stem = dotIdx !== -1 ? currentPath.slice(0, dotIdx) : currentPath;
        const outExt = currentOption.targetExt || '.pdf';
        return `${stem}${outExt}`;
    });

    async function handleConvert() {
        if (!currentPath) {
            statusMessage = 'Error: Please select or open an input file first.';
            return;
        }

        const out = computedOutputPath;
        if (!out) {
            statusMessage = 'Error: Invalid destination path.';
            return;
        }

        isSubmitting = true;
        statusMessage = null;

        const opt = currentOption;
        let req: JobRequest;

        if (opt.id === 'MarkdownToPdf') {
            req = {
                op: 'MarkdownToPdf',
                input_path: currentPath,
                output_path: out
            };
        } else if (opt.id === 'MarkdownToDocx' || opt.id === 'MarkdownToHtml') {
            req = {
                op: 'MarkdownToDoc',
                input_path: currentPath,
                output_path: out,
                format: opt.markdownDocFormat!
            };
        } else if (opt.id === 'DocxToMarkdown' || opt.id === 'HtmlToMarkdown') {
            req = {
                op: 'DocToMarkdown',
                input_path: currentPath,
                output_path: out,
                format: opt.markdownDocFormat!
            };
        } else if (opt.id === 'BatchMarkdownConvert') {
            req = {
                op: 'BatchMarkdownConvert',
                input_dir: currentPath,
                output_dir: out,
                options: {
                    target: batchTarget,
                    inject_front_matter: batchInjectFrontMatter,
                    preserve_front_matter: batchPreserveFrontMatter,
                    rewrite_assets: batchRewriteAssets
                }
            };
        } else if (opt.kind === 'office' && opt.officeFormat) {
            req = {
                op: 'OfficeToPdf',
                input_path: currentPath,
                output_path: out,
                format: opt.officeFormat
            };
        } else if (opt.nativeOp) {
            req = {
                op: opt.nativeOp,
                input_path: currentPath,
                output_path: out
            } as JobRequest;
        } else {
            statusMessage = 'Error: Unsupported conversion option.';
            isSubmitting = false;
            return;
        }

        try {
            const jobId = await IpcClient.submitJob(req);
            lastJobId = jobId;
            statusMessage = `Conversion job submitted successfully (ID: ${jobId.slice(0, 8)}...)`;
        } catch (e: any) {
            statusMessage = `Conversion failed: ${e?.message || e || 'Unknown error'}`;
        } finally {
            isSubmitting = false;
        }
    }
</script>

<div class="view-container">
    <div class="header">
        <div class="title-wrap">
            <h2>Convert to PDF</h2>
            <p class="subtitle">Convert Office documents, spreadsheets, presentations, and images to PDF.</p>
        </div>
    </div>

    <div class="content">
        <div class="controls-panel">
            <div class="section-card file-card">
                <div class="card-head-row">
                    <span class="card-title">Source File</span>
                    <button type="button" class="btn-picker" onclick={handleBrowseSource}>
                        📁 Browse File...
                    </button>
                </div>
                {#if currentPath}
                    <div class="file-info-box">
                        <span class="file-icon">📁</span>
                        <div class="file-text">
                            <span class="file-name">{currentPath.split(/[\/\\]/).pop()}</span>
                            <span class="file-path">{currentPath}</span>
                        </div>
                    </div>
                {:else}
                    <div class="empty-file-box">
                        <p>No file loaded. Open a file via the Organize tab or drag and drop a document.</p>
                        <button type="button" class="btn-picker" onclick={handleBrowseSource} style="margin-top: 0.5rem;">
                            📁 Browse File...
                        </button>
                    </div>
                {/if}
            </div>

            <div class="section-card format-card">
                <span class="card-title">Select Conversion Mode</span>
                <div class="formats-grid">
                    {#each convertOptions as opt}
                        <button
                            type="button"
                            class="format-tile"
                            class:active={selectedOptionId === opt.id}
                            aria-pressed={selectedOptionId === opt.id}
                            onclick={() => { selectedOptionId = opt.id; }}
                        >
                            <div class="tile-header">
                                <span class="tile-icon">{opt.icon}</span>
                                <span class="badge" class:sidecar-badge={opt.kind === 'office'}>{opt.badge}</span>
                            </div>
                            <span class="tile-title">{opt.title}</span>
                            <span class="tile-desc">{opt.desc}</span>
                        </button>
                    {/each}
                </div>
            </div>

            {#if selectedOptionId === 'BatchMarkdownConvert'}
                <div class="section-card options-card">
                    <span class="card-title">Batch Folder Settings</span>
                    <div class="batch-settings">
                        <div class="form-group">
                            <label for="batch-target-select">Batch Target Format</label>
                            <select id="batch-target-select" class="select-input" bind:value={batchTarget}>
                                <option value="Pdf">PDF (.pdf)</option>
                                <option value="Docx">Microsoft Word (.docx)</option>
                                <option value="Html">HTML (.html)</option>
                                <option value="Markdown">Markdown (.md)</option>
                            </select>
                        </div>
                        <div class="checkboxes-group">
                            <label class="checkbox-label">
                                <input type="checkbox" bind:checked={batchInjectFrontMatter} />
                                <span>Inject YAML front-matter (title, source, timestamp)</span>
                            </label>
                            <label class="checkbox-label">
                                <input type="checkbox" bind:checked={batchPreserveFrontMatter} />
                                <span>Preserve existing YAML front-matter</span>
                            </label>
                            <label class="checkbox-label">
                                <input type="checkbox" bind:checked={batchRewriteAssets} />
                                <span>Rewrite image links to relative assets/</span>
                            </label>
                        </div>
                    </div>
                </div>
            {/if}

            <div class="section-card output-card">
                <label class="card-title" for="target-output-input">Output Destination</label>
                <div class="output-row">
                    <input
                        id="target-output-input"
                        type="text"
                        class="text-input"
                        placeholder={computedOutputPath}
                        bind:value={customOutputPath}
                    />
                    <button type="button" class="btn-picker" onclick={handleBrowseSaveAs} title="Select destination file">
                        💾 Save As...
                    </button>
                    <button type="button" class="btn-picker" onclick={handleBrowseOutputFolder} title="Select output directory">
                        📁 Select Folder...
                    </button>
                </div>
                <span class="help-text">
                    Leave blank to save as a new PDF next to the original file. Output is written atomically.
                </span>
            </div>

            {#if completedResult}
                <div class="toast-banner success-toast" role="status" aria-live="polite">
                    <div class="toast-content">
                        <div class="toast-header">
                            <span class="toast-icon">✅</span>
                            <span class="toast-title">{completedResult.op} — Completed</span>
                        </div>
                        <p class="toast-path"><strong>Saved to:</strong> {completedResult.outputPath}</p>
                    </div>
                    <button type="button" class="btn-toast" onclick={resetConversion}>
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
            {:else if statusMessage}
                <div class="status-banner" class:error={statusMessage.startsWith('Error') || statusMessage.startsWith('Conversion failed')}>
                    <span>{statusMessage}</span>
                </div>
            {/if}

            <div class="action-row">
                <button
                    type="button"
                    class="btn-primary convert-btn"
                    disabled={!currentPath || isSubmitting}
                    onclick={handleConvert}
                >
                    {#if isSubmitting}
                        <span class="spinner">⏳</span> Converting...
                    {:else}
                        <span>⚡ Convert to PDF</span>
                    {/if}
                </button>
            </div>
        </div>

        <div class="preview-panel">
            <div class="preview-header">
                <h3>Conversion Summary</h3>
            </div>
            <div class="summary-details">
                <div class="summary-item">
                    <span class="summary-label">Mode:</span>
                    <span class="summary-val">{currentOption.title}</span>
                </div>
                <div class="summary-item">
                    <span class="summary-label">Engine:</span>
                    <span class="summary-val highlight">{currentOption.engine}</span>
                </div>
                <div class="summary-item">
                    <span class="summary-label">Target Format:</span>
                    <span class="summary-val">PDF (.pdf)</span>
                </div>
                <div class="summary-item">
                    <span class="summary-label">Destination:</span>
                    <span class="summary-val target-val">{computedOutputPath || 'None'}</span>
                </div>
                {#if currentOption.kind === 'office'}
                    <div class="summary-item">
                        <span class="summary-label">Sidecar Isolation:</span>
                        <span class="summary-val">Disposable profile, arguments isolated</span>
                    </div>
                {/if}
            </div>

            <div class="offline-guarantee-box">
                <span class="lock-icon">🔒</span>
                <div>
                    <strong>100% Offline & Private</strong>
                    <p>Conversions run strictly inside local process isolation with zero network access.</p>
                </div>
            </div>
        </div>
    </div>
</div>

<style>
    .view-container {
        display: flex;
        flex-direction: column;
        height: 100%;
        padding: 1.5rem;
        gap: 1.25rem;
        overflow-y: auto;
        background: transparent;
        color: var(--text-primary);
    }

    .header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        border-bottom: 1px solid var(--glass-border-subtle);
        padding-bottom: 1rem;
    }

    .title-wrap h2 {
        margin: 0;
        font-size: 1.6rem;
        color: var(--text-primary);
        font-weight: 700;
    }

    .subtitle {
        margin: 0.25rem 0 0;
        font-size: 0.9rem;
        color: var(--text-secondary);
    }

    .content {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(min(100%, 320px), 1fr));
        gap: 1.25rem;
        flex: 1;
        min-height: 0;
        min-width: 0;
        width: 100%;
    }

    .controls-panel {
        display: flex;
        flex-direction: column;
        gap: 1.25rem;
        min-width: 0;
    }

    @media (max-width: 1100px) {
        .controls-panel,
        .preview-panel {
            width: 100%;
            box-sizing: border-box;
        }
    }

    .section-card {
        background: var(--glass-bg-surface);
        -webkit-backdrop-filter: var(--glass-blur);
        backdrop-filter: var(--glass-blur);
        border: 1px solid var(--glass-border-subtle);
        border-radius: var(--radius-lg, 14px);
        padding: 1.25rem;
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
        box-shadow: inset 0 1px 0 0 var(--glass-border-specular), var(--glass-shadow-sm);
    }

    .card-title {
        font-size: 0.95rem;
        font-weight: 600;
        color: var(--text-primary);
        text-transform: uppercase;
        letter-spacing: 0.05em;
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
        padding: 1.25rem;
        text-align: center;
        color: var(--text-secondary);
        font-size: 0.9rem;
    }

    .formats-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(210px, 1fr));
        gap: 0.75rem;
    }

    .format-tile {
        background: var(--glass-bg-input);
        border: 1px solid var(--glass-border-subtle);
        border-radius: var(--radius-md, 10px);
        padding: 1rem;
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
        cursor: pointer;
        text-align: left;
        min-width: 0;
        word-break: break-word;
        transition: all var(--dur-fast, 120ms) ease;
    }

    .format-tile:hover {
        border-color: var(--accent-primary);
        background: var(--glass-bg-hover);
        transform: translateY(-2px);
    }

    .format-tile.active {
        border-color: var(--accent-primary);
        background: var(--glass-bg-active);
        box-shadow: 0 0 0 1px var(--accent-primary);
    }

    .tile-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    .tile-icon {
        font-size: 1.4rem;
    }

    .badge {
        font-size: 0.65rem;
        font-weight: 700;
        background: var(--glass-bg-hover);
        color: var(--accent-primary);
        border: 1px solid var(--glass-border-subtle);
        padding: 0.2rem 0.5rem;
        border-radius: var(--radius-xs, 6px);
        text-transform: uppercase;
    }

    .badge.sidecar-badge {
        background: var(--glass-bg-hover);
        color: var(--accent-primary);
        border: 1px solid var(--accent-primary);
    }

    .tile-title {
        font-size: 0.9rem;
        font-weight: 600;
        color: var(--text-primary);
    }

    .tile-desc {
        font-size: 0.75rem;
        color: var(--text-secondary);
        line-height: 1.35;
    }

    .text-input {
        width: 100%;
        background: var(--glass-bg-input);
        border: 1px solid var(--glass-border-subtle);
        border-radius: var(--radius-md, 10px);
        padding: 0.65rem 0.85rem;
        color: var(--text-primary);
        font-size: 0.85rem;
        font-family: var(--font-mono, monospace);
        box-sizing: border-box;
    }

    .text-input:focus {
        outline: none;
        border-color: var(--accent-primary);
    }

    .help-text {
        font-size: 0.75rem;
        color: var(--text-tertiary);
        line-height: 1.4;
    }

    .status-banner {
        background: var(--glass-bg-input);
        border: 1px solid var(--semantic-success);
        color: var(--semantic-success);
        padding: 0.75rem 1rem;
        border-radius: var(--radius-sm, 8px);
        font-size: 0.85rem;
        font-weight: 500;
    }

    .status-banner.error {
        background: var(--glass-bg-input);
        border-color: var(--semantic-danger);
        color: var(--semantic-danger);
    }

    .action-row {
        margin-top: 0.5rem;
    }

    .convert-btn {
        width: 100%;
        background: var(--accent-primary);
        color: var(--text-on-accent);
        border: none;
        border-radius: var(--radius-md, 10px);
        padding: 0.9rem;
        font-size: 1rem;
        font-weight: 700;
        cursor: pointer;
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 0.5rem;
        box-shadow: inset 0 1px 0 0 var(--glass-border-specular), var(--glass-shadow-sm);
        transition: opacity var(--dur-fast, 120ms) ease, transform var(--dur-fast, 120ms) ease;
    }

    .convert-btn:hover:not(:disabled) {
        opacity: 0.9;
        transform: translateY(-1px);
    }

    .convert-btn:disabled {
        opacity: 0.45;
        cursor: not-allowed;
    }

    .preview-panel {
        background: var(--glass-bg-surface);
        -webkit-backdrop-filter: var(--glass-blur);
        backdrop-filter: var(--glass-blur);
        border: 1px solid var(--glass-border-subtle);
        border-radius: var(--radius-lg, 14px);
        padding: 1.5rem;
        display: flex;
        flex-direction: column;
        gap: 1.25rem;
        box-shadow: inset 0 1px 0 0 var(--glass-border-specular), var(--glass-shadow-sm);
        height: fit-content;
        min-width: 0;
    }

    .preview-header h3 {
        margin: 0;
        font-size: 1.1rem;
        color: var(--text-primary);
    }

    .summary-details {
        display: flex;
        flex-direction: column;
        gap: 0.85rem;
        border-top: 1px solid var(--glass-border-subtle);
        padding-top: 1rem;
    }

    .summary-item {
        display: flex;
        justify-content: space-between;
        align-items: flex-start;
        font-size: 0.85rem;
        gap: 0.5rem;
    }

    .summary-label {
        color: var(--text-secondary);
        font-weight: 500;
    }

    .summary-val {
        color: var(--text-primary);
        font-weight: 600;
        text-align: right;
    }

    .summary-val.highlight {
        color: var(--semantic-success);
    }

    .summary-val.target-val {
        font-family: var(--font-mono, monospace);
        font-size: 0.75rem;
        word-break: break-all;
        color: var(--accent-primary);
        max-width: 65%;
    }

    .offline-guarantee-box {
        margin-top: auto;
        display: flex;
        gap: 0.75rem;
        align-items: center;
        background: var(--glass-bg-input);
        border: 1px solid var(--glass-border-subtle);
        border-radius: var(--radius-md, 10px);
        padding: 0.85rem;
        font-size: 0.8rem;
        color: var(--text-primary);
    }

    .offline-guarantee-box strong {
        color: var(--semantic-success);
        display: block;
        margin-bottom: 0.2rem;
    }

    .offline-guarantee-box p {
        margin: 0;
        color: var(--text-secondary);
        font-size: 0.75rem;
        line-height: 1.35;
    }

    .lock-icon {
        font-size: 1.5rem;
    }

    .card-head-row {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 0.5rem;
    }

    .btn-picker {
        display: inline-flex;
        align-items: center;
        gap: 0.4rem;
        padding: 0.45rem 0.85rem;
        background: var(--glass-bg-input);
        color: var(--text-primary);
        border: 1px solid var(--glass-border-subtle);
        border-radius: var(--radius-sm, 8px);
        font-size: 0.85rem;
        font-weight: 500;
        cursor: pointer;
        white-space: nowrap;
        transition: background-color var(--dur-fast, 120ms) ease, border-color var(--dur-fast, 120ms) ease;
    }

    .btn-picker:hover {
        background: var(--glass-bg-hover);
        color: var(--text-primary);
        border-color: var(--accent-primary);
    }

    .output-row {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        flex-wrap: wrap;
    }

    .output-row .text-input {
        flex: 1;
        min-width: 200px;
    }

    .toast-banner {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 1rem;
        padding: 1rem 1.25rem;
        border-radius: var(--radius-md, 10px);
        margin-top: 1rem;
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
</style>

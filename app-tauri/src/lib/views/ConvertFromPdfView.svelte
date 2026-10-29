<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import { appState } from '$lib/state.svelte';
    import { IpcClient, pickFilesDialog, pickFolderDialog, saveFileDialog } from '$lib/ipc';
    import type { JobRequest } from '$lib/bindings/JobRequest';
    import type { PdfAConformance } from '$lib/bindings/PdfAConformance';
    import type { OfficeOutputFormat } from '$lib/bindings/OfficeOutputFormat';

    let currentPath = $derived(appState.currentFile);
    type ExportMode =
        | 'PdfToDocx'
        | 'PdfToWordSidecar'
        | 'PdfToExcelSidecar'
        | 'PdfToPowerPointSidecar'
        | 'PdfToMarkdown'
        | 'BatchMarkdownConvert'
        | 'PdfToPdfA'
        | 'PdfToImages'
        | 'ExtractAllImages'
        | 'PdfToHtml'
        | 'PdfToText'
        | 'PdfToRtf';

    let selectedFormat = $state<ExportMode>('PdfToDocx');
    let selectedConformance = $state<PdfAConformance>('PdfA2b');
    let customOutputPath = $state<string>('');
    let statusMessage = $state<string | null>(null);
    let isSubmitting = $state<boolean>(false);
    let lastJobId = $state<string | null>(null);

    let completedResult = $state<{ op: string; outputPath: string } | null>(null);
    let failedResult = $state<{ op: string; message: string } | null>(null);
    let unlistenEvents: (() => void) | undefined;

    let batchInjectFrontMatter = $state<boolean>(true);
    let batchPreserveFrontMatter = $state<boolean>(true);
    let batchRewriteAssets = $state<boolean>(true);

    async function handleBrowseSource() {
        try {
            if (selectedFormat === 'BatchMarkdownConvert') {
                const folder = await pickFolderDialog();
                if (folder) appState.addFiles([folder]);
            } else {
                const files = await pickFilesDialog(false, ['pdf']);
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
            const defaultName = computedOutputPath.split(/[\/\\]/).pop() || 'converted.docx';
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
                    op: 'Convert from PDF',
                    outputPath: out
                };
                failedResult = null;
                statusMessage = null;
            } else if (event.kind === 'Failed') {
                failedResult = {
                    op: 'Convert from PDF',
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

    const formatDetails = [
        {
            id: 'PdfToDocx' as const,
            title: 'Word Document (.docx)',
            ext: '.docx',
            icon: '📄',
            desc: 'Converts reading blocks, headings, tables, and images to structural Word format natively.',
            badge: 'Structural'
        },
        {
            id: 'PdfToWordSidecar' as const,
            title: 'PDF to Word (Complex Flowable via Sidecar)',
            ext: '.docx',
            icon: '📝',
            desc: 'Converts complex multi-column layouts into flowable Microsoft Word (.docx) via headless sidecar.',
            badge: 'Sidecar Flowable'
        },
        {
            id: 'PdfToExcelSidecar' as const,
            title: 'PDF to Excel (.xlsx)',
            ext: '.xlsx',
            icon: '📊',
            desc: 'Extracts tables and tabular spreadsheets into Microsoft Excel (.xlsx) via headless sidecar.',
            badge: 'Sidecar Spreadsheet'
        },
        {
            id: 'PdfToPowerPointSidecar' as const,
            title: 'PDF to PowerPoint (.pptx)',
            ext: '.pptx',
            icon: '📽️',
            desc: 'Converts presentation slides into Microsoft PowerPoint (.pptx) via headless sidecar.',
            badge: 'Sidecar Slides'
        },
        {
            id: 'PdfToMarkdown' as const,
            title: 'PDF to Markdown (.md + assets/)',
            ext: '.md',
            icon: '📘',
            desc: 'Extracts reading layout to Markdown via Pandoc and extracts all embedded images to assets/.',
            badge: 'Sidecar Markdown'
        },
        {
            id: 'BatchMarkdownConvert' as const,
            title: 'Batch Folder to Markdown (.md)',
            ext: '_markdown_batch',
            icon: '📁',
            desc: 'Batch converts all documents in a folder to clean Markdown with assets/ extraction and YAML front-matter.',
            badge: 'Batch Folder'
        },
        {
            id: 'PdfToPdfA' as const,
            title: 'PDF to PDF/A (Archival)',
            ext: '_pdfa.pdf',
            icon: '🏛️',
            desc: 'Converts to ISO 19005 standard archival format with color intents and preflight verification.',
            badge: 'Archival'
        },
        {
            id: 'PdfToImages' as const,
            title: 'PDF to Images (PNG / JPEG)',
            ext: '_images',
            icon: '🖼️',
            desc: 'Renders each PDF page as high-resolution PNG or JPEG images into a designated output folder.',
            badge: 'Images'
        },
        {
            id: 'ExtractAllImages' as const,
            title: 'Extract Embedded Images',
            ext: '_extracted_images',
            icon: '📷',
            desc: 'Extracts all embedded raw image resources from the document stream.',
            badge: 'Raw Assets'
        },
        {
            id: 'PdfToHtml' as const,
            title: 'HTML Document (.html)',
            ext: '.html',
            icon: '🌐',
            desc: 'Exports structured flowing HTML with inline styles and embedded assets.',
            badge: 'Web'
        },
        {
            id: 'PdfToText' as const,
            title: 'Plain Text (.txt)',
            ext: '.txt',
            icon: '📝',
            desc: 'Extracts full reading-order plain text separated by form feeds.',
            badge: 'Text'
        },
        {
            id: 'PdfToRtf' as const,
            title: 'Rich Text Format (.rtf)',
            ext: '.rtf',
            icon: '📑',
            desc: 'Standard cross-platform rich text document preserving layout and formatting.',
            badge: 'RTF'
        }
    ];

    let computedOutputPath = $derived.by(() => {
        if (!currentPath) return '';
        if (customOutputPath.trim()) return customOutputPath.trim();

        const dotIdx = currentPath.lastIndexOf('.');
        const stem = dotIdx !== -1 ? currentPath.slice(0, dotIdx) : currentPath;
        const currentDetail = formatDetails.find(f => f.id === selectedFormat);
        const ext = currentDetail ? currentDetail.ext : '.out';

        return stem + ext;
    });

    async function handleExport() {
        if (!currentPath) {
            statusMessage = 'Error: Please open or select a PDF first.';
            return;
        }

        const out = computedOutputPath;
        if (!out) {
            statusMessage = 'Error: Invalid output path.';
            return;
        }

        isSubmitting = true;
        statusMessage = null;

        let req: JobRequest;
        if (selectedFormat === 'PdfToPdfA') {
            req = {
                op: 'PdfToPdfA',
                input_path: currentPath,
                output_path: out,
                conformance: selectedConformance
            };
        } else if (selectedFormat === 'PdfToWordSidecar') {
            req = {
                op: 'PdfToOffice',
                input_path: currentPath,
                output_path: out,
                format: 'Docx'
            };
        } else if (selectedFormat === 'PdfToExcelSidecar') {
            req = {
                op: 'PdfToOffice',
                input_path: currentPath,
                output_path: out,
                format: 'Xlsx'
            };
        } else if (selectedFormat === 'PdfToPowerPointSidecar') {
            req = {
                op: 'PdfToOffice',
                input_path: currentPath,
                output_path: out,
                format: 'Pptx'
            };
        } else if (selectedFormat === 'PdfToMarkdown') {
            req = {
                op: 'PdfToMarkdown',
                input_path: currentPath,
                output_path: out
            };
        } else if (selectedFormat === 'BatchMarkdownConvert') {
            req = {
                op: 'BatchMarkdownConvert',
                input_dir: currentPath,
                output_dir: out,
                options: {
                    target: 'Markdown',
                    inject_front_matter: batchInjectFrontMatter,
                    preserve_front_matter: batchPreserveFrontMatter,
                    rewrite_assets: batchRewriteAssets
                }
            };
        } else {
            req = {
                op: selectedFormat,
                input_path: currentPath,
                output_path: out
            } as JobRequest;
        }

        try {
            const jobId = await IpcClient.submitJob(req);
            lastJobId = jobId;
            statusMessage = `Export job submitted successfully (ID: ${jobId.slice(0, 8)}...)`;
        } catch (e: any) {
            statusMessage = `Export failed: ${e?.message || e || 'Unknown error'}`;
        } finally {
            isSubmitting = false;
        }
    }
</script>

<div class="view-container">
    <div class="header">
        <div class="title-wrap">
            <h2>Convert from PDF</h2>
            <p class="subtitle">Extract documents, structured formats, and images using native memory-safe engine.</p>
        </div>
    </div>

    <div class="content">
        <div class="controls-panel">
            <div class="section-card file-card">
                <div class="card-head-row">
                    <span class="card-title">Source PDF</span>
                    <button type="button" class="btn-picker" onclick={handleBrowseSource}>
                        📁 Browse File...
                    </button>
                </div>
                {#if currentPath}
                    <div class="file-info-box">
                        <span class="file-icon">📕</span>
                        <div class="file-text">
                            <span class="file-name">{currentPath.split(/[\/\\]/).pop()}</span>
                            <span class="file-path">{currentPath}</span>
                        </div>
                    </div>
                {:else}
                    <div class="empty-file-box">
                        <p>No PDF loaded. Open a file via the Organize tab or drag and drop a PDF.</p>
                        <button type="button" class="btn-picker" onclick={handleBrowseSource} style="margin-top: 0.5rem;">
                            📁 Browse File...
                        </button>
                    </div>
                {/if}
            </div>

            <div class="section-card format-card">
                <span class="card-title">Select Target Format</span>
                <div class="formats-grid">
                    {#each formatDetails as fmt}
                        <button
                            type="button"
                            class="format-tile"
                            class:active={selectedFormat === fmt.id}
                            aria-pressed={selectedFormat === fmt.id}
                            onclick={() => { selectedFormat = fmt.id; }}
                        >
                            <div class="tile-header">
                                <span class="tile-icon">{fmt.icon}</span>
                                <span class="badge">{fmt.badge}</span>
                            </div>
                            <span class="tile-title">{fmt.title}</span>
                            <span class="tile-desc">{fmt.desc}</span>
                        </button>
                    {/each}
                </div>
            </div>

            {#if selectedFormat === 'PdfToPdfA'}
                <div class="section-card conformance-card">
                    <span class="card-title">PDF/A Conformance Level</span>
                    <div class="conformance-selector">
                        <label class="radio-label" class:selected={selectedConformance === 'PdfA2b'}>
                            <input
                                type="radio"
                                name="conformance"
                                value="PdfA2b"
                                bind:group={selectedConformance}
                            />
                            <div class="radio-content">
                                <div class="radio-head">
                                    <span class="radio-title">PDF/A-2b</span>
                                    <span class="rec-badge">Recommended</span>
                                </div>
                                <span class="radio-desc">ISO 19005-2. Preserves modern PDF 1.7 visual appearance, layers, transparency, and sRGB output intent.</span>
                            </div>
                        </label>
                        <label class="radio-label" class:selected={selectedConformance === 'PdfA3b'}>
                            <input
                                type="radio"
                                name="conformance"
                                value="PdfA3b"
                                bind:group={selectedConformance}
                            />
                            <div class="radio-content">
                                <div class="radio-head">
                                    <span class="radio-title">PDF/A-3b</span>
                                </div>
                                <span class="radio-desc">ISO 19005-3. Extends 2b by permitting embedded external files and invoice XML (ZUGFeRD/Factur-X).</span>
                            </div>
                        </label>
                        <label class="radio-label" class:selected={selectedConformance === 'PdfA1b'}>
                            <input
                                type="radio"
                                name="conformance"
                                value="PdfA1b"
                                bind:group={selectedConformance}
                            />
                            <div class="radio-content">
                                <div class="radio-head">
                                    <span class="radio-title">PDF/A-1b</span>
                                </div>
                                <span class="radio-desc">ISO 19005-1. Legacy baseline format based on PDF 1.4 (forbids transparency and JPEG2000).</span>
                            </div>
                        </label>
                    </div>
                </div>
            {/if}

            {#if selectedFormat === 'BatchMarkdownConvert'}
                <div class="section-card options-card">
                    <span class="card-title">Batch Markdown Settings</span>
                    <div class="batch-settings">
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
                            <span>Extract images and rewrite links to assets/</span>
                        </label>
                    </div>
                </div>
            {/if}

            <div class="section-card output-card">
                <label class="card-title" for="output-path-input">Destination Path</label>
                <div class="output-row">
                    <input
                        id="output-path-input"
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
                    Leave blank to automatically save next to the source file. Multiple files or assets will be safely isolated in a folder.
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
                <div class="status-banner" class:error={statusMessage.startsWith('Error') || statusMessage.startsWith('Export failed')}>
                    <span>{statusMessage}</span>
                </div>
            {/if}

            <div class="action-row">
                <button
                    type="button"
                    class="btn-primary export-btn"
                    disabled={!currentPath || isSubmitting}
                    onclick={handleExport}
                >
                    {#if isSubmitting}
                        <span class="spinner">⏳</span> Exporting...
                    {:else}
                        <span>⚡ Convert Now</span>
                    {/if}
                </button>
            </div>
        </div>

        <div class="preview-panel">
            <div class="preview-header">
                <h3>Export Summary</h3>
            </div>
            <div class="summary-details">
                <div class="summary-item">
                    <span class="summary-label">Operation:</span>
                    <span class="summary-val">{selectedFormat}</span>
                </div>
                <div class="summary-item">
                    <span class="summary-label">Format:</span>
                    <span class="summary-val">{formatDetails.find(f => f.id === selectedFormat)?.title}</span>
                </div>
                {#if selectedFormat === 'PdfToPdfA'}
                    <div class="summary-item">
                        <span class="summary-label">Conformance:</span>
                        <span class="summary-val highlight">{selectedConformance}</span>
                    </div>
                {/if}
                <div class="summary-item">
                    <span class="summary-label">Engine:</span>
                    <span class="summary-val highlight">
                        {selectedFormat === 'PdfToMarkdown'
                            ? 'Headless Sidecar (Pandoc)'
                            : selectedFormat.includes('Sidecar')
                            ? 'Headless Sidecar (LibreOffice / OpenOffice)'
                            : 'Pure Rust Native (`pdfcraft`)'}
                    </span>
                </div>
                <div class="summary-item">
                    <span class="summary-label">File Safety:</span>
                    <span class="summary-val">Atomic write & isolated subfolders</span>
                </div>
                <div class="summary-item">
                    <span class="summary-label">Output Target:</span>
                    <span class="summary-val target-val">{computedOutputPath || 'None'}</span>
                </div>
            </div>

            <div class="offline-guarantee-box">
                <span class="lock-icon">🔒</span>
                <div>
                    <strong>100% Offline & Private</strong>
                    <p>All conversions run purely on your local machine with zero external network access.</p>
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
    }

    .header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        border-bottom: 1px solid var(--border-color, #313244);
        padding-bottom: 1rem;
    }

    .title-wrap h2 {
        margin: 0;
        font-size: 1.6rem;
        color: var(--text-color, #cdd6f4);
        font-weight: 700;
    }

    .subtitle {
        margin: 0.25rem 0 0;
        font-size: 0.9rem;
        color: var(--text-muted, #a6adc8);
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
        color: var(--text-color, #cdd6f4);
        text-transform: uppercase;
        letter-spacing: 0.05em;
    }

    .file-info-box {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        background: #181825;
        border: 1px solid #45475a;
        padding: 0.75rem 1rem;
        border-radius: 6px;
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
        color: #89b4fa;
        font-size: 0.95rem;
    }

    .file-path {
        font-size: 0.75rem;
        color: #a6adc8;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .empty-file-box {
        background: #181825;
        border: 1px dashed #45475a;
        border-radius: 6px;
        padding: 1.25rem;
        text-align: center;
        color: #a6adc8;
        font-size: 0.9rem;
    }

    .formats-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(210px, 1fr));
        gap: 0.75rem;
    }

    .format-tile {
        background: #181825;
        border: 1px solid #313244;
        border-radius: 8px;
        padding: 1rem;
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
        cursor: pointer;
        text-align: left;
        min-width: 0;
        word-break: break-word;
        transition: all 0.2s ease-in-out;
    }

    .format-tile:hover {
        border-color: #89b4fa;
        transform: translateY(-2px);
    }

    .format-tile.active {
        border-color: #89b4fa;
        background: rgba(137, 180, 250, 0.1);
        box-shadow: 0 0 0 1px #89b4fa;
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
        background: #313244;
        color: #cba6f7;
        padding: 0.2rem 0.5rem;
        border-radius: 4px;
        text-transform: uppercase;
    }

    .tile-title {
        font-size: 0.9rem;
        font-weight: 600;
        color: var(--text-color, #cdd6f4);
    }

    .tile-desc {
        font-size: 0.75rem;
        color: #a6adc8;
        line-height: 1.35;
    }

    .text-input {
        width: 100%;
        background: #181825;
        border: 1px solid #45475a;
        border-radius: 6px;
        padding: 0.65rem 0.85rem;
        color: #cdd6f4;
        font-size: 0.85rem;
        font-family: monospace;
        box-sizing: border-box;
    }

    .text-input:focus {
        outline: none;
        border-color: #89b4fa;
    }

    .help-text {
        font-size: 0.75rem;
        color: #6c7086;
        line-height: 1.4;
    }

    .conformance-selector {
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
    }

    .radio-label {
        display: flex;
        align-items: flex-start;
        gap: 0.75rem;
        background: #181825;
        border: 1px solid #313244;
        border-radius: 8px;
        padding: 0.85rem 1rem;
        cursor: pointer;
        transition: all 0.2s ease;
    }

    .radio-label:hover {
        border-color: #89b4fa;
    }

    .radio-label.selected {
        border-color: #89b4fa;
        background: rgba(137, 180, 250, 0.08);
        box-shadow: 0 0 0 1px #89b4fa;
    }

    .radio-label input[type='radio'] {
        margin-top: 0.25rem;
        accent-color: #89b4fa;
    }

    .radio-content {
        display: flex;
        flex-direction: column;
        gap: 0.25rem;
    }

    .radio-head {
        display: flex;
        align-items: center;
        gap: 0.5rem;
    }

    .radio-title {
        font-weight: 600;
        font-size: 0.9rem;
        color: #cdd6f4;
    }

    .radio-desc {
        font-size: 0.75rem;
        color: #a6adc8;
        line-height: 1.35;
    }

    .rec-badge {
        font-size: 0.65rem;
        font-weight: 700;
        background: rgba(166, 227, 161, 0.2);
        color: #a6e3a1;
        border: 1px solid #a6e3a1;
        padding: 0.15rem 0.45rem;
        border-radius: 4px;
        text-transform: uppercase;
    }

    .status-banner {
        background: rgba(166, 227, 161, 0.15);
        border: 1px solid #a6e3a1;
        color: #a6e3a1;
        padding: 0.75rem 1rem;
        border-radius: 6px;
        font-size: 0.85rem;
        font-weight: 500;
    }

    .status-banner.error {
        background: rgba(243, 139, 168, 0.15);
        border-color: #f38ba8;
        color: #f38ba8;
    }

    .action-row {
        margin-top: 0.5rem;
    }

    .export-btn {
        width: 100%;
        background: #89b4fa;
        color: #11111b;
        border: none;
        border-radius: 8px;
        padding: 0.9rem;
        font-size: 1rem;
        font-weight: 700;
        cursor: pointer;
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 0.5rem;
        transition: all 0.2s ease;
    }

    .export-btn:hover:not(:disabled) {
        background: #b4befe;
        transform: translateY(-1px);
    }

    .export-btn:disabled {
        opacity: 0.5;
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
        color: #cdd6f4;
    }

    .summary-details {
        display: flex;
        flex-direction: column;
        gap: 0.85rem;
        border-top: 1px solid #313244;
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
        color: #a6adc8;
        font-weight: 500;
    }

    .summary-val {
        color: #cdd6f4;
        font-weight: 600;
        text-align: right;
    }

    .summary-val.highlight {
        color: #a6e3a1;
    }

    .summary-val.target-val {
        font-family: monospace;
        font-size: 0.75rem;
        word-break: break-all;
        color: #89b4fa;
        max-width: 65%;
    }

    .offline-guarantee-box {
        margin-top: auto;
        display: flex;
        gap: 0.75rem;
        align-items: center;
        background: rgba(166, 227, 161, 0.1);
        border: 1px solid rgba(166, 227, 161, 0.3);
        border-radius: 8px;
        padding: 0.85rem;
        font-size: 0.8rem;
        color: #cdd6f4;
    }

    .offline-guarantee-box strong {
        color: #a6e3a1;
        display: block;
        margin-bottom: 0.2rem;
    }

    .offline-guarantee-box p {
        margin: 0;
        color: #a6adc8;
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
        background: #313244;
        color: #cdd6f4;
        border: 1px solid #45475a;
        border-radius: 6px;
        font-size: 0.85rem;
        font-weight: 500;
        cursor: pointer;
        white-space: nowrap;
        transition: all 0.15s ease;
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
        border-radius: 8px;
        margin-top: 1rem;
        animation: fadeIn 0.2s ease-in-out;
    }

    .success-toast {
        background: rgba(166, 227, 161, 0.12);
        border: 1px solid #a6e3a1;
        color: #a6e3a1;
    }

    .error-toast {
        background: rgba(243, 139, 168, 0.12);
        border: 1px solid #f38ba8;
        color: #f38ba8;
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
        color: #cdd6f4;
        word-break: break-all;
    }

    .btn-toast {
        padding: 0.5rem 1rem;
        background: #89b4fa;
        color: #11111b;
        font-weight: 600;
        font-size: 0.85rem;
        border: none;
        border-radius: 6px;
        cursor: pointer;
        white-space: nowrap;
        transition: background 0.15s ease;
    }

    .btn-toast:hover {
        background: #b4befe;
    }
</style>

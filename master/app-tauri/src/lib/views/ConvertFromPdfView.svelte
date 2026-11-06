<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import { appState } from '$lib/state.svelte';
    import { resolveFreeOutput } from '$lib/views/studio/outputNames';
    import { IpcClient, pickFilesDialog, pickFolderDialog, saveFileDialog } from '$lib/ipc';
    import { friendlyJobError } from '$lib/core/jobError';
    import ErrorState from '$lib/components/ui/ErrorState.svelte';
    import Button from '$lib/components/ui/Button.svelte';
    import Input from '$lib/components/ui/Input.svelte';
    import Select from '$lib/components/ui/Select.svelte';
    import Toggle from '$lib/components/ui/Toggle.svelte';
    import For from '$lib/components/For.svelte';
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
    const CONFORMANCE = [
        { value: 'PdfA2b', label: 'PDF/A-2b (recommended)', desc: 'ISO 19005-2. Preserves modern PDF 1.7 visual appearance, layers, transparency, and sRGB output intent.' },
        { value: 'PdfA3b', label: 'PDF/A-3b', desc: 'ISO 19005-3. Extends 2b by permitting embedded external files and invoice XML (ZUGFeRD/Factur-X).' },
        { value: 'PdfA1b', label: 'PDF/A-1b', desc: 'ISO 19005-1. Legacy baseline format based on PDF 1.4 (forbids transparency and JPEG2000).' },
    ];
    let customOutputPath = $state<string>('');
    // What the screen tells the user after a click (the old free-text status string, now structured): an error with
    // what happened and what to do (ErrorState, with Retry where it applies) or a non-blocking success notice.
    let notice = $state<
        | { kind: 'error'; what: string; next: string; retry?: () => void }
        | { kind: 'success'; text: string }
        | null
    >(null);
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
                notice = null;
            } else if (event.kind === 'Failed') {
                failedResult = {
                    op: 'Convert from PDF',
                    message: (event as any)?.data?.error_message || 'Conversion failed'
                };
                completedResult = null;
                notice = null;
            }
        });
    });

    onDestroy(() => {
        if (unlistenEvents) unlistenEvents();
    });

    function resetConversion() {
        completedResult = null;
        failedResult = null;
        notice = null;
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
        // A new action replaces whatever the previous one left on screen.
        completedResult = null;
        failedResult = null;
        notice = null;
        if (!currentPath) {
            notice = { kind: 'error', what: 'No PDF is selected.', next: 'Open or select a file, then convert again.' };
            return;
        }

        const wanted = computedOutputPath;
        if (!wanted) {
            notice = { kind: 'error', what: "The output path isn't valid.", next: 'Enter a different output file name, or leave it blank to save next to the source.' };
            return;
        }
        if (isSubmitting) return; // set before any await: a double click starts one job
        isSubmitting = true;
        // A name that is the input or already exists gets the next free name (BUG-051); folders are left as chosen.
        let out: string;
        try {
            out = selectedFormat === 'BatchMarkdownConvert' ? wanted : await resolveFreeOutput(wanted, currentPath);
        } catch (e) {
            console.error('No free output name', e);
            notice = { kind: 'error', what: "Couldn't find a free output file name.", next: 'Choose another location and try again.', retry: () => void handleExport() };
            isSubmitting = false;
            return;
        }
        if (out !== wanted) customOutputPath = out;

        isSubmitting = true;
        notice = null;

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
                output_path: out,
                native_text_only: appState.preferNativeText
            } as JobRequest;
        }

        try {
            const jobId = await IpcClient.submitJob(req);
            lastJobId = jobId;
            notice = { kind: 'success', text: `Job started. Output: ${out.split(/[\/]/).pop()}` };
        } catch (e: any) {
            console.error('Export failed to start:', e);
            notice = { kind: 'error', what: "Couldn't start the export.", next: 'Try again. If it keeps failing, check the file and the output location.', retry: () => void handleExport() };
        } finally {
            isSubmitting = false;
        }
    }
</script>

<div class="view-container">
    <div class="header">
        <div class="title-wrap">
            <h2>Convert from PDF</h2>
            <p class="subtitle">Extract documents, structured formats, and images with the built-in engine.</p>
        </div>
    </div>

    <div class="content">
        <div class="controls-panel">
            <div class="section-card file-card">
                <div class="card-head-row">
                    <span class="card-title">Source PDF</span>
                    <Button variant="secondary" size="sm" onclick={handleBrowseSource}>Browse file</Button>
                </div>
                {#if currentPath}
                    <div class="file-info-box">
                        <span class="file-icon" aria-hidden="true">📕</span>
                        <div class="file-text">
                            <span class="file-name">{currentPath.split(/[\/\\]/).pop()}</span>
                            <span class="file-path">{currentPath}</span>
                        </div>
                    </div>
                {:else}
                    <div class="empty-file-box">
                        <p>No PDF loaded. Open a file from the Organize tab or drag and drop a PDF.</p>
                    </div>
                {/if}
            </div>

            <div class="section-card format-card">
                <span class="card-title">Target format</span>
                <div class="formats-grid">
                    <For items={formatDetails}>
                        {#snippet children(fmt)}
                            <Button
                                variant="secondary"
                                block
                                class="format-tile"
                                pressed={selectedFormat === fmt.id}
                                onclick={() => { selectedFormat = fmt.id; }}
                            >
                                <span class="tile-body">
                                    <span class="tile-header">
                                        <span class="tile-icon" aria-hidden="true">{fmt.icon}</span>
                                        <span class="badge">{fmt.badge}</span>
                                    </span>
                                    <span class="tile-title">{fmt.title}</span>
                                    <span class="tile-desc">{fmt.desc}</span>
                                </span>
                            </Button>
                        {/snippet}
                    </For>
                </div>
            </div>

            <div class="section-card native-toggle-card">
                <Toggle
                    label="Native text only (skip OCR)"
                    bind:checked={() => appState.preferNativeText, (v) => appState.setPreferNativeText(v)}
                />
                <span class="toggle-subtitle">Speeds up conversion for digital PDFs by skipping image rasterization.</span>
            </div>

            {#if selectedFormat === 'PdfToPdfA'}
                <div class="section-card conformance-card">
                    <Select label="PDF/A conformance level" bind:value={selectedConformance} options={CONFORMANCE.map((c) => ({ value: c.value, label: c.label }))} />
                    <span class="radio-desc">{CONFORMANCE.find((c) => c.value === selectedConformance)?.desc}</span>
                </div>
            {/if}

            {#if selectedFormat === 'BatchMarkdownConvert'}
                <div class="section-card options-card">
                    <span class="card-title">Batch Markdown settings</span>
                    <Toggle label="Inject YAML front-matter (title, source, timestamp)" bind:checked={batchInjectFrontMatter} />
                    <Toggle label="Preserve existing YAML front-matter" bind:checked={batchPreserveFrontMatter} />
                    <Toggle label="Extract images and rewrite links to assets/" bind:checked={batchRewriteAssets} />
                </div>
            {/if}

            <div class="section-card output-card">
                <Input label="Destination path" placeholder={computedOutputPath} bind:value={customOutputPath} data-testid="output-path-input" />
                <div class="output-row">
                    <Button variant="secondary" size="sm" title="Select destination file" onclick={handleBrowseSaveAs}>Save as…</Button>
                    <Button variant="secondary" size="sm" title="Select output directory" onclick={handleBrowseOutputFolder}>Select folder…</Button>
                </div>
                <span class="help-text">
                    Leave blank to save next to the source file. Multiple files or assets are placed in a folder.
                </span>
            </div>

            {#if completedResult}
                <div class="toast-banner success-toast" role="status" aria-live="polite">
                    <div class="toast-content">
                        <span class="toast-title">{completedResult.op}: completed</span>
                        <p class="toast-path"><strong>Saved to:</strong> {completedResult.outputPath}</p>
                    </div>
                    <Button variant="secondary" size="sm" onclick={resetConversion}>Clear and convert another</Button>
                </div>
            {:else if failedResult}
                {@const f = friendlyJobError(failedResult.message, { what: `${failedResult.op} failed.`, next: 'Check the file and the output location, then try again.' })}
                <ErrorState what={f.what} next={f.next} retry={() => { failedResult = null; void handleExport(); }} data-testid="convert-failure" />
            {:else if notice}
                {#if notice.kind === 'error'}
                    <ErrorState what={notice.what} next={notice.next} retry={notice.retry} data-testid="convert-status-error" />
                {:else}
                    <div class="status-banner" role="status" data-testid="convert-submitted">
                        <span>{notice.text}</span>
                    </div>
                {/if}
            {/if}

            <div class="action-row">
                <Button variant="primary" size="lg" busy={isSubmitting} onclick={handleExport}>
                    {isSubmitting ? 'Exporting…' : 'Convert now'}
                </Button>
            </div>
        </div>

        <div class="preview-panel">
            <div class="preview-header">
                <h3>Export summary</h3>
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
                            ? 'Headless sidecar (Pandoc)'
                            : selectedFormat.includes('Sidecar')
                            ? 'Headless sidecar (LibreOffice / OpenOffice)'
                            : 'Built-in Rust engine (pdfcraft)'}
                    </span>
                </div>
                <div class="summary-item">
                    <span class="summary-label">File safety:</span>
                    <span class="summary-val">Atomic write, never over the original</span>
                </div>
                <div class="summary-item">
                    <span class="summary-label">Output target:</span>
                    <span class="summary-val target-val">{computedOutputPath || 'None'}</span>
                </div>
            </div>

            <div class="offline-guarantee-box">
                <span class="lock-icon" aria-hidden="true">🔒</span>
                <div>
                    <strong>Works offline</strong>
                    <p>Conversions run on this computer and make no network connections.</p>
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
</style>

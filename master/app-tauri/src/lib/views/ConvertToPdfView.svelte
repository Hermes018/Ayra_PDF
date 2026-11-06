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
    import type { OfficeInputFormat } from '$lib/bindings/OfficeInputFormat';
    import type { MarkdownDocFormat } from '$lib/bindings/MarkdownDocFormat';
    import type { MarkdownBatchTarget } from '$lib/bindings/MarkdownBatchTarget';

    let currentPath = $derived(appState.currentFile);
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

    let batchTarget = $state<MarkdownBatchTarget>('Pdf');
    const BATCH_TARGETS = [
        { value: 'Pdf', label: 'PDF (.pdf)' },
        { value: 'Docx', label: 'Microsoft Word (.docx)' },
        { value: 'Html', label: 'HTML (.html)' },
        { value: 'Markdown', label: 'Markdown (.md)' },
    ];
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
                notice = null;
            } else if (event.kind === 'Failed') {
                failedResult = {
                    op: 'Convert to PDF',
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
            desc: 'Converts Word documents (.docx/.odt) using LibreOffice running locally in the background. Page breaks and fonts can differ slightly from Word.',
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
            desc: 'Renders local HTML documents to PDF on your device, with scripts disabled and no network requests.',
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
        // A new action replaces whatever the previous one left on screen.
        completedResult = null;
        failedResult = null;
        notice = null;
        if (!currentPath) {
            notice = { kind: 'error', what: 'No file is selected.', next: 'Open or select a file, then convert again.' };
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
            out = currentOption.id === 'BatchMarkdownConvert' ? wanted : await resolveFreeOutput(wanted, currentPath);
        } catch (e) {
            console.error('No free output name', e);
            notice = { kind: 'error', what: "Couldn't find a free output file name.", next: 'Choose another location and try again.', retry: () => void handleConvert() };
            isSubmitting = false;
            return;
        }
        if (out !== wanted) customOutputPath = out;

        isSubmitting = true;
        notice = null;

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
            notice = { kind: 'error', what: 'That conversion is not supported.', next: 'Choose another conversion option.' };
            isSubmitting = false;
            return;
        }

        try {
            const jobId = await IpcClient.submitJob(req);
            lastJobId = jobId;
            notice = { kind: 'success', text: `Job started. Output: ${out.split(/[\/]/).pop()}` };
        } catch (e: any) {
            console.error('Conversion failed to start:', e);
            notice = { kind: 'error', what: "Couldn't start the conversion.", next: 'Try again. If it keeps failing, check the file and the output location.', retry: () => void handleConvert() };
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
                    <span class="card-title">Source file</span>
                    <Button variant="secondary" size="sm" onclick={handleBrowseSource}>Browse file</Button>
                </div>
                {#if currentPath}
                    <div class="file-info-box">
                        <span class="file-icon" aria-hidden="true">📁</span>
                        <div class="file-text">
                            <span class="file-name">{currentPath.split(/[\/\\]/).pop()}</span>
                            <span class="file-path">{currentPath}</span>
                        </div>
                    </div>
                {:else}
                    <div class="empty-file-box">
                        <p>No file loaded. Open a file from the Organize tab or drag and drop a document.</p>
                    </div>
                {/if}
            </div>

            <div class="section-card format-card">
                <span class="card-title">Conversion mode</span>
                <div class="formats-grid">
                    <For items={convertOptions}>
                        {#snippet children(opt)}
                            <Button
                                variant="secondary"
                                block
                                class="format-tile"
                                pressed={selectedOptionId === opt.id}
                                onclick={() => { selectedOptionId = opt.id; }}
                            >
                                <span class="tile-body">
                                    <span class="tile-header">
                                        <span class="tile-icon" aria-hidden="true">{opt.icon}</span>
                                        <span class="badge">{opt.badge}</span>
                                    </span>
                                    <span class="tile-title">{opt.title}</span>
                                    <span class="tile-desc">{opt.desc}</span>
                                </span>
                            </Button>
                        {/snippet}
                    </For>
                </div>
            </div>

            {#if selectedOptionId === 'BatchMarkdownConvert'}
                <div class="section-card options-card">
                    <span class="card-title">Batch folder settings</span>
                    <Select label="Batch target format" bind:value={batchTarget} options={BATCH_TARGETS} />
                    <Toggle label="Inject YAML front-matter (title, source, timestamp)" bind:checked={batchInjectFrontMatter} />
                    <Toggle label="Preserve existing YAML front-matter" bind:checked={batchPreserveFrontMatter} />
                    <Toggle label="Rewrite image links to relative assets/" bind:checked={batchRewriteAssets} />
                </div>
            {/if}

            <div class="section-card output-card">
                <Input label="Output destination" placeholder={computedOutputPath} bind:value={customOutputPath} data-testid="target-output-input" />
                <div class="output-row">
                    <Button variant="secondary" size="sm" title="Select destination file" onclick={handleBrowseSaveAs}>Save as…</Button>
                    <Button variant="secondary" size="sm" title="Select output directory" onclick={handleBrowseOutputFolder}>Select folder…</Button>
                </div>
                <span class="help-text">
                    Leave blank to save as a new PDF next to the original file. Output is written atomically.
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
                <ErrorState what={f.what} next={f.next} retry={() => { failedResult = null; void handleConvert(); }} data-testid="convert-failure" />
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
                <Button variant="primary" size="lg" busy={isSubmitting} onclick={handleConvert}>
                    {isSubmitting ? 'Converting…' : 'Convert to PDF'}
                </Button>
            </div>
        </div>

        <div class="preview-panel">
            <div class="preview-header">
                <h3>Conversion summary</h3>
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
                    <span class="summary-label">Target format:</span>
                    <span class="summary-val">PDF (.pdf)</span>
                </div>
                <div class="summary-item">
                    <span class="summary-label">Destination:</span>
                    <span class="summary-val target-val">{computedOutputPath || 'None'}</span>
                </div>
                {#if currentOption.kind === 'office'}
                    <div class="summary-item">
                        <span class="summary-label">Sidecar isolation:</span>
                        <span class="summary-val">Disposable profile, arguments isolated</span>
                    </div>
                {/if}
            </div>

            <div class="offline-guarantee-box">
                <span class="lock-icon" aria-hidden="true">🔒</span>
                <div>
                    <strong>Works offline</strong>
                    <p>Conversions run in an isolated local process and make no network connections.</p>
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

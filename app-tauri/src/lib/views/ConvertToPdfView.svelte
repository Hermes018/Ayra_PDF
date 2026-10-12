<script lang="ts">
    import { appState } from '$lib/state.svelte';
    import { IpcClient } from '$lib/ipc';
    import type { JobRequest } from '$lib/bindings/JobRequest';
    import type { OfficeInputFormat } from '$lib/bindings/OfficeInputFormat';
    import type { MarkdownDocFormat } from '$lib/bindings/MarkdownDocFormat';

    let currentPath = $derived(appState.currentFile);
    let customOutputPath = $state<string>('');
    let statusMessage = $state<string | null>(null);
    let isSubmitting = $state<boolean>(false);
    let lastJobId = $state<string | null>(null);

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
                <span class="card-title">Source File</span>
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
                </div>
                <span class="help-text">
                    Leave blank to save as a new PDF next to the original file. Output is written atomically.
                </span>
            </div>

            {#if statusMessage}
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
        display: flex;
        gap: 1.5rem;
        flex: 1;
        min-height: 0;
    }

    .controls-panel {
        flex: 1.6;
        display: flex;
        flex-direction: column;
        gap: 1.25rem;
    }

    .section-card {
        background: var(--surface-bg, #1e1e2e);
        border: 1px solid var(--border-color, #313244);
        border-radius: 10px;
        padding: 1.25rem;
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
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

    .badge.sidecar-badge {
        background: rgba(203, 166, 247, 0.2);
        color: #cba6f7;
        border: 1px solid rgba(203, 166, 247, 0.4);
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

    .convert-btn {
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

    .convert-btn:hover:not(:disabled) {
        background: #b4befe;
        transform: translateY(-1px);
    }

    .convert-btn:disabled {
        opacity: 0.5;
        cursor: not-allowed;
    }

    .preview-panel {
        flex: 1;
        background: var(--surface-bg, #1e1e2e);
        border: 1px solid var(--border-color, #313244);
        border-radius: 10px;
        padding: 1.5rem;
        display: flex;
        flex-direction: column;
        gap: 1.25rem;
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
        height: fit-content;
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
</style>

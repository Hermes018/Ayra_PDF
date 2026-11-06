<script lang="ts">
    import GridEditor from '$lib/components/GridEditor.svelte';
    import For from '$lib/components/For.svelte';
    import { appState } from '$lib/state.svelte';
    import { IpcClient, pickFilesDialog } from '$lib/ipc';
    import type { JobRequest } from '$lib/bindings/JobRequest';
    import Button from '$lib/components/ui/Button.svelte';
    import Input from '$lib/components/ui/Input.svelte';
    import Select from '$lib/components/ui/Select.svelte';
    import ErrorState from '$lib/components/ui/ErrorState.svelte';
    import Progress from '$lib/components/ui/Progress.svelte';

    // The viewer reactive path comes directly from the global active files
    let currentPath = $derived(appState.currentFile);
    let activeFiles = $derived(appState.activeFiles);

    let extractPagesInput = $state("");
    let deletePagesInput = $state("");
    let rotatePagesInput = $state("");
    // Select values are strings; converted to degrees when the job is built.
    let rotateDegrees = $state('90');
    let rotateDocDegrees = $state('90');
    const DEGREE_OPTIONS = [
        { value: '90', label: '90° clockwise' },
        { value: '180', label: '180°' },
        { value: '-90', label: '90° counter-clockwise' },
    ];

    // Field errors replace the old alert() dialogs; a running job shows Progress with Cancel; a failed
    // job or file picker shows an ErrorState with Retry.
    let extractError = $state<string | undefined>(undefined);
    let deleteError = $state<string | undefined>(undefined);
    let rotateError = $state<string | undefined>(undefined);
    let busyLabel = $state<string | null>(null);
    let opAbort: AbortController | null = null;
    let opError = $state<{ what: string; next: string; retry: () => void } | null>(null);
    let browseError = $state(false);

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

    const PAGES_HINT = 'Enter page numbers like 1, 3, 5-7.';

    /** Runs one job and waits for it (existing IPC job signals); Cancel aborts it through cancel_job. */
    async function runOp(label: string, what: string, req: JobRequest): Promise<void> {
        if (busyLabel) return;
        opError = null;
        busyLabel = label;
        opAbort = new AbortController();
        const signal = opAbort.signal;
        try {
            await IpcClient.runJob(req, { signal });
        } catch (e) {
            if (!signal.aborted) {
                console.error(`${what} failed:`, e);
                opError = {
                    what,
                    next: 'Check the page numbers and that the file is still there, then try again. The original file is unchanged.',
                    retry: () => void runOp(label, what, req),
                };
            }
        } finally {
            busyLabel = null;
            opAbort = null;
        }
    }

    async function handleExtract() {
        if (!currentPath) return;
        const pages = parsePages(extractPagesInput);
        extractError = pages.length === 0 ? PAGES_HINT : undefined;
        if (pages.length === 0) return;
        await runOp('Extracting pages', "Couldn't extract the pages.", {
            op: 'ExtractPages',
            input_path: currentPath,
            output_path: currentPath.replace(/\.pdf$/i, '_extracted.pdf'),
            pages,
        });
    }

    async function handleDelete() {
        if (!currentPath) return;
        const pages = parsePages(deletePagesInput);
        deleteError = pages.length === 0 ? PAGES_HINT : undefined;
        if (pages.length === 0) return;
        await runOp('Deleting pages', "Couldn't delete the pages.", {
            op: 'DeletePages',
            input_path: currentPath,
            output_path: currentPath.replace(/\.pdf$/i, '_deleted.pdf'),
            pages,
        });
    }

    async function handleRotate() {
        if (!currentPath) return;
        const pages = parsePages(rotatePagesInput);
        rotateError = pages.length === 0 ? PAGES_HINT : undefined;
        if (pages.length === 0) return;
        await runOp('Rotating pages', "Couldn't rotate the pages.", {
            op: 'RotatePages',
            input_path: currentPath,
            output_path: currentPath.replace(/\.pdf$/i, '_rotated.pdf'),
            pages,
            degrees: Number(rotateDegrees),
        });
    }

    async function handleRotateDoc() {
        if (!currentPath) return;
        await runOp('Rotating the document', "Couldn't rotate the document.", {
            op: 'RotateDocument',
            input_path: currentPath,
            output_path: currentPath.replace(/\.pdf$/i, `_rotdoc_${rotateDocDegrees}.pdf`),
            degrees: Number(rotateDocDegrees),
        });
    }

    async function handleMerge() {
        if (activeFiles.length < 2) return;
        await runOp('Merging files', "Couldn't merge the files.", {
            op: 'Merge',
            input_paths: activeFiles,
            output_path: activeFiles[0].replace(/\.pdf$/i, '_merged.pdf'),
        });
    }

    async function handleBrowseFiles() {
        browseError = false;
        try {
            const files = await pickFilesDialog(true, ['pdf']);
            if (files && files.length > 0) {
                appState.addFiles(files);
            }
        } catch (e) {
            console.error("Failed to open file picker:", e);
            browseError = true;
        }
    }
</script>

<div class="view-container">
    <div class="header">
        <h2>Organize PDFs</h2>
        <div class="controls">
            <Button variant="primary" onclick={handleBrowseFiles}>Browse files</Button>
            <span class="active-file-path">
                {currentPath || 'No file selected. Drag and drop a PDF.'}
            </span>
        </div>
    </div>

    {#if browseError}
        <ErrorState
            what="Couldn't open the file picker."
            next="Try again, or drag a PDF into the window."
            retry={handleBrowseFiles}
        />
    {/if}
    {#if opError}
        <ErrorState what={opError.what} next={opError.next} retry={opError.retry} data-testid="organize-op-error" />
    {/if}
    {#if busyLabel}
        <Progress value={null} label={busyLabel} data-testid="organize-op-progress" />
        <div class="progress-cancel">
            <Button variant="secondary" size="sm" onclick={() => opAbort?.abort()} data-testid="organize-op-cancel">Cancel</Button>
        </div>
    {/if}

    <div class="viewer-layout">
        <div class="tools-sidebar">
            <h3>Tools</h3>
            <p>Select pages to extract, delete, or reorder.</p>

            <div class="tool-section">
                <h4>Extract pages</h4>
                {#if currentPath}
                    <Input label="Pages to extract" bind:value={extractPagesInput} placeholder="e.g. 1, 3, 5-10" error={extractError} />
                    <Button
                        variant="primary"
                        onclick={handleExtract}
                        disabled={!extractPagesInput.trim() || !!busyLabel}
                        disabledReason={busyLabel ? 'Wait for the current job to finish, or cancel it' : 'Enter the pages to extract'}
                    >
                        Extract
                    </Button>
                {:else}
                    <p class="empty-text">Select a file to extract pages.</p>
                {/if}
            </div>

            <div class="tool-section">
                <h4>Delete pages</h4>
                {#if currentPath}
                    <Input label="Pages to delete" bind:value={deletePagesInput} placeholder="e.g. 1, 3, 5-10" error={deleteError} />
                    <Button
                        variant="destructive"
                        onclick={handleDelete}
                        disabled={!deletePagesInput.trim() || !!busyLabel}
                        disabledReason={busyLabel ? 'Wait for the current job to finish, or cancel it' : 'Enter the pages to delete'}
                    >
                        Delete pages
                    </Button>
                {:else}
                    <p class="empty-text">Select a file to delete pages.</p>
                {/if}
            </div>

            <div class="tool-section">
                <h4>Rotate pages</h4>
                {#if currentPath}
                    <Input label="Pages to rotate" bind:value={rotatePagesInput} placeholder="e.g. 1, 3, 5-10" error={rotateError} data-testid="organize-rotate-pages-input" />
                    <Select label="Direction" bind:value={rotateDegrees} options={DEGREE_OPTIONS} />
                    <Button
                        variant="primary"
                        data-testid="organize-rotate-pages-btn"
                        onclick={handleRotate}
                        disabled={!rotatePagesInput.trim() || !!busyLabel}
                        disabledReason={busyLabel ? 'Wait for the current job to finish, or cancel it' : 'Enter the pages to rotate'}
                    >
                        Rotate pages
                    </Button>
                {:else}
                    <p class="empty-text">Select a file to rotate pages.</p>
                {/if}
            </div>

            <div class="tool-section">
                <h4>Rotate document</h4>
                {#if currentPath}
                    <Select label="Direction" bind:value={rotateDocDegrees} options={DEGREE_OPTIONS} />
                    <Button
                        variant="primary"
                        onclick={handleRotateDoc}
                        disabled={!!busyLabel}
                        disabledReason="Wait for the current job to finish, or cancel it"
                    >
                        Rotate all pages
                    </Button>
                {:else}
                    <p class="empty-text">Select a file to rotate.</p>
                {/if}
            </div>

            <div class="tool-section">
                <h4>Files to merge</h4>
                {#if activeFiles.length > 0}
                    <ul class="file-list">
                        <For items={activeFiles}>
                            {#snippet children(file)}
                                <li>{file.split(/[\/\\]/).pop()}</li>
                            {/snippet}
                        </For>
                    </ul>
                    <Button
                        variant="primary"
                        data-testid="organize-merge-btn"
                        onclick={handleMerge}
                        disabled={activeFiles.length < 2 || !!busyLabel}
                        disabledReason={busyLabel ? 'Wait for the current job to finish, or cancel it' : 'Open at least two PDFs to enable Merge all'}
                    >
                        Merge all
                    </Button>
                {:else}
                    <p class="empty-text">Drag files here to merge.</p>
                {/if}
            </div>
        </div>

        <!-- Main PDF Viewer Area -->
        <div class="viewer-main">
            <GridEditor filePath={currentPath} />
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
    }

    .header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        background-color: var(--glass-bg-surface);
        -webkit-backdrop-filter: var(--glass-blur);
        backdrop-filter: var(--glass-blur);
        padding: var(--space-4) var(--space-6);
        border-radius: var(--radius-lg, 14px);
        border: 1px solid var(--glass-border-subtle);
        box-shadow: inset 0 1px 0 0 var(--glass-border-specular), var(--glass-shadow-sm);
    }

    h2 {
        margin: 0;
        font-size: var(--font-size-title1);
        color: var(--accent-primary);
    }

    .controls {
        display: flex;
        align-items: center;
        gap: var(--space-3);
        flex-wrap: wrap;
    }

    .active-file-path {
        background-color: var(--glass-bg-input);
        padding: var(--space-2) var(--space-4);
        border-radius: var(--radius-sm, 8px);
        color: var(--text-secondary);
        font-family: var(--font-mono, monospace);
        font-size: var(--font-size-body);
        border: 1px solid var(--glass-border-subtle);
        word-break: break-all;
    }

    .viewer-layout {
        display: grid;
        grid-template-columns: minmax(min(100%, 260px), 280px) 1fr;
        gap: var(--space-5);
        flex: 1;
        min-height: 0;
        min-width: 0;
        width: 100%;
    }

    .tools-sidebar {
        min-width: 0;
        word-break: break-word;
        box-sizing: border-box;
        background-color: var(--glass-bg-surface);
        -webkit-backdrop-filter: var(--glass-blur);
        backdrop-filter: var(--glass-blur);
        border: 1px solid var(--glass-border-subtle);
        border-radius: var(--radius-lg, 14px);
        padding: var(--space-5);
        box-shadow: inset 0 1px 0 0 var(--glass-border-specular), var(--glass-shadow-sm);
        height: fit-content;
    }

    .tools-sidebar h3 {
        margin-top: 0;
        color: var(--text-primary);
        font-size: var(--font-size-title2);
    }

    .tools-sidebar p {
        color: var(--text-secondary);
        font-size: var(--font-size-body);
    }

    .viewer-main {
        flex: 1;
        min-width: 0;
        border: 1px solid var(--glass-border-subtle);
        border-radius: var(--radius-lg, 14px);
        overflow: hidden;
        background: var(--glass-bg-surface);
    }

    @media (max-width: 900px) {
        .viewer-layout {
            grid-template-columns: 1fr;
        }
    }

    .tool-section {
        margin-top: var(--space-6);
        padding-top: var(--space-4);
        border-top: 1px solid var(--glass-border-subtle);
    }

    .tool-section h4 {
        margin-top: 0;
        margin-bottom: var(--space-2);
        color: var(--accent-primary);
        font-size: var(--font-size-title3);
    }

    .file-list {
        list-style: none;
        padding: 0;
        margin: 0 0 var(--space-4) 0;
        max-height: 150px;
        overflow-y: auto;
    }

    .file-list li {
        padding: var(--space-1) var(--space-2);
        background-color: var(--glass-bg-input);
        margin-bottom: var(--space-1);
        border-radius: var(--radius-xs, 6px);
        font-size: var(--font-size-body);
        color: var(--text-secondary);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .progress-cancel {
        display: flex;
        justify-content: flex-end;
    }

    .empty-text {
        font-size: var(--font-size-body);
        color: var(--text-tertiary);
        font-style: italic;
    }
</style>

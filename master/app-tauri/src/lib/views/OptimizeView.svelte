<script lang="ts">
    import { appState } from '$lib/state.svelte';
    import PdfViewer from '$lib/components/PdfViewer.svelte';
    import ViewShell from '$lib/components/ViewShell.svelte';
    import Button from '$lib/components/ui/Button.svelte';
    import Select from '$lib/components/ui/Select.svelte';
    import EmptyState from '$lib/components/ui/EmptyState.svelte';
    import { freeOutputPath } from '$lib/views/studio/outputNames';
    import { JobRunner } from '$lib/core/jobRunner.svelte';
    import type { OptimizeProfile } from '$lib/bindings/OptimizeProfile';

    let currentPath = $derived(appState.currentFile);
    let selectedProfile = $state<string>('Medium');
    const PROFILES = [
        { value: 'Low', label: 'Low (high quality, large size)' },
        { value: 'Medium', label: 'Medium (balanced)' },
        { value: 'High', label: 'High (low quality, small size)' },
    ];
    const NEXT = 'Check that the file is still there and not open elsewhere, then try again. The original file is unchanged.';
    const jobs = new JobRunner();
    const busyReason = 'Wait for the current job to finish, or cancel it';

    // Output names come from the collision-safe naming: never the input, never an existing file (BUG-050).
    async function compress() {
        if (!currentPath) return;
        const out = await freeOutputPath(currentPath, `compressed-${selectedProfile.toLowerCase()}`);
        void jobs.run('Compressing the PDF', "Couldn't compress the PDF.", NEXT, {
            op: 'Compress',
            input_path: currentPath,
            output_path: out,
            profile: selectedProfile as OptimizeProfile,
        });
    }
    async function repair() {
        if (!currentPath) return;
        const out = await freeOutputPath(currentPath, 'repaired');
        void jobs.run('Repairing the PDF', "Couldn't repair the PDF.", NEXT, {
            op: 'Repair',
            input_path: currentPath,
            output_path: out,
        });
    }
    async function ocr() {
        if (!currentPath) return;
        const out = await freeOutputPath(currentPath, 'ocr');
        void jobs.run('Recognizing text', "Couldn't recognize the text.", NEXT, {
            op: 'Ocr',
            input_path: currentPath,
            output_path: out,
            lang: 'en', // Only Latin supported right now natively
        });
    }
</script>

<ViewShell title="Optimize PDF" busyLabel={jobs.busyLabel} oncancel={() => jobs.cancel()} error={jobs.error}>
    {#snippet tools()}
        <div class="tool-section">
            <h4>Compress document</h4>
            {#if currentPath}
                <p class="file-label">Active: {currentPath.split(/[\/\\]/).pop()}</p>
                <Select label="Compression profile" bind:value={selectedProfile} options={PROFILES} />
                <Button variant="primary" onclick={compress} disabled={!!jobs.busyLabel} disabledReason={busyReason}>Compress PDF</Button>
                <p class="help-text">The compressed file is saved next to the original.</p>
            {:else}
                <p class="empty-text">Select or drag a file to compress.</p>
            {/if}
        </div>

        <div class="tool-section">
            <h4>Repair document</h4>
            {#if currentPath}
                <p class="help-text">Tries to fix broken cross-reference tables, recover orphaned objects and rewrite the PDF cleanly.</p>
                <Button variant="secondary" onclick={repair} disabled={!!jobs.busyLabel} disabledReason={busyReason}>Repair PDF</Button>
            {:else}
                <p class="empty-text">Select or drag a file to repair.</p>
            {/if}
        </div>

        <div class="tool-section">
            <h4>OCR (make text searchable)</h4>
            {#if currentPath}
                <p class="help-text">Recognizes Latin-alphabet text on this computer and adds an invisible searchable layer.</p>
                <Button variant="secondary" onclick={ocr} disabled={!!jobs.busyLabel} disabledReason={busyReason}>Recognize text (OCR)</Button>
            {:else}
                <p class="empty-text">Select or drag a file to OCR.</p>
            {/if}
        </div>
    {/snippet}
    {#snippet main()}
        {#if currentPath}
            <PdfViewer filePath={currentPath} />
        {:else}
            <EmptyState title="No file selected" body="Open or drag in a PDF to compress, repair or run OCR on it." />
        {/if}
    {/snippet}
</ViewShell>

<style>
    .tool-section {
        background-color: var(--glass-bg-input);
        padding: var(--space-4);
        border-radius: var(--radius-md);
        border: 1px solid var(--glass-border-subtle);
        display: flex;
        flex-direction: column;
        gap: var(--space-3);
    }

    .tool-section h4 {
        margin: 0;
        color: var(--text-primary);
        font-size: var(--font-size-title3);
    }

    .file-label {
        margin: 0;
        color: var(--text-secondary);
        font-size: var(--font-size-footnote);
        word-break: break-all;
    }

    .help-text {
        margin: 0;
        color: var(--text-secondary);
        font-size: var(--font-size-footnote);
        line-height: 1.4;
    }

    .empty-text {
        margin: 0;
        color: var(--text-tertiary);
        font-size: var(--font-size-footnote);
        font-style: italic;
    }
</style>

<script lang="ts">
    import { appState } from '$lib/state.svelte';
    import PdfViewer from '$lib/components/PdfViewer.svelte';
    import ViewShell from '$lib/components/ViewShell.svelte';
    import Button from '$lib/components/ui/Button.svelte';
    import Input from '$lib/components/ui/Input.svelte';
    import EmptyState from '$lib/components/ui/EmptyState.svelte';
    import { SAMPLE_HIGHLIGHT_ID, SAMPLE_HIGHLIGHT_LABS, LABS_BADGE, labsVisible } from '$lib/components/studio/labs.svelte';
    import { parseNumberOr } from '$lib/core/parseNumber';
    import { freeOutputPath } from '$lib/views/studio/outputNames';
    import { JobRunner } from '$lib/core/jobRunner.svelte';
    import type { WatermarkSettings } from '$lib/bindings/WatermarkSettings';
    import type { AnnotationDef } from '$lib/bindings/AnnotationDef';
    import type { AnnotationShape } from '$lib/bindings/AnnotationShape';

    let currentPath = $derived(appState.currentFile);

    // Watermark state (ui/Input works with strings; converted when the job is built)
    let watermarkText = $state('CONFIDENTIAL');
    let watermarkOpacity = $state('0.5');
    let watermarkRotation = $state('45');

    // Annotation state
    let stickyNoteText = $state('Please review this section.');
    let annotX = $state('100');
    let annotY = $state('100');
    const authorName = 'Admin';

    const NEXT = 'Check that the file is still there and not open elsewhere, then try again. The original file is unchanged.';
    const jobs = new JobRunner();
    const busyReason = 'Wait for the current job to finish, or cancel it';

    // Requests are built per attempt (a builder), so a Retry after a refused name gets the next free name.
    function watermark() {
        const input = currentPath;
        if (!input || !watermarkText) return;
        const settings: WatermarkSettings = {
            text: watermarkText,
            opacity: parseNumberOr(watermarkOpacity, 0.5),
            rotation: parseNumberOr(watermarkRotation, 45),
            color: [0.8, 0.2, 0.2], // Red-ish watermark
        };
        void jobs.run('Adding the watermark', "Couldn't add the watermark.", NEXT, async () => ({
            op: 'WatermarkDocument',
            input_path: input,
            output_path: await freeOutputPath(input, 'watermarked'),
            settings,
        }));
    }

    function annotate(label: string, what: string, tag: string, shape: AnnotationShape) {
        const input = currentPath;
        if (!input) return;
        const annotation: AnnotationDef = { page: 0, shape, author: authorName };
        void jobs.run(label, what, NEXT, async () => ({
            op: 'AnnotateDocument',
            input_path: input,
            output_path: await freeOutputPath(input, tag),
            annotation,
        }));
    }

    function stickyNote() {
        if (!stickyNoteText) return;
        annotate('Adding the sticky note', "Couldn't add the sticky note.", 'annotated', {
            StickyNote: { x: parseNumberOr(annotX, 100), y: parseNumberOr(annotY, 100), text: stickyNoteText },
        });
    }

    function highlight() {
        // Labs demo: a fixed example box on page 1 (the real highlight tool is in Drawboard Studio).
        annotate('Adding the highlight', "Couldn't add the highlight.", 'highlighted', {
            Highlight: { quads: [[100.0, 200.0, 300.0, 200.0, 100.0, 180.0, 300.0, 180.0]] },
        });
    }

    function flatten() {
        const input = currentPath;
        if (!input) return;
        void jobs.run('Flattening comments', "Couldn't flatten the comments.", NEXT, async () => ({
            op: 'FlattenComments',
            input_path: input,
            output_path: await freeOutputPath(input, 'flattened'),
        }));
    }
</script>

<ViewShell title="Edit and annotate PDF" showFileBar={false} busyLabel={jobs.busyLabel} oncancel={() => jobs.cancel()} error={jobs.error}>
    {#snippet tools()}
        <div class="tool-section">
            <h4>Watermark</h4>
            {#if currentPath}
                <Input label="Text" bind:value={watermarkText} placeholder="CONFIDENTIAL" />
                <div class="row-group">
                    <Input type="number" label="Opacity" bind:value={watermarkOpacity} />
                    <Input type="number" label="Angle" bind:value={watermarkRotation} />
                </div>
                <Button variant="primary" onclick={watermark} disabled={!watermarkText || !!jobs.busyLabel} disabledReason={jobs.busyLabel ? busyReason : 'Enter the watermark text'}>
                    Add watermark
                </Button>
            {:else}
                <p class="empty-text">Select or drag a file to add a watermark.</p>
            {/if}
        </div>

        <div class="tool-section">
            <h4>Sticky note</h4>
            {#if currentPath}
                <Input label="Note content" bind:value={stickyNoteText} placeholder="Enter note text..." />
                <div class="row-group">
                    <Input type="number" label="X position" bind:value={annotX} />
                    <Input type="number" label="Y position" bind:value={annotY} />
                </div>
                <Button variant="secondary" onclick={stickyNote} disabled={!stickyNoteText || !!jobs.busyLabel} disabledReason={jobs.busyLabel ? busyReason : 'Enter the note text'}>
                    Add sticky note
                </Button>
            {:else}
                <p class="empty-text">Select or drag a file to annotate.</p>
            {/if}
        </div>

        <div class="tool-section">
            <h4>Quick tools</h4>
            {#if currentPath}
                <p class="help-text">Simple tools for marking up and finalizing.</p>
                <div class="button-row">
                    {#if labsVisible(SAMPLE_HIGHLIGHT_ID)}
                        <Button variant="secondary" ariaLabel={SAMPLE_HIGHLIGHT_LABS.label} title={`${SAMPLE_HIGHLIGHT_LABS.label}: ${SAMPLE_HIGHLIGHT_LABS.note}`} onclick={highlight} disabled={!!jobs.busyLabel} disabledReason={busyReason}>
                            {SAMPLE_HIGHLIGHT_LABS.name} <span class="labs-badge">{LABS_BADGE}</span>
                        </Button>
                    {/if}
                    <Button variant="destructive" onclick={flatten} disabled={!!jobs.busyLabel} disabledReason={busyReason}>Flatten comments</Button>
                </div>
            {:else}
                <p class="empty-text">Select or drag a file to access quick tools.</p>
            {/if}
        </div>
    {/snippet}
    {#snippet main()}
        {#if currentPath}
            <PdfViewer filePath={currentPath} />
        {:else}
            <EmptyState title="No file selected" body="Open or drag in a PDF to add a watermark, a note or a highlight." />
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

    .row-group {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: var(--space-3);
    }

    /* Two number fields side by side must fit the 300px tools column (they were clipped at 1200x800). */
    .row-group :global(.lp-field) {
        min-width: 0;
    }

    .row-group :global(.lp-input) {
        width: 100%;
        min-width: 0;
        box-sizing: border-box;
    }

    .button-row {
        display: flex;
        flex-direction: column;
        gap: var(--space-2);
    }

    .labs-badge {
        margin-left: var(--space-1);
        font-size: var(--font-size-caption);
        font-weight: 700;
        text-transform: uppercase;
    }

    .help-text {
        margin: 0;
        color: var(--text-secondary);
        font-size: var(--font-size-footnote);
    }

    .empty-text {
        margin: 0;
        color: var(--text-tertiary);
        font-size: var(--font-size-footnote);
        font-style: italic;
    }
</style>

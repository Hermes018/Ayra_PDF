<script lang="ts">
    import { untrack } from 'svelte';
    import { appState } from '$lib/state.svelte';
    import { freeOutputPath } from '$lib/views/studio/outputNames';
    import PdfViewer from '$lib/components/PdfViewer.svelte';
    import ViewShell from '$lib/components/ViewShell.svelte';
    import Button from '$lib/components/ui/Button.svelte';
    import Input from '$lib/components/ui/Input.svelte';
    import Toggle from '$lib/components/ui/Toggle.svelte';
    import EmptyState from '$lib/components/ui/EmptyState.svelte';
    import { JobRunner } from '$lib/core/jobRunner.svelte';
    import type { Permissions } from '$lib/bindings/Permissions';

    // Presentation only: the same three operations (protect, unlock, redact) with the same requests. The password
    // fields are read when each attempt starts and cleared after the job succeeds.
    let currentPath = $derived(appState.currentFile);

    // Protect state
    let protectPassword = $state('');
    let allowPrint = $state(true);
    let allowCopy = $state(false);
    let allowEdit = $state(false);

    // Unlock state
    let unlockPassword = $state('');

    // Redact state (ui/Input works with strings; parsed when the job is built)
    // Pages are 1-based and checked against the real page count (BUG-053); regions need all four values.
    let pageCount = $state<number | null>(null);
    let redactPage = $state('1');
    let redactX = $state('0');
    let redactY = $state('0');
    let redactW = $state('100');
    let redactH = $state('100');

    // A password kept for a Retry must never be applied to a different file.
    $effect(() => {
        void currentPath;
        untrack(() => {
            protectPassword = '';
            unlockPassword = '';
            jobs.invalidate();
            pageCount = null;
            redactError = null;
        });
    });

    let redactError = $state<string | null>(null);

    /** All four values entered as numbers, width and height above 0, page a whole number in 1..=pageCount. */
    function validateRedaction(): { page: number; x: number; y: number; width: number; height: number } | null {
        const text = (v: string | number | null | undefined) => (v === null || v === undefined ? '' : String(v)).trim();
        if (pageCount === null) {
            redactError = 'Wait until the page count is shown, then try again.';
            return null;
        }
        const pageText = text(redactPage);
        const page = Number(pageText);
        if (pageText === '' || !Number.isInteger(page) || page < 1 || page > pageCount) {
            redactError = `Enter a page from 1 to ${pageCount}.`;
            return null;
        }
        const nums = [redactX, redactY, redactW, redactH].map((v) => (text(v) === '' ? NaN : Number(text(v))));
        if (nums.some((n) => !Number.isFinite(n))) {
            redactError = 'Enter all four values: X, Y, width and height, as numbers.';
            return null;
        }
        const [x, y, width, height] = nums;
        if (width <= 0 || height <= 0) {
            redactError = 'Enter all four values: the width and height must be above 0.';
            return null;
        }
        redactError = null;
        return { page, x, y, width, height };
    }

    const NEXT = 'Check the password and that the file is still there, then try again. The original file is unchanged.';
    const jobs = new JobRunner();
    const busyReason = 'Wait for the current job to finish, or cancel it';

    async function handleProtect() {
        const input = currentPath;
        if (!input || !protectPassword) return;
        await jobs.run('Protecting the PDF', "Couldn't protect the PDF.", NEXT, async () => {
            const permissions: Permissions = { allow_print: allowPrint, allow_copy: allowCopy, allow_edit: allowEdit };
            return {
                op: 'ProtectDocument',
                input_path: input,
                output_path: await freeOutputPath(input, 'protected'),
                password: protectPassword,
                permissions,
            };
        });
        if (!jobs.error) protectPassword = '';
    }

    async function handleUnlock() {
        const input = currentPath;
        if (!input || !unlockPassword) return;
        await jobs.run('Removing the password', "Couldn't remove the password.", NEXT, async () => ({
            op: 'UnlockDocument',
            input_path: input,
            output_path: await freeOutputPath(input, 'unlocked'),
            password: unlockPassword,
        }));
        if (!jobs.error) unlockPassword = '';
    }

    async function handleRedact() {
        const input = currentPath;
        if (!input) return;
        const region = validateRedaction();
        if (!region) return; // nothing is defaulted: a bad value is refused here (the backend refuses it too)
        await jobs.run('Redacting the region', "Couldn't redact the region.", 'Check the region values and that the file is still there, then try again. The original file is unchanged.', async () => ({
            op: 'RedactDocument',
            input_path: input,
            output_path: await freeOutputPath(input, 'redacted'),
            regions: [region],
        }));
    }
</script>

<ViewShell title="PDF security" showFileBar={false} busyLabel={jobs.busyLabel} oncancel={() => jobs.cancel()} error={jobs.error}>
    {#snippet tools()}
        <div class="tool-section">
            <h4>Protect document</h4>
            {#if currentPath}
                <p class="file-label">Active: {currentPath.split(/[\/\\]/).pop()}</p>
                <Input type="password" label="Password (user and owner)" bind:value={protectPassword} placeholder="Enter a secure password" autocomplete="off" data-testid="protect-password" />
                <div class="permissions-group">
                    <span class="group-label">Permissions (owner capabilities)</span>
                    <Toggle label="Allow printing" bind:checked={allowPrint} />
                    <Toggle label="Allow copying" bind:checked={allowCopy} />
                    <Toggle label="Allow editing" bind:checked={allowEdit} />
                </div>
                <Button variant="primary" onclick={handleProtect} disabled={!protectPassword || !!jobs.busyLabel} disabledReason={jobs.busyLabel ? busyReason : 'Enter a password to protect the PDF'}>
                    Protect PDF
                </Button>
                <p class="help-text">Encrypts with AES-256 on this computer. Save your password: it cannot be recovered.</p>
            {:else}
                <p class="empty-text">Select or drag a file to protect.</p>
            {/if}
        </div>

        <div class="tool-section">
            <h4>Unlock document</h4>
            {#if currentPath}
                <Input type="password" label="Document password" bind:value={unlockPassword} placeholder="Enter the password to unlock" autocomplete="off" data-testid="unlock-password" />
                <Button variant="secondary" onclick={handleUnlock} disabled={!unlockPassword || !!jobs.busyLabel} disabledReason={jobs.busyLabel ? busyReason : 'Enter the document password'}>
                    Remove security
                </Button>
                <p class="help-text">Creates an unlocked copy.</p>
            {:else}
                <p class="empty-text">Select or drag a file to unlock.</p>
            {/if}
        </div>

        <div class="tool-section">
            <h4>Redaction</h4>
            {#if currentPath}
                <Input type="number" label={pageCount === null ? 'Page' : `Page (1\u2013${pageCount})`} bind:value={redactPage} />
                <div class="row-group">
                    <Input type="number" label="X" bind:value={redactX} />
                    <Input type="number" label="Y" bind:value={redactY} />
                </div>
                <div class="row-group">
                    <Input type="number" label="Width" bind:value={redactW} />
                    <Input type="number" label="Height" bind:value={redactH} />
                </div>
                {#if redactError}
                    <p class="redact-error" role="alert" data-testid="redact-error">{redactError}</p>
                {/if}
                <Button variant="destructive" onclick={handleRedact} disabled={!!jobs.busyLabel} disabledReason={busyReason}>
                    Apply redaction
                </Button>
                <p class="help-text">Removes the text and images in the selected region from the new file. Cannot be undone there; the original is unchanged.</p>
            {:else}
                <p class="empty-text">Select or drag a file to redact.</p>
            {/if}
        </div>
    {/snippet}
    {#snippet main()}
        {#if currentPath}
            <PdfViewer filePath={currentPath} onpagecount={(n) => (pageCount = n)} />
        {:else}
            <EmptyState title="No file selected" body="Open or drag in a PDF to protect, unlock or redact it." />
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

    .permissions-group {
        display: flex;
        flex-direction: column;
        gap: var(--space-2);
    }

    .group-label {
        font-size: var(--font-size-footnote);
        font-weight: 600;
        color: var(--text-secondary);
    }

    .row-group {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: var(--space-3);
    }

    .row-group :global(.lp-field) {
        min-width: 0;
    }

    .row-group :global(.lp-input) {
        width: 100%;
        min-width: 0;
        box-sizing: border-box;
    }

    .redact-error {
        margin: 0;
        color: var(--semantic-danger);
        font-size: var(--font-size-footnote);
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

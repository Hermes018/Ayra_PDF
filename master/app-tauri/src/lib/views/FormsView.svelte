<script lang="ts">
    import { untrack } from 'svelte';
    import { appState } from '$lib/state.svelte';
    import { freeOutputPath } from '$lib/views/studio/outputNames';
    import Button from '$lib/components/ui/Button.svelte';
    import Input from '$lib/components/ui/Input.svelte';
    import EmptyState from '$lib/components/ui/EmptyState.svelte';
    import ErrorState from '$lib/components/ui/ErrorState.svelte';
    import Progress from '$lib/components/ui/Progress.svelte';
    import { JobRunner } from '$lib/core/jobRunner.svelte';
    import { open } from '@tauri-apps/plugin-dialog';

    // Presentation only: the same three operations (fill, flatten, sign) with the same requests. The signing password
    // is read when each attempt starts and cleared after the job succeeds.
    let currentPath = $derived(appState.currentFile);

    // Forms fill state
    let formFieldName = $state('Text1');
    let formFieldValue = $state('John Doe');

    // Sign state
    let p12Path = $state('');
    let signPassword = $state('');
    let pickerFailed = $state(false);

    // A password kept for a Retry must never be applied to a different file.
    $effect(() => {
        void currentPath;
        untrack(() => {
            signPassword = '';
            jobs.invalidate();
        });
    });

    const NEXT = 'Check that the file is still there and not open elsewhere, then try again. The original file is unchanged.';
    const jobs = new JobRunner();
    const busyReason = 'Wait for the current job to finish, or cancel it';

    function handleFlattenForms() {
        const input = currentPath;
        if (!input) return;
        void jobs.run('Flattening the forms', "Couldn't flatten the forms.", NEXT, async () => ({
            op: 'FlattenForms',
            input_path: input,
            output_path: await freeOutputPath(input, 'forms-flattened'),
        }));
    }

    function handleFillForm() {
        const input = currentPath;
        if (!input || !formFieldName.trim()) return;
        void jobs.run('Filling the field', "Couldn't fill the field.", NEXT, async () => {
            const fields: Record<string, string> = {};
            fields[formFieldName] = formFieldValue;
            return {
                op: 'FillForm',
                input_path: input,
                output_path: await freeOutputPath(input, 'filled'),
                fields,
            };
        });
    }

    async function selectP12() {
        pickerFailed = false;
        try {
            const selected = await open({
                title: 'Select PKCS#12 Certificate',
                filters: [{ name: 'Certificates', extensions: ['p12', 'pfx'] }],
            });
            if (typeof selected === 'string') {
                p12Path = selected;
            }
        } catch (e) {
            console.error('Failed to select p12:', e);
            pickerFailed = true;
        }
    }

    async function handleSign() {
        const input = currentPath;
        if (!input || !p12Path || !signPassword) return;
        await jobs.run('Signing the document', "Couldn't sign the document.", 'Check the certificate and its password, then try again. The original file is unchanged.', async () => ({
            op: 'SignDocument',
            input_path: input,
            output_path: await freeOutputPath(input, 'signed'),
            p12_path: p12Path,
            password: signPassword,
        }));
        if (!jobs.error) signPassword = '';
    }
</script>

<div class="forms-view">
    <header class="view-header">
        <h1>Forms and signatures</h1>
        <p>Fill interactive forms, flatten fields, and digitally sign documents on this computer.</p>
    </header>

    {#if pickerFailed}
        <ErrorState what="Couldn't open the file picker." next="Try again." retry={selectP12} />
    {/if}
    {#if jobs.error}
        <ErrorState what={jobs.error.what} next={jobs.error.next} retry={jobs.error.retry} data-testid="view-op-error" />
    {/if}
    {#if jobs.busyLabel}
        <Progress value={null} label={jobs.busyLabel} oncancel={() => jobs.cancel()} data-testid="view-op-progress" />
    {/if}

    {#if !currentPath}
        <EmptyState title="No document selected" body="Select or open a PDF to use forms and signatures." />
    {:else}
        <div class="content-grid">
            <!-- Form filling -->
            <div class="tools-card">
                <h3>Fill form</h3>
                <p class="description">Fill out a text field in the document.</p>
                <Input label="Field name" bind:value={formFieldName} placeholder="e.g. Text1" />
                <Input label="Field value" bind:value={formFieldValue} placeholder="Enter value..." />
                <Button variant="primary" onclick={handleFillForm} disabled={!formFieldName.trim() || !!jobs.busyLabel} disabledReason={jobs.busyLabel ? busyReason : 'Enter the field name'}>
                    Apply value
                </Button>
            </div>

            <!-- Forms management -->
            <div class="tools-card">
                <h3>Manage forms</h3>
                <p class="description">Bake form fields permanently into the page, removing interactivity.</p>
                <Button variant="secondary" onclick={handleFlattenForms} disabled={!!jobs.busyLabel} disabledReason={busyReason}>
                    Flatten all forms
                </Button>
            </div>

            <!-- Signature -->
            <div class="tools-card">
                <h3>Digital signature</h3>
                <p class="description">Sign the document with a local .p12 or .pfx certificate.</p>
                <div class="file-picker">
                    <Input label="Certificate (.p12 / .pfx)" value={p12Path} placeholder="Select a certificate..." class="flex-1" data-testid="cert-path" />
                    <Button variant="secondary" onclick={selectP12}>Browse</Button>
                </div>
                <Input type="password" label="Password" bind:value={signPassword} placeholder="Certificate password" autocomplete="off" data-testid="sign-password" />
                <Button variant="primary" onclick={handleSign} disabled={!p12Path || !signPassword || !!jobs.busyLabel} disabledReason={jobs.busyLabel ? busyReason : 'Choose a certificate and enter its password'}>
                    Sign document
                </Button>
            </div>
        </div>
    {/if}
</div>

<style>
    .forms-view {
        padding: var(--space-8);
        max-width: 1200px;
        margin: 0 auto;
        color: var(--text-primary);
        display: flex;
        flex-direction: column;
        gap: var(--space-4);
    }

    .view-header h1 {
        margin: 0 0 var(--space-2) 0;
        font-size: var(--font-size-title1);
        font-weight: 600;
        color: var(--text-primary);
    }

    .view-header p {
        margin: 0;
        color: var(--text-secondary);
        font-size: var(--font-size-body);
    }

    .content-grid {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
        gap: var(--space-6);
    }

    .tools-card {
        background-color: var(--glass-bg-surface);
        border: 1px solid var(--glass-border-subtle);
        border-radius: var(--radius-lg);
        padding: var(--space-5);
        display: flex;
        flex-direction: column;
        gap: var(--space-3);
        box-shadow: var(--glass-shadow-sm);
    }

    .tools-card h3 {
        margin: 0;
        font-size: var(--font-size-title3);
    }

    .description {
        margin: 0;
        color: var(--text-secondary);
        font-size: var(--font-size-footnote);
    }

    .file-picker {
        display: flex;
        align-items: flex-end;
        gap: var(--space-2);
    }

    .file-picker :global(.flex-1) {
        flex: 1;
        min-width: 0;
    }
</style>

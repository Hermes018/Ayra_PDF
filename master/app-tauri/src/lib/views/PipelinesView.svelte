<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import { IpcClient } from '$lib/ipc';
    import { friendlyJobError } from '$lib/core/jobError';
    import ErrorState from '$lib/components/ui/ErrorState.svelte';
    import Button from '$lib/components/ui/Button.svelte';
    import Input from '$lib/components/ui/Input.svelte';
    import Select from '$lib/components/ui/Select.svelte';
    import Toggle from '$lib/components/ui/Toggle.svelte';
    import EmptyState from '$lib/components/ui/EmptyState.svelte';
    import { parseNumberOr } from '$lib/core/parseNumber';
    import For from '$lib/components/For.svelte';
    import { appState } from '$lib/state.svelte';
    import { siblingOutputPath, resolveFreeOutput } from '$lib/views/studio/outputNames';
    import type { PipelinePreset } from '$lib/bindings/PipelinePreset';
    import type { PipelineStep } from '$lib/bindings/PipelineStep';
    import type { JobRequest } from '$lib/bindings/JobRequest';

    // State via Svelte 5 runes
    let presets = $state<PipelinePreset[]>([]);
    let activeSteps = $state<PipelineStep[]>([]);
    let selectedPresetId = $state<string | null>(null);
    let newStepType = $state<string>('Compress');
    let inputPath = $state<string>('');
    let outputPath = $state<string>('');
    let executionMessage = $state<string | null>(null);
    let isSubmitting = $state(false);

    // Synchronize inputPath with appState if available
    $effect(() => {
        if (!inputPath && appState.currentFile) {
            inputPath = appState.currentFile;
            outputPath = siblingOutputPath(inputPath, '_workflow.pdf');
        }
    });

    onDestroy(() => unlistenJobs?.());

    // A queued run that later fails replaces the "enqueued successfully" banner with the plain ErrorState.
    let lastJobId: string | null = null;
    let unlistenJobs: (() => void) | undefined;
    onMount(async () => {
        try {
            unlistenJobs = await IpcClient.onJobEvent((event) => {
                const ev = event as { kind?: string; data?: { job_id?: string; error_message?: string } };
                if (!lastJobId || ev.data?.job_id !== lastJobId) return;
                if (ev.kind === 'Failed') {
                    executionMessage = null;
                    const f = friendlyJobError(ev.data?.error_message ?? '', {
                        what: 'The workflow failed.',
                        next: 'Check the input file and the steps, then try again.',
                    });
                    runFailure = { what: f.what, next: f.next, retry: () => void handleRunWorkflow() };
                }
            });
        } catch (e) {
            console.error('Failed to listen for job events:', e);
        }
        try {
            presets = await IpcClient.listPipelinePresets();
            if (presets.length > 0 && activeSteps.length === 0) {
                applyPreset(presets[0]);
            }
        } catch (e) {
            console.error('Failed to load pipeline presets:', e);
        }
    });

    function applyPreset(preset: PipelinePreset) {
        selectedPresetId = preset.id;
        activeSteps = JSON.parse(JSON.stringify(preset.steps));
        runFailure = null;
        executionMessage = `Loaded preset: ${preset.name}`;
    }

    const STEP_OPTIONS = [
        { value: 'Repair', label: '🛠️ Repair' },
        { value: 'Compress', label: '⚡ Compress' },
        { value: 'Ocr', label: '🔍 OCR' },
        { value: 'Rotate', label: '🔄 Rotate' },
        { value: 'FlattenComments', label: '💬 Flatten comments' },
        { value: 'FlattenForms', label: '📝 Flatten forms' },
        { value: 'Watermark', label: '🏷️ Watermark' },
        { value: 'Protect', label: '🔒 Protect' },
        { value: 'PdfToPdfA', label: '🏛️ PDF/A archival' },
    ];
    const PROFILE_OPTIONS = [
        { value: 'Low', label: 'Low (maximum quality)' },
        { value: 'Medium', label: 'Medium (recommended)' },
        { value: 'High', label: 'High (maximum compression)' },
    ];
    const DEGREE_OPTIONS = [
        { value: '90', label: '90° clockwise' },
        { value: '180', label: '180° half turn' },
        { value: '270', label: '270° counter-clockwise' },
    ];
    const CONFORMANCE_OPTIONS = [
        { value: 'PdfA1b', label: 'PDF/A-1b (basic)' },
        { value: 'PdfA2b', label: 'PDF/A-2b (recommended)' },
        { value: 'PdfA3b', label: 'PDF/A-3b (embedded files)' },
    ];

    function createDefaultStep(type: string): PipelineStep {
        switch (type) {
            case 'Repair':
                return { type: 'Repair' };
            case 'Compress':
                return { type: 'Compress', profile: 'Medium' };
            case 'Ocr':
                return { type: 'Ocr' };
            case 'Rotate':
                return { type: 'Rotate', degrees: 90, pages: [] };
            case 'FlattenComments':
                return { type: 'FlattenComments' };
            case 'FlattenForms':
                return { type: 'FlattenForms' };
            case 'Watermark':
                return {
                    type: 'Watermark',
                    settings: {
                        text: 'CONFIDENTIAL',
                        opacity: 0.35,
                        rotation: 45.0,
                        color: [0.8, 0.2, 0.2]
                    }
                };
            case 'Protect':
                return {
                    type: 'Protect',
                    password: '', // no hidden default: the user types it, and a workflow with an empty one is refused
                    permissions: {
                        allow_print: true,
                        allow_copy: false,
                        allow_edit: false
                    }
                };
            case 'PdfToPdfA':
                return { type: 'PdfToPdfA', conformance: 'PdfA2b' };
            default:
                return { type: 'Repair' };
        }
    }

    function handleAddStep() {
        const step = createDefaultStep(newStepType);
        activeSteps = [...activeSteps, step];
        selectedPresetId = null; // Custom modified
    }

    function handleRemoveStep(index: number) {
        activeSteps = activeSteps.filter((_, idx) => idx !== index);
        selectedPresetId = null;
    }

    function handleMoveUp(index: number) {
        if (index <= 0) return;
        const copy = [...activeSteps];
        const temp = copy[index];
        copy[index] = copy[index - 1];
        copy[index - 1] = temp;
        activeSteps = copy;
        selectedPresetId = null;
    }

    function handleMoveDown(index: number) {
        if (index >= activeSteps.length - 1) return;
        const copy = [...activeSteps];
        const temp = copy[index];
        copy[index] = copy[index + 1];
        copy[index + 1] = temp;
        activeSteps = copy;
        selectedPresetId = null;
    }

    // A failed run: what happened and what to do, with a Retry that picks the next free name.
    let runFailure = $state<{ what: string; next: string; retry: () => void } | null>(null);

    async function handleRunWorkflow() {
        if (isSubmitting) return; // set before any await: a double click starts one job
        executionMessage = null; // a new run replaces the previous message
        runFailure = null;
        if (!inputPath || !outputPath) {
            executionMessage = 'Please select both input and output paths.';
            return;
        }
        if (activeSteps.length === 0) {
            executionMessage = 'Workflow must contain at least one transformation step.';
            return;
        }
        if (activeSteps.some((st) => st.type === 'Watermark' && (!(st.settings.opacity >= 0.1) || st.settings.opacity > 1))) {
            executionMessage = 'Watermark opacity must be between 0.1 and 1.';
            return;
        }
        if (activeSteps.some((st) => st.type === 'Protect' && !st.password)) {
            executionMessage = 'Enter a password for the Protect step before running the workflow.';
            return;
        }

        isSubmitting = true;
        runFailure = null;
        try {
            const preset = presets.find(p => p.id === selectedPresetId);
            // Never the input, never an existing file: next free name (BUG-051).
            const out = await resolveFreeOutput(outputPath.trim(), inputPath);
            if (out !== outputPath.trim()) outputPath = out;
            const req: JobRequest = {
                op: 'RunPipeline',
                input_path: inputPath,
                output_path: out,
                steps: activeSteps,
                preset_name: preset ? preset.name : null
            };
            const jobId = await IpcClient.submitJob(req);
            lastJobId = jobId;
            executionMessage = `Workflow enqueued successfully (Job ID: ${jobId.slice(0, 8)}). Track progress in Batch Queue.`;
        } catch (e) {
            console.error('Failed to dispatch workflow:', e);
            const f = friendlyJobError(e instanceof Error ? e.message : String(e), {
                what: "Couldn't start the workflow.",
                next: 'Check the input and output paths, then try again.',
            });
            runFailure = { what: f.what, next: f.next, retry: () => void handleRunWorkflow() };
        } finally {
            isSubmitting = false;
        }
    }

    function getStepTitle(step: PipelineStep): string {
        switch (step.type) {
            case 'Repair': return 'Repair PDF Structure';
            case 'Compress': return 'Optimize & Compress';
            case 'Ocr': return 'OCR Text Recognition';
            case 'Rotate': return 'Rotate Pages / Document';
            case 'FlattenComments': return 'Flatten Comments & Annotations';
            case 'FlattenForms': return 'Flatten Interactive Forms';
            case 'Watermark': return 'Burn-in Watermark';
            case 'Protect': return 'Password & Security Permissions';
            case 'PdfToPdfA': return 'Archival PDF/A Conversion';
            default: return 'Custom Step';
        }
    }

    function getStepIcon(step: PipelineStep): string {
        switch (step.type) {
            case 'Repair': return '🛠️';
            case 'Compress': return '⚡';
            case 'Ocr': return '🔍';
            case 'Rotate': return '🔄';
            case 'FlattenComments': return '💬';
            case 'FlattenForms': return '📝';
            case 'Watermark': return '🏷️';
            case 'Protect': return '🔒';
            case 'PdfToPdfA': return '🏛️';
            default: return '⚙️';
        }
    }
</script>

<div class="workflows-container" data-testid="pipelines-view">
    <header class="header">
        <div class="header-title">
            <span class="icon" aria-hidden="true">🔀</span>
            <div>
                <h2>Workflows & Tool Chaining</h2>
                <p class="subtitle">Chain tools into sequential pipelines with built-in presets. A failed step stops the chain.</p>
            </div>
        </div>
    </header>

    {#if runFailure}
        <ErrorState what={runFailure.what} next={runFailure.next} retry={runFailure.retry} data-testid="run-failure" />
    {/if}

    {#if executionMessage}
        <div class="alert-banner" data-testid="execution-message" role="status">
            <span>{executionMessage}</span>
            <Button variant="plain" size="sm" ariaLabel="Dismiss this message" title="Dismiss this message" onclick={() => { executionMessage = null; }}>✕</Button>
        </div>
    {/if}

    <!-- Preset workflows -->
    <section class="section" data-testid="presets-section">
        <div class="section-header">
            <h3>Built-in presets</h3>
        </div>
        <div class="presets-grid">
            <For items={presets}>
                {#snippet children(preset)}
                    <Button
                        variant="secondary"
                        block
                        class="preset-card"
                        pressed={selectedPresetId === preset.id}
                        data-testid={`preset-card-${preset.id}`}
                        onclick={() => applyPreset(preset)}
                    >
                        <span class="preset-body">
                            <span class="preset-header">
                                <span class="preset-title">{preset.name}</span>
                                {#if selectedPresetId === preset.id}
                                    <span class="preset-active-badge">Active</span>
                                {/if}
                            </span>
                            <span class="preset-desc">{preset.description}</span>
                            <span class="preset-steps-chips">
                                <For items={preset.steps}>
                                    {#snippet children(step, idx)}
                                        <span class="step-chip">{step.type}</span>
                                        {#if idx < preset.steps.length - 1}
                                            <span class="chip-arrow" aria-hidden="true">➔</span>
                                        {/if}
                                    {/snippet}
                                </For>
                            </span>
                        </span>
                    </Button>
                {/snippet}
            </For>
        </div>
    </section>

    <!-- Pipeline builder -->
    <section class="section builder-section" data-testid="builder-section">
        <div class="section-header">
            <h3>Workflow chain ({activeSteps.length} steps)</h3>
            <div class="add-step-bar">
                <Select label="Step to add" hideLabel compact bind:value={newStepType} options={STEP_OPTIONS} data-testid="select-new-step" />
                <Button variant="secondary" size="sm" onclick={handleAddStep} data-testid="btn-add-step">Add step</Button>
            </div>
        </div>

        {#if activeSteps.length === 0}
            <EmptyState title="No steps yet" body="Select a preset above or add a step to begin." />
        {:else}
            <div class="steps-list" data-testid="active-steps-list">
                <For items={activeSteps}>
                    {#snippet children(step, index)}
                        <div class="step-card" data-testid="step-card">
                            <div class="step-card-header">
                                <div class="step-card-title">
                                    <span class="step-number">#{index + 1}</span>
                                    <span class="step-icon" aria-hidden="true">{getStepIcon(step)}</span>
                                    <span class="step-name">{getStepTitle(step)}</span>
                                </div>
                                <div class="step-controls">
                                    <Button variant="plain" size="sm" ariaLabel="Move step up" title="Move up" disabled={index === 0} disabledReason="This is the first step" data-testid={`btn-move-up-${index}`} onclick={() => handleMoveUp(index)}>▲</Button>
                                    <Button variant="plain" size="sm" ariaLabel="Move step down" title="Move down" disabled={index === activeSteps.length - 1} disabledReason="This is the last step" data-testid={`btn-move-down-${index}`} onclick={() => handleMoveDown(index)}>▼</Button>
                                    <Button variant="plain" size="sm" ariaLabel="Remove step" title="Remove step" data-testid={`btn-remove-${index}`} onclick={() => handleRemoveStep(index)}>✕</Button>
                                </div>
                            </div>

                            <div class="step-config-body">
                                {#if step.type === 'Compress'}
                                    <Select label="Compression profile" options={PROFILE_OPTIONS} data-testid="compress-profile-select" bind:value={() => step.profile, (v) => (step.profile = v as typeof step.profile)} />
                                {:else if step.type === 'Rotate'}
                                    <Select label="Rotation" options={DEGREE_OPTIONS} bind:value={() => String(step.degrees), (v) => (step.degrees = Number(v))} />
                                {:else if step.type === 'Watermark'}
                                    <div class="field-row">
                                        <Input label="Watermark text" placeholder="CONFIDENTIAL" data-testid="watermark-text-input" bind:value={step.settings.text} />
                                        <Input type="number" label="Opacity" min={0.1} max={1} step={0.05} bind:value={() => String(step.settings.opacity), (v) => (step.settings.opacity = parseNumberOr(v, 0.35))} />
                                    </div>
                                {:else if step.type === 'Protect'}
                                    <div class="field-row">
                                        <Input type="password" label="Password" placeholder="Enter a password" autocomplete="off" data-testid="protect-password-input" bind:value={step.password} />
                                        <div class="checkbox-group">
                                            <Toggle label="Allow printing" bind:checked={step.permissions.allow_print} />
                                            <Toggle label="Allow copying" bind:checked={step.permissions.allow_copy} />
                                        </div>
                                    </div>
                                {:else if step.type === 'PdfToPdfA'}
                                    <Select label="PDF/A conformance standard" options={CONFORMANCE_OPTIONS} bind:value={() => step.conformance, (v) => (step.conformance = v as typeof step.conformance)} />
                                {:else}
                                    <span class="step-hint">Built-in transformation. No extra parameters needed.</span>
                                {/if}
                            </div>
                        </div>
                    {/snippet}
                </For>
            </div>
        {/if}
    </section>

    <!-- Execution -->
    <section class="section execution-section" data-testid="execution-section">
        <h3>Run the workflow</h3>
        <div class="execution-inputs">
            <Input label="Input PDF file" placeholder="/path/to/source.pdf" data-testid="input-path-field" bind:value={inputPath} />
            <Input label="Output destination" placeholder="/path/to/destination.pdf" data-testid="output-path-field" bind:value={outputPath} />
        </div>

        <div class="execution-footer">
            <p class="fail-closed-notice">Intermediate steps run in a staging folder. The destination is only written when every step succeeds.</p>
            <Button
                variant="primary"
                size="lg"
                busy={isSubmitting}
                disabled={activeSteps.length === 0 || !inputPath}
                disabledReason={activeSteps.length === 0 ? 'Add at least one step' : 'Enter the input file'}
                data-testid="btn-run-workflow"
                onclick={handleRunWorkflow}
            >
                Run workflow ({activeSteps.length} steps)
            </Button>
        </div>
    </section>
</div>

<style>
    .workflows-container {
        display: flex;
        flex-direction: column;
        gap: var(--space-4);
        padding: var(--space-4);
        color: var(--text-primary);
    }

    .header-title {
        display: flex;
        align-items: center;
        gap: var(--space-3);
    }

    .header-title h2 {
        margin: 0;
        font-size: var(--font-size-title1);
        color: var(--accent-primary);
    }

    .subtitle {
        margin: var(--space-1) 0 0;
        color: var(--text-secondary);
        font-size: var(--font-size-footnote);
    }

    .section {
        background: var(--glass-bg-surface);
        border: 1px solid var(--glass-border-subtle);
        border-radius: var(--radius-lg);
        padding: var(--space-4);
        display: flex;
        flex-direction: column;
        gap: var(--space-3);
    }

    .section h3 {
        margin: 0;
        font-size: var(--font-size-title3);
    }

    .section-header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: var(--space-3);
        flex-wrap: wrap;
    }

    .add-step-bar {
        display: flex;
        align-items: flex-end;
        gap: var(--space-2);
    }

    .alert-banner {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: var(--space-3);
        padding: var(--space-3) var(--space-4);
        border-radius: var(--radius-md);
        background: var(--glass-bg-input);
        border: 1px solid var(--glass-border-subtle);
        font-size: var(--font-size-footnote);
    }

    .presets-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
        gap: var(--space-3);
    }

    .presets-grid :global(.preset-card .lp-btn) {
        height: auto;
        min-height: 0;
        justify-content: flex-start;
        text-align: left;
        padding: var(--space-3);
        font-weight: 400;
    }

    .preset-body {
        display: flex;
        flex-direction: column;
        gap: var(--space-2);
        white-space: normal;
    }

    .preset-header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: var(--space-2);
    }

    .preset-title {
        font-weight: 600;
        font-size: var(--font-size-body);
    }

    .preset-active-badge,
    .step-chip {
        font-size: var(--font-size-caption);
        padding: 0 var(--space-2);
        border-radius: var(--radius-pill);
        background: var(--glass-bg-active);
    }

    .preset-desc,
    .step-hint,
    .fail-closed-notice {
        margin: 0;
        color: var(--text-secondary);
        font-size: var(--font-size-footnote);
        line-height: 1.4;
    }

    .preset-steps-chips {
        display: flex;
        flex-wrap: wrap;
        align-items: center;
        gap: var(--space-1);
    }

    .steps-list {
        display: flex;
        flex-direction: column;
        gap: var(--space-3);
    }

    .step-card {
        border: 1px solid var(--glass-border-subtle);
        border-radius: var(--radius-md);
        background: var(--glass-bg-input);
        padding: var(--space-3);
        display: flex;
        flex-direction: column;
        gap: var(--space-3);
    }

    .step-card-header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: var(--space-3);
    }

    .step-card-title,
    .step-controls {
        display: flex;
        align-items: center;
        gap: var(--space-2);
    }

    .step-number {
        font-weight: 700;
        color: var(--accent-primary);
    }

    .step-name {
        font-weight: 600;
    }

    .field-row {
        display: flex;
        gap: var(--space-3);
        align-items: flex-end;
        flex-wrap: wrap;
    }

    .checkbox-group {
        display: flex;
        flex-direction: column;
        gap: var(--space-2);
    }

    .execution-inputs {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: var(--space-3);
    }

    .execution-footer {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: var(--space-4);
        flex-wrap: wrap;
    }
</style>

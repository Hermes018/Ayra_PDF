<script lang="ts">
    import { onMount } from 'svelte';
    import { IpcClient } from '$lib/ipc';
    import For from '$lib/components/For.svelte';
    import { appState } from '$lib/state.svelte';
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
            outputPath = inputPath.replace(/\.pdf$/i, '_workflow.pdf');
        }
    });

    onMount(async () => {
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
        executionMessage = `Loaded preset: ${preset.name}`;
    }

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
                    password: 'password123',
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

    async function handleRunWorkflow() {
        if (!inputPath || !outputPath) {
            executionMessage = 'Please select both input and output paths.';
            return;
        }
        if (activeSteps.length === 0) {
            executionMessage = 'Workflow must contain at least one transformation step.';
            return;
        }

        const preset = presets.find(p => p.id === selectedPresetId);
        const req: JobRequest = {
            op: 'RunPipeline',
            input_path: inputPath,
            output_path: outputPath,
            steps: activeSteps,
            preset_name: preset ? preset.name : null
        };

        try {
            isSubmitting = true;
            const jobId = await IpcClient.submitJob(req);
            executionMessage = `Workflow enqueued successfully (Job ID: ${jobId.slice(0, 8)}). Track progress in Batch Queue.`;
        } catch (e) {
            executionMessage = `Failed to dispatch workflow: ${e}`;
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
            <span class="icon">🔀</span>
            <div>
                <h2>Workflows & Tool Chaining</h2>
                <p class="subtitle">Chain multiple tools into sequential, fail-closed pipelines with built-in presets</p>
            </div>
        </div>
    </header>

    {#if executionMessage}
        <div class="alert-banner" data-testid="execution-message">
            <span>{executionMessage}</span>
            <button class="close-alert" onclick={() => { executionMessage = null; }}>×</button>
        </div>
    {/if}

    <!-- Preset Workflows Grid -->
    <section class="section" data-testid="presets-section">
        <div class="section-header">
            <h3>Built-in Presets (Workflows)</h3>
            <span class="badge">iLovePDF Equivalent</span>
        </div>
        <div class="presets-grid">
            <For items={presets}>
                {#snippet children(preset)}
                <div
                    class="preset-card"
                    class:selected={selectedPresetId === preset.id}
                    onclick={() => applyPreset(preset)}
                    data-testid={`preset-card-${preset.id}`}
                    role="button"
                    tabindex="0"
                    aria-pressed={selectedPresetId === preset.id}
                    onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); applyPreset(preset); } }}
                >
                    <div class="preset-header">
                        <span class="preset-title">{preset.name}</span>
                        {#if selectedPresetId === preset.id}
                            <span class="preset-active-badge">Active</span>
                        {/if}
                    </div>
                    <p class="preset-desc">{preset.description}</p>
                    <div class="preset-steps-chips">
                        <For items={preset.steps}>
                            {#snippet children(step, idx)}
                            <span class="step-chip">{step.type}</span>
                            {#if idx < preset.steps.length - 1}
                                <span class="chip-arrow">➔</span>
                            {/if}
                            {/snippet}
                        </For>
                    </div>
                </div>
                {/snippet}
            </For>
        </div>
    </section>

    <!-- Interactive Pipeline Builder -->
    <section class="section builder-section" data-testid="builder-section">
        <div class="section-header">
            <h3>Active Workflow Chain ({activeSteps.length} Steps)</h3>
            <div class="add-step-bar">
                <select bind:value={newStepType} class="select-input" data-testid="select-new-step">
                    <option value="Repair">🛠️ Repair</option>
                    <option value="Compress">⚡ Compress</option>
                    <option value="Ocr">🔍 OCR</option>
                    <option value="Rotate">🔄 Rotate</option>
                    <option value="FlattenComments">💬 Flatten Comments</option>
                    <option value="FlattenForms">📝 Flatten Forms</option>
                    <option value="Watermark">🏷️ Watermark</option>
                    <option value="Protect">🔒 Protect</option>
                    <option value="PdfToPdfA">🏛️ PDF/A Archival</option>
                </select>
                <button class="btn btn-secondary" onclick={handleAddStep} data-testid="btn-add-step">
                    ➕ Add Step
                </button>
            </div>
        </div>

        {#if activeSteps.length === 0}
            <div class="empty-steps">
                <p>No steps in workflow. Select a preset above or add custom steps to begin.</p>
            </div>
        {:else}
            <div class="steps-list" data-testid="active-steps-list">
                <For items={activeSteps}>
                    {#snippet children(step, index)}
                    <div class="step-card" data-testid="step-card">
                        <div class="step-card-header">
                            <div class="step-card-title">
                                <span class="step-number">#{index + 1}</span>
                                <span class="step-icon">{getStepIcon(step)}</span>
                                <span class="step-name">{getStepTitle(step)}</span>
                            </div>
                            <div class="step-controls">
                                <button
                                    class="icon-btn"
                                    onclick={() => handleMoveUp(index)}
                                    disabled={index === 0}
                                    title="Move Up"
                                    data-testid={`btn-move-up-${index}`}
                                >
                                    ▲
                                </button>
                                <button
                                    class="icon-btn"
                                    onclick={() => handleMoveDown(index)}
                                    disabled={index === activeSteps.length - 1}
                                    title="Move Down"
                                    data-testid={`btn-move-down-${index}`}
                                >
                                    ▼
                                </button>
                                <button
                                    class="icon-btn btn-delete"
                                    onclick={() => handleRemoveStep(index)}
                                    title="Remove Step"
                                    data-testid={`btn-remove-${index}`}
                                >
                                    ✕
                                </button>
                            </div>
                        </div>

                        <!-- Step Specific Configuration -->
                        <div class="step-config-body">
                            {#if step.type === 'Compress'}
                                <label class="field-label">
                                    Compression Profile:
                                    <select bind:value={step.profile} class="select-input" data-testid="compress-profile-select">
                                        <option value="Low">Low (Maximum Quality)</option>
                                        <option value="Medium">Medium (Recommended)</option>
                                        <option value="High">High (Maximum Compression)</option>
                                    </select>
                                </label>
                            {:else if step.type === 'Rotate'}
                                <label class="field-label">
                                    Rotation Degrees:
                                    <select bind:value={step.degrees} class="select-input">
                                        <option value={90}>90° Clockwise</option>
                                        <option value={180}>180° Half Turn</option>
                                        <option value={270}>270° Counter-Clockwise</option>
                                    </select>
                                </label>
                            {:else if step.type === 'Watermark'}
                                <div class="field-row">
                                    <label class="field-label">
                                        Watermark Text:
                                        <input
                                            type="text"
                                            bind:value={step.settings.text}
                                            class="text-input"
                                            placeholder="CONFIDENTIAL"
                                            data-testid="watermark-text-input"
                                        />
                                    </label>
                                    <label class="field-label">
                                        Opacity:
                                        <input
                                            type="number"
                                            step="0.05"
                                            min="0.1"
                                            max="1.0"
                                            bind:value={step.settings.opacity}
                                            class="text-input small"
                                        />
                                    </label>
                                </div>
                            {:else if step.type === 'Protect'}
                                <div class="field-row">
                                    <label class="field-label">
                                        Password:
                                        <input
                                            type="password"
                                            bind:value={step.password}
                                            class="text-input"
                                            placeholder="Password"
                                        />
                                    </label>
                                    <div class="checkbox-group">
                                        <label class="checkbox-label">
                                            <input type="checkbox" bind:checked={step.permissions.allow_print} />
                                            Print
                                        </label>
                                        <label class="checkbox-label">
                                            <input type="checkbox" bind:checked={step.permissions.allow_copy} />
                                            Copy
                                        </label>
                                    </div>
                                </div>
                            {:else if step.type === 'PdfToPdfA'}
                                <label class="field-label">
                                    PDF/A Conformance Standard:
                                    <select bind:value={step.conformance} class="select-input">
                                        <option value="PdfA1b">PDF/A-1b (Basic)</option>
                                        <option value="PdfA2b">PDF/A-2b (Recommended)</option>
                                        <option value="PdfA3b">PDF/A-3b (Embedded Files)</option>
                                    </select>
                                </label>
                            {:else}
                                <span class="step-hint">Native self-contained transformation. No additional parameters needed.</span>
                            {/if}
                        </div>
                    </div>
                    {/snippet}
                </For>
            </div>
        {/if}
    </section>

    <!-- Execution Panel -->
    <section class="section execution-section" data-testid="execution-section">
        <h3>Execute Workflow</h3>
        <div class="execution-inputs">
            <div class="input-group">
                <label for="workflow-input-file">Input PDF File:</label>
                <input
                    id="workflow-input-file"
                    type="text"
                    bind:value={inputPath}
                    placeholder="/path/to/source.pdf"
                    class="text-input"
                    data-testid="input-path-field"
                />
            </div>
            <div class="input-group">
                <label for="workflow-output-file">Output Destination:</label>
                <input
                    id="workflow-output-file"
                    type="text"
                    bind:value={outputPath}
                    placeholder="/path/to/destination.pdf"
                    class="text-input"
                    data-testid="output-path-field"
                />
            </div>
        </div>

        <div class="execution-footer">
            <div class="fail-closed-notice">
                <span>🛡️ Fail-closed execution: All intermediate steps execute in an isolated staging workspace. Destination will only be updated upon complete chain success.</span>
            </div>
            <button
                class="btn btn-primary btn-large"
                onclick={handleRunWorkflow}
                disabled={isSubmitting || activeSteps.length === 0 || !inputPath}
                data-testid="btn-run-workflow"
            >
                🚀 Run Workflow ({activeSteps.length} Steps)
            </button>
        </div>
    </section>
</div>

<style>
    .workflows-container {
        padding: 2rem;
        display: flex;
        flex-direction: column;
        gap: 1.5rem;
        color: var(--text-primary);
        max-width: 1200px;
        margin: 0 auto;
        width: 100%;
    }

    .header-title {
        display: flex;
        align-items: center;
        gap: 1rem;
    }

    .header-title .icon {
        font-size: 2.2rem;
    }

    .header-title h2 {
        font-size: 1.75rem;
        margin: 0;
        font-weight: 700;
        color: var(--text-primary);
    }

    .subtitle {
        margin: 0.25rem 0 0;
        font-size: 0.9rem;
        color: var(--text-secondary);
    }

    .alert-banner {
        display: flex;
        justify-content: space-between;
        align-items: center;
        background: var(--glass-bg-input);
        border-left: 4px solid var(--accent-primary);
        padding: 0.75rem 1rem;
        border-radius: var(--radius-sm, 8px);
        font-size: 0.9rem;
        color: var(--text-primary);
    }

    .close-alert {
        background: none;
        border: none;
        color: var(--text-secondary);
        font-size: 1.25rem;
        cursor: pointer;
    }

    .section {
        background: var(--glass-bg-surface);
        border: 1px solid var(--glass-border-subtle);
        border-radius: var(--radius-lg, 14px);
        box-shadow: inset 0 1px 0 0 var(--glass-border-specular), var(--glass-shadow-sm);
        padding: 1.5rem;
        display: flex;
        flex-direction: column;
        gap: 1.25rem;
    }

    .section-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        flex-wrap: wrap;
        gap: 0.75rem;
    }

    .section-header h3 {
        margin: 0;
        font-size: 1.2rem;
        color: var(--text-primary);
    }

    .badge {
        font-size: 0.75rem;
        background: var(--accent-primary);
        color: var(--text-on-accent);
        font-weight: 700;
        padding: 0.25rem 0.5rem;
        border-radius: var(--radius-xs, 6px);
        text-transform: uppercase;
    }

    /* Presets Grid */
    .presets-grid {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
        gap: 1rem;
    }

    .preset-card {
        background: var(--glass-bg-input);
        border: 1px solid var(--glass-border-subtle);
        border-radius: var(--radius-md, 10px);
        padding: 1.25rem;
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
        cursor: pointer;
        transition: all var(--dur-fast, 120ms) ease;
    }

    .preset-card:hover {
        border-color: var(--accent-primary);
        background: var(--glass-bg-hover);
        transform: translateY(-2px);
    }

    .preset-card.selected {
        border-color: var(--accent-primary);
        background: var(--glass-bg-active);
        box-shadow: 0 0 12px var(--accent-ring);
    }

    .preset-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    .preset-title {
        font-weight: 700;
        font-size: 1.05rem;
        color: var(--text-primary);
    }

    .preset-active-badge {
        background: var(--accent-primary);
        color: var(--text-on-accent);
        font-size: 0.7rem;
        font-weight: 700;
        padding: 0.2rem 0.5rem;
        border-radius: var(--radius-xs, 6px);
        text-transform: uppercase;
    }

    .preset-desc {
        font-size: 0.85rem;
        color: var(--text-secondary);
        margin: 0;
        line-height: 1.4;
    }

    .preset-steps-chips {
        display: flex;
        flex-wrap: wrap;
        align-items: center;
        gap: 0.4rem;
        margin-top: auto;
    }

    .step-chip {
        background: var(--glass-bg-input);
        color: var(--accent-primary);
        border: 1px solid var(--glass-border-subtle);
        font-size: 0.75rem;
        font-weight: 600;
        padding: 0.2rem 0.5rem;
        border-radius: var(--radius-xs, 6px);
    }

    .chip-arrow {
        color: var(--text-tertiary);
        font-size: 0.75rem;
    }

    /* Add Step Bar */
    .add-step-bar {
        display: flex;
        gap: 0.5rem;
        align-items: center;
    }

    /* Step Cards List */
    .steps-list {
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
    }

    .step-card {
        background: var(--glass-bg-surface);
        border: 1px solid var(--glass-border-subtle);
        border-radius: var(--radius-md, 10px);
        padding: 1rem;
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
    }

    .step-card-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    .step-card-title {
        display: flex;
        align-items: center;
        gap: 0.5rem;
    }

    .step-number {
        font-size: 0.8rem;
        font-family: var(--font-mono, monospace);
        color: var(--text-secondary);
        background: var(--glass-bg-input);
        border: 1px solid var(--glass-border-subtle);
        padding: 0.15rem 0.4rem;
        border-radius: var(--radius-xs, 6px);
    }

    .step-icon {
        font-size: 1.1rem;
    }

    .step-name {
        font-weight: 600;
        color: var(--text-primary);
        font-size: 0.95rem;
    }

    .step-controls {
        display: flex;
        gap: 0.35rem;
    }

    .icon-btn {
        background: var(--glass-bg-input);
        border: 1px solid var(--glass-border-subtle);
        color: var(--text-primary);
        padding: 0.3rem 0.6rem;
        border-radius: var(--radius-xs, 6px);
        cursor: pointer;
        font-size: 0.8rem;
        transition: background var(--dur-fast, 120ms) ease;
    }

    .icon-btn:hover:not(:disabled) {
        background: var(--glass-bg-hover);
    }

    .icon-btn:disabled {
        opacity: 0.4;
        cursor: not-allowed;
    }

    .btn-delete:hover:not(:disabled) {
        background: var(--semantic-danger);
        color: var(--text-on-accent);
    }

    .step-config-body {
        border-top: 1px solid var(--glass-border-subtle);
        padding-top: 0.6rem;
        font-size: 0.85rem;
    }

    .step-hint {
        color: var(--text-secondary);
        font-style: italic;
    }

    .field-label {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        color: var(--text-secondary);
    }

    .field-row {
        display: flex;
        gap: 1.5rem;
        align-items: center;
        flex-wrap: wrap;
    }

    .checkbox-group {
        display: flex;
        gap: 1rem;
    }

    .checkbox-label {
        display: flex;
        align-items: center;
        gap: 0.35rem;
        color: var(--text-primary);
        cursor: pointer;
    }

    /* Execution Inputs */
    .execution-inputs {
        display: flex;
        flex-direction: column;
        gap: 1rem;
    }

    .input-group {
        display: flex;
        flex-direction: column;
        gap: 0.35rem;
    }

    .input-group label {
        font-size: 0.85rem;
        font-weight: 600;
        color: var(--text-secondary);
    }

    .text-input {
        background: var(--glass-bg-input);
        border: 1px solid var(--glass-border-subtle);
        border-radius: var(--radius-sm, 8px);
        padding: 0.6rem 0.85rem;
        color: var(--text-primary);
        font-size: 0.9rem;
    }

    .text-input.small {
        width: 80px;
    }

    .select-input {
        background: var(--glass-bg-input);
        border: 1px solid var(--glass-border-subtle);
        border-radius: var(--radius-sm, 8px);
        padding: 0.5rem 0.75rem;
        color: var(--text-primary);
        font-size: 0.85rem;
    }

    .execution-footer {
        display: flex;
        justify-content: space-between;
        align-items: center;
        flex-wrap: wrap;
        gap: 1rem;
        border-top: 1px solid var(--glass-border-subtle);
        padding-top: 1rem;
    }

    .fail-closed-notice {
        font-size: 0.8rem;
        color: var(--text-secondary);
        max-width: 600px;
        line-height: 1.4;
    }

    .btn {
        padding: 0.5rem 1rem;
        border-radius: var(--radius-sm, 8px);
        font-size: 0.9rem;
        font-weight: 600;
        cursor: pointer;
        border: 1px solid transparent;
        transition: all var(--dur-fast, 120ms) ease;
        display: inline-flex;
        align-items: center;
        gap: 0.4rem;
    }

    .btn:disabled {
        opacity: 0.45;
        cursor: not-allowed;
    }

    .btn-primary {
        background: var(--accent-primary);
        color: var(--text-on-accent);
    }
    .btn-primary:hover:not(:disabled) {
        opacity: 0.9;
    }

    .btn-secondary {
        background: var(--glass-bg-input);
        color: var(--text-primary);
        border: 1px solid var(--glass-border-subtle);
    }
    .btn-secondary:hover:not(:disabled) {
        background: var(--glass-bg-hover);
    }

    .btn-large {
        padding: 0.75rem 1.5rem;
        font-size: 1rem;
    }

    .empty-steps {
        padding: 2rem;
        text-align: center;
        color: var(--text-secondary);
        background: var(--glass-bg-surface);
        border-radius: var(--radius-md, 10px);
        border: 1px dashed var(--glass-border-subtle);
    }
</style>

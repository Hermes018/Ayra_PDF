<script lang="ts">
    import StudioIcon from '$lib/components/studio/StudioIcon.svelte';
    import { onMount } from 'svelte';
    import { IpcClient } from '$lib/ipc';
    import { friendlyJobError } from '$lib/core/jobError';
    import ErrorState from '$lib/components/ui/ErrorState.svelte';
    import Button from '$lib/components/ui/Button.svelte';
    import Input from '$lib/components/ui/Input.svelte';
    import Select from '$lib/components/ui/Select.svelte';
    import Toggle from '$lib/components/ui/Toggle.svelte';
    import { parseNumberOr } from '$lib/core/parseNumber';
    import { appState } from '$lib/state.svelte';
    import { siblingOutputPath, resolveFreeOutput } from '$lib/views/studio/outputNames';
    import For from '$lib/components/For.svelte';
    import type { SummaryLength } from '$lib/bindings/SummaryLength';
    import type { JobRequest } from '$lib/bindings/JobRequest';
    import type { AppSettings } from '$lib/bindings/AppSettings';

    let { onNavigate = (_view: string) => {} } = $props<{
        onNavigate?: (view: string) => void;
    }>();

    // Svelte 5 runes state
    let activeMode = $state<'extractive' | 'llm' | 'translate'>('extractive');
    let selectedLength = $state<SummaryLength>('Medium');
    let chunkWordLimit = $state<number>(400);
    let temperature = $state<number>(0.2);
    let sourceLang = $state<string>('Auto');
    let targetLang = $state<string>('es');
    let preserveLayoutBlocks = $state<boolean>(true);
    let inputPath = $state<string>('');
    let outputPath = $state<string>('');
    let isSubmitting = $state(false);
    let submissionStatus = $state<string | null>(null);
    let errorMessage = $state<string | null>(null);
    let appSettings = $state<AppSettings | null>(null);

    let isLlmReady = $derived(
        Boolean(appSettings?.ai_local_enabled && appSettings?.ai_license_acknowledged && appSettings?.local_ai_model_path)
    );

    onMount(async () => {
        try {
            appSettings = await IpcClient.getSettings();
        } catch (e) {
            console.warn('Failed to load settings in AiSummarizeView:', e);
        }
    });

    // Synchronize inputPath with appState if available
    $effect(() => {
        if (!inputPath && appState.currentFile) {
            inputPath = appState.currentFile;
            outputPath = siblingOutputPath(inputPath, '_summary.md');
        }
    });

    // Auto-update output path when input path changes if output is blank or follows default convention
    function handleInputChange(newInput: string) {
        inputPath = newInput;
        if (!outputPath || outputPath.endsWith('_summary.md') || outputPath.includes('_translated_')) {
            if (activeMode === 'translate') {
                outputPath = siblingOutputPath(newInput, `_translated_${targetLang}.pdf`);
            } else {
                outputPath = siblingOutputPath(newInput, '_summary.md');
            }
        }
    }

    function selectMode(mode: 'extractive' | 'llm' | 'translate') {
        activeMode = mode;
        if (inputPath && (!outputPath || outputPath.endsWith('_summary.md') || outputPath.includes('_translated_'))) {
            if (mode === 'translate') {
                outputPath = siblingOutputPath(inputPath, `_translated_${targetLang}.pdf`);
            } else {
                outputPath = siblingOutputPath(inputPath, '_summary.md');
            }
        }
    }

    const LANG_OPTIONS = [
        { value: 'es', label: 'Spanish (es)' },
        { value: 'en', label: 'English (en)' },
        { value: 'fr', label: 'French (fr)' },
        { value: 'de', label: 'German (de)' },
        { value: 'bn', label: 'Bengali (bn)' },
    ];
    const SOURCE_LANG_OPTIONS = [{ value: 'Auto', label: 'Auto-detect' }, ...LANG_OPTIONS];

    function setTargetLang(v: string) {
        targetLang = v;
        if (inputPath && (outputPath.includes('_translated_') || !outputPath)) {
            outputPath = siblingOutputPath(inputPath, `_translated_${targetLang}.pdf`);
        }
    }

    const lengthOptions: { id: SummaryLength; label: string; desc: string }[] = [
        {
            id: 'Short',
            label: 'Short',
            desc: '3–5 key sentences. Quick executive gist and bullet highlights.',
        },
        {
            id: 'Medium',
            label: 'Medium',
            desc: '8–10 sentences. Balanced coverage of main points and context.',
        },
        {
            id: 'Detailed',
            label: 'Detailed',
            desc: '15–20 sentences. In-depth extraction covering all sections.',
        },
    ];

    // A failed submit: what happened and what to do, with a Retry that picks the next free name.
    let genFailure = $state<{ what: string; next: string; retry: () => void } | null>(null);

    async function handleGenerateSummary() {
        if (isSubmitting) return; // set before any await: a double click starts one job
        errorMessage = null;
        genFailure = null;
        submissionStatus = null;

        if (!inputPath.trim()) {
            errorMessage = 'Please provide an input PDF file path.';
            return;
        }

        if (!outputPath.trim()) {
            errorMessage = 'Please provide an output Markdown/Text file path.';
            return;
        }

        isSubmitting = true;
        try {
            // Never the input, never an existing file: next free name (BUG-051).
            const outName = await resolveFreeOutput(outputPath.trim(), inputPath.trim());
            if (outName !== outputPath.trim()) outputPath = outName;
            let req: JobRequest;
            if (activeMode === 'translate') {
                if (!appSettings?.local_ai_model_path) {
                    throw new Error('Local AI model path is not configured.');
                }
                const modelPath = appSettings.local_ai_model_path;
                let fmt = 'GGUF';
                if (modelPath.endsWith('.onnx')) {
                    fmt = 'ONNX';
                } else if (modelPath.endsWith('.safetensors')) {
                    fmt = 'SafeTensors';
                }

                req = {
                    op: 'TranslatePdf',
                    input_path: inputPath.trim(),
                    output_path: outName,
                    options: {
                        manifest: {
                            model_path: modelPath,
                            sha256_hex: appSettings.ai_model_sha256 || '',
                            format: fmt,
                            license_spdx: appSettings.ai_model_license || 'Custom',
                            license_acknowledged: Boolean(appSettings.ai_license_acknowledged),
                        },
                        ai_local_enabled: Boolean(appSettings.ai_local_enabled),
                        source_lang: sourceLang,
                        target_lang: targetLang,
                        preserve_layout_blocks: preserveLayoutBlocks,
                    },
                };
            } else if (activeMode === 'llm') {
                if (!appSettings?.local_ai_model_path) {
                    throw new Error('Local AI model path is not configured.');
                }
                const modelPath = appSettings.local_ai_model_path;
                let fmt = 'GGUF';
                if (modelPath.endsWith('.onnx')) {
                    fmt = 'ONNX';
                } else if (modelPath.endsWith('.safetensors')) {
                    fmt = 'SafeTensors';
                }

                req = {
                    op: 'SummarizeLlm',
                    input_path: inputPath.trim(),
                    output_path: outName,
                    options: {
                        manifest: {
                            model_path: modelPath,
                            sha256_hex: appSettings.ai_model_sha256 || '',
                            format: fmt,
                            license_spdx: appSettings.ai_model_license || 'Custom',
                            license_acknowledged: Boolean(appSettings.ai_license_acknowledged),
                        },
                        ai_local_enabled: Boolean(appSettings.ai_local_enabled),
                        length: selectedLength,
                        chunk_word_limit: Number(chunkWordLimit) || 400,
                        max_tokens: 512,
                        temperature: Number(temperature) || 0.2,
                    },
                };
            } else {
                req = {
                    op: 'SummarizeExtractive',
                    input_path: inputPath.trim(),
                    output_path: outName,
                    length: selectedLength,
                };
            }

            const jobId = await IpcClient.submitJob(req);
            const actionName = activeMode === 'translate' ? 'Translation' : 'Summary';
            submissionStatus = `${actionName} job queued successfully (ID: ${jobId}). Check Batch Queue for progress.`;
        } catch (e: any) {
            console.error('Failed to submit summarization job:', e);
            const msg = e?.message || String(e) || '';
            if (msg.startsWith('Local AI model path')) {
                errorMessage = msg; // a plain configuration message, already in words
            } else {
                const f = friendlyJobError(msg, {
                    what: "Couldn't start the job.",
                    next: 'Check the file paths and settings, then try again.',
                });
                genFailure = { what: f.what, next: f.next, retry: () => void handleGenerateSummary() };
            }
        } finally {
            isSubmitting = false;
        }
    }
</script>

<div class="summarize-view">
    <header class="view-header">
        <div class="header-content">
            <h1>AI document summarizer</h1>
            <p class="subtitle">Extract key takeaways, topics, and structured summaries on this computer, with no cloud service.</p>
        </div>
        <div class="privacy-badge">Works offline</div>
    </header>

    <!-- Mode cards -->
    <div class="modes-grid">
        <div class="mode-card" class:active={activeMode === 'extractive'}>
            <Button
                variant="secondary"
                block
                class="mode-select"
                pressed={activeMode === 'extractive'}
                data-testid="mode-extractive"
                onclick={() => selectMode('extractive')}
            >
                <span class="mode-body">
                    <span class="mode-header">
                        <span class="mode-icon" aria-hidden="true"><StudioIcon name="bolt" size={24} /></span>
                        <span class="mode-badge">Default mode</span>
                    </span>
                    <span class="mode-title">Basic extractive</span>
                    <span class="mode-desc">Fast, no model needed. Picks the most informative sentences using TF-IDF centrality and position weighting.</span>
                    <span class="mode-features">
                        <span>No model download required</span>
                        <span>Runs on the CPU, no GPU needed</span>
                        <span>Low memory use</span>
                    </span>
                </span>
            </Button>
        </div>

        <div class="mode-card" class:gated-mode={!isLlmReady} class:active={activeMode === 'llm'}>
            <Button
                variant="secondary"
                block
                class="mode-select"
                pressed={activeMode === 'llm'}
                disabled={!isLlmReady}
                disabledReason="Set up a local model in Settings to enable this mode"
                data-testid="mode-local-llm"
                onclick={() => selectMode('llm')}
            >
                <span class="mode-body">
                    <span class="mode-header">
                        <span class="mode-icon" aria-hidden="true"><StudioIcon name="chip" size={24} /></span>
                        {#if isLlmReady}
                            <span class="mode-badge" data-testid="llm-status-badge">Model ready</span>
                        {:else}
                            <span class="mode-badge" data-testid="llm-status-badge">Gated opt-in</span>
                        {/if}
                    </span>
                    <span class="mode-title">Local LLM mode (map-reduce)</span>
                    <span class="mode-desc">Abstractive summaries from a local quantized GGUF model via llama.cpp or Candle. Runs on your hardware; nothing is sent anywhere.</span>
                </span>
            </Button>

            {#if isLlmReady}
                <div class="llm-ready-info" data-testid="llm-ready-info">
                    <div class="info-row">
                        <span class="info-label">Model:</span>
                        <span class="info-val" data-testid="llm-model-path">{appSettings?.local_ai_model_path}</span>
                    </div>
                    {#if appSettings?.ai_model_sha256}
                        <div class="info-row">
                            <span class="info-label">SHA-256:</span>
                            <code class="info-hash" data-testid="llm-model-hash">{appSettings.ai_model_sha256.slice(0, 16)}...</code>
                        </div>
                    {/if}
                    <div class="info-row">
                        <span><StudioIcon name="check" size={14} /> Licence acknowledged ({appSettings?.ai_model_license || 'SPDX'})</span>
                    </div>
                </div>
            {:else}
                <div class="gate-notice">
                    <span>Requires the <code>ai-local</code> feature, a local model import with its SHA-256 checksum, and licence acknowledgement.</span>
                    <Button variant="plain" size="sm" data-testid="settings-link" onclick={() => onNavigate('Settings')}>Configure in Settings →</Button>
                </div>
            {/if}
        </div>

        <div class="mode-card" class:gated-mode={!isLlmReady} class:active={activeMode === 'translate'}>
            <Button
                variant="secondary"
                block
                class="mode-select"
                pressed={activeMode === 'translate'}
                disabled={!isLlmReady}
                disabledReason="Set up a local model in Settings to enable this mode"
                data-testid="mode-translate"
                onclick={() => selectMode('translate')}
            >
                <span class="mode-body">
                    <span class="mode-header">
                        <span class="mode-icon" aria-hidden="true"><StudioIcon name="globe" size={24} /></span>
                        {#if isLlmReady}
                            <span class="mode-badge" data-testid="translate-status-badge">Experimental</span>
                        {:else}
                            <span class="mode-badge" data-testid="translate-status-badge">Gated opt-in</span>
                        {/if}
                    </span>
                    <span class="mode-title">Translate PDF (experimental)</span>
                    <span class="mode-desc">Block-level translation and re-typesetting. Writes a new PDF and a Markdown companion report.</span>
                </span>
            </Button>

            {#if isLlmReady}
                <div class="llm-ready-info" data-testid="translate-ready-info">
                    <div class="info-row">
                        <span class="info-label">Model:</span>
                        <span class="info-val">{appSettings?.local_ai_model_path}</span>
                    </div>
                    <div class="info-row">
                        <span><StudioIcon name="check" size={14} /> Block-level re-typesetting and report</span>
                    </div>
                </div>
            {:else}
                <div class="gate-notice">
                    <span>Requires the <code>ai-local</code> feature, a local model import, and licence acknowledgement.</span>
                    <Button variant="plain" size="sm" data-testid="translate-settings-link" onclick={() => onNavigate('Settings')}>Configure in Settings →</Button>
                </div>
            {/if}
        </div>
    </div>

    <!-- Configuration -->
    <section class="config-panel" data-testid="config-panel">
        {#if activeMode === 'translate'}
            <div class="panel-section" data-testid="translate-config-section">
                <div class="section-header-row">
                    <h2>1. Translation configuration</h2>
                    <span class="badge-experimental">Experimental (ADR-006)</span>
                </div>
                <div class="info-callout">
                    <span><StudioIcon name="warning" size={14} /> Block-level re-typesetting replaces document paragraphs and writes a companion report (<code>.report.md</code>) listing block mappings and layout warnings.</span>
                </div>
                <div class="form-row-2">
                    <Select label="Source language" bind:value={sourceLang} options={SOURCE_LANG_OPTIONS} data-testid="source-lang-select" />
                    <Select label="Target language" bind:value={() => targetLang, setTargetLang} options={LANG_OPTIONS} data-testid="target-lang-select" />
                </div>
                <Toggle label="Preserve block-level layout (extracts per-page text blocks and re-typesets paragraphs into a valid PDF)" bind:checked={preserveLayoutBlocks} data-testid="preserve-layout-checkbox" />
            </div>
        {:else}
            <div class="panel-section">
                <h2>1. Summary length</h2>
                <div class="length-selector">
                    <For items={lengthOptions}>
                        {#snippet children(opt)}
                            <Button
                                variant="secondary"
                                block
                                class="length-btn"
                                pressed={selectedLength === opt.id}
                                data-testid={`length-${opt.id.toLowerCase()}`}
                                onclick={() => { selectedLength = opt.id; }}
                            >
                                <span class="length-body">
                                    <span class="length-label">{opt.label}</span>
                                    <span class="length-desc">{opt.desc}</span>
                                </span>
                            </Button>
                        {/snippet}
                    </For>
                </div>
            </div>
        {/if}

        {#if activeMode === 'llm'}
            <div class="panel-section" data-testid="map-reduce-section">
                <h2>2. Map-reduce parameters</h2>
                <div class="form-row-2">
                    <div class="range-group">
                        <Input
                            type="range"
                            min={100}
                            max={2000}
                            step={50}
                            label="Chunk word limit: {chunkWordLimit} words (splits the input into bounded chunks)"
                            data-testid="chunk-words-slider"
                            bind:value={() => String(chunkWordLimit), (v) => (chunkWordLimit = parseNumberOr(v, 400))}
                        />
                        <Input
                            type="number"
                            min={50}
                            max={5000}
                            step={50}
                            label="Chunk word limit (number)"
                            hideLabel
                            data-testid="chunk-words-input"
                            bind:value={() => String(chunkWordLimit), (v) => (chunkWordLimit = parseNumberOr(v, 400))}
                        />
                    </div>
                    <div class="range-group">
                        <Input
                            type="range"
                            min={0}
                            max={1}
                            step={0.05}
                            label="Temperature: {temperature} (0.3 or lower is more deterministic)"
                            data-testid="temperature-slider"
                            bind:value={() => String(temperature), (v) => (temperature = parseNumberOr(v, 0.2))}
                        />
                        <Input
                            type="number"
                            min={0}
                            max={1}
                            step={0.05}
                            label="Temperature (number)"
                            hideLabel
                            data-testid="temperature-input"
                            bind:value={() => String(temperature), (v) => (temperature = parseNumberOr(v, 0.2))}
                        />
                    </div>
                </div>
            </div>
        {/if}

        <div class="panel-section">
            <h2>{activeMode === 'llm' ? '3. File selection' : '2. File selection'}</h2>
            <div class="form-row-2">
                <Input
                    label="Input PDF document"
                    placeholder="/path/to/document.pdf"
                    data-testid="input-path"
                    bind:value={() => inputPath, handleInputChange}
                />
                <Input
                    label={activeMode === 'translate' ? 'Output PDF destination (.pdf)' : 'Output summary destination (.md or .txt)'}
                    placeholder={activeMode === 'translate' ? '/path/to/document_translated.pdf' : '/path/to/document_summary.md'}
                    data-testid="output-path"
                    bind:value={outputPath}
                />
            </div>
        </div>

        {#if genFailure}
            <ErrorState what={genFailure.what} next={genFailure.next} retry={genFailure.retry} data-testid="gen-failure" />
        {/if}

        {#if errorMessage}
            <ErrorState what={errorMessage} next="Fix the entry above and try again." data-testid="error-message" />
        {/if}

        {#if submissionStatus}
            <div class="status-box" role="status" aria-live="polite" data-testid="submission-status">
                <span>{submissionStatus}</span>
            </div>
        {/if}

        <div class="panel-actions">
            <Button variant="primary" size="lg" busy={isSubmitting} data-testid="generate-summary-btn" onclick={handleGenerateSummary}>
                {#if isSubmitting}
                    Queueing job…
                {:else if activeMode === 'translate'}
                    Translate document (experimental)
                {:else if activeMode === 'llm'}
                    Generate map-reduce summary
                {:else}
                    Generate extractive summary
                {/if}
            </Button>
        </div>
    </section>
</div>

<style>
    .summarize-view {
        display: flex;
        flex-direction: column;
        gap: var(--space-4);
        padding: var(--space-4);
        color: var(--text-primary);
    }

    .view-header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: var(--space-4);
        padding: var(--space-4) var(--space-6);
        background: var(--glass-bg-surface);
        border: 1px solid var(--glass-border-subtle);
        border-radius: var(--radius-lg);
    }

    .view-header h1 {
        margin: 0;
        font-size: var(--font-size-title1);
        color: var(--accent-primary);
    }

    .subtitle {
        margin: var(--space-1) 0 0;
        color: var(--text-secondary);
        font-size: var(--font-size-footnote);
    }

    .privacy-badge,
    .mode-badge,
    .badge-experimental {
        font-size: var(--font-size-caption);
        padding: 0 var(--space-2);
        border-radius: var(--radius-pill);
        background: var(--glass-bg-active);
        white-space: nowrap;
    }

    .modes-grid {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
        gap: var(--space-4);
    }

    .mode-card {
        display: flex;
        flex-direction: column;
        gap: var(--space-2);
        min-width: 0;
    }

    .mode-card :global(.mode-select .lp-btn),
    .length-selector :global(.length-btn .lp-btn) {
        height: auto;
        min-height: 0;
        justify-content: flex-start;
        text-align: left;
        padding: var(--space-3);
        font-weight: 400;
    }

    .mode-body,
    .length-body {
        display: flex;
        flex-direction: column;
        gap: var(--space-2);
        white-space: normal;
        align-items: flex-start;
    }

    .mode-header {
        display: flex;
        align-items: center;
        gap: var(--space-2);
    }

    .mode-title,
    .length-label {
        font-weight: 600;
        font-size: var(--font-size-body);
    }

    .mode-desc,
    .length-desc {
        color: var(--text-secondary);
        font-size: var(--font-size-footnote);
        line-height: 1.4;
    }

    .mode-features {
        display: flex;
        flex-direction: column;
        gap: var(--space-1);
        font-size: var(--font-size-caption);
        color: var(--text-secondary);
    }

    .llm-ready-info,
    .gate-notice {
        padding: var(--space-3);
        border-radius: var(--radius-md);
        background: var(--glass-bg-input);
        font-size: var(--font-size-footnote);
        display: flex;
        flex-direction: column;
        gap: var(--space-2);
    }

    .info-row {
        display: flex;
        gap: var(--space-2);
        flex-wrap: wrap;
    }

    .info-label {
        color: var(--text-secondary);
    }

    .info-val {
        word-break: break-all;
    }

    .config-panel {
        display: flex;
        flex-direction: column;
        gap: var(--space-4);
    }

    .panel-section {
        background: var(--glass-bg-surface);
        border: 1px solid var(--glass-border-subtle);
        border-radius: var(--radius-lg);
        padding: var(--space-4);
        display: flex;
        flex-direction: column;
        gap: var(--space-3);
    }

    .panel-section h2 {
        margin: 0;
        font-size: var(--font-size-title3);
    }

    .section-header-row {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: var(--space-3);
    }

    .info-callout {
        padding: var(--space-3);
        border-radius: var(--radius-md);
        background: var(--glass-bg-input);
        font-size: var(--font-size-footnote);
    }

    .form-row-2 {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: var(--space-4);
    }

    .length-selector {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
        gap: var(--space-3);
    }

    .range-group {
        display: flex;
        flex-direction: column;
        gap: var(--space-2);
    }

    .status-box {
        padding: var(--space-3) var(--space-4);
        border-radius: var(--radius-md);
        background: var(--glass-bg-input);
        border: 1px solid var(--semantic-success);
        font-size: var(--font-size-footnote);
    }

    .panel-actions {
        display: flex;
        justify-content: flex-end;
    }
</style>

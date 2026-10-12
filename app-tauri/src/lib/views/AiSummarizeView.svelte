<script lang="ts">
    import { onMount } from 'svelte';
    import { IpcClient } from '$lib/ipc';
    import { appState } from '$lib/state.svelte';
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
            outputPath = inputPath.replace(/\.pdf$/i, '_summary.md');
        }
    });

    // Auto-update output path when input path changes if output is blank or follows default convention
    function handleInputChange(newInput: string) {
        inputPath = newInput;
        if (!outputPath || outputPath.endsWith('_summary.md') || outputPath.includes('_translated_')) {
            if (activeMode === 'translate') {
                outputPath = newInput.replace(/\.pdf$/i, `_translated_${targetLang}.pdf`);
            } else {
                outputPath = newInput.replace(/\.pdf$/i, '_summary.md');
            }
        }
    }

    function selectMode(mode: 'extractive' | 'llm' | 'translate') {
        activeMode = mode;
        if (inputPath && (!outputPath || outputPath.endsWith('_summary.md') || outputPath.includes('_translated_'))) {
            if (mode === 'translate') {
                outputPath = inputPath.replace(/\.pdf$/i, `_translated_${targetLang}.pdf`);
            } else {
                outputPath = inputPath.replace(/\.pdf$/i, '_summary.md');
            }
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

    async function handleGenerateSummary() {
        errorMessage = null;
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
                    output_path: outputPath.trim(),
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
                    output_path: outputPath.trim(),
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
                    output_path: outputPath.trim(),
                    length: selectedLength,
                };
            }

            const jobId = await IpcClient.submitJob(req);
            const actionName = activeMode === 'translate' ? 'Translation' : 'Summary';
            submissionStatus = `${actionName} job queued successfully (ID: ${jobId}). Check Batch Queue for progress.`;
        } catch (e: any) {
            errorMessage = e?.message || String(e) || 'Failed to submit summarization job.';
        } finally {
            isSubmitting = false;
        }
    }
</script>

<div class="summarize-view">
    <header class="view-header">
        <div class="header-content">
            <h1>AI Document Summarizer</h1>
            <p class="subtitle">Extract executive takeaways, key topics, and structured summaries offline with zero cloud dependency.</p>
        </div>
        <div class="privacy-badge">
            <span class="badge-dot"></span>
            100% Offline & Private
        </div>
    </header>

    <!-- Mode Selector Cards -->
    <div class="modes-grid">
        <button
            type="button"
            class="mode-card"
            class:active={activeMode === 'extractive'}
            onclick={() => selectMode('extractive')}
            data-testid="mode-extractive"
        >
            <div class="mode-header">
                <span class="mode-icon">⚡</span>
                <span class="mode-badge active-badge">Default Mode</span>
            </div>
            <h3>Basic Extractive</h3>
            <p class="mode-desc">Instant, zero-model summarization. Uses deterministic TF-IDF centrality and position weighting to select the most informative sentences natively.</p>
            <ul class="mode-features">
                <li>Zero model download required</li>
                <li>Instant execution (&lt;100ms)</li>
                <li>Zero RAM/VRAM memory spikes</li>
            </ul>
        </button>

        <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
        <div
            role={isLlmReady ? 'button' : undefined}
            tabindex={isLlmReady ? 0 : undefined}
            class="mode-card"
            class:gated-mode={!isLlmReady}
            class:active={activeMode === 'llm'}
            onclick={() => { if (isLlmReady) selectMode('llm'); }}
            onkeydown={(e) => { if (isLlmReady && (e.key === 'Enter' || e.key === ' ')) selectMode('llm'); }}
            data-testid="mode-local-llm"
        >
            <div class="mode-header">
                <span class="mode-icon">🧠</span>
                {#if isLlmReady}
                    <span class="mode-badge ready-badge" data-testid="llm-status-badge">Model Ready</span>
                {:else}
                    <span class="mode-badge gated-badge" data-testid="llm-status-badge">Gated Opt-In</span>
                {/if}
            </div>
            <h3>Local LLM Mode (Map-Reduce)</h3>
            <p class="mode-desc">Abstractive neural summarization with local quantized GGUF models via llama.cpp or Candle. Completely private and runs locally on your hardware.</p>

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
                    <div class="info-row status-success">
                        <span>✓ Licence Acknowledged ({appSettings?.ai_model_license || 'SPDX'})</span>
                    </div>
                </div>
            {:else}
                <div class="gate-notice">
                    <span>Requires <code>ai-local</code> feature, local model import, SHA-256 checksum, and licence acknowledgement.</span>
                    <button
                        type="button"
                        class="settings-link-btn"
                        onclick={(e) => { e.stopPropagation(); onNavigate('Settings'); }}
                        data-testid="settings-link"
                    >
                        Configure in Settings &rarr;
                    </button>
                </div>
            {/if}
        </div>

        <!-- Translate PDF (Experimental) Mode Card -->
        <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
        <div
            role={isLlmReady ? 'button' : undefined}
            tabindex={isLlmReady ? 0 : undefined}
            class="mode-card"
            class:gated-mode={!isLlmReady}
            class:active={activeMode === 'translate'}
            onclick={() => { if (isLlmReady) selectMode('translate'); }}
            onkeydown={(e) => { if (isLlmReady && (e.key === 'Enter' || e.key === ' ')) selectMode('translate'); }}
            data-testid="mode-translate"
        >
            <div class="mode-header">
                <span class="mode-icon">🌐</span>
                {#if isLlmReady}
                    <span class="mode-badge experimental-badge" data-testid="translate-status-badge">Experimental</span>
                {:else}
                    <span class="mode-badge gated-badge" data-testid="translate-status-badge">Gated Opt-In</span>
                {/if}
            </div>
            <h3>Translate PDF (Experimental)</h3>
            <p class="mode-desc">Block-level neural translation and re-typesetting. Generates a new PDF alongside an exhaustive Markdown companion report.</p>

            {#if isLlmReady}
                <div class="llm-ready-info" data-testid="translate-ready-info">
                    <div class="info-row">
                        <span class="info-label">Model:</span>
                        <span class="info-val">{appSettings?.local_ai_model_path}</span>
                    </div>
                    <div class="info-row status-success">
                        <span>✓ Block-level re-typesetting &amp; report</span>
                    </div>
                </div>
            {:else}
                <div class="gate-notice">
                    <span>Requires <code>ai-local</code> feature, local model import, and licence acknowledgement.</span>
                    <button
                        type="button"
                        class="settings-link-btn"
                        onclick={(e) => { e.stopPropagation(); onNavigate('Settings'); }}
                        data-testid="translate-settings-link"
                    >
                        Configure in Settings &rarr;
                    </button>
                </div>
            {/if}
        </div>
    </div>

    <!-- Configuration Panel -->
    <section class="config-panel" data-testid="config-panel">
        {#if activeMode === 'translate'}
            <div class="panel-section" data-testid="translate-config-section">
                <div class="section-header-row">
                    <h2>1. Translation Configuration</h2>
                    <span class="badge-experimental">Experimental (ADR-006)</span>
                </div>
                <div class="info-callout">
                    <span>⚠️ Block-level re-typesetting replaces document paragraphs and generates an atomic companion report (<code>.report.md</code>) recording block mappings and layout warnings.</span>
                </div>
                <div class="form-grid">
                    <div class="form-row-2">
                        <div class="form-group">
                            <label for="source-lang-select">Source Language</label>
                            <select
                                id="source-lang-select"
                                bind:value={sourceLang}
                                data-testid="source-lang-select"
                            >
                                <option value="Auto">Auto-Detect</option>
                                <option value="en">English (en)</option>
                                <option value="es">Spanish (es)</option>
                                <option value="fr">French (fr)</option>
                                <option value="de">German (de)</option>
                                <option value="bn">Bengali (bn)</option>
                            </select>
                        </div>

                        <div class="form-group">
                            <label for="target-lang-select">Target Language</label>
                            <select
                                id="target-lang-select"
                                bind:value={targetLang}
                                onchange={() => {
                                    if (inputPath && (outputPath.includes('_translated_') || !outputPath)) {
                                        outputPath = inputPath.replace(/\.pdf$/i, `_translated_${targetLang}.pdf`);
                                    }
                                }}
                                data-testid="target-lang-select"
                            >
                                <option value="es">Spanish (es)</option>
                                <option value="en">English (en)</option>
                                <option value="fr">French (fr)</option>
                                <option value="de">German (de)</option>
                                <option value="bn">Bengali (bn)</option>
                            </select>
                        </div>
                    </div>

                    <div class="form-group">
                        <label class="toggle-checkbox-label">
                            <input
                                type="checkbox"
                                bind:checked={preserveLayoutBlocks}
                                data-testid="preserve-layout-checkbox"
                            />
                            <span>
                                <strong>Preserve Block-Level Layout</strong>
                                <span class="help-text">(Extracts per-page text blocks and re-typesets paragraphs into valid PDF)</span>
                            </span>
                        </label>
                    </div>
                </div>
            </div>
        {:else}
            <div class="panel-section">
                <h2>1. Summary Length</h2>
                <div class="length-selector">
                    {#each lengthOptions as opt}
                        <button
                            type="button"
                            class="length-btn"
                            class:selected={selectedLength === opt.id}
                            onclick={() => { selectedLength = opt.id; }}
                            data-testid={`length-${opt.id.toLowerCase()}`}
                        >
                            <span class="length-label">{opt.label}</span>
                            <span class="length-desc">{opt.desc}</span>
                        </button>
                    {/each}
                </div>
            </div>
        {/if}

        {#if activeMode === 'llm'}
            <div class="panel-section" data-testid="map-reduce-section">
                <h2>2. Map-Reduce Parameters</h2>
                <div class="form-grid">
                    <div class="form-group">
                        <label for="chunk-words-input">
                            Chunk Word Limit: <strong>{chunkWordLimit} words</strong>
                            <span class="help-text">(Partitions input into bounded semantic chunks)</span>
                        </label>
                        <div class="range-with-number">
                            <input
                                id="chunk-words-range"
                                type="range"
                                min="100"
                                max="2000"
                                step="50"
                                bind:value={chunkWordLimit}
                                data-testid="chunk-words-slider"
                            />
                            <input
                                id="chunk-words-input"
                                type="number"
                                min="50"
                                max="5000"
                                step="50"
                                bind:value={chunkWordLimit}
                                data-testid="chunk-words-input"
                            />
                        </div>
                    </div>

                    <div class="form-group">
                        <label for="temperature-input">
                            Temperature: <strong>{temperature}</strong>
                            <span class="help-text">(Deterministic inference ≤ 0.3 recommended)</span>
                        </label>
                        <div class="range-with-number">
                            <input
                                id="temperature-range"
                                type="range"
                                min="0.0"
                                max="1.0"
                                step="0.05"
                                bind:value={temperature}
                                data-testid="temperature-slider"
                            />
                            <input
                                id="temperature-input"
                                type="number"
                                min="0.0"
                                max="1.0"
                                step="0.05"
                                bind:value={temperature}
                                data-testid="temperature-input"
                            />
                        </div>
                    </div>
                </div>
            </div>
        {/if}

        <div class="panel-section">
            <h2>{activeMode === 'llm' ? '3. File Selection' : '2. File Selection'}</h2>
            <div class="form-grid">
                <div class="form-group">
                    <label for="input-pdf-path">Input PDF Document</label>
                    <div class="input-with-button">
                        <input
                            id="input-pdf-path"
                            type="text"
                            placeholder="/path/to/document.pdf"
                            value={inputPath}
                            oninput={(e) => handleInputChange(e.currentTarget.value)}
                            data-testid="input-path"
                        />
                    </div>
                </div>

                <div class="form-group">
                    <label for="output-md-path">{activeMode === 'translate' ? 'Output PDF Destination (.pdf)' : 'Output Summary Destination (.md or .txt)'}</label>
                    <div class="input-with-button">
                        <input
                            id="output-md-path"
                            type="text"
                            placeholder={activeMode === 'translate' ? '/path/to/document_translated.pdf' : '/path/to/document_summary.md'}
                            bind:value={outputPath}
                            data-testid="output-path"
                        />
                    </div>
                </div>
            </div>
        </div>

        {#if errorMessage}
            <div class="status-box error" role="alert" data-testid="error-message">
                <span class="status-icon">⚠️</span>
                <span>{errorMessage}</span>
            </div>
        {/if}

        {#if submissionStatus}
            <div class="status-box success" role="status" data-testid="submission-status">
                <span class="status-icon">✅</span>
                <span>{submissionStatus}</span>
            </div>
        {/if}

        <div class="panel-actions">
            <button
                type="button"
                id="btn-generate-summary"
                class="primary-btn"
                disabled={isSubmitting}
                onclick={handleGenerateSummary}
                data-testid="generate-summary-btn"
            >
                {#if isSubmitting}
                    <span class="spinner"></span>
                    Queueing Job...
                {:else if activeMode === 'translate'}
                    <span>🌐 Translate Document (Experimental)</span>
                {:else if activeMode === 'llm'}
                    <span>🧠 Generate Map-Reduce Summary</span>
                {:else}
                    <span>✨ Generate Extractive Summary</span>
                {/if}
            </button>
        </div>
    </section>
</div>

<style>
    .summarize-view {
        padding: 2rem;
        max-width: 1000px;
        margin: 0 auto;
        color: #cdd6f4;
        display: flex;
        flex-direction: column;
        gap: 2rem;
    }

    .view-header {
        display: flex;
        justify-content: space-between;
        align-items: flex-start;
        border-bottom: 1px solid #313244;
        padding-bottom: 1.5rem;
        gap: 1rem;
    }

    h1 {
        font-size: 1.75rem;
        font-weight: 700;
        color: #cba6f7;
        margin: 0 0 0.5rem 0;
    }

    .subtitle {
        color: #a6adc8;
        font-size: 0.95rem;
        margin: 0;
    }

    .privacy-badge {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        background-color: #1e1e2e;
        border: 1px solid #a6e3a1;
        color: #a6e3a1;
        padding: 0.4rem 0.8rem;
        border-radius: 9999px;
        font-size: 0.85rem;
        font-weight: 500;
        white-space: nowrap;
    }

    .badge-dot {
        width: 8px;
        height: 8px;
        border-radius: 50%;
        background-color: #a6e3a1;
        box-shadow: 0 0 6px rgba(166, 227, 161, 0.6);
    }

    .modes-grid {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
        gap: 1.5rem;
    }

    .mode-card {
        background-color: #1e1e2e;
        border: 2px solid #313244;
        border-radius: 12px;
        padding: 1.5rem;
        text-align: left;
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
        cursor: pointer;
        transition: all 0.2s ease;
        color: inherit;
    }

    .mode-card:hover {
        border-color: #585b70;
        background-color: #242438;
    }

    .mode-card.active {
        border-color: #cba6f7;
        background-color: rgba(203, 166, 247, 0.08);
        box-shadow: 0 4px 16px rgba(0, 0, 0, 0.2);
    }

    .mode-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    .mode-icon {
        font-size: 1.5rem;
    }

    .mode-badge {
        font-size: 0.75rem;
        padding: 0.2rem 0.6rem;
        border-radius: 6px;
        font-weight: 600;
        text-transform: uppercase;
        letter-spacing: 0.03em;
    }

    .active-badge {
        background-color: rgba(166, 227, 161, 0.2);
        color: #a6e3a1;
        border: 1px solid #a6e3a1;
    }

    .gated-badge {
        background-color: rgba(249, 226, 175, 0.2);
        color: #f9e2af;
        border: 1px solid #f9e2af;
    }

    .ready-badge {
        background-color: rgba(137, 180, 250, 0.2);
        color: #89b4fa;
        border: 1px solid #89b4fa;
    }

    .experimental-badge {
        background-color: rgba(250, 179, 135, 0.2);
        color: #fab387;
        border: 1px solid #fab387;
    }

    .badge-experimental {
        font-size: 0.75rem;
        font-weight: 600;
        text-transform: uppercase;
        background-color: rgba(250, 179, 135, 0.15);
        color: #fab387;
        padding: 0.2rem 0.5rem;
        border-radius: 6px;
        border: 1px solid #fab387;
    }

    .section-header-row {
        display: flex;
        align-items: center;
        justify-content: space-between;
        margin-bottom: 0.75rem;
    }

    .section-header-row h2 {
        margin: 0;
    }

    .info-callout {
        background-color: rgba(249, 226, 175, 0.08);
        border: 1px solid #fab387;
        border-radius: 8px;
        padding: 0.75rem 1rem;
        font-size: 0.85rem;
        color: #fab387;
        margin-bottom: 1rem;
    }

    .form-row-2 {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 1.25rem;
    }

    select {
        background-color: #252538;
        border: 1px solid #313244;
        border-radius: 8px;
        padding: 0.75rem 1rem;
        color: #cdd6f4;
        font-size: 0.92rem;
        outline: none;
        transition: border-color 0.2s ease;
        box-sizing: border-box;
    }

    select:focus {
        border-color: #cba6f7;
    }

    .toggle-checkbox-label {
        display: flex;
        align-items: flex-start;
        gap: 0.75rem;
        cursor: pointer;
        user-select: none;
    }

    .toggle-checkbox-label input[type="checkbox"] {
        margin-top: 0.2rem;
        accent-color: #cba6f7;
        cursor: pointer;
        width: 1.1rem;
        height: 1.1rem;
    }

    .llm-ready-info {
        margin-top: 0.5rem;
        padding: 0.75rem;
        background-color: #181825;
        border-radius: 8px;
        border: 1px solid #313244;
        display: flex;
        flex-direction: column;
        gap: 0.4rem;
        font-size: 0.82rem;
    }

    .info-row {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        color: #cdd6f4;
    }

    .info-label {
        font-weight: 600;
        color: #9399b2;
    }

    .info-val {
        word-break: break-all;
    }

    .info-hash {
        background-color: #313244;
        padding: 0.1rem 0.3rem;
        border-radius: 4px;
        color: #a6e3a1;
        font-size: 0.78rem;
    }

    .status-success {
        color: #a6e3a1;
        font-weight: 500;
    }

    .mode-card h3 {
        margin: 0;
        font-size: 1.2rem;
        color: #cdd6f4;
    }

    .mode-desc {
        font-size: 0.88rem;
        color: #a6adc8;
        line-height: 1.4;
        margin: 0;
    }

    .mode-features {
        list-style: none;
        padding: 0;
        margin: 0.5rem 0 0 0;
        display: flex;
        flex-direction: column;
        gap: 0.35rem;
        font-size: 0.82rem;
        color: #bac2de;
    }

    .mode-features li::before {
        content: "✓ ";
        color: #a6e3a1;
        font-weight: bold;
    }

    .gated-mode {
        cursor: default;
        opacity: 0.85;
    }

    .gate-notice {
        margin-top: 0.5rem;
        padding: 0.75rem;
        background-color: #181825;
        border-radius: 8px;
        border: 1px dashed #45475a;
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
        font-size: 0.82rem;
        color: #9399b2;
    }

    .gate-notice code {
        background-color: #313244;
        padding: 0.15rem 0.35rem;
        border-radius: 4px;
        color: #fab387;
    }

    .settings-link-btn {
        background: transparent;
        border: none;
        color: #89b4fa;
        cursor: pointer;
        font-size: 0.82rem;
        font-weight: 600;
        padding: 0;
        text-align: left;
        text-decoration: underline;
    }

    .settings-link-btn:hover {
        color: #b4befe;
    }

    .config-panel {
        background-color: #1e1e2e;
        border: 1px solid #313244;
        border-radius: 12px;
        padding: 1.75rem;
        display: flex;
        flex-direction: column;
        gap: 1.75rem;
    }

    .panel-section h2 {
        font-size: 1.05rem;
        font-weight: 600;
        color: #b4befe;
        margin: 0 0 0.85rem 0;
    }

    .length-selector {
        display: grid;
        grid-template-columns: repeat(3, 1fr);
        gap: 1rem;
    }

    .length-btn {
        background-color: #252538;
        border: 2px solid #313244;
        border-radius: 10px;
        padding: 1rem;
        text-align: left;
        cursor: pointer;
        display: flex;
        flex-direction: column;
        gap: 0.35rem;
        color: #cdd6f4;
        transition: all 0.2s ease;
    }

    .length-btn:hover {
        border-color: #585b70;
        background-color: #2c2c42;
    }

    .length-btn.selected {
        border-color: #cba6f7;
        background-color: rgba(203, 166, 247, 0.12);
    }

    .length-label {
        font-size: 1rem;
        font-weight: 700;
        color: #cba6f7;
    }

    .length-desc {
        font-size: 0.78rem;
        color: #a6adc8;
        line-height: 1.35;
    }

    .form-grid {
        display: flex;
        flex-direction: column;
        gap: 1.25rem;
    }

    .form-group {
        display: flex;
        flex-direction: column;
        gap: 0.45rem;
    }

    .form-group label {
        font-size: 0.88rem;
        font-weight: 500;
        color: #bac2de;
    }

    .help-text {
        font-size: 0.78rem;
        color: #9399b2;
        font-weight: normal;
        margin-left: 0.4rem;
    }

    .range-with-number {
        display: flex;
        align-items: center;
        gap: 1rem;
    }

    .range-with-number input[type="range"] {
        flex: 1;
        accent-color: #cba6f7;
        cursor: pointer;
    }

    .range-with-number input[type="number"] {
        width: 90px;
        background-color: #252538;
        border: 1px solid #313244;
        border-radius: 8px;
        padding: 0.5rem 0.75rem;
        color: #cdd6f4;
        font-size: 0.9rem;
        text-align: right;
    }

    .range-with-number input[type="number"]:focus {
        border-color: #cba6f7;
        outline: none;
    }

    .input-with-button input {
        width: 100%;
        background-color: #252538;
        border: 1px solid #313244;
        border-radius: 8px;
        padding: 0.75rem 1rem;
        color: #cdd6f4;
        font-size: 0.92rem;
        outline: none;
        transition: border-color 0.2s ease;
        box-sizing: border-box;
    }

    .input-with-button input:focus {
        border-color: #cba6f7;
    }

    .status-box {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        padding: 0.85rem 1rem;
        border-radius: 8px;
        font-size: 0.9rem;
    }

    .status-box.error {
        background-color: rgba(243, 139, 168, 0.15);
        border: 1px solid #f38ba8;
        color: #f38ba8;
    }

    .status-box.success {
        background-color: rgba(166, 227, 161, 0.15);
        border: 1px solid #a6e3a1;
        color: #a6e3a1;
    }

    .panel-actions {
        display: flex;
        justify-content: flex-end;
    }

    .primary-btn {
        background-color: #cba6f7;
        color: #11111b;
        font-weight: 600;
        font-size: 0.95rem;
        padding: 0.8rem 1.6rem;
        border: none;
        border-radius: 8px;
        cursor: pointer;
        display: inline-flex;
        align-items: center;
        gap: 0.5rem;
        transition: all 0.2s ease;
    }

    .primary-btn:hover:not(:disabled) {
        background-color: #b4befe;
        transform: translateY(-1px);
    }

    .primary-btn:disabled {
        opacity: 0.6;
        cursor: not-allowed;
    }

    .spinner {
        width: 1rem;
        height: 1rem;
        border: 2px solid #11111b;
        border-top-color: transparent;
        border-radius: 50%;
        animation: spin 0.8s linear infinite;
    }

    @keyframes spin {
        to {
            transform: rotate(360deg);
        }
    }
</style>

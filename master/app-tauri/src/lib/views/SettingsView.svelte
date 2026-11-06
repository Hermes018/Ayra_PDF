<script lang="ts">
    import StudioIcon from '$lib/components/studio/StudioIcon.svelte';
    import { onMount } from 'svelte';
    import { IpcClient, pickFilesDialog } from '$lib/ipc';
    import For from '$lib/components/For.svelte';
    import { appState } from '$lib/state.svelte';
    import Dialog from '$lib/components/ui/Dialog.svelte';
    import Button from '$lib/components/ui/Button.svelte';
    import Input from '$lib/components/ui/Input.svelte';
    import Select from '$lib/components/ui/Select.svelte';
    import Toggle from '$lib/components/ui/Toggle.svelte';
    import ErrorState from '$lib/components/ui/ErrorState.svelte';
    import { friendlyJobError } from '$lib/core/jobError';
    import type { AppSettings } from '$lib/bindings/AppSettings';
    import type { HardwareTier } from '$lib/bindings/HardwareTier';
    import type { OutputPolicy } from '$lib/bindings/OutputPolicy';
    import type { ThemePreference } from '$lib/bindings/ThemePreference';
    import type { AiModelInfo } from '$lib/bindings/AiModelInfo';
    import { INK_GUESS_LABS, MATH_TEXT_LABS, SPLIT_VIEW_LABS, isLabsOn, setLabs } from '$lib/components/studio/labs.svelte';

    // State via Svelte 5 runes
    let settings = $state<AppSettings>({
        tier_override: null,
        output_policy: 'SuffixNewFile',
        backups_enabled: true,
        language: 'en',
        theme: 'System',
        local_ai_model_path: null,
        ai_local_enabled: false,
        ai_license_acknowledged: false,
        ai_model_sha256: null,
        ai_model_license: null,
    });

    let recentLogs = $state<string[]>([]);
    let isLoadingLogs = $state(false);
    let logError = $state<string | null>(null);

    let modelPathInput = $state('');
    let importedModelInfo = $state<AiModelInfo | null>(null);
    let isImportingModel = $state(false);
    let modelError = $state<{ what: string; next: string } | null>(null);
    let modelLicense = $state('Apache-2.0');
    let licenseAcknowledged = $state(false);

    let diagnosticsPath = $state('localpdf-diagnostics.txt');
    let isExportingDiag = $state(false);
    let diagSuccessMessage = $state<string | null>(null);
    let diagErrorMessage = $state<{ what: string; next: string } | null>(null);

    let loadFailed = $state(false);
    let saveMessage = $state<string | null>(null);
    let saveError = $state<string | null>(null);
    let isSaving = $state(false);
    let isLoaded = $state(false);

    async function loadSettings() {
        loadFailed = false;
        try {
            const fetched = await IpcClient.getSettings();
            if (fetched) {
                settings = fetched;
                // The theme the user is using right now wins: opening Settings must not snap the app back
                // to a stale saved value. Saving writes the current choice.
                settings.theme = appState.themePreference;
                if (settings.local_ai_model_path) {
                    modelPathInput = settings.local_ai_model_path;
                }
                if (settings.ai_model_license) {
                    modelLicense = settings.ai_model_license;
                }
                licenseAcknowledged = settings.ai_license_acknowledged;
            }
        } catch (e: any) {
            console.error('Failed to load settings:', e);
            loadFailed = true;
        } finally {
            isLoaded = true;
        }
    }

    onMount(async () => {
        await loadSettings();
        await refreshLogs();
    });

    async function refreshLogs() {
        isLoadingLogs = true;
        logError = null;
        try {
            recentLogs = await IpcClient.getRecentLogs(150);
        } catch (e: any) {
            console.error('Failed to read logs:', e);
            logError = 'read';
        } finally {
            isLoadingLogs = false;
        }
    }

    // Exported so tests can call the handler directly (the guard must hold even without the disabled button).
    export async function handleSaveSettings() {
        // Fail closed: never write the on-screen defaults over the user's real settings.
        if (loadFailed || !isLoaded) return;
        isSaving = true;
        saveMessage = null;
        saveError = null;
        try {
            settings.ai_license_acknowledged = licenseAcknowledged;
            settings.ai_model_license = modelLicense;
            if (!licenseAcknowledged) {
                settings.ai_local_enabled = false;
            }
            const saved = await IpcClient.saveSettings(settings);
            settings = saved;
            saveMessage = 'Settings saved.';
            setTimeout(() => {
                saveMessage = null;
            }, 3000);
        } catch (e: any) {
            console.error('Failed to save settings:', e);
            saveError = 'save';
        } finally {
            isSaving = false;
        }
    }

    function handleLicenseAckChange(acknowledged: boolean) {
        licenseAcknowledged = acknowledged;
        settings.ai_license_acknowledged = acknowledged;
        if (!acknowledged) {
            settings.ai_local_enabled = false;
        }
    }

    async function handleBrowseModel() {
        try {
            const files = await pickFilesDialog(false, ['gguf', 'onnx', 'bin']);
            if (files && files.length > 0) {
                modelPathInput = files[0];
            }
        } catch (e) {
            console.error('Failed to browse model file:', e);
        }
    }

    export async function handleImportModel() {
        // Fail closed: the backend also refuses, but never ask it to rewrite settings we could not read.
        if (loadFailed || !isLoaded) return;
        if (!modelPathInput.trim()) {
            modelError = { what: 'No model file is chosen.', next: 'Enter or browse to a local model file (.gguf or .onnx), then import it.' };
            return;
        }
        isImportingModel = true;
        modelError = null;
        try {
            const info = await IpcClient.importLocalAiModel(
                modelPathInput.trim(),
                modelLicense.trim(),
                licenseAcknowledged
            );
            importedModelInfo = info;
            settings.local_ai_model_path = info.path;
            settings.ai_model_sha256 = info.sha256_hex;
            settings.ai_model_license = modelLicense.trim();
            settings.ai_license_acknowledged = licenseAcknowledged;
            settings.ai_local_enabled = licenseAcknowledged;
            saveMessage = `Imported model: ${info.file_name} (${info.format}).`;
        } catch (e: any) {
            console.error('Failed to import model:', e);
            modelError = { what: "Couldn't import the model.", next: 'Check that the file is a valid model, the licence is set and acknowledged, then try again.' };
        } finally {
            isImportingModel = false;
        }
    }

    // Reset preferences: asks first (Cancel has focus). The backend keeps the licence and refuses when the settings
    // file cannot be read; the screen mirrors that by disabling the button after a failed load.
    let confirmReset = $state(false);
    let isResetting = $state(false);
    let resetError = $state(false);

    // Exported so tests can call the handler directly (the guard must hold even without the disabled button).
    export async function handleResetPreferences() {
        if (loadFailed || !isLoaded) return;
        isResetting = true;
        resetError = false;
        confirmReset = false;
        try {
            const reset = await IpcClient.resetPreferences();
            settings = reset;
            appState.setTheme(reset.theme);
            licenseAcknowledged = reset.ai_license_acknowledged;
            if (reset.ai_model_license) {
                modelLicense = reset.ai_model_license;
            }
            saveMessage = 'Preferences reset to the defaults. Your model licence was kept.';
            setTimeout(() => {
                saveMessage = null;
            }, 3000);
        } catch (e: any) {
            console.error('Failed to reset preferences:', e);
            resetError = true;
        } finally {
            isResetting = false;
        }
    }

    // An existing file is replaced only after the user confirms; an open document is never a valid target.
    let confirmReplaceDiag = $state(false);

    async function handleExportDiagnostics(overwrite = false) {
        isExportingDiag = true;
        diagSuccessMessage = null;
        diagErrorMessage = null;
        confirmReplaceDiag = false;
        try {
            const savedPath = await IpcClient.exportDiagnostics(
                diagnosticsPath.trim() || 'localpdf-diagnostics.txt',
                [...appState.activeFiles],
                overwrite
            );
            diagSuccessMessage = `Diagnostics report exported to: ${savedPath}`;
        } catch (e: any) {
            const msg = String(e?.message ?? e);
            if (!overwrite && msg.includes('OUTPUT_PATH_UNSAFE') && msg.includes('already exists')) {
                confirmReplaceDiag = true;
            } else {
                console.error('Failed to export diagnostics:', e);
                diagErrorMessage = friendlyJobError(msg, {
                    what: "Couldn't export the diagnostics report.",
                    next: 'Choose another file name or location, then try again.',
                }, false);
            }
        } finally {
            isExportingDiag = false;
        }
    }

    const POLICY_OPTIONS = [
        { value: 'SuffixNewFile', label: 'Suffix new file (doc_1.pdf, doc_2.pdf)' },
        { value: 'Subfolder', label: 'Save to a dedicated subfolder' },
        { value: 'OverwriteWithBackup', label: 'Overwrite original with automatic backup (F8)' },
    ];
    const LANGUAGE_OPTIONS = [
        { value: 'en', label: 'English (en)' },
        { value: 'de', label: 'Deutsch (de)' },
        { value: 'fr', label: 'Français (fr)' },
        { value: 'es', label: 'Español (es)' },
    ];
    const LICENSE_OPTIONS = ['Apache-2.0', 'MIT', 'Llama-3-Community', 'OpenRAIL-M'].map((v) => ({ value: v, label: v })).concat([{ value: 'Custom', label: 'Custom / proprietary' }]);
    const TIERS: { id: string; value: HardwareTier | null; name: string; desc: string }[] = [
        { id: 'auto', value: null, name: 'Auto-detect', desc: 'Adapts to system CPU & RAM' },
        { id: 'Low', value: 'Low', name: 'Tier Low (L)', desc: '1 job · 64MB cache · ≤4GB RAM' },
        { id: 'Medium', value: 'Medium', name: 'Tier Medium (M)', desc: '2 jobs · 256MB cache · Balanced' },
        { id: 'High', value: 'High', name: 'Tier High (H)', desc: 'Parallel jobs · 512MB cache · 16GB+' },
    ];
    const THEMES: { value: ThemePreference; label: string }[] = [
        { value: 'System', label: 'System' },
        { value: 'Light', label: 'Light' },
        { value: 'Dark', label: 'Dark' },
    ];

    function setTier(tier: HardwareTier | null) {
        settings.tier_override = tier;
    }

    function setTheme(theme: ThemePreference) {
        settings.theme = theme;
        appState.setTheme(theme);
    }
</script>

<Dialog
    bind:open={confirmReplaceDiag}
    variant="alert"
    title="Replace the existing file?"
    onclose={() => (confirmReplaceDiag = false)}
>
    <p>A file with that name already exists. Replacing it cannot be undone.</p>
    {#snippet actions()}
        <Button variant="secondary" attrs={{ 'data-autofocus': '' }} onclick={() => (confirmReplaceDiag = false)}>Cancel</Button>
        <Button variant="destructive" onclick={() => handleExportDiagnostics(true)}>Replace file</Button>
    {/snippet}
</Dialog>

<Dialog
    bind:open={confirmReset}
    variant="alert"
    title="Reset preferences?"
    onclose={() => (confirmReset = false)}
>
    <p>
        This resets the theme, language, hardware tier, output policy, backups and the local AI switch to their defaults.
        Your model licence is kept. Your PDFs are not touched.
    </p>
    {#snippet actions()}
        <Button variant="secondary" attrs={{ 'data-autofocus': '' }} onclick={() => (confirmReset = false)}>Cancel</Button>
        <Button variant="destructive" onclick={handleResetPreferences}>Reset now</Button>
    {/snippet}
</Dialog>

<div class="settings-view" data-testid={isLoaded ? 'settings-loaded' : 'settings-loading'}>
    <div class="header">
        <h2>Settings & Diagnostics</h2>
        <p class="subtitle">Hardware tiers, collision safety rules, local AI models, and offline system diagnostics</p>
    </div>

    {#if loadFailed}
        <ErrorState what="Couldn't load your settings." next="The defaults are shown. Try again." retry={loadSettings} />
    {/if}
    {#if saveMessage}
        <div class="banner success" role="status">{saveMessage}</div>
    {/if}
    {#if resetError}
        <ErrorState what="Couldn't reset your preferences." next="Nothing was changed. Try again." retry={handleResetPreferences} />
    {/if}
    {#if saveError}
        <ErrorState what="Couldn't save your settings." next="Nothing was changed on disk. Try again." retry={handleSaveSettings} />
    {/if}

    <div class="settings-grid">
        <!-- Performance & hardware tier -->
        <section class="card" aria-labelledby="hw-title">
            <h3 id="hw-title">Performance & hardware tier</h3>
            <p class="description">Control CPU core allocation, worker concurrency, and tile memory cache limits.</p>

            <div class="tier-options">
                {#each TIERS as t (t.id)}
                    <Button
                        variant="secondary"
                        block
                        class="tier-btn"
                        pressed={settings.tier_override === t.value}
                        data-testid={`tier-btn-${t.id}`}
                        onclick={() => setTier(t.value)}
                    >
                        <span class="tier-body">
                            <span class="tier-name">{t.name}</span>
                            <span class="tier-desc">{t.desc}</span>
                        </span>
                    </Button>
                {/each}
            </div>
        </section>

        <!-- Output policy & backups -->
        <section class="card" aria-labelledby="policy-title">
            <h3 id="policy-title">Output policy & collision safety</h3>
            <p class="description">Writes files atomically and protects originals from accidental overwrite.</p>
            <Select label="Output collision rule" bind:value={() => settings.output_policy, (v) => (settings.output_policy = v as OutputPolicy)} options={POLICY_OPTIONS} />
            <Toggle label="Enable automatic backups (F8): keeps the original before overwrite transformations" bind:checked={settings.backups_enabled} />
        </section>

        <!-- Appearance & language -->
        <section class="card" aria-labelledby="appearance-title">
            <h3 id="appearance-title">Appearance & language</h3>
            <p class="description">Choose the interface color theme and language.</p>
            <div class="theme-group" role="group" aria-label="Interface theme">
                <span class="field-label">Interface theme</span>
                <div class="theme-buttons">
                    {#each THEMES as th (th.value)}
                        <Button variant="secondary" size="sm" pressed={settings.theme === th.value} onclick={() => setTheme(th.value)}>{th.label}</Button>
                    {/each}
                </div>
            </div>
            <Select label="Interface language" bind:value={settings.language} options={LANGUAGE_OPTIONS} />
        </section>

        <!-- Labs -->
        <section class="card" aria-labelledby="labs-title">
            <h3 id="labs-title">Labs (experimental)</h3>
            <p class="description">Unfinished Studio tools. They stay hidden unless you turn this on, and may change or be removed.</p>
            <Toggle
                label="Show Labs tools (off by default; saved on this device as soon as you change it)"
                data-testid="labs-toggle"
                bind:checked={() => isLabsOn(), (v) => setLabs(v)}
            />
            <ul class="labs-list" data-testid="labs-feature-list">
                <li><strong>{MATH_TEXT_LABS.label}</strong> — {MATH_TEXT_LABS.note}</li>
                <li><strong>{INK_GUESS_LABS.label}</strong> — {INK_GUESS_LABS.note}</li>
                <li><strong>{SPLIT_VIEW_LABS.label}</strong> — {SPLIT_VIEW_LABS.note}</li>
            </ul>
        </section>

        <!-- Local AI model registry -->
        <section class="card" aria-labelledby="ai-title">
            <h3 id="ai-title">Local AI model import (offline)</h3>
            <p class="description">Import a local model weights file (.gguf or .onnx). It runs on this computer with no network access.</p>

            <div class="input-action-row">
                <Input label="Local model file path" placeholder="/path/to/model.gguf or model.onnx" bind:value={modelPathInput} />
                <Button variant="secondary" title="Browse local storage for a model file" onclick={handleBrowseModel}>Browse…</Button>
                <Button
                    variant="secondary"
                    busy={isImportingModel}
                    disabled={loadFailed || !isLoaded}
                    disabledReason={loadFailed ? "Settings couldn't be loaded. Retry first." : 'Your settings are still loading'}
                    onclick={handleImportModel}
                >
                    {isImportingModel ? 'Verifying…' : 'Import model'}
                </Button>
            </div>

            {#if modelError}
                <ErrorState what={modelError.what} next={modelError.next} />
            {/if}

            {#if importedModelInfo}
                <div class="model-badge-card">
                    <div class="badge-row">
                        <span class="model-name"><StudioIcon name="file" size={14} /> {importedModelInfo.file_name}</span>
                        <span class="format-pill">{importedModelInfo.format}</span>
                        <span class="size-pill">{(Number(importedModelInfo.size_bytes) / (1024 * 1024)).toFixed(1)} MB</span>
                    </div>
                    <div class="hash-row">
                        <span class="hash-label">SHA-256:</span>
                        <code data-testid="model-sha256">{importedModelInfo.sha256_hex}</code>
                    </div>
                </div>
            {:else if settings.local_ai_model_path}
                <div class="model-badge-card">
                    <div class="badge-row">
                        <span class="model-name"><StudioIcon name="file" size={14} /> {settings.local_ai_model_path}</span>
                        <span class="format-pill">Configured</span>
                    </div>
                    {#if settings.ai_model_sha256}
                        <div class="hash-row">
                            <span class="hash-label">SHA-256:</span>
                            <code data-testid="model-sha256">{settings.ai_model_sha256}</code>
                        </div>
                    {/if}
                </div>
            {/if}

            <div class="license-panel" data-testid="license-verification-panel">
                <h4>Model licence & verification</h4>
                <Select label="Model licence (SPDX)" bind:value={modelLicense} options={LICENSE_OPTIONS} data-testid="model-license-select" />
                <Toggle
                    label="I acknowledge the model licence terms and verify the SHA-256 hash for offline local execution (required by Failsafe F19 before local LLM execution can be enabled)"
                    data-testid="license-acknowledge-checkbox"
                    bind:checked={() => licenseAcknowledged, handleLicenseAckChange}
                />
            </div>

            <Toggle
                label={`Enable AI features (ai-local): ${licenseAcknowledged ? 'offline summarization and translation using local weights' : 'acknowledge the model licence and verify the SHA-256 first'}`}
                data-testid="ai-local-toggle"
                disabled={!licenseAcknowledged}
                disabledReason="Acknowledge the model licence terms first"
                bind:checked={settings.ai_local_enabled}
            />
        </section>
    </div>

    <!-- Log viewer & diagnostics export -->
    <section class="card full-width" aria-labelledby="diag-title">
        <div class="card-header-actions">
            <div>
                <h3 id="diag-title">Local log viewer & diagnostics</h3>
                <p class="description">Inspect recent logs or export a sanitized local diagnostics report.</p>
            </div>
            <Button variant="secondary" busy={isLoadingLogs} onclick={refreshLogs}>
                {isLoadingLogs ? 'Loading…' : 'Refresh logs'}
            </Button>
        </div>

        {#if logError}
            <ErrorState what="Couldn't read the logs." next="Try again." retry={refreshLogs} />
        {/if}

        <div class="log-viewer" role="region" aria-label="Recent logs">
            {#if recentLogs.length === 0}
                <div class="log-empty">No recent logs recorded.</div>
            {:else}
                <For items={recentLogs}>
                    {#snippet children(line)}
                        <div class="log-line">{line}</div>
                    {/snippet}
                </For>
            {/if}
        </div>

        <div class="export-box">
            <div class="export-row">
                <Input label="Export diagnostics report" bind:value={diagnosticsPath} placeholder="localpdf-diagnostics.txt" />
                <Button variant="primary" busy={isExportingDiag} onclick={() => handleExportDiagnostics(false)}>
                    {isExportingDiag ? 'Exporting…' : 'Export diagnostics'}
                </Button>
            </div>
            {#if diagSuccessMessage}
                <div class="banner success" role="status">{diagSuccessMessage}</div>
            {/if}
            {#if diagErrorMessage}
                <ErrorState what={diagErrorMessage.what} next={diagErrorMessage.next} />
            {/if}
        </div>
    </section>

    <!-- Save -->
    <div class="footer-actions">
        <Button
            variant="secondary"
            size="lg"
            busy={isResetting}
            disabled={loadFailed || !isLoaded}
            disabledReason={loadFailed ? "Settings couldn't be loaded. Retry first." : 'Your settings are still loading'}
            onclick={() => (confirmReset = true)}
        >
            Reset preferences
        </Button>
        <Button
            variant="primary"
            size="lg"
            busy={isSaving}
            disabled={loadFailed || !isLoaded}
            disabledReason={loadFailed ? "Settings couldn't be loaded. Retry first." : 'Your settings are still loading'}
            onclick={handleSaveSettings}
        >
            {isSaving ? 'Saving…' : 'Save settings'}
        </Button>
    </div>
</div>

<style>
    .settings-view {
        display: flex;
        flex-direction: column;
        gap: var(--space-4);
        padding: var(--space-4);
        color: var(--text-primary);
    }

    .header h2 {
        margin: 0;
        font-size: var(--font-size-title1);
        color: var(--accent-primary);
    }

    .subtitle,
    .description {
        margin: var(--space-1) 0 0;
        color: var(--text-secondary);
        font-size: var(--font-size-footnote);
    }

    .settings-grid {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(460px, 1fr));
        align-items: start;
        gap: var(--space-4);
    }

    .card {
        background: var(--glass-bg-surface);
        border: 1px solid var(--glass-border-subtle);
        border-radius: var(--radius-lg);
        padding: var(--space-4);
        display: flex;
        flex-direction: column;
        gap: var(--space-3);
        min-width: 0;
    }

    .card h3 {
        margin: 0;
        font-size: var(--font-size-title3);
    }

    .tier-options {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: var(--space-2);
    }

    .tier-options :global(.tier-btn .lp-btn) {
        height: auto;
        min-height: 0;
        justify-content: flex-start;
        text-align: left;
        padding: var(--space-3);
        font-weight: 400;
    }

    .tier-body {
        display: flex;
        flex-direction: column;
        gap: var(--space-1);
        white-space: normal;
    }

    .tier-name {
        font-weight: 600;
    }

    .tier-desc {
        color: var(--text-secondary);
        font-size: var(--font-size-footnote);
    }

    .field-label {
        font-size: var(--font-size-footnote);
        font-weight: 600;
        color: var(--text-secondary);
    }

    .theme-group {
        display: flex;
        flex-direction: column;
        gap: var(--space-2);
    }

    .theme-buttons,
    .input-action-row,
    .export-row {
        display: flex;
        gap: var(--space-2);
        align-items: flex-end;
        flex-wrap: wrap;
    }

    .input-action-row :global(.lp-field),
    .export-row :global(.lp-field) {
        flex: 1;
        min-width: 14em;
    }

    .labs-list {
        margin: 0;
        padding-left: var(--space-5);
        font-size: var(--font-size-footnote);
        color: var(--text-secondary);
        display: flex;
        flex-direction: column;
        gap: var(--space-2);
    }

    .model-badge-card,
    .license-panel {
        padding: var(--space-3);
        border-radius: var(--radius-md);
        background: var(--glass-bg-input);
        display: flex;
        flex-direction: column;
        gap: var(--space-2);
    }

    .license-panel h4 {
        margin: 0;
    }

    .badge-row {
        display: flex;
        gap: var(--space-2);
        align-items: center;
        flex-wrap: wrap;
    }

    .format-pill,
    .size-pill {
        font-size: var(--font-size-caption);
        padding: 0 var(--space-2);
        border-radius: var(--radius-pill);
        background: var(--glass-bg-active);
    }

    .hash-row code {
        word-break: break-all;
        font-size: var(--font-size-caption);
    }

    .card-header-actions {
        display: flex;
        justify-content: space-between;
        gap: var(--space-3);
        align-items: flex-start;
        flex-wrap: wrap;
    }

    .log-viewer {
        max-height: 220px;
        overflow: auto;
        padding: var(--space-3);
        border-radius: var(--radius-md);
        background: var(--glass-bg-input);
        font-family: var(--font-mono);
        font-size: var(--font-size-caption);
    }

    .log-line {
        white-space: pre-wrap;
        word-break: break-all;
    }

    .banner {
        padding: var(--space-3) var(--space-4);
        border-radius: var(--radius-md);
        font-size: var(--font-size-footnote);
        background: var(--glass-bg-input);
        border: 1px solid var(--glass-border-subtle);
    }

    .banner.success {
        border-color: var(--semantic-success);
    }

    .footer-actions {
        display: flex;
        justify-content: flex-end;
        gap: var(--space-2);
    }
</style>

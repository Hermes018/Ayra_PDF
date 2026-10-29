<script lang="ts">
    import { onMount } from 'svelte';
    import { IpcClient, pickFilesDialog } from '$lib/ipc';
    import { appState } from '$lib/state.svelte';
    import type { AppSettings } from '$lib/bindings/AppSettings';
    import type { HardwareTier } from '$lib/bindings/HardwareTier';
    import type { OutputPolicy } from '$lib/bindings/OutputPolicy';
    import type { ThemePreference } from '$lib/bindings/ThemePreference';
    import type { AiModelInfo } from '$lib/bindings/AiModelInfo';

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
    let modelError = $state<string | null>(null);
    let modelLicense = $state('Apache-2.0');
    let licenseAcknowledged = $state(false);

    let diagnosticsPath = $state('localpdf-diagnostics.txt');
    let isExportingDiag = $state(false);
    let diagSuccessMessage = $state<string | null>(null);
    let diagErrorMessage = $state<string | null>(null);

    let saveMessage = $state<string | null>(null);
    let saveError = $state<string | null>(null);
    let isSaving = $state(false);
    let isLoaded = $state(false);

    onMount(async () => {
        try {
            const fetched = await IpcClient.getSettings();
            if (fetched) {
                settings = fetched;
                if (settings.theme) {
                    appState.setTheme(settings.theme);
                } else if (appState.themePreference) {
                    settings.theme = appState.themePreference;
                }
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
            saveError = `Failed to load settings: ${e?.message ?? e}`;
        } finally {
            isLoaded = true;
        }

        await refreshLogs();
    });

    async function refreshLogs() {
        isLoadingLogs = true;
        logError = null;
        try {
            recentLogs = await IpcClient.getRecentLogs(150);
        } catch (e: any) {
            logError = `Failed to read logs: ${e?.message ?? e}`;
        } finally {
            isLoadingLogs = false;
        }
    }

    async function handleSaveSettings() {
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
            saveMessage = 'Settings saved successfully!';
            setTimeout(() => {
                saveMessage = null;
            }, 3000);
        } catch (e: any) {
            saveError = `Failed to save settings: ${e?.message ?? e}`;
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

    async function handleImportModel() {
        if (!modelPathInput.trim()) {
            modelError = 'Please specify a local model file path (.gguf or .onnx).';
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
            saveMessage = `Imported model: ${info.file_name} (${info.format})`;
        } catch (e: any) {
            modelError = `Failed to import model: ${e?.message ?? e}`;
        } finally {
            isImportingModel = false;
        }
    }

    async function handleExportDiagnostics() {
        isExportingDiag = true;
        diagSuccessMessage = null;
        diagErrorMessage = null;
        try {
            const savedPath = await IpcClient.exportDiagnostics(diagnosticsPath.trim() || 'localpdf-diagnostics.txt');
            diagSuccessMessage = `Diagnostics report exported to: ${savedPath}`;
        } catch (e: any) {
            diagErrorMessage = `Failed to export diagnostics: ${e?.message ?? e}`;
        } finally {
            isExportingDiag = false;
        }
    }

    function setTier(tier: HardwareTier | null) {
        settings.tier_override = tier;
    }

    function setTheme(theme: ThemePreference) {
        settings.theme = theme;
        appState.setTheme(theme);
    }
</script>

<div class="settings-view" data-testid={isLoaded ? 'settings-loaded' : 'settings-loading'}>
    <div class="header">
        <h2>Settings & Diagnostics</h2>
        <p class="subtitle">Hardware tiers, collision safety rules, local AI models, and offline system diagnostics</p>
    </div>

    {#if saveMessage}
        <div class="banner success">{saveMessage}</div>
    {/if}
    {#if saveError}
        <div class="banner error">{saveError}</div>
    {/if}

    <div class="settings-grid">
        <!-- 1. Performance & Hardware Tier -->
        <section class="card" aria-labelledby="hw-title">
            <h3 id="hw-title">⚡ Performance & Hardware Tier</h3>
            <p class="description">Control CPU core allocation, worker concurrency, and tile memory cache limits.</p>

            <div class="tier-options">
                <button
                    type="button"
                    class="tier-btn"
                    data-testid="tier-btn-auto"
                    class:active={settings.tier_override === null}
                    onclick={() => setTier(null)}
                >
                    <div class="tier-name">Auto-Detect</div>
                    <div class="tier-desc">Adapts to system CPU & RAM</div>
                </button>
                <button
                    type="button"
                    class="tier-btn"
                    data-testid="tier-btn-Low"
                    class:active={settings.tier_override === 'Low'}
                    onclick={() => setTier('Low')}
                >
                    <div class="tier-name">Tier Low (L)</div>
                    <div class="tier-desc">1 job · 64MB cache · &le;4GB RAM</div>
                </button>
                <button
                    type="button"
                    class="tier-btn"
                    data-testid="tier-btn-Medium"
                    class:active={settings.tier_override === 'Medium'}
                    onclick={() => setTier('Medium')}
                >
                    <div class="tier-name">Tier Medium (M)</div>
                    <div class="tier-desc">2 jobs · 256MB cache · Balanced</div>
                </button>
                <button
                    type="button"
                    class="tier-btn"
                    data-testid="tier-btn-High"
                    class:active={settings.tier_override === 'High'}
                    onclick={() => setTier('High')}
                >
                    <div class="tier-name">Tier High (H)</div>
                    <div class="tier-desc">Parallel jobs · 512MB cache · 16GB+</div>
                </button>
            </div>
        </section>

        <!-- 2. Output Policy & Backups -->
        <section class="card" aria-labelledby="policy-title">
            <h3 id="policy-title">🛡️ Output Policy & Collision Safety</h3>
            <p class="description">Guarantees atomic file writes and safeguards originals against accidental overwrite.</p>

            <div class="form-group">
                <label for="output-policy-select">Output Collision Rule</label>
                <select
                    id="output-policy-select"
                    bind:value={settings.output_policy}
                >
                    <option value="SuffixNewFile">Suffix New File (doc_1.pdf, doc_2.pdf)</option>
                    <option value="Subfolder">Save to dedicated subfolder</option>
                    <option value="OverwriteWithBackup">Overwrite original with automatic backup (F8)</option>
                </select>
            </div>

            <div class="checkbox-row">
                <input
                    type="checkbox"
                    id="backups-toggle"
                    bind:checked={settings.backups_enabled}
                />
                <label for="backups-toggle">
                    <strong>Enable Automatic Backups (F8)</strong>
                    <span class="subtext">Retains original versions before overwrite transformations</span>
                </label>
            </div>
        </section>

        <!-- 3. Appearance & Language -->
        <section class="card" aria-labelledby="appearance-title">
            <h3 id="appearance-title">🎨 Appearance & Language</h3>
            <p class="description">Configure interface color theme and localization options.</p>

            <div class="form-group">
                <span class="field-label">Interface Theme</span>
                <div class="theme-buttons">
                    <button
                        type="button"
                        class="theme-btn"
                        class:active={settings.theme === 'System'}
                        onclick={() => setTheme('System')}
                    >
                        🖥️ System
                    </button>
                    <button
                        type="button"
                        class="theme-btn"
                        class:active={settings.theme === 'Light'}
                        onclick={() => setTheme('Light')}
                    >
                        ☀️ Light
                    </button>
                    <button
                        type="button"
                        class="theme-btn"
                        class:active={settings.theme === 'Dark'}
                        onclick={() => setTheme('Dark')}
                    >
                        🌙 Dark
                    </button>
                </div>
            </div>

            <div class="form-group">
                <label for="language-select">Interface Language</label>
                <select id="language-select" bind:value={settings.language}>
                    <option value="en">English (en)</option>
                    <option value="de">Deutsch (de)</option>
                    <option value="fr">Français (fr)</option>
                    <option value="es">Español (es)</option>
                </select>
            </div>
        </section>

        <!-- 4. Local AI Model Registry -->
        <section class="card" aria-labelledby="ai-title">
            <h3 id="ai-title">🧠 Local AI Model Import (Offline)</h3>
            <p class="description">Import a local LLM weights file (.gguf or .onnx). Strictly local execution, zero network access.</p>

            <div class="form-group">
                <label for="model-path-input">Local Model File Path</label>
                <div class="input-action-row">
                    <input
                        type="text"
                        id="model-path-input"
                        placeholder="/path/to/model.gguf or model.onnx"
                        bind:value={modelPathInput}
                    />
                    <button
                        type="button"
                        class="btn secondary"
                        onclick={handleBrowseModel}
                        title="Browse local storage for model file"
                    >
                        📁 Browse...
                    </button>
                    <button
                        type="button"
                        class="btn secondary"
                        disabled={isImportingModel}
                        onclick={handleImportModel}
                    >
                        {isImportingModel ? 'Verifying...' : 'Import Model'}
                    </button>
                </div>
            </div>

            {#if modelError}
                <div class="banner error">{modelError}</div>
            {/if}

            {#if importedModelInfo}
                <div class="model-badge-card">
                    <div class="badge-row">
                        <span class="model-name">📄 {importedModelInfo.file_name}</span>
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
                        <span class="model-name">📄 {settings.local_ai_model_path}</span>
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

            <!-- Model Licence & Checksum Verification panel -->
            <div class="license-panel" data-testid="license-verification-panel">
                <h4>📜 Model Licence & Verification</h4>
                <div class="form-group">
                    <label for="model-license-select">Model Licence (SPDX)</label>
                    <select
                        id="model-license-select"
                        data-testid="model-license-select"
                        bind:value={modelLicense}
                    >
                        <option value="Apache-2.0">Apache-2.0</option>
                        <option value="MIT">MIT</option>
                        <option value="Llama-3-Community">Llama-3-Community</option>
                        <option value="OpenRAIL-M">OpenRAIL-M</option>
                        <option value="Custom">Custom / Proprietary</option>
                    </select>
                </div>

                <div class="checkbox-row">
                    <input
                        type="checkbox"
                        id="license-acknowledge-checkbox"
                        data-testid="license-acknowledge-checkbox"
                        checked={licenseAcknowledged}
                        onchange={(e) => handleLicenseAckChange((e.target as HTMLInputElement).checked)}
                    />
                    <label for="license-acknowledge-checkbox">
                        <strong>I acknowledge the model licence terms and verify the SHA-256 hash for offline local execution</strong>
                        <span class="subtext">Required by Failsafe F19 before local LLM execution can be enabled</span>
                    </label>
                </div>
            </div>

            <div class="checkbox-row">
                <input
                    type="checkbox"
                    id="ai-toggle"
                    data-testid="ai-local-toggle"
                    disabled={!licenseAcknowledged}
                    bind:checked={settings.ai_local_enabled}
                />
                <label for="ai-toggle">
                    <strong>Enable AI Features (ai-local)</strong>
                    <span class="subtext">{licenseAcknowledged ? 'Offline summarization and translation using local weights' : 'Disabled: Acknowledge model licence terms and verify SHA-256 to enable'}</span>
                </label>
            </div>
        </section>
    </div>

    <!-- 5. Local Log Viewer & Diagnostics Export -->
    <section class="card full-width" aria-labelledby="diag-title">
        <div class="card-header-actions">
            <div>
                <h3 id="diag-title">📋 Local Log Viewer & Diagnostics</h3>
                <p class="description">Inspect active system logs or export a sanitized local diagnostics report.</p>
            </div>
            <div class="actions">
                <button
                    type="button"
                    class="btn secondary"
                    disabled={isLoadingLogs}
                    onclick={refreshLogs}
                >
                    {isLoadingLogs ? 'Loading...' : '🔄 Refresh Logs'}
                </button>
            </div>
        </div>

        {#if logError}
            <div class="banner error">{logError}</div>
        {/if}

        <div class="log-viewer" role="region" aria-label="Recent Logs">
            {#if recentLogs.length === 0}
                <div class="log-empty">No recent logs recorded.</div>
            {:else}
                {#each recentLogs as line}
                    <div class="log-line">{line}</div>
                {/each}
            {/if}
        </div>

        <div class="export-box">
            <div class="export-row">
                <label for="diag-path-input">Export Diagnostics Report:</label>
                <input
                    type="text"
                    id="diag-path-input"
                    bind:value={diagnosticsPath}
                    placeholder="localpdf-diagnostics.txt"
                />
                <button
                    type="button"
                    class="btn primary"
                    disabled={isExportingDiag}
                    onclick={handleExportDiagnostics}
                >
                    {isExportingDiag ? 'Exporting...' : 'Export Diagnostics'}
                </button>
            </div>
            {#if diagSuccessMessage}
                <div class="banner success">{diagSuccessMessage}</div>
            {/if}
            {#if diagErrorMessage}
                <div class="banner error">{diagErrorMessage}</div>
            {/if}
        </div>
    </section>

    <!-- Save Settings Action Footer -->
    <div class="footer-actions">
        <button
            type="button"
            class="btn primary large"
            disabled={isSaving}
            onclick={handleSaveSettings}
        >
            {isSaving ? 'Saving...' : '💾 Save Settings'}
        </button>
    </div>
</div>

<style>
    .settings-view {
        padding: 1.5rem;
        max-width: 1200px;
        margin: 0 auto;
        color: var(--text-primary);
        display: flex;
        flex-direction: column;
        gap: 1.5rem;
    }

    .header h2 {
        font-size: 1.8rem;
        margin: 0 0 0.25rem 0;
        color: var(--accent-primary);
    }

    .subtitle {
        color: var(--text-secondary);
        margin: 0;
        font-size: 0.95rem;
    }

    .banner {
        padding: 0.75rem 1rem;
        border-radius: var(--radius-sm, 8px);
        font-size: 0.9rem;
    }

    .banner.success {
        background-color: rgba(52, 199, 89, 0.15);
        border: 1px solid var(--semantic-success);
        color: var(--semantic-success);
    }

    .banner.error {
        background-color: rgba(255, 59, 48, 0.15);
        border: 1px solid var(--semantic-danger);
        color: var(--semantic-danger);
    }

    .settings-grid {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(min(100%, 320px), 1fr));
        gap: 1.5rem;
    }

    .card {
        background-color: var(--glass-bg-surface);
        -webkit-backdrop-filter: var(--glass-blur);
        backdrop-filter: var(--glass-blur);
        border: 1px solid var(--glass-border-subtle);
        border-radius: var(--radius-lg, 14px);
        box-shadow: inset 0 1px 0 0 var(--glass-border-specular), var(--glass-shadow-sm);
        padding: 1.5rem;
        display: flex;
        flex-direction: column;
        gap: 1rem;
        min-width: 0;
        box-sizing: border-box;
    }

    .card.full-width {
        grid-column: 1 / -1;
    }

    .card h3 {
        margin: 0;
        font-size: 1.15rem;
        color: var(--text-primary);
    }

    .description {
        margin: 0;
        font-size: 0.85rem;
        color: var(--text-secondary);
    }

    .tier-options {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(min(100%, 180px), 1fr));
        gap: 0.75rem;
    }

    .tier-btn {
        background-color: var(--glass-bg-input);
        border: 1px solid var(--glass-border-subtle);
        border-radius: var(--radius-sm, 8px);
        padding: 0.75rem;
        text-align: left;
        cursor: pointer;
        color: var(--text-primary);
        transition: all var(--dur-fast, 120ms) ease;
    }

    .tier-btn:hover {
        border-color: var(--accent-primary);
        background-color: var(--glass-bg-hover);
    }

    .tier-btn.active {
        border-color: var(--accent-primary);
        background-color: var(--glass-bg-active);
        color: var(--accent-primary);
    }

    .tier-name {
        font-weight: 600;
        font-size: 0.95rem;
        margin-bottom: 0.2rem;
    }

    .tier-desc {
        font-size: 0.75rem;
        color: var(--text-secondary);
    }

    .form-group {
        display: flex;
        flex-direction: column;
        gap: 0.4rem;
    }

    .form-group label,
    .field-label {
        font-size: 0.85rem;
        font-weight: 500;
        color: var(--text-primary);
    }

    select, input[type="text"] {
        background-color: var(--glass-bg-input);
        border: 1px solid var(--glass-border-subtle);
        border-radius: var(--radius-md, 10px);
        padding: 0.6rem 0.8rem;
        color: var(--text-primary);
        font-size: 0.9rem;
        outline: none;
    }

    select option {
        background-color: var(--option-bg) !important;
        color: var(--option-fg) !important;
        padding: 0.4rem;
    }

    select:focus, input[type="text"]:focus {
        border-color: var(--accent-primary);
        box-shadow: 0 0 0 3px var(--accent-ring);
    }

    .checkbox-row {
        display: flex;
        align-items: flex-start;
        gap: 0.6rem;
        padding-top: 0.25rem;
    }

    .checkbox-row input[type="checkbox"] {
        margin-top: 0.2rem;
        accent-color: var(--accent-primary);
        cursor: pointer;
    }

    .checkbox-row label {
        display: flex;
        flex-direction: column;
        cursor: pointer;
        font-size: 0.9rem;
    }

    .subtext {
        font-size: 0.75rem;
        color: var(--text-secondary);
    }

    .theme-buttons {
        display: inline-flex;
        padding: 3px;
        border-radius: var(--radius-sm, 8px);
        background: var(--glass-bg-input);
        border: 1px solid var(--glass-border-subtle);
        width: 100%;
        max-width: 360px;
        gap: 3px;
    }

    .theme-btn {
        flex: 1;
        padding: 0.45rem 0.75rem;
        background-color: transparent;
        border: none;
        border-radius: var(--radius-xs, 6px);
        color: var(--text-primary);
        cursor: pointer;
        font-size: 0.85rem;
        font-weight: 500;
        transition: background-color var(--dur-fast, 120ms) ease, box-shadow var(--dur-fast, 120ms) ease;
    }

    .theme-btn:hover {
        background-color: var(--glass-bg-hover);
    }

    .theme-btn.active {
        background-color: var(--glass-bg-elevated);
        box-shadow: 0 1px 3px rgba(0, 0, 0, 0.18), inset 0 1px 0 0 var(--glass-border-specular);
        color: var(--accent-primary);
        font-weight: 600;
    }

    .input-action-row {
        display: flex;
        gap: 0.5rem;
    }

    .input-action-row input {
        flex: 1;
    }

    .model-badge-card {
        background-color: var(--glass-bg-elevated);
        border: 1px solid var(--glass-border-subtle);
        border-radius: var(--radius-md, 10px);
        padding: 0.75rem;
        display: flex;
        flex-direction: column;
        gap: 0.4rem;
    }

    .badge-row {
        display: flex;
        align-items: center;
        gap: 0.5rem;
    }

    .model-name {
        font-weight: 500;
        font-size: 0.85rem;
        flex: 1;
    }

    .format-pill, .size-pill {
        font-size: 0.75rem;
        padding: 0.15rem 0.5rem;
        border-radius: 12px;
        background-color: var(--glass-bg-input);
        color: var(--semantic-warning);
    }

    .size-pill {
        color: var(--semantic-success);
    }

    .hash-row {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        font-size: 0.75rem;
        color: var(--text-secondary);
    }

    .hash-row code {
        background-color: var(--glass-bg-input);
        padding: 0.1rem 0.3rem;
        border-radius: 4px;
        color: var(--text-primary);
        word-break: break-all;
    }

    .license-panel {
        background-color: var(--glass-bg-surface);
        border: 1px solid var(--glass-border-subtle);
        border-radius: var(--radius-md, 10px);
        padding: 0.75rem;
        display: flex;
        flex-direction: column;
        gap: 0.6rem;
    }

    .license-panel h4 {
        margin: 0;
        font-size: 0.9rem;
        font-weight: 600;
        color: var(--text-primary);
    }

    .card-header-actions {
        display: flex;
        justify-content: space-between;
        align-items: flex-start;
    }

    .log-viewer {
        background-color: var(--glass-bg-input);
        border: 1px solid var(--glass-border-subtle);
        border-radius: var(--radius-md, 10px);
        padding: 0.75rem;
        height: 180px;
        overflow-y: auto;
        font-family: var(--font-mono, monospace);
        font-size: 0.8rem;
        color: var(--text-secondary);
        display: flex;
        flex-direction: column;
        gap: 0.2rem;
    }

    .log-empty {
        color: var(--text-tertiary);
        font-style: italic;
    }

    .log-line {
        line-height: 1.3;
        word-break: break-all;
    }

    .export-box {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
        padding-top: 0.5rem;
        border-top: 1px solid var(--glass-border-subtle);
    }

    .export-row {
        display: flex;
        align-items: center;
        gap: 0.75rem;
    }

    .export-row label {
        font-size: 0.85rem;
        color: var(--text-primary);
        white-space: nowrap;
    }

    .export-row input {
        flex: 1;
    }

    .btn {
        padding: 0.6rem 1rem;
        border-radius: var(--radius-md, 10px);
        border: 1px solid var(--glass-border-subtle);
        cursor: pointer;
        font-size: 0.85rem;
        font-weight: 500;
        transition: all var(--dur-fast, 120ms) ease;
    }

    .btn.primary {
        background-color: var(--accent-primary);
        color: var(--text-on-accent);
        border-color: transparent;
    }

    .btn.primary:hover:not(:disabled) {
        background-color: var(--accent-primary-hover);
    }

    .btn.secondary {
        background-color: var(--glass-bg-input);
        color: var(--text-primary);
    }

    .btn.secondary:hover:not(:disabled) {
        background-color: var(--glass-bg-hover);
    }

    .btn.large {
        padding: 0.8rem 1.75rem;
        font-size: 1rem;
        font-weight: 600;
    }

    .btn:disabled {
        opacity: 0.5;
        cursor: not-allowed;
    }

    .footer-actions {
        display: flex;
        justify-content: flex-end;
        padding-top: 0.5rem;
    }
</style>

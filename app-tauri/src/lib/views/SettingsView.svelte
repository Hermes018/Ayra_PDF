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
        padding: 2rem;
        max-width: 1200px;
        margin: 0 auto;
        color: #cdd6f4;
        display: flex;
        flex-direction: column;
        gap: 1.5rem;
    }

    .header h2 {
        font-size: 1.8rem;
        margin: 0 0 0.25rem 0;
        color: #cba6f7;
    }

    .subtitle {
        color: #a6adc8;
        margin: 0;
        font-size: 0.95rem;
    }

    .banner {
        padding: 0.75rem 1rem;
        border-radius: 8px;
        font-size: 0.9rem;
    }

    .banner.success {
        background-color: rgba(166, 227, 161, 0.15);
        border: 1px solid #a6e3a1;
        color: #a6e3a1;
    }

    .banner.error {
        background-color: rgba(243, 139, 168, 0.15);
        border: 1px solid #f38ba8;
        color: #f38ba8;
    }

    .settings-grid {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
        gap: 1.5rem;
    }

    .card {
        background-color: var(--surface-bg, #1e1e2e);
        border: 1px solid var(--border-color, #313244);
        border-radius: 12px;
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
        color: var(--text-color, #f5e0dc);
    }

    .description {
        margin: 0;
        font-size: 0.85rem;
        color: var(--text-muted, #9399b2);
    }

    .tier-options {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
        gap: 0.75rem;
    }

    .tier-btn {
        background-color: var(--surface-elevated, #252538);
        border: 1px solid var(--border-color, #313244);
        border-radius: 8px;
        padding: 0.75rem;
        text-align: left;
        cursor: pointer;
        color: var(--text-color, #cdd6f4);
        transition: all 0.15s ease;
    }

    .tier-btn:hover {
        border-color: var(--primary-color, #cba6f7);
        background-color: var(--surface-elevated, #2e2e42);
    }

    .tier-btn.active {
        border-color: var(--primary-color, #cba6f7);
        background-color: rgba(203, 166, 247, 0.15);
        color: var(--primary-color, #cba6f7);
    }

    .tier-name {
        font-weight: 600;
        font-size: 0.95rem;
        margin-bottom: 0.2rem;
    }

    .tier-desc {
        font-size: 0.75rem;
        color: var(--text-muted, #9399b2);
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
        color: var(--text-color, #bac2de);
    }

    select, input[type="text"] {
        background-color: var(--input-bg, #252538);
        border: 1px solid var(--input-border, #313244);
        border-radius: 6px;
        padding: 0.6rem 0.8rem;
        color: var(--input-text, #cdd6f4);
        font-size: 0.9rem;
        outline: none;
    }

    select option {
        background-color: var(--option-bg, #1e293b);
        color: var(--option-text, #f8fafc);
        padding: 0.4rem;
    }

    select:focus, input[type="text"]:focus {
        border-color: var(--primary-color, #cba6f7);
    }

    .checkbox-row {
        display: flex;
        align-items: flex-start;
        gap: 0.6rem;
        padding-top: 0.25rem;
    }

    .checkbox-row input[type="checkbox"] {
        margin-top: 0.2rem;
        accent-color: var(--primary-color, #cba6f7);
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
        color: var(--text-muted, #9399b2);
    }

    .theme-buttons {
        display: flex;
        gap: 0.5rem;
    }

    .theme-btn {
        flex: 1;
        padding: 0.6rem;
        background-color: var(--surface-elevated, #252538);
        border: 1px solid var(--border-color, #313244);
        border-radius: 6px;
        color: var(--text-color, #cdd6f4);
        cursor: pointer;
        font-size: 0.85rem;
        transition: all 0.15s ease;
    }

    .theme-btn:hover {
        border-color: var(--primary-color, #89b4fa);
    }

    .theme-btn.active {
        border-color: var(--primary-color, #89b4fa);
        background-color: rgba(137, 180, 250, 0.15);
        color: var(--primary-color, #89b4fa);
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
        background-color: #252538;
        border: 1px solid #45475a;
        border-radius: 6px;
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
        background-color: #313244;
        color: #fab387;
    }

    .size-pill {
        color: #a6e3a1;
    }

    .hash-row {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        font-size: 0.75rem;
        color: #9399b2;
    }

    .hash-row code {
        background-color: #181825;
        padding: 0.1rem 0.3rem;
        border-radius: 4px;
        color: #cdd6f4;
        word-break: break-all;
    }

    .license-panel {
        background-color: #1e1e2e;
        border: 1px solid #313244;
        border-radius: 6px;
        padding: 0.75rem;
        display: flex;
        flex-direction: column;
        gap: 0.6rem;
    }

    .license-panel h4 {
        margin: 0;
        font-size: 0.9rem;
        font-weight: 600;
        color: #cdd6f4;
    }

    .card-header-actions {
        display: flex;
        justify-content: space-between;
        align-items: flex-start;
    }

    .log-viewer {
        background-color: #11111b;
        border: 1px solid #313244;
        border-radius: 6px;
        padding: 0.75rem;
        height: 180px;
        overflow-y: auto;
        font-family: monospace;
        font-size: 0.8rem;
        color: #a6adc8;
        display: flex;
        flex-direction: column;
        gap: 0.2rem;
    }

    .log-empty {
        color: #585b70;
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
        border-top: 1px solid #313244;
    }

    .export-row {
        display: flex;
        align-items: center;
        gap: 0.75rem;
    }

    .export-row label {
        font-size: 0.85rem;
        color: #bac2de;
        white-space: nowrap;
    }

    .export-row input {
        flex: 1;
    }

    .btn {
        padding: 0.6rem 1rem;
        border-radius: 6px;
        border: none;
        cursor: pointer;
        font-size: 0.85rem;
        font-weight: 500;
        transition: all 0.15s ease;
    }

    .btn.primary {
        background-color: #cba6f7;
        color: #11111b;
    }

    .btn.primary:hover:not(:disabled) {
        background-color: #dcb8ff;
    }

    .btn.secondary {
        background-color: #313244;
        color: #cdd6f4;
    }

    .btn.secondary:hover:not(:disabled) {
        background-color: #45475a;
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

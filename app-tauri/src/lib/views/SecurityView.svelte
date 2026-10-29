<script lang="ts">
    import { appState } from '$lib/state.svelte';
    import { IpcClient } from '$lib/ipc';
    import type { JobRequest } from '$lib/bindings/JobRequest';
    import type { Permissions } from '$lib/bindings/Permissions';

    let currentPath = $derived(appState.currentFile);

    // Protect State
    let protectPassword = $state("");
    let allowPrint = $state(true);
    let allowCopy = $state(false);
    let allowEdit = $state(false);

    // Unlock State
    let unlockPassword = $state("");

    // Redact State
    let redactPage = $state(0);
    let redactX = $state(0);
    let redactY = $state(0);
    let redactW = $state(100);
    let redactH = $state(100);

    async function handleProtect() {
        if (!currentPath || !protectPassword) return;

        const outputPath = currentPath.replace(/\.pdf$/i, `_protected.pdf`);
        const permissions: Permissions = {
            allow_print: allowPrint,
            allow_copy: allowCopy,
            allow_edit: allowEdit,
        };

        const req: JobRequest = {
            op: "ProtectDocument",
            input_path: currentPath,
            output_path: outputPath,
            password: protectPassword,
            permissions
        };

        try {
            const jobId = await IpcClient.submitJob(req);
            console.log("Protect job submitted with ID:", jobId);
            protectPassword = "";
        } catch (e) {
            console.error("Failed to submit protect job:", e);
        }
    }

    async function handleUnlock() {
        if (!currentPath || !unlockPassword) return;

        const outputPath = currentPath.replace(/\.pdf$/i, `_unlocked.pdf`);
        
        const req: JobRequest = {
            op: "UnlockDocument",
            input_path: currentPath,
            output_path: outputPath,
            password: unlockPassword
        };

        try {
            const jobId = await IpcClient.submitJob(req);
            console.log("Unlock job submitted with ID:", jobId);
            unlockPassword = "";
        } catch (e) {
            console.error("Failed to submit unlock job:", e);
        }
    }

    async function handleRedact() {
        if (!currentPath) return;

        const outputPath = currentPath.replace(/\.pdf$/i, `_redacted.pdf`);
        
        const req: JobRequest = {
            op: "RedactDocument",
            input_path: currentPath,
            output_path: outputPath,
            regions: [
                {
                    page: redactPage,
                    x: redactX,
                    y: redactY,
                    width: redactW,
                    height: redactH
                }
            ]
        };

        try {
            const jobId = await IpcClient.submitJob(req);
            console.log("Redact job submitted with ID:", jobId);
        } catch (e) {
            console.error("Failed to submit redact job:", e);
        }
    }
</script>

<div class="view-container">
    <div class="header">
        <h2>PDF Security</h2>
    </div>

    <div class="content">
        <div class="sidebar">
            <div class="tool-section">
                <h4>Protect Document</h4>
                {#if currentPath}
                    <p class="file-label">Active: {currentPath.split(/[\/\\]/).pop()}</p>
                    
                    <div class="form-group">
                        <label for="protectPassword">Password (User & Owner)</label>
                        <input id="protectPassword" type="password" bind:value={protectPassword} placeholder="Enter secure password" />
                    </div>

                    <div class="form-group permissions-group">
                        <span class="group-label">Permissions (Owner capabilities)</span>
                        <label class="checkbox-label">
                            <input type="checkbox" bind:checked={allowPrint} />
                            Allow Printing
                        </label>
                        <label class="checkbox-label">
                            <input type="checkbox" bind:checked={allowCopy} />
                            Allow Copying
                        </label>
                        <label class="checkbox-label">
                            <input type="checkbox" bind:checked={allowEdit} />
                            Allow Editing
                        </label>
                    </div>

                    <button class="action-button" onclick={handleProtect} disabled={!protectPassword}>
                        Protect PDF
                    </button>
                    
                    <p class="help-text">
                        Uses AES-256 natively via pdfcraft_crypt. Save your password!
                    </p>
                {:else}
                    <p class="empty-text">Select or drag a file to protect.</p>
                {/if}
            </div>

            <div class="tool-section">
                <h4>Unlock Document</h4>
                {#if currentPath}
                    <div class="form-group">
                        <label for="unlockPassword">Document Password</label>
                        <input id="unlockPassword" type="password" bind:value={unlockPassword} placeholder="Enter password to unlock" />
                    </div>

                    <button class="action-button secondary-button" onclick={handleUnlock} disabled={!unlockPassword}>
                        Remove Security
                    </button>
                    <p class="help-text">
                        Creates an unlocked copy.
                    </p>
                {:else}
                    <p class="empty-text">Select or drag a file to unlock.</p>
                {/if}
            </div>

            <div class="tool-section">
                <h4>Semantic Redaction</h4>
                {#if currentPath}
                    <div class="form-group row-group">
                        <div class="col">
                            <label for="redactPage">Page (0-idx)</label>
                            <input id="redactPage" type="number" min="0" bind:value={redactPage} />
                        </div>
                    </div>
                    <div class="form-group row-group">
                        <div class="col">
                            <label for="redactX">X</label>
                            <input id="redactX" type="number" bind:value={redactX} />
                        </div>
                        <div class="col">
                            <label for="redactY">Y</label>
                            <input id="redactY" type="number" bind:value={redactY} />
                        </div>
                    </div>
                    <div class="form-group row-group">
                        <div class="col">
                            <label for="redactW">Width</label>
                            <input id="redactW" type="number" bind:value={redactW} />
                        </div>
                        <div class="col">
                            <label for="redactH">Height</label>
                            <input id="redactH" type="number" bind:value={redactH} />
                        </div>
                    </div>

                    <button class="action-button danger-button" onclick={handleRedact}>
                        Apply Redaction
                    </button>
                    <p class="help-text">
                        Mathematically destroys text and images in the selected region. Cannot be undone!
                    </p>
                {:else}
                    <p class="empty-text">Select or drag a file to redact.</p>
                {/if}
            </div>
        </div>
        
        <div class="viewer-main">
            {#if currentPath}
                <div class="placeholder-preview">
                    <p>Security preview & properties will be displayed here in a future update.</p>
                </div>
            {:else}
                <div class="empty-state">
                    <p>No file selected.</p>
                </div>
            {/if}
        </div>
    </div>
</div>

<style>
    .view-container {
        display: flex;
        flex-direction: column;
        height: 100%;
        padding: 1rem;
        gap: 1rem;
    }

    .header h2 {
        margin: 0;
        font-size: 1.5rem;
        color: var(--text-color);
    }

    .content {
        display: flex;
        flex: 1;
        gap: 1.5rem;
        min-height: 0;
    }

    .sidebar {
        width: 300px;
        display: flex;
        flex-direction: column;
        gap: 1.5rem;
        overflow-y: auto;
    }

    .tool-section {
        background: var(--surface-bg);
        border: 1px solid var(--border-color);
        border-radius: 8px;
        padding: 1rem;
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
    }

    .tool-section h4 {
        margin: 0;
        color: var(--text-color);
        font-size: 1.1rem;
    }

    .file-label {
        font-size: 0.85rem;
        color: var(--primary-color);
        word-break: break-all;
        margin: 0;
    }

    .form-group {
        display: flex;
        flex-direction: column;
        gap: 0.4rem;
    }

    .form-group label,
    .form-group .group-label {
        font-size: 0.85rem;
        color: var(--text-muted);
    }

    .form-group input[type="password"],
    .form-group input[type="number"] {
        padding: 0.5rem;
        border: 1px solid var(--border-color);
        border-radius: 4px;
        background: var(--bg-color);
        color: var(--text-color);
    }
    
    .row-group {
        flex-direction: row;
        gap: 0.5rem;
    }
    
    .col {
        display: flex;
        flex-direction: column;
        flex: 1;
        gap: 0.2rem;
    }
    
    .permissions-group {
        margin-top: 0.5rem;
    }

    .checkbox-label {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        font-size: 0.9rem;
        color: var(--text-color);
        cursor: pointer;
    }

    .action-button {
        background: var(--primary-color);
        color: white;
        border: none;
        border-radius: 4px;
        padding: 0.75rem;
        font-weight: 600;
        cursor: pointer;
        transition: opacity 0.2s;
        margin-top: 0.5rem;
    }
    
    .action-button:disabled {
        opacity: 0.5;
        cursor: not-allowed;
    }

    .action-button:hover:not(:disabled) {
        opacity: 0.9;
    }
    
    .secondary-button {
        background: var(--glass-bg-elevated);
        color: var(--text-primary);
    }

    .danger-button {
        background: var(--semantic-danger);
        color: var(--text-on-accent);
    }

    .help-text {
        font-size: 0.8rem;
        color: var(--text-muted);
        margin: 0;
        line-height: 1.4;
    }

    .empty-text {
        color: var(--text-muted);
        font-style: italic;
        margin: 0;
        font-size: 0.9rem;
    }

    .viewer-main {
        flex: 1;
        background: var(--surface-bg);
        border: 1px solid var(--border-color);
        border-radius: 8px;
        display: flex;
        align-items: center;
        justify-content: center;
        overflow: hidden;
    }

    .placeholder-preview, .empty-state {
        color: var(--text-muted);
        text-align: center;
        padding: 2rem;
    }
</style>

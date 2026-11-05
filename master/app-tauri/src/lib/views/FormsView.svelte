<script lang="ts">
    import { appState } from '$lib/state.svelte';
    import { IpcClient } from '$lib/ipc';
    import type { JobRequest } from '$lib/bindings/JobRequest';
    import { open } from '@tauri-apps/plugin-dialog';

    let currentPath = $derived(appState.currentFile);

    // Forms Fill State
    let formFieldName = $state("Text1");
    let formFieldValue = $state("John Doe");

    // Sign State
    let p12Path = $state("");
    let signPassword = $state("");

    async function handleFlattenForms() {
        if (!currentPath) return;

        const outputPath = currentPath.replace(/\.pdf$/i, `_forms_flattened.pdf`);
        const req: JobRequest = {
            op: "FlattenForms",
            input_path: currentPath,
            output_path: outputPath,
        };

        try {
            const jobId = await IpcClient.submitJob(req);
            console.log("Flatten forms job submitted with ID:", jobId);
        } catch (e) {
            console.error("Failed to submit flatten forms job:", e);
        }
    }

    async function handleFillForm() {
        if (!currentPath) return;
        if (!formFieldName.trim()) return;

        const outputPath = currentPath.replace(/\.pdf$/i, `_filled.pdf`);
        const fields: Record<string, string> = {};
        fields[formFieldName] = formFieldValue;

        const req: JobRequest = {
            op: "FillForm",
            input_path: currentPath,
            output_path: outputPath,
            fields
        };

        try {
            const jobId = await IpcClient.submitJob(req);
            console.log("Fill form job submitted with ID:", jobId);
        } catch (e) {
            console.error("Failed to submit fill form job:", e);
        }
    }

    async function selectP12() {
        try {
            const selected = await open({
                title: "Select PKCS#12 Certificate",
                filters: [{
                    name: "Certificates",
                    extensions: ["p12", "pfx"]
                }]
            });
            if (typeof selected === 'string') {
                p12Path = selected;
            }
        } catch (e) {
            console.error("Failed to select p12:", e);
        }
    }

    async function handleSign() {
        if (!currentPath || !p12Path || !signPassword) return;

        const outputPath = currentPath.replace(/\.pdf$/i, `_signed.pdf`);
        const req: JobRequest = {
            op: "SignDocument",
            input_path: currentPath,
            output_path: outputPath,
            p12_path: p12Path,
            password: signPassword
        };

        try {
            const jobId = await IpcClient.submitJob(req);
            console.log("Sign job submitted with ID:", jobId);
        } catch (e) {
            console.error("Failed to submit sign job:", e);
        }
    }
</script>

<div class="forms-view">
    <header class="view-header">
        <h1>Forms & Signatures</h1>
        <p>Fill interactive forms, flatten fields, and digitally sign documents natively.</p>
    </header>

    {#if !currentPath}
        <div class="empty-state">
            <div class="empty-icon">📝</div>
            <h3>No Document Selected</h3>
            <p>Please select or open a PDF document to use forms and signatures.</p>
        </div>
    {:else}
        <div class="content-grid">
            <!-- Form Filling Section -->
            <div class="tools-card">
                <div class="card-header">
                    <span class="icon">✏️</span>
                    <h3>Fill Form</h3>
                </div>
                <div class="card-body">
                    <p class="description">Fill out text fields in the document.</p>
                    
                    <div class="form-group">
                        <label for="fieldName">Field Name</label>
                        <input id="fieldName" type="text" bind:value={formFieldName} placeholder="e.g. Text1" class="input-field" />
                    </div>

                    <div class="form-group">
                        <label for="fieldValue">Field Value</label>
                        <input id="fieldValue" type="text" bind:value={formFieldValue} placeholder="Enter value..." class="input-field" />
                    </div>

                    <button class="action-btn" onclick={handleFillForm} disabled={!formFieldName.trim()}>
                        Apply Value
                    </button>
                </div>
            </div>

            <!-- Forms Management -->
            <div class="tools-card">
                <div class="card-header">
                    <span class="icon">🧱</span>
                    <h3>Manage Forms</h3>
                </div>
                <div class="card-body">
                    <p class="description">Bake form fields permanently into the page, removing interactivity.</p>
                    
                    <button class="action-btn secondary" onclick={handleFlattenForms}>
                        Flatten All Forms
                    </button>
                </div>
            </div>

            <!-- Signatures Section -->
            <div class="tools-card sign-card">
                <div class="card-header">
                    <span class="icon">✒️</span>
                    <h3>Digital Signature</h3>
                </div>
                <div class="card-body">
                    <p class="description">Cryptographically sign the document using a local .p12 or .pfx certificate.</p>
                    
                    <div class="form-group">
                        <label for="cert-path">Certificate (.p12 / .pfx)</label>
                        <div class="file-picker">
                            <input id="cert-path" type="text" readonly value={p12Path} placeholder="Select a certificate..." class="input-field flex-1" />
                            <button class="browse-btn" onclick={selectP12} aria-label="Browse for certificate file">Browse</button>
                        </div>
                    </div>

                    <div class="form-group">
                        <label for="password">Password</label>
                        <input id="password" type="password" bind:value={signPassword} placeholder="Certificate password" class="input-field" />
                    </div>

                    <button class="action-btn primary" onclick={handleSign} disabled={!p12Path || !signPassword}>
                        Sign Document
                    </button>
                </div>
            </div>
        </div>
    {/if}
</div>

<style>
    .forms-view {
        padding: 2rem;
        max-width: 1200px;
        margin: 0 auto;
        color: var(--text-primary);
    }

    .view-header {
        margin-bottom: 2rem;
    }

    .view-header h1 {
        margin: 0 0 0.5rem 0;
        font-size: 2rem;
        font-weight: 600;
        color: var(--text-primary);
    }

    .view-header p {
        margin: 0;
        color: var(--text-secondary);
    }

    .empty-state {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        padding: 4rem 2rem;
        background-color: var(--glass-bg-surface);
        border-radius: var(--radius-lg, 14px);
        border: 1px dashed var(--glass-border-subtle);
        text-align: center;
    }

    .empty-icon {
        font-size: 4rem;
        margin-bottom: 1rem;
        opacity: 0.5;
    }

    .content-grid {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
        gap: 1.5rem;
    }

    .tools-card {
        background-color: var(--glass-bg-surface);
        border: 1px solid var(--glass-border-subtle);
        border-radius: var(--radius-lg, 14px);
        box-shadow: inset 0 1px 0 0 var(--glass-border-specular), var(--glass-shadow-sm);
        overflow: hidden;
        display: flex;
        flex-direction: column;
    }

    .sign-card {
        grid-column: 1 / -1;
    }

    .card-header {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        padding: 1.25rem;
        background-color: var(--glass-bg-input);
        border-bottom: 1px solid var(--glass-border-subtle);
    }

    .card-header h3 {
        margin: 0;
        font-size: 1.25rem;
        font-weight: 500;
        color: var(--accent-primary);
    }

    .card-header .icon {
        font-size: 1.5rem;
    }

    .card-body {
        padding: 1.5rem;
        display: flex;
        flex-direction: column;
        gap: 1.25rem;
        flex: 1;
    }

    .description {
        margin: 0;
        font-size: 0.9rem;
        color: var(--text-secondary);
        line-height: 1.4;
    }

    .form-group {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    .form-group label {
        font-size: 0.9rem;
        color: var(--text-secondary);
        font-weight: 500;
    }

    .input-field {
        background-color: var(--glass-bg-input);
        border: 1px solid var(--glass-border-subtle);
        color: var(--text-primary);
        padding: 0.75rem;
        border-radius: var(--radius-md, 10px);
        font-family: inherit;
        font-size: 0.95rem;
        transition: all var(--dur-fast, 120ms) ease;
    }

    .input-field:focus {
        outline: none;
        border-color: var(--accent-primary);
        box-shadow: 0 0 0 2px var(--accent-ring);
    }

    .file-picker {
        display: flex;
        gap: 0.5rem;
    }

    .flex-1 {
        flex: 1;
        min-width: 0;
    }

    .browse-btn {
        background-color: var(--glass-bg-input);
        color: var(--text-primary);
        border: 1px solid var(--glass-border-subtle);
        border-radius: var(--radius-sm, 8px);
        padding: 0 1rem;
        font-weight: 500;
        cursor: pointer;
        transition: background-color var(--dur-fast, 120ms) ease;
    }

    .browse-btn:hover {
        background-color: var(--glass-bg-hover);
    }

    .action-btn {
        margin-top: auto;
        padding: 0.75rem 1rem;
        border-radius: var(--radius-md, 10px);
        font-weight: 600;
        font-size: 1rem;
        cursor: pointer;
        transition: all var(--dur-fast, 120ms) ease;
        border: none;
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 0.5rem;
    }

    .action-btn:disabled {
        opacity: 0.5;
        cursor: not-allowed;
    }

    .action-btn:not(.secondary):not(.primary) {
        background-color: var(--accent-primary);
        color: var(--text-on-accent);
    }

    .action-btn:not(.secondary):not(.primary):hover:not(:disabled) {
        opacity: 0.9;
    }

    .action-btn.secondary {
        background-color: transparent;
        border: 1px solid var(--semantic-danger);
        color: var(--semantic-danger);
    }

    .action-btn.secondary:hover {
        background-color: var(--glass-bg-hover);
    }

    .action-btn.primary {
        background-color: var(--semantic-success);
        color: var(--text-on-accent);
    }

    .action-btn.primary:hover:not(:disabled) {
        opacity: 0.9;
    }
</style>

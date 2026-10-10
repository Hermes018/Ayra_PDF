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
                        <label>Certificate (.p12 / .pfx)</label>
                        <div class="file-picker">
                            <input type="text" readonly value={p12Path} placeholder="Select a certificate..." class="input-field flex-1" />
                            <button class="browse-btn" onclick={selectP12}>Browse</button>
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
        color: #cdd6f4;
    }

    .view-header {
        margin-bottom: 2rem;
    }

    .view-header h1 {
        margin: 0 0 0.5rem 0;
        font-size: 2rem;
        font-weight: 600;
        color: #f5c2e7;
    }

    .view-header p {
        margin: 0;
        color: #a6adc8;
    }

    .empty-state {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        padding: 4rem 2rem;
        background-color: #1e1e2e;
        border-radius: 12px;
        border: 1px dashed #45475a;
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
        background-color: #1e1e2e;
        border: 1px solid #313244;
        border-radius: 12px;
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
        background-color: #181825;
        border-bottom: 1px solid #313244;
    }

    .card-header h3 {
        margin: 0;
        font-size: 1.25rem;
        font-weight: 500;
        color: #89b4fa;
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
        color: #a6adc8;
        line-height: 1.4;
    }

    .form-group {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    .form-group label {
        font-size: 0.9rem;
        color: #bac2de;
        font-weight: 500;
    }

    .input-field {
        background-color: #11111b;
        border: 1px solid #45475a;
        color: #cdd6f4;
        padding: 0.75rem;
        border-radius: 6px;
        font-family: inherit;
        font-size: 0.95rem;
        transition: all 0.2s;
    }

    .input-field:focus {
        outline: none;
        border-color: #89b4fa;
        box-shadow: 0 0 0 2px rgba(137, 180, 250, 0.2);
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
        background-color: #313244;
        color: #cdd6f4;
        border: none;
        border-radius: 6px;
        padding: 0 1rem;
        font-weight: 500;
        cursor: pointer;
        transition: background-color 0.2s;
    }

    .browse-btn:hover {
        background-color: #45475a;
    }

    .action-btn {
        margin-top: auto;
        padding: 0.75rem 1rem;
        border-radius: 6px;
        font-weight: 600;
        font-size: 1rem;
        cursor: pointer;
        transition: all 0.2s;
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
        background-color: #89b4fa;
        color: #11111b;
    }

    .action-btn:not(.secondary):not(.primary):hover:not(:disabled) {
        background-color: #b4befe;
    }

    .action-btn.secondary {
        background-color: transparent;
        border: 1px solid #f38ba8;
        color: #f38ba8;
    }

    .action-btn.secondary:hover {
        background-color: rgba(243, 139, 168, 0.1);
    }

    .action-btn.primary {
        background-color: #a6e3a1;
        color: #11111b;
    }

    .action-btn.primary:hover:not(:disabled) {
        background-color: #94e2d5;
    }
</style>

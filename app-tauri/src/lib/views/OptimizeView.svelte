<script lang="ts">
    import { appState } from '$lib/state.svelte';
    import { IpcClient } from '$lib/ipc';
    import type { JobRequest } from '$lib/bindings/JobRequest';
    import type { OptimizeProfile } from '$lib/bindings/OptimizeProfile';

    let currentPath = $derived(appState.currentFile);
    let activeFiles = $derived(appState.activeFiles);

    let selectedProfile = $state<OptimizeProfile>("Medium");

    async function handleCompress() {
        if (!currentPath) return;

        const outputPath = currentPath.replace(/\.pdf$/i, `_compressed_${selectedProfile}.pdf`);
        const req: JobRequest = {
            op: "Compress",
            input_path: currentPath,
            output_path: outputPath,
            profile: selectedProfile
        };

        try {
            const jobId = await IpcClient.submitJob(req);
            console.log("Compress job submitted with ID:", jobId);
        } catch (e) {
            console.error("Failed to submit compress job:", e);
        }
    }
</script>

<div class="view-container">
    <div class="header">
        <h2>Optimize PDF</h2>
    </div>

    <div class="content">
        <div class="sidebar">
            <div class="tool-section">
                <h4>Compress Document</h4>
                {#if currentPath}
                    <p class="file-label">Active: {currentPath.split(/[\/\\]/).pop()}</p>
                    
                    <label class="profile-label" for="profileSelect">Compression Profile:</label>
                    <select id="profileSelect" class="profile-select" bind:value={selectedProfile}>
                        <option value="Low">Low (High Quality, Large Size)</option>
                        <option value="Medium">Medium (Balanced)</option>
                        <option value="High">High (Low Quality, Small Size)</option>
                    </select>

                    <button class="action-button" onclick={handleCompress}>
                        Compress Now
                    </button>
                    
                    <p class="help-text">
                        The compressed file will be saved alongside the original.
                    </p>
                {:else}
                    <p class="empty-text">Select or drag a file to compress.</p>
                {/if}
            </div>
        </div>
        
        <div class="viewer-main">
            {#if currentPath}
                <div class="placeholder-preview">
                    <p>Compression preview & sizes will be displayed here in a future update.</p>
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

    .header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        background-color: #1e1e2e;
        padding: 1rem 1.5rem;
        border-radius: 8px;
        border: 1px solid #313244;
    }

    h2 {
        margin: 0;
        font-size: 1.5rem;
        color: #cba6f7;
    }

    .content {
        display: flex;
        flex: 1;
        gap: 1rem;
        min-height: 0;
    }

    .sidebar {
        width: 300px;
        background-color: #181825;
        border-radius: 8px;
        border: 1px solid #313244;
        padding: 1rem;
        display: flex;
        flex-direction: column;
        gap: 1.5rem;
        overflow-y: auto;
    }

    .viewer-main {
        flex: 1;
        background-color: #11111b;
        border-radius: 8px;
        border: 1px solid #313244;
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .tool-section {
        background-color: #1e1e2e;
        padding: 1rem;
        border-radius: 6px;
        border: 1px solid #313244;
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
    }

    .tool-section h4 {
        margin: 0 0 0.5rem 0;
        color: #f5c2e7;
        font-size: 1.1rem;
    }

    .file-label {
        margin: 0;
        color: #a6adc8;
        font-size: 0.9rem;
        word-break: break-all;
    }

    .profile-label {
        font-size: 0.9rem;
        color: #bac2de;
    }

    .profile-select {
        width: 100%;
        background-color: #11111b;
        border: 1px solid #45475a;
        color: #cdd6f4;
        padding: 0.5rem;
        border-radius: 4px;
        outline: none;
    }

    .action-button {
        width: 100%;
        background-color: #cba6f7;
        color: #11111b;
        border: none;
        padding: 0.5rem 1rem;
        border-radius: 4px;
        font-weight: 600;
        cursor: pointer;
        transition: background-color 0.2s;
    }

    .action-button:hover {
        background-color: #b4befe;
    }

    .help-text {
        font-size: 0.8rem;
        color: #6c7086;
        margin-top: 0.5rem;
        line-height: 1.4;
    }

    .empty-text, .empty-state p {
        color: #6c7086;
        text-align: center;
        font-style: italic;
    }
    
    .placeholder-preview {
        color: #a6adc8;
        text-align: center;
    }
</style>

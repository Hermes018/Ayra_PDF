<script lang="ts">
    import { appState } from '$lib/state.svelte';
    import { IpcClient } from '$lib/ipc';
    import type { JobRequest } from '$lib/bindings/JobRequest';
    import type { WatermarkSettings } from '$lib/bindings/WatermarkSettings';
    import type { AnnotationDef } from '$lib/bindings/AnnotationDef';
    import type { AnnotationShape } from '$lib/bindings/AnnotationShape';

    let currentPath = $derived(appState.currentFile);

    // Watermark State
    let watermarkText = $state("CONFIDENTIAL");
    let watermarkOpacity = $state(0.5);
    let watermarkRotation = $state(45.0); // degrees
    
    // Annotation State
    let stickyNoteText = $state("Please review this section.");
    let annotX = $state(100.0);
    let annotY = $state(100.0);
    let authorName = $state("Admin");

    async function handleWatermark() {
        if (!currentPath || !watermarkText) return;

        const outputPath = currentPath.replace(/\.pdf$/i, `_watermarked.pdf`);
        const settings: WatermarkSettings = {
            text: watermarkText,
            opacity: watermarkOpacity,
            rotation: watermarkRotation,
            color: [0.8, 0.2, 0.2] // Red-ish watermark
        };

        const req: JobRequest = {
            op: "WatermarkDocument",
            input_path: currentPath,
            output_path: outputPath,
            settings
        };

        try {
            const jobId = await IpcClient.submitJob(req);
            console.log("Watermark job submitted with ID:", jobId);
        } catch (e) {
            console.error("Failed to submit watermark job:", e);
        }
    }

    async function handleStickyNote() {
        if (!currentPath || !stickyNoteText) return;

        const outputPath = currentPath.replace(/\.pdf$/i, `_annotated.pdf`);
        
        const shape: AnnotationShape = {
            StickyNote: {
                x: annotX,
                y: annotY,
                text: stickyNoteText
            }
        };

        const annotation: AnnotationDef = {
            page: 0,
            shape,
            author: authorName
        };

        const req: JobRequest = {
            op: "AnnotateDocument",
            input_path: currentPath,
            output_path: outputPath,
            annotation
        };

        try {
            const jobId = await IpcClient.submitJob(req);
            console.log("Sticky note job submitted with ID:", jobId);
        } catch (e) {
            console.error("Failed to submit sticky note job:", e);
        }
    }

    async function handleHighlight() {
        if (!currentPath) return;

        const outputPath = currentPath.replace(/\.pdf$/i, `_highlighted.pdf`);
        
        // Example quads for a highlight box (top-left, top-right, bottom-left, bottom-right)
        const shape: AnnotationShape = {
            Highlight: {
                quads: [[100.0, 200.0, 300.0, 200.0, 100.0, 180.0, 300.0, 180.0]]
            }
        };

        const annotation: AnnotationDef = {
            page: 0,
            shape,
            author: authorName
        };

        const req: JobRequest = {
            op: "AnnotateDocument",
            input_path: currentPath,
            output_path: outputPath,
            annotation
        };

        try {
            const jobId = await IpcClient.submitJob(req);
            console.log("Highlight job submitted with ID:", jobId);
        } catch (e) {
            console.error("Failed to submit highlight job:", e);
        }
    }

    async function handleFlatten() {
        if (!currentPath) return;

        const outputPath = currentPath.replace(/\.pdf$/i, `_flattened.pdf`);
        
        const req: JobRequest = {
            op: "FlattenComments",
            input_path: currentPath,
            output_path: outputPath
        };

        try {
            const jobId = await IpcClient.submitJob(req);
            console.log("Flatten comments job submitted with ID:", jobId);
        } catch (e) {
            console.error("Failed to submit flatten job:", e);
        }
    }
</script>

<div class="view-container">
    <div class="header">
        <h2>Edit & Annotate PDF</h2>
    </div>

    <div class="content">
        <div class="sidebar">
            <div class="tool-section">
                <h4>Watermark</h4>
                {#if currentPath}
                    <div class="form-group">
                        <label for="watermarkText">Text</label>
                        <input id="watermarkText" type="text" bind:value={watermarkText} placeholder="CONFIDENTIAL" />
                    </div>
                    <div class="form-group row-group">
                        <div class="sub-group">
                            <label for="watermarkOpacity">Opacity</label>
                            <input id="watermarkOpacity" type="number" step="0.1" min="0.1" max="1.0" bind:value={watermarkOpacity} />
                        </div>
                        <div class="sub-group">
                            <label for="watermarkRotation">Angle</label>
                            <input id="watermarkRotation" type="number" bind:value={watermarkRotation} />
                        </div>
                    </div>
                    <button class="action-button" onclick={handleWatermark}>
                        Add Watermark
                    </button>
                {:else}
                    <p class="empty-text">Select or drag a file to add a watermark.</p>
                {/if}
            </div>

            <div class="tool-section">
                <h4>Sticky Note</h4>
                {#if currentPath}
                    <div class="form-group">
                        <label for="stickyText">Note Content</label>
                        <input id="stickyText" type="text" bind:value={stickyNoteText} placeholder="Enter note text..." />
                    </div>
                    <div class="form-group row-group">
                        <div class="sub-group">
                            <label for="annotX">X Position</label>
                            <input id="annotX" type="number" bind:value={annotX} />
                        </div>
                        <div class="sub-group">
                            <label for="annotY">Y Position</label>
                            <input id="annotY" type="number" bind:value={annotY} />
                        </div>
                    </div>
                    <button class="action-button secondary-button" onclick={handleStickyNote}>
                        Add Sticky Note
                    </button>
                {:else}
                    <p class="empty-text">Select or drag a file to annotate.</p>
                {/if}
            </div>

            <div class="tool-section">
                <h4>Quick Tools</h4>
                {#if currentPath}
                    <p class="help-text">Foundation tools for text markup and finalizing.</p>
                    <div class="button-row">
                        <button class="action-button tertiary-button" onclick={handleHighlight}>
                            Highlight Text (Demo)
                        </button>
                        <button class="action-button danger-button" onclick={handleFlatten}>
                            Flatten Comments
                        </button>
                    </div>
                {:else}
                    <p class="empty-text">Select or drag a file to access quick tools.</p>
                {/if}
            </div>
        </div>
        
        <div class="viewer-main">
            {#if currentPath}
                <div class="canvas-placeholder">
                    <p>PDF Viewer & Interactive Canvas</p>
                    <small>Annotations and drawings will be interactive here in a future update.</small>
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
        width: 320px;
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

    .form-group {
        display: flex;
        flex-direction: column;
        gap: 0.4rem;
    }

    .row-group {
        flex-direction: row;
        gap: 1rem;
    }

    .sub-group {
        display: flex;
        flex-direction: column;
        flex: 1;
        gap: 0.4rem;
    }

    .form-group label, .sub-group label {
        font-size: 0.85rem;
        color: var(--text-muted);
    }

    .form-group input, .sub-group input {
        padding: 0.5rem;
        border: 1px solid var(--border-color);
        border-radius: 4px;
        background: var(--bg-color);
        color: var(--text-color);
        width: 100%;
        box-sizing: border-box;
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
        width: 100%;
    }
    
    .action-button:hover {
        opacity: 0.9;
    }

    .secondary-button {
        background: var(--secondary-color, #4f46e5);
    }
    
    .tertiary-button {
        background: var(--secondary-color, #10b981);
        margin-top: 0;
    }

    .danger-button {
        background: #ef4444;
        margin-top: 0;
    }

    .button-row {
        display: flex;
        gap: 0.5rem;
        flex-direction: column;
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

    .canvas-placeholder, .empty-state {
        color: var(--text-muted);
        text-align: center;
        padding: 2rem;
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }
</style>

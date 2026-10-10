<script lang="ts">
    import PdfViewer from '$lib/components/PdfViewer.svelte';
    import { appState } from '$lib/state.svelte';

    // The viewer reactive path comes directly from the global active files
    let currentPath = $derived(appState.currentFile);
</script>

<div class="view-container">
    <div class="header">
        <h2>Organize PDFs</h2>
        <div class="controls">
            <span class="active-file-path">
                {currentPath || 'No file selected. Drag and drop a PDF.'}
            </span>
        </div>
    </div>

    <div class="viewer-layout">
        <!-- Sidebar or tools can go here in the future -->
        <div class="tools-sidebar">
            <h3>Tools</h3>
            <p>Select pages to extract, delete, or reorder.</p>
        </div>
        
        <!-- Main PDF Viewer Area -->
        <div class="viewer-main">
            <PdfViewer filePath={currentPath} />
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

    .controls {
        display: flex;
        gap: 0.5rem;
    }

    .active-file-path {
        background-color: #11111b;
        padding: 0.5rem 1rem;
        border-radius: 6px;
        color: #a6adc8;
        font-family: monospace;
        font-size: 0.9rem;
        border: 1px solid #45475a;
    }

    .viewer-layout {
        display: flex;
        flex: 1;
        gap: 1rem;
        min-height: 0; /* Important for flex child scrolling */
    }

    .tools-sidebar {
        width: 250px;
        background-color: #1e1e2e;
        border: 1px solid #313244;
        border-radius: 8px;
        padding: 1rem;
    }

    .tools-sidebar h3 {
        margin-top: 0;
        color: #f38ba8;
    }

    .viewer-main {
        flex: 1;
        border: 1px solid #313244;
        border-radius: 8px;
        overflow: hidden;
    }
</style>

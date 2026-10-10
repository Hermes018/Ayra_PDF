<script lang="ts">
    import PdfViewer from '$lib/components/PdfViewer.svelte';

    let pdfPath = $state("../../corpus/basic_10_pages.pdf");
    let currentPath = $state<string | null>(null);

    function loadPdf() {
        currentPath = pdfPath;
    }
</script>

<div class="view-container">
    <div class="header">
        <h2>Organize PDFs</h2>
        <div class="controls">
            <input 
                type="text" 
                bind:value={pdfPath} 
                placeholder="Enter absolute or relative path to PDF..." 
                class="path-input"
            />
            <button onclick={loadPdf} class="btn-primary">
                Load PDF
            </button>
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
        width: 50%;
    }

    .path-input {
        flex: 1;
        padding: 0.5rem 1rem;
        border-radius: 6px;
        border: 1px solid #45475a;
        background-color: #11111b;
        color: #cdd6f4;
        outline: none;
    }

    .path-input:focus {
        border-color: #89b4fa;
    }

    .btn-primary {
        padding: 0.5rem 1.25rem;
        border-radius: 6px;
        border: none;
        background-color: #89b4fa;
        color: #11111b;
        font-weight: 600;
        cursor: pointer;
        transition: background-color 0.2s;
    }

    .btn-primary:hover {
        background-color: #74c7ec;
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

<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import { invoke } from '@tauri-apps/api/core';

    let { filePath } = $props<{ filePath: string | null }>();

    let canvas: HTMLCanvasElement;
    let loading = $state(false);
    let error = $state<string | null>(null);

    // Track the current pdf document so we can destroy it if the filePath changes
    let currentPdfDoc: any = null;

    async function renderPdf(path: string) {
        if (!path) return;
        
        loading = true;
        error = null;
        
        try {
            // @ts-ignore - pdfjsLib is loaded globally via script tag in app.html
            const pdfjs = window['pdfjs-dist/build/pdf'] || window.pdfjsLib;
            
            if (!pdfjs) {
                throw new Error("pdf.js is not loaded.");
            }

            // Set worker source
            pdfjs.GlobalWorkerOptions.workerSrc = '/pdf.worker.min.js';

            // 1. Read file bytes from Rust backend
            const bytes: number[] = await invoke('read_file_bytes', { path });
            const uint8Array = new Uint8Array(bytes);

            // 2. Load the PDF document
            const loadingTask = pdfjs.getDocument({ data: uint8Array });
            const pdf = await loadingTask.promise;
            currentPdfDoc = pdf;

            // 3. Render the first page
            const page = await pdf.getPage(1);
            
            // Calculate scale based on container width or use a fixed scale
            const scale = 1.5;
            const viewport = page.getViewport({ scale });

            const context = canvas.getContext('2d');
            if (!context) throw new Error("Could not get canvas context");

            canvas.height = viewport.height;
            canvas.width = viewport.width;

            const renderContext = {
                canvasContext: context,
                viewport: viewport
            };

            await page.render(renderContext).promise;
            
        } catch (err: any) {
            console.error("Failed to render PDF:", err);
            error = err.message || "Failed to render PDF";
        } finally {
            loading = false;
        }
    }

    // Effect to reactively load when filePath changes
    $effect(() => {
        if (filePath) {
            renderPdf(filePath);
        }
    });

    onDestroy(() => {
        if (currentPdfDoc) {
            currentPdfDoc.destroy();
        }
    });
</script>

<div class="pdf-viewer-container">
    {#if loading}
        <div class="loading-overlay">
            <span class="spinner">⏳</span> Loading PDF...
        </div>
    {/if}
    
    {#if error}
        <div class="error-message">
            {error}
        </div>
    {/if}

    <div class="canvas-wrapper" style="display: {filePath && !error ? 'flex' : 'none'}">
        <canvas bind:this={canvas}></canvas>
    </div>
    
    {#if !filePath}
        <div class="empty-state">
            <p>No PDF selected</p>
        </div>
    {/if}
</div>

<style>
    .pdf-viewer-container {
        position: relative;
        width: 100%;
        height: 100%;
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        background-color: var(--glass-bg-canvas);
        border-radius: 8px;
        overflow: hidden;
    }

    .canvas-wrapper {
        width: 100%;
        height: 100%;
        overflow: auto;
        display: flex;
        justify-content: center;
        align-items: flex-start;
        padding: 2rem;
    }

    canvas {
        box-shadow: 0 4px 6px rgba(0, 0, 0, 0.3);
        background-color: white;
    }

    .loading-overlay {
        position: absolute;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        background-color: var(--glass-bg-canvas);
        backdrop-filter: var(--glass-blur);
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        color: var(--text-primary);
        z-index: 10;
    }

    .error-message {
        color: var(--semantic-danger);
        padding: 1rem;
        background-color: var(--glass-bg-surface);
        border: 1px solid var(--semantic-danger);
        border-radius: 4px;
        margin: 2rem;
    }

    .empty-state {
        color: var(--text-secondary);
        font-size: 1.2rem;
    }
</style>

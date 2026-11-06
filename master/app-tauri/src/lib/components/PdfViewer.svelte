<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import { invoke } from '@tauri-apps/api/core';
    import ErrorState from '$lib/components/ui/ErrorState.svelte';
    import Progress from '$lib/components/ui/Progress.svelte';
    import EmptyState from '$lib/components/ui/EmptyState.svelte';

    // `onpagecount` reports the real page count once the document is open (used by screens that take a page number).
    let { filePath, onpagecount } = $props<{ filePath: string | null; onpagecount?: (pages: number) => void }>();

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
            onpagecount?.(pdf.numPages);

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
            error = 'render'; // details go to the log, not the screen
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
            <Progress value={null} label="Loading the PDF" />
        </div>
    {/if}

    {#if error}
        <ErrorState
            what="Couldn't show this PDF."
            next="The file may be damaged or no longer there. Check it, then try again."
            retry={() => filePath && renderPdf(filePath)}
            data-testid="pdf-viewer-error"
        />
    {/if}

    <div class="canvas-wrapper" style="display: {filePath && !error ? 'flex' : 'none'}">
        <canvas bind:this={canvas}></canvas>
    </div>

    {#if !filePath}
        <EmptyState title="No PDF selected" body="Open a PDF to preview it here." />
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
        border-radius: var(--radius-sm);
        overflow: hidden;
    }

    .canvas-wrapper {
        width: 100%;
        height: 100%;
        overflow: auto;
        display: flex;
        align-items: flex-start;
        padding: var(--space-8);
    }

    canvas {
        /* auto margins centre a narrow page and let a wide one start at the left edge (no clipped left side) */
        margin: 0 auto;
        flex: none;
        box-shadow: var(--glass-shadow-sm);
        background-color: var(--laser-core);
    }

    .loading-overlay {
        position: absolute;
        inset: 0;
        background-color: var(--glass-bg-canvas);
        backdrop-filter: var(--glass-blur);
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        color: var(--text-primary);
        z-index: 10;
        padding: var(--space-8);
    }
</style>

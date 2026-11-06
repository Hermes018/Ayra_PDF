<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import { invoke } from '@tauri-apps/api/core';
    import { IpcClient } from '$lib/ipc';
    import { applyOrganize } from './organizeApply';

    let { filePath } = $props<{ filePath: string | null }>();

    type PageState = {
        id: string;
        originalIndex: number;
        rotation: number;
        deleted: boolean;
        dataUrl: string;
    };

    let pages = $state<PageState[]>([]);
    let loading = $state(false);
    let error = $state<string | null>(null);
    let currentPdfDoc: any = null;

    let dragIndex = $state<number | null>(null);

    // BUG-014: truthful outcome of the last Apply (never "success" after a failed step).
    let applyStatus = $state<{ state: 'idle' | 'running' | 'success' | 'error'; message: string }>({
        state: 'idle',
        message: '',
    });

    async function loadPdf(path: string) {
        if (!path) return;
        loading = true;
        error = null;
        pages = [];
        try {
            // @ts-ignore
            const pdfjs = window['pdfjs-dist/build/pdf'] || window.pdfjsLib;
            pdfjs.GlobalWorkerOptions.workerSrc = '/pdf.worker.min.js';

            const bytes: number[] = await invoke('read_file_bytes', { path });
            const uint8Array = new Uint8Array(bytes);
            currentPdfDoc = await pdfjs.getDocument({ data: uint8Array }).promise;

            const numPages = currentPdfDoc.numPages;
            for (let i = 1; i <= numPages; i++) {
                const page = await currentPdfDoc.getPage(i);
                // Render at a small scale for thumbnails
                const viewport = page.getViewport({ scale: 0.5 });
                const canvas = document.createElement('canvas');
                canvas.width = viewport.width;
                canvas.height = viewport.height;
                const context = canvas.getContext('2d');
                await page.render({ canvasContext: context, viewport }).promise;
                
                pages.push({
                    id: `page-${i}-${Date.now()}`,
                    originalIndex: i,
                    rotation: 0,
                    deleted: false,
                    dataUrl: canvas.toDataURL('image/jpeg', 0.8)
                });
            }
        } catch (err: any) {
            error = err.message;
        } finally {
            loading = false;
        }
    }

    $effect(() => {
        if (filePath) loadPdf(filePath);
    });

    onDestroy(() => {
        // Review r3: leaving the view stops a running Apply (cancel_job), no orphan work.
        applyAbort?.abort();
        if (currentPdfDoc) currentPdfDoc.destroy();
    });

    // Drag and Drop
    function handleDragStart(e: DragEvent, index: number) {
        dragIndex = index;
        if (e.dataTransfer) {
            e.dataTransfer.effectAllowed = "move";
            // Set a transparent drag image to not obscure
            const img = new Image();
            img.src = 'data:image/gif;base64,R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7';
            e.dataTransfer.setDragImage(img, 0, 0);
        }
    }

    function handleDragOver(e: DragEvent, index: number) {
        e.preventDefault();
    }

    function handleDrop(e: DragEvent, targetIndex: number) {
        e.preventDefault();
        if (dragIndex !== null && dragIndex !== targetIndex) {
            const newPages = [...pages];
            const [dragged] = newPages.splice(dragIndex, 1);
            newPages.splice(targetIndex, 0, dragged);
            pages = newPages;
        }
        dragIndex = null;
    }

    // Actions
    function toggleDelete(index: number) {
        pages[index].deleted = !pages[index].deleted;
    }

    function rotatePage(index: number) {
        pages[index].rotation = (pages[index].rotation + 90) % 360;
    }

    let applyAbort: AbortController | null = null;

    async function applyChanges() {
        if (!filePath || applyStatus.state === 'running') return;
        applyStatus = { state: 'running', message: 'Applying changes…' };
        applyAbort = new AbortController();
        const signal = applyAbort.signal;
        const outcome = await applyOrganize(filePath, pages, (req) => IpcClient.runJob(req, { signal }).then(() => undefined));
        applyAbort = null;
        if (outcome.ok) {
            applyStatus = outcome.outputPath
                ? { state: 'success', message: `Saved as a new file: ${outcome.outputPath}` }
                : { state: 'success', message: 'Nothing to apply: no pages were moved, deleted or rotated.' };
        } else {
            const where = outcome.failedStep ? `"${outcome.failedStep}" failed` : 'Could not apply';
            const kept = outcome.completedSteps.length > 0 ? ` Completed before the failure: ${outcome.completedSteps.join('; ')}.` : '';
            applyStatus = { state: 'error', message: `${where}: ${outcome.error}.${kept} The original file is unchanged.` };
        }
    }
</script>

<div class="grid-editor-container">
    {#if loading}
        <div class="loading">Loading pages...</div>
    {:else if error}
        <div class="error">{error}</div>
    {:else if pages.length > 0}
        <div class="toolbar">
            <span class="info">{pages.filter(p => !p.deleted).length} of {pages.length} pages selected</span>
            <button
                class="apply-btn"
                data-testid="organize-apply-btn"
                onclick={applyChanges}
                disabled={pages.length === 0 || applyStatus.state === 'running'}
            >
                {applyStatus.state === 'running' ? 'Applying…' : 'Apply Changes'}
            </button>
            {#if applyStatus.state === 'running'}
                <button class="apply-btn cancel" data-testid="organize-cancel-btn" onclick={() => applyAbort?.abort()}>
                    Cancel
                </button>
            {/if}
        </div>
        {#if applyStatus.state !== 'idle'}
            <div
                class="apply-status {applyStatus.state}"
                role="status"
                aria-live="polite"
                data-testid="organize-status"
                data-state={applyStatus.state}
            >
                {applyStatus.message}
            </div>
        {/if}
        
        <div class="grid-layout">
            {#each pages as page, i (page.id)}
                <!-- svelte-ignore a11y_no_static_element_interactions -->
                <div 
                    class="page-card {page.deleted ? 'deleted' : ''} {dragIndex === i ? 'dragging' : ''}" 
                    data-testid="organize-page-card"
                    data-original-page={page.originalIndex}
                    data-rotation={page.rotation}
                    data-deleted={page.deleted}
                    draggable="true"
                    ondragstart={(e) => handleDragStart(e, i)}
                    ondragover={(e) => handleDragOver(e, i)}
                    ondrop={(e) => handleDrop(e, i)}
                >
                    <div class="page-number">{i + 1}</div>
                    
                    <div class="img-container">
                        <img 
                            src={page.dataUrl} 
                            alt="Page {page.originalIndex}" 
                            draggable="false" 
                            style="transform: rotate({page.rotation}deg)"
                        />
                    </div>

                    <div class="page-overlay">
                        <button class="action-btn" data-testid="organize-rotate-btn" onclick={() => rotatePage(i)} title="Rotate" aria-label="Rotate page {i + 1}">↻</button>
                        <button class="action-btn {page.deleted ? 'restore' : 'danger'}" data-testid="organize-delete-btn" onclick={() => toggleDelete(i)} title="Delete/Restore" aria-label="{page.deleted ? 'Restore' : 'Delete'} page {i + 1}">
                            {page.deleted ? '↩' : '×'}
                        </button>
                    </div>
                </div>
            {/each}
        </div>
    {:else}
        <div class="empty">No document loaded</div>
    {/if}
</div>

<style>
    .apply-status {
        margin: 0 0 12px;
        padding: 8px 12px;
        border-radius: 6px;
        font-size: 0.85rem;
        border: 1px solid var(--glass-border-subtle);
        background: var(--glass-bg-elevated);
        color: var(--text-primary);
    }
    .apply-status.success {
        border-color: var(--semantic-success);
    }
    .apply-status.error {
        border-color: var(--semantic-danger);
        color: var(--semantic-danger);
    }
    .grid-editor-container {
        display: flex;
        flex-direction: column;
        width: 100%;
        height: 100%;
        background-color: var(--glass-bg-canvas);
        border-radius: 8px;
        overflow: hidden;
    }

    .toolbar {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 1rem;
        background-color: var(--glass-bg-surface);
        border-bottom: 1px solid var(--glass-border-subtle);
    }

    .info {
        color: var(--text-secondary);
        font-size: 0.9rem;
    }

    .apply-btn {
        background-color: var(--semantic-success);
        color: var(--text-on-accent);
        border: none;
        padding: 0.5rem 1.5rem;
        border-radius: 4px;
        font-weight: bold;
        cursor: pointer;
        transition: opacity 0.2s;
    }

    .apply-btn.cancel {
        background-color: var(--glass-bg-elevated);
        color: var(--text-primary);
        border: 1px solid var(--glass-border-subtle);
        margin-left: 0.5rem;
    }

    .apply-btn:disabled {
        opacity: 0.5;
        cursor: not-allowed;
    }

    .apply-btn:not(:disabled):hover {
        opacity: 0.9;
    }

    .grid-layout {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
        gap: 1.5rem;
        padding: 1.5rem;
        overflow-y: auto;
        flex: 1;
        align-content: flex-start;
    }

    .page-card {
        background-color: var(--glass-bg-surface);
        border-radius: 6px;
        border: 2px solid transparent;
        position: relative;
        cursor: grab;
        display: flex;
        flex-direction: column;
        align-items: center;
        padding: 0.5rem;
        transition: border-color 0.2s, opacity 0.2s;
    }

    .page-card:active {
        cursor: grabbing;
    }

    .page-card.dragging {
        opacity: 0.4;
        border-color: var(--accent-primary);
    }

    .page-card.deleted {
        opacity: 0.5;
        border-color: var(--semantic-danger);
    }

    .page-number {
        position: absolute;
        top: -10px;
        left: -10px;
        background-color: var(--accent-primary);
        color: var(--text-on-accent);
        width: 24px;
        height: 24px;
        border-radius: 12px;
        display: flex;
        align-items: center;
        justify-content: center;
        font-size: 0.8rem;
        font-weight: bold;
        z-index: 2;
    }

    .page-card.deleted .page-number {
        background-color: var(--semantic-danger);
    }

    .img-container {
        width: 120px;
        height: 160px;
        display: flex;
        align-items: center;
        justify-content: center;
        overflow: hidden;
        background-color: var(--glass-bg-input);
        border-radius: 4px;
    }

    .img-container img {
        max-width: 100%;
        max-height: 100%;
        object-fit: contain;
        transition: transform 0.3s ease;
    }

    .page-overlay {
        position: absolute;
        bottom: 10px;
        left: 50%;
        transform: translateX(-50%);
        display: flex;
        gap: 0.5rem;
        opacity: 0;
        transition: opacity 0.2s;
        background: var(--glass-bg-elevated);
        backdrop-filter: var(--glass-blur);
        padding: 0.25rem;
        border-radius: 4px;
    }

    .page-card:hover .page-overlay {
        opacity: 1;
    }

    .action-btn {
        width: 30px;
        height: 30px;
        border: none;
        border-radius: 4px;
        background-color: var(--glass-bg-input);
        color: var(--text-primary);
        cursor: pointer;
        display: flex;
        align-items: center;
        justify-content: center;
        font-size: 1.1rem;
    }

    .action-btn:hover {
        background-color: var(--glass-bg-hover);
    }

    .action-btn.danger {
        color: var(--semantic-danger);
    }

    .action-btn.restore {
        color: var(--semantic-success);
    }

    .loading, .error, .empty {
        padding: 2rem;
        text-align: center;
        color: var(--text-secondary);
    }

    .error {
        color: var(--semantic-danger);
    }
</style>

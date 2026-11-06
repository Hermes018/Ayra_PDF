import { expect, test, vi, beforeEach, afterEach, describe } from 'vitest';
import { render, screen, cleanup, fireEvent, waitFor } from '@testing-library/svelte';

// BUG-013: in Edit Text (structural) mode a click on a PDF text block must open the editor for
// that block. #wet-layer sits above the page layer and the default Pan tool used to swallow it.

const BLOCK = { id: 'b1', pageIndex: 0, bboxPt: [72, 700, 200, 14], text: 'Hello structural', fontSize: 12 };

vi.mock('@tauri-apps/api/core', () => ({
    invoke: vi.fn(async (cmd: string) => {
        if (cmd === 'read_file_bytes') return [37, 80, 68, 70];
        if (cmd === 'studio_extract_text_blocks') return [BLOCK];
        return null;
    }),
}));

import SpatialCanvas from '../studio/SpatialCanvas.svelte';
import { appState } from '$lib/state.svelte';
import { viewport } from '$lib/core/viewportEngine.svelte';
import { tools } from '$lib/core/toolEngine.svelte';
import { shapes } from '$lib/core/shapeAndHatchEngine.svelte';

function fakePdfjs(sizes: Array<[number, number]>) {
    return {
        GlobalWorkerOptions: {},
        getDocument: () => ({
            promise: Promise.resolve({
                numPages: sizes.length,
                getPage: async (n: number) => ({
                    getViewport: ({ scale }: { scale: number }) => ({ width: sizes[n - 1][0] * scale, height: sizes[n - 1][1] * scale }),
                    render: () => ({ promise: Promise.resolve() }),
                }),
                destroy: async () => {},
            }),
        }),
    };
}

describe('Edit Text hotspot reachable by pointer (BUG-013)', () => {
    beforeEach(() => {
        (window as any).pdfjsLib = fakePdfjs([[612, 792]]);
        appState.clear();
        viewport.pages = [];
        tools.selectTool('pan'); // the default tool, which used to take the click
        shapes.editingTextBox = null as any;
        appState.addFiles(['/docs/text.pdf']);
    });

    afterEach(() => {
        cleanup();
        delete (window as any).pdfjsLib;
        appState.clear();
        viewport.pages = [];
        shapes.editingTextBox = null as any;
    });

    test('a pointer click on a text block in Edit Text mode opens that block for editing', async () => {
        render(SpatialCanvas);
        await waitFor(() => expect(viewport.pages).toHaveLength(1));
        const editBtn = await screen.findByRole('button', { name: /Edit text/ });
        await waitFor(() => expect(editBtn).toBeEnabled());
        await fireEvent.click(editBtn);
        await screen.findByRole('button', { name: /Edit text: Hello structural/ });

        const [x, y, w, h] = BLOCK.bboxPt;
        const centre = viewport.pdfToScreen('main-canvas', 0).apply({ x: x + w / 2, y: y + h / 2 });
        const wet = document.getElementById('wet-layer') as Element;
        await fireEvent.pointerDown(wet, { clientX: centre.x, clientY: centre.y, button: 0, pointerId: 1 });
        await fireEvent.pointerUp(wet, { clientX: centre.x, clientY: centre.y, button: 0, pointerId: 1 });

        const editor = await screen.findByTestId('inline-pdf-text-editor');
        expect(editor).toHaveTextContent('Structural PDF text edit');
        expect((screen.getByRole('textbox', { name: 'Annotation or PDF text content' }) as HTMLTextAreaElement).value).toBe(
            'Hello structural'
        );
    });

    test('a click on empty page area in Edit Text mode with Pan does not open a stray editor', async () => {
        render(SpatialCanvas);
        await waitFor(() => expect(viewport.pages).toHaveLength(1));
        const editBtn = await screen.findByRole('button', { name: /Edit text/ });
        await waitFor(() => expect(editBtn).toBeEnabled());
        await fireEvent.click(editBtn);
        await screen.findByRole('button', { name: /Edit text: Hello structural/ });

        const empty = viewport.pdfToScreen('main-canvas', 0).apply({ x: 400, y: 100 });
        const wet = document.getElementById('wet-layer') as Element;
        await fireEvent.pointerDown(wet, { clientX: empty.x, clientY: empty.y, button: 0, pointerId: 1 });
        await fireEvent.pointerUp(wet, { clientX: empty.x, clientY: empty.y, button: 0, pointerId: 1 });
        expect(screen.queryByTestId('inline-pdf-text-editor')).not.toBeInTheDocument();
    });
});

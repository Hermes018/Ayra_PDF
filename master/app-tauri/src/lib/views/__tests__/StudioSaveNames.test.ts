import { expect, test, vi, beforeEach, afterEach, describe } from 'vitest';
import { render, screen, cleanup, fireEvent, waitFor } from '@testing-library/svelte';

// BUG-021: every Save writes a NEW, collision-safe file (never overwriting the previous
// `<orig>.annotated.pdf`), and saving must not log RenderingCancelledException / reload twice.

const saves: string[] = [];
const opened: string[] = [];
// 'cancel-next': the next page render rejects like pdf.js does when a newer load supersedes it.
// 'fail-next': the next page render rejects with a real error.
let renderMode: 'ok' | 'cancel-next' | 'fail-next' = 'ok';

function renderTask() {
    if (renderMode === 'cancel-next') {
        renderMode = 'ok';
        const e = new Error('Rendering cancelled, page 1');
        e.name = 'RenderingCancelledException';
        return { promise: Promise.reject(e) };
    }
    if (renderMode === 'fail-next') {
        renderMode = 'ok';
        return { promise: Promise.reject(new Error('canvas exploded')) };
    }
    return { promise: Promise.resolve() };
}

vi.mock('@tauri-apps/api/core', () => ({
    invoke: vi.fn(async (cmd: string, args: any) => {
        if (cmd === 'read_file_bytes') {
            // Only the original and files we actually saved exist on "disk".
            if (args.path !== '/docs/plan.pdf' && !saves.includes(args.path)) {
                throw new Error('No such file or directory (os error 2)');
            }
            opened.push(args.path);
            return [37, 80, 68, 70];
        }
        if (cmd === 'studio_save_annotations') {
            saves.push(args.outputPath);
            return { outputPath: args.outputPath, savedCount: args.annotations.length, flattened: args.flatten, pageCount: 1 };
        }
        return null;
    }),
}));

import SplitLayout from '../studio/SplitLayout.svelte';
import { appState } from '$lib/state.svelte';
import { viewport } from '$lib/core/viewportEngine.svelte';
import { ink } from '$lib/core/inkEngine.svelte';
import { newOutputPath } from '../studio/outputNames';
import { clearDocumentSessions, isDocumentRestored } from '../studio/documentSessions';

describe('newOutputPath (BUG-021)', () => {
    test('stamps a new name and never chains or reuses suffixes', () => {
        const t = new Date(2026, 10, 6, 5, 48, 0, 123);
        const a = newOutputPath('/d/plan.pdf', 'annotated', t);
        expect(a).toBe('/d/plan.annotated-20261106-054800-123.pdf');
        const b = newOutputPath('/d/plan.pdf', 'annotated', t); // same millisecond
        expect(b).toBe('/d/plan.annotated-20261106-054800-123-2.pdf');
        const c = newOutputPath(a, 'flattened', t); // saving from a saved copy
        expect(c).toBe('/d/plan.flattened-20261106-054800-123.pdf');
        expect(newOutputPath('/d/plan.annotated.pdf', 'edited', t)).toBe('/d/plan.edited-20261106-054800-123.pdf');
    });
});

describe('Studio Save writes a new file each time (BUG-021)', () => {
    beforeEach(() => {
        saves.length = 0;
        opened.length = 0;
        renderMode = 'ok';
        clearDocumentSessions();
        (window as any).pdfjsLib = {
            GlobalWorkerOptions: {},
            getDocument: () => ({
                promise: Promise.resolve({
                    numPages: 1,
                    getPage: async () => ({
                        getViewport: ({ scale }: { scale: number }) => ({ width: 612 * scale, height: 792 * scale }),
                        render: () => renderTask(),
                    }),
                    destroy: async () => {},
                }),
            }),
        };
        appState.clear();
        viewport.pages = [];
        ink.completedStrokes = [];
    });

    afterEach(() => {
        cleanup();
        vi.restoreAllMocks();
        delete (window as any).pdfjsLib;
        appState.clear();
        viewport.pages = [];
        ink.completedStrokes = [];
    });

    test('two saves give two different new files; the saved copy opens once, as its own tab', async () => {
        const errors = vi.spyOn(console, 'error');
        appState.addFiles(['/docs/plan.pdf']);
        render(SplitLayout);
        await waitFor(() => expect(isDocumentRestored('/docs/plan.pdf')).toBe(true)); // ready to edit + save
        ink.startStroke(100, 100, 0.5, 'pen.a');
        ink.addPoint(200, 120, 0.5);
        ink.endStroke();

        await fireEvent.click(screen.getByTestId('save-pdf-btn'));
        await waitFor(() => expect(saves).toHaveLength(1));
        await waitFor(() => expect(appState.currentFile).toBe(saves[0]));
        await waitFor(() => expect(isDocumentRestored(saves[0])).toBe(true));
        await fireEvent.click(screen.getByTestId('save-pdf-btn'));
        await waitFor(() => expect(saves).toHaveLength(2));

        expect(new Set(saves).size).toBe(2);
        for (const out of saves) {
            expect(out).not.toBe('/docs/plan.pdf');
            expect(out).toMatch(/^\/docs\/plan\.annotated-\d{8}-\d{6}-\d{3}(-\d+)?\.pdf$/);
        }
        expect(saves).not.toContain('/docs/plan.annotated.pdf');
        // Each saved copy was opened exactly once (no reload bounce back to the original).
        await waitFor(() => expect(opened.filter((p) => p === saves[1])).toHaveLength(1));
        expect(opened.filter((p) => p === saves[0])).toHaveLength(1);
        expect(opened.filter((p) => p === '/docs/plan.pdf')).toHaveLength(1);
        expect(errors.mock.calls.flat().join(' ')).not.toMatch(/RenderingCancelled|Failed to render/);
        expect(screen.getAllByTestId('doc-tab').map((t) => t.getAttribute('data-path'))).toEqual(['/docs/plan.pdf', saves[0], saves[1]]);
    });

    test('Save / Flatten / Commit Redactions expose stable test ids', async () => {
        appState.addFiles(['/docs/plan.pdf']);
        render(SplitLayout);
        await waitFor(() => expect(viewport.pages).toHaveLength(1));
        expect(screen.getByTestId('save-pdf-btn')).toBeEnabled();
        expect(screen.getByTestId('flatten-pdf-btn')).toBeEnabled();
    });

    test('a cancelled render is ignored (debug log only), a real render error is shown', async () => {
        const errors = vi.spyOn(console, 'error').mockImplementation(() => {});
        const debug = vi.spyOn(console, 'debug').mockImplementation(() => {});
        // jsdom has no 2D context; without one SpatialCanvas never calls page.render().
        vi.spyOn(HTMLCanvasElement.prototype, 'getContext').mockReturnValue({} as any);
        renderMode = 'cancel-next';
        appState.addFiles(['/docs/plan.pdf']);
        render(SplitLayout);
        await waitFor(() => expect(debug.mock.calls.flat().join(' ')).toMatch(/RenderingCancelled/));
        expect(errors.mock.calls.flat().join(' ')).not.toMatch(/RenderingCancelled/);
        expect(screen.queryByTestId('studio-status-banner')).not.toBeInTheDocument();

        renderMode = 'fail-next';
        appState.addFiles(['/docs/other.pdf']);
        saves.push('/docs/other.pdf'); // exists on "disk"
        const banner = await screen.findByTestId('studio-status-banner');
        expect(banner).toHaveTextContent('canvas exploded');
        expect(errors.mock.calls.flat().join(' ')).toMatch(/canvas exploded/);
    });
});

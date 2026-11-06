import { expect, test, vi, beforeEach, afterEach, describe } from 'vitest';
import { render, screen, cleanup, fireEvent, waitFor, within } from '@testing-library/svelte';

// BUG-016: a second document (CLI / "open with" / Open PDF) must be shown, as a tab, and switching
// tabs must keep each document's view state and markups.

const saved: string[] = [];

vi.mock('@tauri-apps/api/core', () => ({
    invoke: vi.fn(async (cmd: string, args: any) => {
        if (cmd === 'read_file_bytes') {
            if (/\.(annotated|flattened)-/.test(args.path) && !saved.includes(args.path)) {
                throw new Error('No such file or directory (os error 2)');
            }
            return [args.path === '/docs/a.pdf' ? 1 : 3];
        }
        if (cmd === 'studio_save_annotations') {
            saved.push(args.outputPath);
            return { outputPath: args.outputPath, savedCount: args.annotations.length, flattened: false, pageCount: 1 };
        }
        return null;
    }),
}));

import SplitLayout from '../studio/SplitLayout.svelte';
import { appState } from '$lib/state.svelte';
import { viewport } from '$lib/core/viewportEngine.svelte';
import { ink } from '$lib/core/inkEngine.svelte';
import { clearDocumentSessions, isDocumentRestored } from '../studio/documentSessions';

function fakePdfjs() {
    return {
        GlobalWorkerOptions: {},
        getDocument: ({ data }: { data: Uint8Array }) => ({
            promise: Promise.resolve({
                numPages: data[0],
                getPage: async () => ({
                    getViewport: ({ scale }: { scale: number }) => ({ width: 612 * scale, height: 792 * scale }),
                    render: () => ({ promise: Promise.resolve() }),
                }),
                destroy: async () => {},
            }),
        }),
    };
}

function drawStroke(y: number) {
    ink.startStroke(100, y, 0.5, 'pen.a');
    ink.addPoint(200, y, 0.5);
    return ink.endStroke()!.id;
}

const view = () => viewport.views['main-canvas'];

describe('Second document is shown as a tab (BUG-016)', () => {
    beforeEach(() => {
        (window as any).pdfjsLib = fakePdfjs();
        clearDocumentSessions();
        appState.clear();
        viewport.pages = [];
        ink.completedStrokes = [];
    });

    afterEach(() => {
        cleanup();
        delete (window as any).pdfjsLib;
        appState.clear();
        viewport.pages = [];
        ink.completedStrokes = [];
    });

    test('opening B shows B; switching keeps each document’s view and markups', async () => {
        appState.addFiles(['/docs/a.pdf']);
        render(SplitLayout);
        await waitFor(() => expect(isDocumentRestored('/docs/a.pdf')).toBe(true));
        const strokeA = drawStroke(100);
        viewport.upsertView({ ...view(), zoom: 2, panX: 11, panY: 22 });

        appState.addFiles(['/docs/b.pdf']); // second file, e.g. via "open with"
        await waitFor(() => expect(isDocumentRestored('/docs/b.pdf')).toBe(true));
        expect(viewport.pages).toHaveLength(3);
        expect(appState.currentFile).toBe('/docs/b.pdf');
        expect(ink.completedStrokes).toHaveLength(0); // A's markups are not drawn on B

        const tabs = screen.getAllByTestId('doc-tab');
        expect(tabs.map((t) => t.textContent?.trim())).toEqual(['a.pdf', 'b.pdf']);
        expect(tabs[1]).toHaveAttribute('aria-selected', 'true');

        viewport.upsertView({ ...view(), zoom: 0.5, panX: 5, panY: 6 });
        const strokeB = drawStroke(300);

        await fireEvent.click(tabs[0]);
        await waitFor(() => expect(viewport.pages).toHaveLength(1));
        await waitFor(() => expect(view().zoom).toBe(2));
        expect([view().panX, view().panY]).toEqual([11, 22]);
        expect(ink.completedStrokes.map((s) => s.id)).toEqual([strokeA]);
        expect(screen.getAllByTestId('doc-tab')[0]).toHaveAttribute('aria-selected', 'true');

        await fireEvent.click(screen.getAllByTestId('doc-tab')[1]);
        await waitFor(() => expect(viewport.pages).toHaveLength(3));
        await waitFor(() => expect(view().zoom).toBe(0.5));
        expect(ink.completedStrokes.map((s) => s.id)).toEqual([strokeB]);
    });

    test('closing the active tab shows the remaining document', async () => {
        appState.addFiles(['/docs/a.pdf', '/docs/b.pdf']);
        render(SplitLayout);
        await waitFor(() => expect(viewport.pages).toHaveLength(3));
        const tabB = screen.getAllByTestId('doc-tab')[1];
        await fireEvent.click(within(tabB.parentElement as HTMLElement).getByTestId('doc-tab-close'));
        await waitFor(() => expect(viewport.pages).toHaveLength(1));
        expect(appState.activeFiles).toEqual(['/docs/a.pdf']);
        expect(appState.currentFile).toBe('/docs/a.pdf');
    });
});

describe('Closing a tab with unsaved markups asks first (F11)', () => {
    beforeEach(async () => {
        saved.length = 0;
        (window as any).pdfjsLib = fakePdfjs();
        clearDocumentSessions();
        appState.clear();
        viewport.pages = [];
        ink.completedStrokes = [];
        appState.addFiles(['/docs/a.pdf', '/docs/b.pdf']);
        render(SplitLayout);
        await waitFor(() => expect(isDocumentRestored('/docs/b.pdf')).toBe(true)); // B shown, ready
    });

    afterEach(() => {
        cleanup();
        delete (window as any).pdfjsLib;
        appState.clear();
        viewport.pages = [];
        ink.completedStrokes = [];
    });

    const closeBtn = (i: number) => screen.getAllByTestId('doc-tab-close')[i];

    test('a clean tab closes at once', async () => {
        await fireEvent.click(closeBtn(1));
        expect(screen.queryByTestId('close-tab-dialog')).not.toBeInTheDocument();
        await waitFor(() => expect(appState.activeFiles).toEqual(['/docs/a.pdf']));
    });

    test('a dirty tab is not discarded without confirmation; Cancel keeps it', async () => {
        drawStroke(300);
        await fireEvent.click(closeBtn(1));
        const dialog = await screen.findByTestId('close-tab-dialog');
        expect(dialog).toHaveTextContent('b.pdf');
        expect(dialog).toHaveTextContent(/unsaved/i);
        expect(appState.activeFiles).toEqual(['/docs/a.pdf', '/docs/b.pdf']);
        await fireEvent.click(screen.getByTestId('close-tab-cancel'));
        expect(screen.queryByTestId('close-tab-dialog')).not.toBeInTheDocument();
        expect(appState.activeFiles).toEqual(['/docs/a.pdf', '/docs/b.pdf']);
        expect(ink.completedStrokes).toHaveLength(1);
    });

    test('Discard closes the dirty tab without saving', async () => {
        drawStroke(300);
        await fireEvent.click(closeBtn(1));
        await fireEvent.click(await screen.findByTestId('close-tab-discard'));
        await waitFor(() => expect(appState.activeFiles).toEqual(['/docs/a.pdf']));
        expect(saved).toHaveLength(0);
    });

    test('Save writes a new file first, then closes the tab', async () => {
        drawStroke(300);
        await fireEvent.click(closeBtn(1));
        await fireEvent.click(await screen.findByTestId('close-tab-save'));
        await waitFor(() => expect(saved).toHaveLength(1));
        await waitFor(() => expect(appState.activeFiles).not.toContain('/docs/b.pdf'));
        expect(appState.activeFiles).toContain(saved[0]); // the saved copy stays open
    });

    test('an inactive dirty tab also asks (its markups live in the stashed session)', async () => {
        drawStroke(300); // on B
        await fireEvent.click(screen.getAllByTestId('doc-tab')[0]); // switch to A
        await waitFor(() => expect(isDocumentRestored('/docs/a.pdf')).toBe(true));
        await fireEvent.click(closeBtn(1)); // close B from A
        expect(await screen.findByTestId('close-tab-dialog')).toHaveTextContent('b.pdf');
        await fireEvent.click(screen.getByTestId('close-tab-save'));
        await waitFor(() => expect(saved).toHaveLength(1));
        await waitFor(() => expect(appState.activeFiles).not.toContain('/docs/b.pdf'));
    });

    test('the tab tooltip says switching clears Undo history', () => {
        expect(screen.getAllByTestId('doc-tab')[0].getAttribute('title')).toMatch(/Undo history/);
    });
});

import { expect, test, vi, beforeEach, afterEach, describe } from 'vitest';
import { render, screen, cleanup, fireEvent, waitFor } from '@testing-library/svelte';

// Review r4 (structural): one document store owns all per-document state.
//  1. Studio → Organize → Studio (canvas remount) keeps erased strokes erased and edits kept;
//  2. switching away before a document is restored keeps markups and inserts made meanwhile;
//  3. Save during restore or during a switch refuses and writes nothing;
//  4. B never gets A's scales (also when A's scales arrive late).

type Gate = { promise: Promise<void>; open: () => void };
function gate(): Gate {
    let open!: () => void;
    const promise = new Promise<void>((r) => (open = r));
    return { promise, open };
}

const st = vi.hoisted(() => ({
    persisted: {} as Record<string, any[]>,
    scales: {} as Record<string, any[]>,
    annotGate: {} as Record<string, { promise: Promise<void> } | undefined>,
    scaleGate: {} as Record<string, { promise: Promise<void> } | undefined>,
    existsGate: null as null | { promise: Promise<void> },
    saves: [] as Array<{ inputPath: string; outputPath: string; count: number }>,
}));

vi.mock('@tauri-apps/api/core', () => ({
    invoke: vi.fn(async (cmd: string, args: any) => {
        if (cmd === 'read_file_bytes') {
            if (/\.(annotated|flattened)-/.test(args.path)) {
                if (st.existsGate) await st.existsGate.promise;
                if (!st.saves.some((s) => s.outputPath === args.path)) throw new Error('No such file or directory (os error 2)');
            }
            return [args.path.endsWith('a.pdf') ? 1 : 2];
        }
        if (cmd === 'studio_load_annotations') {
            const g = st.annotGate[args.inputPath];
            if (g) await g.promise;
            return st.persisted[args.inputPath] ?? [];
        }
        if (cmd === 'studio_load_page_scales') {
            const g = st.scaleGate[args.inputPath];
            if (g) await g.promise;
            return st.scales[args.inputPath] ?? [];
        }
        if (cmd === 'studio_save_annotations') {
            st.saves.push({ inputPath: args.inputPath, outputPath: args.outputPath, count: args.annotations.length });
            return { outputPath: args.outputPath, savedCount: args.annotations.length, flattened: false, pageCount: 1 };
        }
        return null;
    }),
}));

import SplitLayout from '../studio/SplitLayout.svelte';
import { appState } from '$lib/state.svelte';
import { viewport } from '$lib/core/viewportEngine.svelte';
import { ink } from '$lib/core/inkEngine.svelte';
import { inserts } from '$lib/core/insertEngine.svelte';
import { measureEngine } from '$lib/core/measureEngine.svelte';
import { clearDocumentSessions, isDocumentRestored, documentSession } from '../studio/documentSessions';
import { setLabs } from '$lib/components/studio/labs.svelte';

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

const SAVED = {
    id: 'saved-1',
    kind: 'pen.a',
    pageIndex: 0,
    points: [{ x: 72, y: 700, p: 0.5 }, { x: 144, y: 700, p: 0.5 }],
    bboxPt: [72, 700, 144, 700],
    colorRgb: [0, 0, 0],
    widthPt: 1,
    opacity: 1,
};

const FT_SCALE = [{ pageIndex: 0, ptDistance: 18, realDistance: 1, unit: 'ft', precision: 2 }];

function drawStroke(y: number) {
    ink.startStroke(100, y, 0.5, 'pen.a');
    ink.addPoint(200, y, 0.5);
    return ink.endStroke()!.id;
}

const ids = () => ink.completedStrokes.map((s) => s.id);
const tab = (name: string) => screen.getAllByTestId('doc-tab').find((t) => t.textContent?.trim() === name)!;
const settle = () => new Promise((r) => setTimeout(r, 60));

describe('Document store (review r4)', () => {
    beforeEach(() => {
        st.persisted = {};
        st.scales = {};
        st.annotGate = {};
        st.scaleGate = {};
        st.existsGate = null;
        st.saves = [];
        clearDocumentSessions();
        (window as any).pdfjsLib = fakePdfjs();
        appState.clear();
        viewport.pages = [];
        ink.completedStrokes = [];
        measureEngine.measurements = [];
    });

    afterEach(() => {
        cleanup();
        delete (window as any).pdfjsLib;
        appState.clear();
        viewport.pages = [];
        ink.completedStrokes = [];
    });

    test('Studio → Organize → Studio keeps erased strokes erased and edits kept', async () => {
        st.persisted['/s4/a.pdf'] = [SAVED];
        appState.addFiles(['/s4/a.pdf']);
        const first = render(SplitLayout);
        await waitFor(() => expect(isDocumentRestored('/s4/a.pdf')).toBe(true));
        await waitFor(() => expect(ids()).toEqual(['saved-1']));
        ink.completedStrokes = ink.completedStrokes.filter((s) => s.id !== 'saved-1'); // erased
        const edit = drawStroke(300);

        first.unmount(); // e.g. the user opens Organize
        render(SplitLayout); // …and comes back to Studio
        await waitFor(() => expect(viewport.pages).toHaveLength(1));
        await settle();
        expect(ids()).toEqual([edit]);
    });

    test('switching away before B is restored keeps B’s saved markups; an insert during load is refused (r5)', async () => {
        st.persisted['/s4/b.pdf'] = [SAVED];
        appState.addFiles(['/s4/a.pdf']);
        render(SplitLayout);
        await waitFor(() => expect(isDocumentRestored('/s4/a.pdf')).toBe(true));
        const g = gate();
        st.annotGate['/s4/b.pdf'] = g;
        appState.addFiles(['/s4/b.pdf']);
        await waitFor(() => expect(viewport.pages).toHaveLength(2)); // laid out, markups not restored yet
        const note = inserts.stampQuickInsert('insert.note', 120, 140, 0, 'early note');
        await fireEvent.click(tab('a.pdf')); // leave B before it was restored
        await waitFor(() => expect(isDocumentRestored('/s4/a.pdf')).toBe(true));
        g.open();
        st.annotGate['/s4/b.pdf'] = undefined;
        await fireEvent.click(tab('b.pdf'));
        await waitFor(() => expect(isDocumentRestored('/s4/b.pdf')).toBe(true));
        expect(ids()).not.toContain(note.id); // refused while loading (its undo entry would wipe restored strokes)
        expect(ids()).toContain('saved-1');
    });

    test('Save while the document is still being restored refuses and writes nothing', async () => {
        const g = gate();
        st.annotGate['/s4/b.pdf'] = g;
        appState.addFiles(['/s4/b.pdf']);
        render(SplitLayout);
        await waitFor(() => expect(viewport.pages).toHaveLength(2));
        await fireEvent.click(screen.getByTestId('save-pdf-btn'));
        await settle();
        expect(st.saves).toHaveLength(0);
        expect(screen.getByTestId('studio-status-banner')).toHaveTextContent(/nothing was written/i);
        g.open();
    });

    test('Save interrupted by a tab switch refuses and writes nothing', async () => {
        appState.addFiles(['/s4/a.pdf', '/s4/b.pdf']);
        render(SplitLayout);
        await waitFor(() => expect(isDocumentRestored('/s4/b.pdf')).toBe(true));
        drawStroke(100);
        const g = gate();
        st.existsGate = g; // the collision check before writing is slow
        await fireEvent.click(screen.getByTestId('save-pdf-btn'));
        await fireEvent.click(tab('a.pdf')); // switch while the save is being prepared
        await waitFor(() => expect(isDocumentRestored('/s4/a.pdf')).toBe(true));
        g.open();
        await settle();
        expect(st.saves).toHaveLength(0);
        expect(screen.getByTestId('studio-status-banner')).toHaveTextContent(/nothing was written/i);
    });

    test('B never gets A’s scales, even when A’s scales arrive after the switch', async () => {
        st.scales['/s4/a.pdf'] = FT_SCALE;
        const g = gate();
        st.scaleGate['/s4/a.pdf'] = g; // A's scales are slow
        appState.addFiles(['/s4/a.pdf']);
        render(SplitLayout);
        await waitFor(() => expect(isDocumentRestored('/s4/a.pdf')).toBe(true));
        appState.addFiles(['/s4/b.pdf']);
        await waitFor(() => expect(isDocumentRestored('/s4/b.pdf')).toBe(true));
        g.open(); // A's late scales must not land on B
        st.scaleGate['/s4/a.pdf'] = undefined;
        await settle();
        expect(measureEngine.getScaleForPage(0).unit).not.toBe('ft');

        // Now let A's scales load normally, then switch to B: B must not inherit them.
        await fireEvent.click(tab('a.pdf'));
        await waitFor(() => expect(measureEngine.getScaleForPage(0).unit).toBe('ft'));
        await fireEvent.click(tab('b.pdf'));
        await waitFor(() => expect(isDocumentRestored('/s4/b.pdf')).toBe(true));
        await settle();
        expect(measureEngine.getScaleForPage(0).unit).not.toBe('ft');
        expect(documentSession('/s4/b.pdf')?.scales[0]?.unit ?? 'mm').not.toBe('ft');
    });

    test('split view is Labs-only: one canvas by default; with Labs the second pane only follows', async () => {
        st.persisted['/s4/a.pdf'] = [SAVED];
        appState.addFiles(['/s4/a.pdf']);
        const view = render(SplitLayout, { props: { mode: 'horizontal' } });
        await waitFor(() => expect(isDocumentRestored('/s4/a.pdf')).toBe(true));
        expect(document.querySelectorAll('#wet-layer')).toHaveLength(1);
        expect(screen.queryByTestId('split-view-toggle')).not.toBeInTheDocument();
        view.unmount();

        setLabs(true);
        try {
            render(SplitLayout, { props: { mode: 'horizontal' } });
            await waitFor(() => expect(document.querySelectorAll('#wet-layer')).toHaveLength(2));
            expect(screen.getByTestId('split-view-toggle')).toHaveTextContent('Split view');
            await settle();
            // Neither pane re-loaded or merged anything: the restored markups are intact, once.
            expect(ids()).toEqual(['saved-1']);
            expect(isDocumentRestored('/s4/a.pdf')).toBe(true);
        } finally {
            setLabs(false);
        }
    });
});

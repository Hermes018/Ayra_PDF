import { expect, test, vi, beforeEach, afterEach, describe } from 'vitest';
import { render, screen, cleanup, fireEvent, waitFor } from '@testing-library/svelte';

// Review r3: no markups may be lost while a document is still loading/rendering.
//  - switching away from B mid-render keeps B's unsaved markups (no empty stash over B's session);
//  - closing B mid-render still asks (dirty comes from B's stored session, not the empty engines);
//  - strokes drawn while the first load is still rendering survive the restore of saved markups;
//  - strokes drawn while a Save is running stay "unsaved" afterwards.

const st = vi.hoisted(() => ({
    hang: new Set<string>(), // paths whose page renders block until released
    gates: [] as Array<() => void>,
    renderCalls: [] as string[],
    persisted: {} as Record<string, any[]>,
    saveGate: null as null | Promise<void>,
    saved: [] as string[],
    currentPath: '',
}));

vi.mock('@tauri-apps/api/core', () => ({
    invoke: vi.fn(async (cmd: string, args: any) => {
        if (cmd === 'read_file_bytes') {
            if (/\.(annotated|flattened)-/.test(args.path) && !st.saved.includes(args.path)) {
                throw new Error('No such file or directory (os error 2)');
            }
            st.currentPath = args.path;
            return [args.path.endsWith('a.pdf') ? 1 : 2];
        }
        if (cmd === 'studio_load_annotations') return st.persisted[args.inputPath] ?? [];
        if (cmd === 'studio_save_annotations') {
            if (st.saveGate) await st.saveGate;
            st.saved.push(args.outputPath);
            return { outputPath: args.outputPath, savedCount: args.annotations.length, flattened: false, pageCount: 1 };
        }
        return null;
    }),
}));

import SplitLayout from '../studio/SplitLayout.svelte';
import { appState } from '$lib/state.svelte';
import { viewport } from '$lib/core/viewportEngine.svelte';
import { ink } from '$lib/core/inkEngine.svelte';
import { tools } from '$lib/core/toolEngine.svelte';
import { clearDocumentSessions, isDocumentDirty, isDocumentRestored } from '../studio/documentSessions';

function fakePdfjs() {
    return {
        GlobalWorkerOptions: {},
        getDocument: ({ data }: { data: Uint8Array }) => {
            const path = st.currentPath;
            return {
                promise: Promise.resolve({
                    numPages: data[0],
                    getPage: async () => ({
                        getViewport: ({ scale }: { scale: number }) => ({ width: 612 * scale, height: 792 * scale }),
                        render: () => {
                            st.renderCalls.push(path);
                            if (!st.hang.has(path)) return { promise: Promise.resolve() };
                            return { promise: new Promise<void>((resolve) => st.gates.push(resolve)) };
                        },
                    }),
                    destroy: async () => {},
                }),
            };
        },
    };
}

function releaseRenders() {
    st.hang.clear();
    st.gates.splice(0).forEach((g) => g());
}

function drawStroke(y: number) {
    ink.startStroke(100, y, 0.5, 'pen.a');
    ink.addPoint(200, y, 0.5);
    return ink.endStroke()!.id;
}

const ids = () => ink.completedStrokes.map((s) => s.id);
const tab = (name: string) => screen.getAllByTestId('doc-tab').find((t) => t.textContent?.trim() === name)!;

describe('Studio load races (review r3)', () => {
    beforeEach(() => {
        st.hang.clear();
        st.gates.length = 0;
        st.renderCalls.length = 0;
        st.persisted = {};
        st.saveGate = null;
        st.saved.length = 0;
        clearDocumentSessions();
        (window as any).pdfjsLib = fakePdfjs();
        vi.spyOn(HTMLCanvasElement.prototype, 'getContext').mockReturnValue({} as any);
        appState.clear();
        viewport.pages = [];
        ink.completedStrokes = [];
        tools.selectTool('pan');
    });

    afterEach(() => {
        releaseRenders();
        cleanup();
        vi.restoreAllMocks();
        delete (window as any).pdfjsLib;
        appState.clear();
        viewport.pages = [];
        ink.completedStrokes = [];
    });

    /** B is laid out and its page render has started but not finished. */
    async function waitForBMidRender(before: number) {
        await waitFor(() => {
            expect(viewport.pages).toHaveLength(2);
            expect(st.renderCalls.slice(before)).toContain('/r3/b.pdf');
        });
        expect(st.gates.length).toBeGreaterThan(0);
    }

    async function openAThenBWithMarkup() {
        appState.addFiles(['/r3/a.pdf', '/r3/b.pdf']);
        render(SplitLayout);
        await waitFor(() => expect(isDocumentRestored('/r3/b.pdf')).toBe(true)); // B shown, ready to draw on
        expect(isDocumentDirty('/r3/b.pdf')).toBe(false);
        const strokeB = drawStroke(300);
        expect(isDocumentDirty('/r3/b.pdf')).toBe(true);
        await fireEvent.click(tab('a.pdf'));
        await waitFor(() => expect(viewport.pages).toHaveLength(1));
        return strokeB;
    }

    test('switching away from B while it is still rendering keeps B’s unsaved markups', async () => {
        const strokeB = await openAThenBWithMarkup();
        st.hang.add('/r3/b.pdf');
        const before = st.renderCalls.length;
        await fireEvent.click(tab('b.pdf'));
        await waitForBMidRender(before)
        await fireEvent.click(tab('a.pdf')); // leave before B finished rendering
        await waitFor(() => expect(viewport.pages).toHaveLength(1));
        releaseRenders();
        expect(isDocumentDirty('/r3/b.pdf')).toBe(true);

        await fireEvent.click(tab('b.pdf'));
        await waitFor(() => expect(viewport.pages).toHaveLength(2));
        await waitFor(() => expect(ids()).toEqual([strokeB]));
    });

    test('closing B while it is still rendering still asks', async () => {
        await openAThenBWithMarkup();
        st.hang.add('/r3/b.pdf');
        const before = st.renderCalls.length;
        await fireEvent.click(tab('b.pdf'));
        await waitForBMidRender(before);
        const closeB = screen.getAllByTestId('doc-tab-close')[1];
        await fireEvent.click(closeB);
        expect(await screen.findByTestId('close-tab-dialog')).toHaveTextContent('b.pdf');
        expect(appState.activeFiles).toContain('/r3/b.pdf');
    });

    test('a stroke drawn while the first load is still rendering survives the restore', async () => {
        st.persisted['/r3/b.pdf'] = [
            {
                id: 'saved-1',
                kind: 'pen.a',
                pageIndex: 0,
                points: [{ x: 72, y: 700, p: 0.5 }, { x: 144, y: 700, p: 0.5 }],
                bboxPt: [72, 700, 144, 700],
                colorRgb: [0, 0, 0],
                widthPt: 1,
                opacity: 1,
            },
        ];
        st.hang.add('/r3/b.pdf');
        appState.addFiles(['/r3/b.pdf']);
        render(SplitLayout);
        await waitFor(() => expect(st.renderCalls.length).toBeGreaterThan(0)); // rendering, not finished
        tools.selectTool('pen.a');
        const wet = document.getElementById('wet-layer') as Element;
        const pt = viewport.worldToScreen('main-canvas').apply({ x: 150, y: 300 });
        await fireEvent.pointerDown(wet, { clientX: pt.x, clientY: pt.y, button: 0, buttons: 1, pointerId: 9 });
        await fireEvent.pointerUp(wet, { clientX: pt.x, clientY: pt.y, button: 0, pointerId: 9 });
        const drawn = ids().filter((id) => id !== 'saved-1');
        expect(drawn).toHaveLength(1);
        releaseRenders();
        await waitFor(() => expect(ids()).toContain('saved-1'));
        await new Promise((r) => setTimeout(r, 50));
        expect(ids()).toEqual(expect.arrayContaining(['saved-1', drawn[0]]));
        expect(isDocumentDirty('/r3/b.pdf')).toBe(true);
    });

    test('a stroke drawn while Save is running stays unsaved', async () => {
        appState.addFiles(['/r3/b.pdf']);
        render(SplitLayout);
        await waitFor(() => expect(isDocumentRestored('/r3/b.pdf')).toBe(true));
        expect(isDocumentDirty('/r3/b.pdf')).toBe(false);
        drawStroke(100);
        let open!: () => void;
        st.saveGate = new Promise<void>((r) => (open = r));
        await fireEvent.click(screen.getByTestId('save-pdf-btn'));
        await waitFor(() => expect(st.saveGate).not.toBeNull());
        await new Promise((r) => setTimeout(r, 20));
        drawStroke(200); // drawn during the save
        open();
        await waitFor(() => expect(st.saved).toHaveLength(1));
        await new Promise((r) => setTimeout(r, 20));
        expect(isDocumentDirty('/r3/b.pdf')).toBe(true);
    });
});

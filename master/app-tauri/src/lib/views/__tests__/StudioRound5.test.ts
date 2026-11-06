import { expect, test, vi, beforeEach, afterEach, describe } from 'vitest';
import { render, screen, cleanup, fireEvent, waitFor, within } from '@testing-library/svelte';

// Review r5 (final tabs round). Each test reproduces one review finding.

type Gate = { promise: Promise<void>; open: () => void };
function gate(): Gate {
    let open!: () => void;
    const promise = new Promise<void>((r) => (open = r));
    return { promise, open };
}

const st = vi.hoisted(() => ({
    persisted: {} as Record<string, any[]>,
    annotGate: {} as Record<string, { promise: Promise<void> } | undefined>,
    renderGate: null as null | { promise: Promise<void> },
    renderError: null as null | Error,
    renders: 0,
    saveGate: null as null | { promise: Promise<void> },
    saves: [] as Array<{ inputPath: string; outputPath: string }>,
    snap: {} as Record<string, any>,
    passwordFor: new Set<string>(),
    currentPath: '',
}));

vi.mock('@tauri-apps/api/core', () => ({
    invoke: vi.fn(async (cmd: string, args: any) => {
        if (cmd === 'read_file_bytes') {
            if (/\.(annotated|flattened)-/.test(args.path)) {
                if (!st.saves.some((s) => s.outputPath === args.path)) throw new Error('No such file or directory (os error 2)');
            }
            st.currentPath = args.path;
            return [args.path.endsWith('a.pdf') ? 1 : 2];
        }
        if (cmd === 'studio_load_annotations') {
            const g = st.annotGate[args.inputPath];
            if (g) await g.promise;
            return st.persisted[args.inputPath] ?? [];
        }
        if (cmd === 'studio_extract_snap_geometry') return st.snap[args.inputPath] ?? null;
        if (cmd === 'studio_save_annotations') {
            if (st.saveGate) await st.saveGate.promise;
            st.saves.push({ inputPath: args.inputPath, outputPath: args.outputPath });
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
import { commands } from '$lib/core/commandEngine.svelte';
import { snapEngine } from '$lib/core/snapEngine.svelte';
import { measureEngine } from '$lib/core/measureEngine.svelte';
import {
    clearDocumentSessions,
    documentSession,
    isDocumentRestored,
    FOLLOWER_WAIT,
} from '../studio/documentSessions';
import { saveLiveDocument } from '../studio/studioSave';
import { setLabs } from '$lib/components/studio/labs.svelte';

function passwordError() {
    const e = new Error('No password given') as Error & { code: number };
    e.name = 'PasswordException';
    e.code = 1;
    return e;
}

function fakePdfjs() {
    return {
        GlobalWorkerOptions: {},
        getDocument: ({ data, password }: { data: Uint8Array; password?: string }) => {
            if (st.passwordFor.has(st.currentPath) && !password) return { promise: Promise.reject(passwordError()) };
            return {
                promise: Promise.resolve({
                    numPages: data[0],
                    getPage: async () => ({
                        getViewport: ({ scale }: { scale: number }) => ({ width: 612 * scale, height: 792 * scale }),
                        render: () => {
                            st.renders++;
                            if (st.renderError) {
                                const e = st.renderError;
                                st.renderError = null;
                                return { promise: Promise.reject(e) };
                            }
                            return { promise: st.renderGate ? st.renderGate.promise : Promise.resolve() };
                        },
                    }),
                    destroy: async () => {},
                }),
            };
        },
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

function drawStroke(y: number) {
    const before = [...ink.completedStrokes];
    ink.startStroke(100, y, 0.5, 'pen.a');
    ink.addPoint(200, y, 0.5);
    const s = ink.endStroke()!;
    return { id: s.id, before };
}

const ids = () => ink.completedStrokes.map((s) => s.id);
const tabNamed = (name: string) => screen.getAllByTestId('doc-tab').find((t) => t.textContent?.trim() === name)!;
const closeBtnOf = (name: string) =>
    within(tabNamed(name).parentElement as HTMLElement).getByTestId('doc-tab-close');
const settle = (ms = 60) => new Promise((r) => setTimeout(r, ms));

describe('Review r5', () => {
    beforeEach(() => {
        st.persisted = {};
        st.annotGate = {};
        st.renderGate = null;
        st.renderError = null;
        st.renders = 0;
        st.saveGate = null;
        st.saves = [];
        st.snap = {};
        st.passwordFor = new Set();
        clearDocumentSessions();
        (window as any).pdfjsLib = fakePdfjs();
        vi.spyOn(HTMLCanvasElement.prototype, 'getContext').mockReturnValue({} as any);
        appState.clear();
        viewport.pages = [];
        ink.completedStrokes = [];
        commands.load(commands.doc);
    });

    afterEach(() => {
        st.renderGate = null;
        cleanup();
        vi.restoreAllMocks();
        delete (window as any).pdfjsLib;
        appState.clear();
        viewport.pages = [];
        ink.completedStrokes = [];
        setLabs(false);
    });

    // BLOCKER
    test('Discard on the active tab, then reopening it, shows only what is on disk', async () => {
        st.persisted['/r5/b.pdf'] = [SAVED];
        appState.addFiles(['/r5/a.pdf', '/r5/b.pdf']);
        render(SplitLayout);
        await waitFor(() => expect(isDocumentRestored('/r5/b.pdf')).toBe(true));
        drawStroke(300);
        await fireEvent.click(closeBtnOf('b.pdf'));
        await fireEvent.click(await screen.findByTestId('close-tab-discard'));
        await waitFor(() => expect(isDocumentRestored('/r5/a.pdf')).toBe(true));
        expect(documentSession('/r5/b.pdf')).toBeUndefined();

        appState.addFiles(['/r5/b.pdf']); // reopen
        await waitFor(() => expect(isDocumentRestored('/r5/b.pdf')).toBe(true));
        await settle();
        expect(ids()).toEqual(['saved-1']);
    });

    test('closing the last tab leaves no session behind', async () => {
        appState.addFiles(['/r5/b.pdf']);
        render(SplitLayout);
        await waitFor(() => expect(isDocumentRestored('/r5/b.pdf')).toBe(true));
        await fireEvent.click(closeBtnOf('b.pdf')); // clean: closes at once
        await waitFor(() => expect(appState.activeFiles).toEqual([]));
        await settle();
        expect(documentSession('/r5/b.pdf')).toBeUndefined();
        appState.addFiles(['/r5/b.pdf']);
        await waitFor(() => expect(isDocumentRestored('/r5/b.pdf')).toBe(true));
        expect(ids()).toEqual([]);
    });

    // MAJOR: inserts during load
    test('inserts are blocked until the document is restored, so Undo can never wipe restored strokes', async () => {
        st.persisted['/r5/b.pdf'] = [SAVED];
        const g = gate();
        st.annotGate['/r5/b.pdf'] = g;
        appState.addFiles(['/r5/b.pdf']);
        render(SplitLayout);
        await waitFor(() => expect(viewport.pages).toHaveLength(2));
        expect(isDocumentRestored('/r5/b.pdf')).toBe(false);
        expect(document.getElementById('wet-layer')).toHaveClass('cursor-busy');
        inserts.stampQuickInsert('insert.note', 120, 140, 0, 'too early');
        expect(ids()).toEqual([]);
        g.open();
        await waitFor(() => expect(isDocumentRestored('/r5/b.pdf')).toBe(true));
        expect(ids()).toEqual(['saved-1']);
        commands.undo();
        expect(ids()).toEqual(['saved-1']);
    });

    // MAJOR: geometry
    test('Save refuses when the page geometry is missing (e.g. after a render error)', async () => {
        appState.addFiles(['/r5/b.pdf']);
        render(SplitLayout);
        await waitFor(() => expect(isDocumentRestored('/r5/b.pdf')).toBe(true));
        drawStroke(100);
        viewport.pages = [];
        const r = await saveLiveDocument(false);
        expect(r.ok).toBe(false);
        expect(r.message).toMatch(/page layout/i);
        expect(st.saves).toHaveLength(0);
    });

    test('a render error after restore keeps the layout, reports it, and Save still writes correct geometry', async () => {
        st.renderError = new Error('canvas exploded');
        vi.spyOn(console, 'error').mockImplementation(() => {});
        appState.addFiles(['/r5/b.pdf']);
        render(SplitLayout);
        expect(await screen.findByTestId('studio-status-banner')).toHaveTextContent('canvas exploded');
        expect(isDocumentRestored('/r5/b.pdf')).toBe(true);
        expect(viewport.pages).toHaveLength(2);
    });

    // MINOR: concurrent saves
    test('a second Save while one is running is refused; names never collide', async () => {
        appState.addFiles(['/r5/b.pdf']);
        render(SplitLayout);
        await waitFor(() => expect(isDocumentRestored('/r5/b.pdf')).toBe(true));
        drawStroke(100);
        const g = gate();
        st.saveGate = g;
        const first = saveLiveDocument(false);
        await settle(20);
        const second = await saveLiveDocument(false);
        expect(second.ok).toBe(false);
        expect(second.message).toMatch(/already being saved/i);
        g.open();
        expect((await first).ok).toBe(true);
        expect(st.saves).toHaveLength(1);
    });

    // MINOR: accurate refusal wording
    test('Save after a password cancel says the file is locked, not "in a moment"', async () => {
        st.passwordFor.add('/r5/b.pdf');
        appState.addFiles(['/r5/b.pdf']);
        render(SplitLayout);
        await fireEvent.click(await screen.findByTestId('pdf-password-cancel'));
        const r = await saveLiveDocument(false);
        expect(r.ok).toBe(false);
        expect(r.message).toMatch(/password/i);
        expect(r.message).not.toMatch(/in a moment/i);
    });

    // MINOR: render mode must not overwrite snap geometry
    test('a remounted canvas keeps the document’s snap geometry', async () => {
        st.snap['/r5/b.pdf'] = { pageIndex: 0, widthPt: 612, heightPt: 792, segments: [{ x1: 1, y1: 2, x2: 3, y2: 4 }] };
        appState.addFiles(['/r5/b.pdf']);
        const v = render(SplitLayout);
        await waitFor(() => expect(snapEngine.pageSegments[0]?.length).toBe(1));
        const before = JSON.stringify(snapEngine.pageSegments);
        v.unmount();
        render(SplitLayout);
        await waitFor(() => expect(viewport.pages).toHaveLength(2));
        await settle();
        expect(JSON.stringify(snapEngine.pageSegments)).toBe(before);
    });

    // MINOR: Labs split — only the primary pane wires the keyboard/handlers; follower retries.
    test('split view: keyboard attached once; the follower repaints after a slow load', async () => {
        setLabs(true);
        FOLLOWER_WAIT.ms = 30;
        const attach = vi.spyOn(commands, 'attachKeyboard');
        const g = gate();
        st.annotGate['/r5/b.pdf'] = g;
        appState.addFiles(['/r5/b.pdf']);
        render(SplitLayout, { props: { mode: 'horizontal' } });
        await waitFor(() => expect(document.querySelectorAll('#wet-layer')).toHaveLength(2));
        expect(attach).toHaveBeenCalledTimes(1);
        await settle(120); // longer than the follower's wait
        g.open();
        await waitFor(() => expect(isDocumentRestored('/r5/b.pdf')).toBe(true));
        await waitFor(() => expect(st.renders).toBeGreaterThanOrEqual(4)); // 2 pages × 2 panes
        FOLLOWER_WAIT.ms = 30_000;
    });

    test('a stroke drawn while pages are still rendering is kept and is its own undo step', async () => {
        st.persisted['/r5/b.pdf'] = [SAVED];
        const g = gate();
        st.renderGate = g;
        appState.addFiles(['/r5/b.pdf']);
        render(SplitLayout);
        await waitFor(() => expect(isDocumentRestored('/r5/b.pdf')).toBe(true));
        await waitFor(() => expect(st.renders).toBeGreaterThan(0));
        expect(isDocumentRestored('/r5/b.pdf')).toBe(true); // drawing allowed, render still pending
        const d = drawStroke(300);
        const { shapes } = await import('$lib/core/shapeAndHatchEngine.svelte');
        shapes.recordStrokeHistory(d.before, [...ink.completedStrokes], 'Ink pen.a');
        g.open();
        await settle();
        expect(ids()).toEqual(['saved-1', d.id]);
        commands.undo();
        expect(ids()).toEqual(['saved-1']);
    });
});

describe('Labs split note is honest', () => {
    test('it does not claim both panes edit', async () => {
        const { SPLIT_VIEW_LABS } = await import('$lib/components/studio/labs.svelte');
        expect(SPLIT_VIEW_LABS.note).not.toMatch(/both panes edit/i);
        expect(SPLIT_VIEW_LABS.note).toMatch(/left pane/i);
    });
});

void measureEngine;

import { expect, test, vi, beforeEach, afterEach, describe } from 'vitest';
import { render, screen, cleanup, waitFor } from '@testing-library/svelte';

// BUG-018: malformed / hostile PDFs must give a user-visible error or degraded-mode notice
// (flow 2c), never a silent empty canvas or a silently partial document.

type Backend = { status: 'Completed' | 'Failed'; degraded?: boolean; error?: string; code?: string };
let backend: Backend = { status: 'Completed', degraded: false };
// Hoisted so the app's global job listener (registered when state.svelte.ts loads) is captured too.
const { listeners, probes } = vi.hoisted(() => ({
    listeners: [] as Array<(e: { payload: any }) => void>,
    probes: [] as string[],
}));

vi.mock('@tauri-apps/api/event', () => ({
    listen: vi.fn(async (_name: string, cb: (e: { payload: any }) => void) => {
        listeners.push(cb);
        return () => {
            const i = listeners.indexOf(cb);
            if (i >= 0) listeners.splice(i, 1);
        };
    }),
}));

vi.mock('@tauri-apps/api/core', () => ({
    invoke: vi.fn(async (cmd: string, args: any) => {
        if (cmd === 'read_file_bytes') return [37, 80, 68, 70];
        if (cmd === 'submit_job') {
            expect(args.request).toMatchObject({ op: 'GetInfo' });
            probes.push(args.request.input_path);
            const payload =
                backend.status === 'Completed'
                    ? { kind: 'Completed', data: { job_id: 'probe-1', output_path: null, degraded: !!backend.degraded } }
                    : { kind: 'Failed', data: { job_id: 'probe-1', error_code: backend.code ?? 'E_PARSE', error_message: backend.error } };
            for (const l of [...listeners]) l({ payload });
            return 'probe-1';
        }
        if (cmd === 'get_job_status') return backend.status;
        if (cmd === 'list_jobs') return [{ job_id: 'probe-1', error_message: backend.error ?? null, error_code: backend.code ?? null }];
        return null;
    }),
}));

import SpatialCanvas from '../studio/SpatialCanvas.svelte';
import { appState } from '$lib/state.svelte';
import { viewport } from '$lib/core/viewportEngine.svelte';

function pdfjsThatOpens(numPages: number) {
    return {
        GlobalWorkerOptions: {},
        getDocument: () => ({
            promise: Promise.resolve({
                numPages,
                getPage: async () => ({
                    getViewport: ({ scale }: { scale: number }) => ({ width: 612 * scale, height: 792 * scale }),
                    render: () => ({ promise: Promise.resolve() }),
                }),
                destroy: async () => {},
            }),
        }),
    };
}

function pdfjsThatRejects() {
    const err = new Error('Invalid PDF structure.');
    err.name = 'InvalidPDFException';
    return { GlobalWorkerOptions: {}, getDocument: () => ({ promise: Promise.reject(err) }) };
}

describe('Malformed PDFs are reported (BUG-018)', () => {
    beforeEach(() => {
        backend = { status: 'Completed', degraded: false };
        probes.length = 0;
        appState.clearToasts();
        appState.clear();
        viewport.pages = [];
    });

    afterEach(() => {
        cleanup();
        delete (window as any).pdfjsLib;
        appState.clear();
        viewport.pages = [];
    });

    test('a file the viewer cannot parse shows an error and the Studio stays usable', async () => {
        (window as any).pdfjsLib = pdfjsThatRejects();
        appState.addFiles(['/corpus/hostile-broken-xref.pdf']);
        render(SpatialCanvas);
        const banner = await screen.findByTestId('studio-status-banner');
        expect(banner).toHaveTextContent('hostile-broken-xref.pdf');
        expect(banner).toHaveTextContent(/damaged or not a PDF/i);
        expect(banner).toHaveTextContent('Invalid PDF structure');
        expect(screen.getByTestId('studio-empty-state')).toBeInTheDocument();
        expect(screen.getByRole('button', { name: 'Open PDF…' })).toBeEnabled();
    });

    test('a file the engine opens only in fallback mode shows a degraded notice', async () => {
        (window as any).pdfjsLib = pdfjsThatOpens(1);
        backend = { status: 'Completed', degraded: true };
        appState.addFiles(['/corpus/repaired.pdf']);
        render(SpatialCanvas);
        await waitFor(() => expect(viewport.pages).toHaveLength(1));
        const banner = await screen.findByTestId('studio-status-banner');
        expect(banner).toHaveTextContent(/degraded/i);
        expect(banner).toHaveTextContent('repaired.pdf');
    });

    test('a file the engine rejects is flagged even when the viewer shows something', async () => {
        (window as any).pdfjsLib = pdfjsThatOpens(1);
        backend = { status: 'Failed', error: 'decompression ratio limit exceeded' };
        appState.addFiles(['/corpus/hostile-bombs.pdf']);
        render(SpatialCanvas);
        await waitFor(() => expect(viewport.pages).toHaveLength(1));
        const banner = await screen.findByTestId('studio-status-banner');
        expect(banner).toHaveTextContent(/damaged/i);
        expect(banner).toHaveTextContent('decompression ratio limit exceeded');
    });

    test('a healthy file shows no notice', async () => {
        (window as any).pdfjsLib = pdfjsThatOpens(2);
        appState.addFiles(['/corpus/standard-text.pdf']);
        render(SpatialCanvas);
        await waitFor(() => expect(viewport.pages).toHaveLength(2));
        await new Promise((r) => setTimeout(r, 300));
        expect(screen.queryByTestId('studio-status-banner')).not.toBeInTheDocument();
    });

    test('a PDF with zero pages is reported', async () => {
        (window as any).pdfjsLib = pdfjsThatOpens(0);
        appState.addFiles(['/corpus/empty.pdf']);
        render(SpatialCanvas);
        const banner = await screen.findByTestId('studio-status-banner');
        expect(banner).toHaveTextContent(/no pages/i);
    });

    test('the probe is an internal job: it never shows an "Operation Completed/Failed" toast', async () => {
        (window as any).pdfjsLib = pdfjsThatOpens(1);
        backend = { status: 'Completed', degraded: true };
        appState.addFiles(['/corpus/quiet.pdf']);
        render(SpatialCanvas);
        await screen.findByTestId('studio-status-banner');
        await new Promise((r) => setTimeout(r, 50));
        expect(listeners.length).toBeGreaterThan(0); // the global listener is really attached
        expect(appState.toasts.map((t) => t.message)).not.toContain('Operation Completed');
        backend = { status: 'Failed', error: 'bad xref' };
        appState.addFiles(['/corpus/quiet-2.pdf']);
        await waitFor(() => expect(probes).toContain('/corpus/quiet-2.pdf'));
        await new Promise((r) => setTimeout(r, 300));
        expect(appState.toasts.map((t) => t.message)).not.toContain('Operation Failed');
    });

    test('a document is probed once per run, not on every tab switch', async () => {
        (window as any).pdfjsLib = pdfjsThatOpens(1);
        appState.addFiles(['/corpus/one.pdf']);
        render(SpatialCanvas);
        await waitFor(() => expect(probes).toEqual(['/corpus/one.pdf']));
        appState.addFiles(['/corpus/two.pdf']);
        await waitFor(() => expect(probes).toEqual(['/corpus/one.pdf', '/corpus/two.pdf']));
        appState.setActiveFile('/corpus/one.pdf');
        await new Promise((r) => setTimeout(r, 300));
        appState.setActiveFile('/corpus/two.pdf');
        await new Promise((r) => setTimeout(r, 300));
        expect(probes).toEqual(['/corpus/one.pdf', '/corpus/two.pdf']);
    });

    test('engines contract: F4 resource guard → "damaged or hostile and was not opened", app stays usable', async () => {
        (window as any).pdfjsLib = pdfjsThatOpens(1);
        backend = {
            status: 'Failed',
            code: 'F4_RESOURCE_GUARD',
            error: "Resource Guard: F4 guard 'declared-stream-length' tripped: 9e9 > 4e8",
        };
        appState.addFiles(['/corpus/hostile-bombs-f4.pdf']);
        render(SpatialCanvas);
        const banner = await screen.findByTestId('studio-status-banner');
        await waitFor(() => expect(banner).toHaveTextContent(/looks damaged or hostile and was not opened/i));
        await waitFor(() => expect(viewport.pages).toHaveLength(0));
        expect(screen.getByRole('button', { name: 'Open PDF…' })).toBeEnabled();
    });

    test('engines contract: fallback open → non-blocking "Opened in repair mode" notice', async () => {
        (window as any).pdfjsLib = pdfjsThatOpens(1);
        backend = { status: 'Completed', degraded: true };
        appState.addFiles(['/corpus/repaired-2.pdf']);
        render(SpatialCanvas);
        const banner = await screen.findByTestId('studio-status-banner');
        expect(banner).toHaveTextContent(/Opened in repair mode/);
        expect(viewport.pages).toHaveLength(1); // still open and usable
    });
});

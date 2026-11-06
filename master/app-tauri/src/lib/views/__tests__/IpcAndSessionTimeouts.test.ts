import { expect, test, vi, beforeEach, describe } from 'vitest';

// Review r3 minors: nothing may wait forever.
//  - whenDocumentLive resolves false when another document becomes ready, or after a timeout;
//  - probeDocument gives up after a timeout and always removes its event listener;
//  - runJob's no-progress timer does not count time spent waiting in the queue.

const st = vi.hoisted(() => ({
    statuses: [] as string[], // statuses returned by successive get_job_status calls (last one repeats)
    unlistened: 0,
    cancelled: [] as string[],
}));

vi.mock('@tauri-apps/api/event', () => ({
    listen: vi.fn(async () => () => {
        st.unlistened++;
    }),
}));

vi.mock('@tauri-apps/api/core', () => ({
    invoke: vi.fn(async (cmd: string, args: any) => {
        if (cmd === 'submit_job') return 'job-1';
        if (cmd === 'get_job_status') return st.statuses.length > 1 ? st.statuses.shift() : st.statuses[0];
        if (cmd === 'list_jobs') return [{ job_id: 'job-1', progress: 0, error_message: null, error_code: null }];
        if (cmd === 'cancel_job') {
            st.cancelled.push(args.jobId);
            return true;
        }
        return null;
    }),
}));

import { IpcClient } from '$lib/ipc';
import {
    clearDocumentSessions,
    notifyDocumentLoaded,
    whenDocumentLive,
} from '$lib/views/studio/documentSessions';

const within = <T,>(p: Promise<T>, ms: number) =>
    Promise.race([p, new Promise<'pending'>((r) => setTimeout(() => r('pending'), ms))]);

describe('whenDocumentLive never hangs', () => {
    beforeEach(() => clearDocumentSessions());

    test('resolves true when the document becomes ready', async () => {
        const p = whenDocumentLive('/t/b.pdf');
        notifyDocumentLoaded('/t/b.pdf', true);
        expect(await within(p, 50)).toBe(true);
    });

    test('resolves false when a different document becomes ready first (superseded)', async () => {
        const p = whenDocumentLive('/t/b.pdf');
        notifyDocumentLoaded('/t/a.pdf', true);
        expect(await within(p, 50)).toBe(false);
    });

    test('resolves false after its timeout', async () => {
        const p = whenDocumentLive('/t/c.pdf', 30);
        expect(await within(p, 200)).toBe(false);
    });
});

describe('probeDocument never hangs', () => {
    beforeEach(() => {
        st.statuses = ['Running'];
        st.unlistened = 0;
        st.cancelled = [];
    });

    test('a stuck probe times out quietly, is cancelled and removes its listener', async () => {
        const res = await within(IpcClient.probeDocument('/t/x.pdf', { pollMs: 5, timeoutMs: 40 }), 1000);
        expect(res).toEqual({ ok: true, degraded: false, timedOut: true });
        expect(st.cancelled).toEqual(['job-1']);
        expect(st.unlistened).toBe(1);
    });

    test('the listener is removed when the probe completes too', async () => {
        st.statuses = ['Completed'];
        await IpcClient.probeDocument('/t/x.pdf', { pollMs: 5, timeoutMs: 1000 });
        expect(st.unlistened).toBe(1);
    });
});

describe('runJob stall timer ignores queue time', () => {
    beforeEach(() => {
        st.cancelled = [];
    });

    test('a job that waits in the queue longer than stallMs, then runs, still completes', async () => {
        st.statuses = [...Array(20).fill('Queued'), 'Running', 'Running', 'Completed'];
        const id = await IpcClient.runJob({ op: 'GetInfo', input_path: '/t/x.pdf' }, { pollMs: 5, stallMs: 40 });
        expect(id).toBe('job-1');
        expect(st.cancelled).toEqual([]);
    });

    test('a running job with no progress is still stopped', async () => {
        st.statuses = ['Queued', 'Running'];
        await expect(
            IpcClient.runJob({ op: 'GetInfo', input_path: '/t/x.pdf' }, { pollMs: 5, stallMs: 40 })
        ).rejects.toThrow(/no progress/);
        expect(st.cancelled).toEqual(['job-1']);
    });
});

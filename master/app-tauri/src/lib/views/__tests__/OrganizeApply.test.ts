import { expect, test, vi, beforeEach, afterEach, describe } from 'vitest';
import { render, screen, cleanup, fireEvent, waitFor } from '@testing-library/svelte';

// BUG-014: Organize Grid "Apply" must keep rotation (each job waits for the previous job's
// output) and must report the real outcome, never "success" after a failed step.
// Review r2: outputs are collision-safe new files (DO-5), at most one intermediate file, and a
// stuck job can be cancelled / times out with a clear error.

type Job = { request: any; polls: number; status: 'Running' | 'Completed' | 'Failed' | 'Cancelled'; error?: string };
const jobs = new Map<string, Job>();
const files = new Set<string>();
const submitted: any[] = [];
const cancelled: string[] = [];
let nextId = 1;
let failOp: string | null = null;
let hangOp: string | null = null;
const deleted: string[] = [];
let failDelete = false;
/** null = command not available (stopgap used); otherwise the engine's answer. */
let pathExistsAnswer: (p: string) => boolean | null = () => null;

vi.mock('@tauri-apps/api/core', () => ({
    invoke: vi.fn(async (cmd: string, args: any) => {
        if (cmd === 'read_file_bytes') {
            if (files.has(args.path)) return [37, 80, 68, 70];
            throw new Error('No such file or directory (os error 2)');
        }
        if (cmd === 'submit_job') {
            const id = `job-${nextId++}`;
            submitted.push(args.request);
            jobs.set(id, { request: args.request, polls: 0, status: 'Running' });
            return id;
        }
        if (cmd === 'delete_organize_intermediate') {
            deleted.push(args.path);
            if (failDelete) throw new Error('refused');
            files.delete(args.path);
            return null;
        }
        if (cmd === 'path_exists') return pathExistsAnswer(args.path);
        if (cmd === 'cancel_job') {
            cancelled.push(args.jobId);
            const job = jobs.get(args.jobId);
            if (job) job.status = 'Cancelled';
            return true;
        }
        if (cmd === 'get_job_status') {
            const job = jobs.get(args.jobId);
            if (!job) return null;
            if (job.status === 'Running' && hangOp !== job.request.op && ++job.polls >= 2) {
                // Like the real worker: a job whose input does not exist (yet) fails.
                if (!files.has(job.request.input_path)) {
                    job.status = 'Failed';
                    job.error = `input not found: ${job.request.input_path}`;
                } else if (failOp === job.request.op) {
                    job.status = 'Failed';
                    job.error = 'engine refused rotation';
                } else {
                    job.status = 'Completed';
                    files.add(job.request.output_path);
                }
            }
            return job.status;
        }
        if (cmd === 'list_jobs') {
            return [...jobs.entries()].map(([job_id, j]) => ({ job_id, progress: 0, error_message: j.error ?? null, error_code: null }));
        }
        return null;
    }),
}));

import GridEditor from '$lib/components/GridEditor.svelte';
import { planOrganizeJobs } from '$lib/components/organizeApply';
import { IpcClient } from '$lib/ipc';
import { freeOutputPath } from '$lib/views/studio/outputNames';

const STAMPED = /^\/docs\/plan\.organized-\d{8}-\d{6}-\d{3}(-\d+)?\.pdf$/;

function fakePdfjs(numPages: number) {
    return {
        GlobalWorkerOptions: {},
        getDocument: () => ({
            promise: Promise.resolve({
                numPages,
                getPage: async () => ({
                    getViewport: () => ({ width: 10, height: 14 }),
                    render: () => ({ promise: Promise.resolve() }),
                }),
                destroy: () => {},
            }),
        }),
    };
}

describe('Organize Grid Apply (BUG-014)', () => {
    beforeEach(() => {
        jobs.clear();
        files.clear();
        submitted.length = 0;
        cancelled.length = 0;
        nextId = 1;
        failOp = null;
        hangOp = null;
        deleted.length = 0;
        failDelete = false;
        pathExistsAnswer = () => null;
        files.add('/docs/plan.pdf');
        (window as any).pdfjsLib = fakePdfjs(3);
        vi.spyOn(HTMLCanvasElement.prototype, 'getContext').mockReturnValue({} as any);
        vi.spyOn(HTMLCanvasElement.prototype, 'toDataURL').mockReturnValue('data:image/jpeg;base64,AA==');
        vi.spyOn(window, 'alert').mockImplementation(() => {});
    });

    afterEach(() => {
        cleanup();
        vi.restoreAllMocks();
        delete (window as any).pdfjsLib;
    });

    async function renderGrid() {
        render(GridEditor, { props: { filePath: '/docs/plan.pdf' } });
        await waitFor(() => expect(screen.getAllByTestId('organize-page-card')).toHaveLength(3));
        return screen.getAllByTestId('organize-page-card');
    }

    test('reorder + delete + rotate: rotation runs on the finished output and is kept', async () => {
        const cards = await renderGrid();
        // Move page 1 to the end: order 2, 3, 1.
        await fireEvent.dragStart(cards[0]);
        await fireEvent.drop(cards[2]);
        // Delete what is now first (orig 2), rotate what is now last (orig 1).
        await fireEvent.click(screen.getAllByTestId('organize-delete-btn')[0]);
        await fireEvent.click(screen.getAllByTestId('organize-rotate-btn')[2]);

        await fireEvent.click(screen.getByTestId('organize-apply-btn'));
        const status = await screen.findByTestId('organize-status');
        await waitFor(() => expect(status).toHaveAttribute('data-state', 'success'));

        expect(submitted.map((r) => r.op)).toEqual(['ExtractPages', 'RunPipeline']);
        expect(submitted[0].pages).toEqual([3, 1]);
        expect(submitted[0].output_path).toMatch(/^\/docs\/plan\.organizing-\d{8}-\d{6}-\d{3}(-\d+)?\.pdf$/);
        expect(submitted[1]).toMatchObject({
            input_path: submitted[0].output_path,
            steps: [{ type: 'Rotate', degrees: 90, pages: [2] }],
        });
        expect(submitted[1].output_path).toMatch(STAMPED);
        expect(files.has(submitted[1].output_path)).toBe(true);
        expect(status).toHaveTextContent(submitted[1].output_path);
        expect(window.alert).not.toHaveBeenCalled();
    });

    test('a second Apply writes a different new file and never overwrites the first', async () => {
        await renderGrid();
        await fireEvent.click(screen.getAllByTestId('organize-rotate-btn')[0]);
        await fireEvent.click(screen.getByTestId('organize-apply-btn'));
        await waitFor(() => expect(screen.getByTestId('organize-status')).toHaveAttribute('data-state', 'success'));
        await fireEvent.click(screen.getByTestId('organize-apply-btn'));
        await waitFor(() => expect(submitted).toHaveLength(2));
        await waitFor(() => expect(screen.getByTestId('organize-status')).toHaveAttribute('data-state', 'success'));
        // Rotation only → one job, no intermediate file.
        expect(submitted.map((r) => r.op)).toEqual(['RunPipeline', 'RunPipeline']);
        const outs = submitted.map((r) => r.output_path);
        expect(outs[0]).toMatch(STAMPED);
        expect(outs[1]).toMatch(STAMPED);
        expect(outs[0]).not.toBe(outs[1]);
        expect(outs).not.toContain('/docs/plan.pdf');
    });

    test('a name that already exists on disk is skipped', async () => {
        const fixed = new Date(2026, 10, 6, 6, 30, 0, 0);
        files.add('/docs/plan.organized-20261106-063000-000.pdf'); // e.g. from an earlier run
        const p = await freeOutputPath('/docs/plan.pdf', 'organized', fixed);
        expect(p).toBe('/docs/plan.organized-20261106-063000-000-2.pdf');
    });

    test('a failed step is reported as a failure, not success', async () => {
        failOp = 'RunPipeline';
        await renderGrid();
        await fireEvent.click(screen.getAllByTestId('organize-rotate-btn')[1]);
        await fireEvent.click(screen.getByTestId('organize-apply-btn'));
        const status = await screen.findByTestId('organize-status');
        await waitFor(() => expect(status).toHaveAttribute('data-state', 'error'));
        expect(status).toHaveTextContent(/engine refused rotation/);
        expect(status).toHaveTextContent(/Rotate page\(s\) 2 by 90°/);
        expect(status).not.toHaveTextContent(/success/i);
    });

    test('a stuck job can be cancelled from the grid', async () => {
        hangOp = 'RunPipeline';
        await renderGrid();
        await fireEvent.click(screen.getAllByTestId('organize-rotate-btn')[0]);
        await fireEvent.click(screen.getByTestId('organize-apply-btn'));
        await waitFor(() => expect(screen.getByTestId('organize-apply-btn')).toBeDisabled());
        await fireEvent.click(await screen.findByTestId('organize-cancel-btn'));
        const status = screen.getByTestId('organize-status');
        await waitFor(() => expect(status).toHaveAttribute('data-state', 'error'));
        expect(status).toHaveTextContent(/cancelled/i);
        expect(cancelled).toHaveLength(1);
        expect(screen.getByTestId('organize-apply-btn')).toBeEnabled();
    });

    test('leaving the Organize view while Apply runs cancels the job (no orphan work)', async () => {
        hangOp = 'RunPipeline';
        const view = render(GridEditor, { props: { filePath: '/docs/plan.pdf' } });
        await waitFor(() => expect(screen.getAllByTestId('organize-page-card')).toHaveLength(3));
        await fireEvent.click(screen.getAllByTestId('organize-rotate-btn')[0]);
        await fireEvent.click(screen.getByTestId('organize-apply-btn'));
        await waitFor(() => expect(submitted).toHaveLength(1));
        view.unmount();
        await waitFor(() => expect(cancelled).toHaveLength(1));
    });

    test('runJob gives up with a clear error when a job makes no progress', async () => {
        hangOp = 'GetInfo';
        await expect(
            IpcClient.runJob({ op: 'GetInfo', input_path: '/docs/plan.pdf' }, { pollMs: 5, stallMs: 60 })
        ).rejects.toThrow(/no progress for 0\.06 s/i);
        expect(cancelled).toHaveLength(1);
    });

    test('rotation numbers refer to positions in the reordered output', () => {
        const names = { intermediate: '/a.organizing.pdf', final: '/a.organized.pdf' };
        const steps = planOrganizeJobs('/a.pdf', [
            { originalIndex: 3, rotation: 270, deleted: false },
            { originalIndex: 1, rotation: 0, deleted: true },
            { originalIndex: 2, rotation: 180, deleted: false },
        ], names);
        expect(steps.map((s) => s.request)).toEqual([
            { op: 'ExtractPages', input_path: '/a.pdf', output_path: '/a.organizing.pdf', pages: [3, 2] },
            {
                op: 'RunPipeline',
                input_path: '/a.organizing.pdf',
                output_path: '/a.organized.pdf',
                steps: [
                    { type: 'Rotate', degrees: -90, pages: [1] },
                    { type: 'Rotate', degrees: 180, pages: [2] },
                ],
                preset_name: null,
            },
        ]);
        expect(planOrganizeJobs('/a.pdf', [{ originalIndex: 1, rotation: 0, deleted: true }, { originalIndex: 2, rotation: 0, deleted: false }], names))
            .toEqual([{ label: 'Delete 1 page(s)', request: { op: 'DeletePages', input_path: '/a.pdf', output_path: '/a.organized.pdf', pages: [1] } }]);
        expect(() => planOrganizeJobs('/a.pdf', [{ originalIndex: 1, rotation: 0, deleted: true }], names)).toThrow(/at least one/);
    });

    test('engines contract: the organizing intermediate is deleted after a successful apply', async () => {
        const cards = await renderGrid();
        await fireEvent.dragStart(cards[0]);
        await fireEvent.drop(cards[2]);
        await fireEvent.click(screen.getAllByTestId('organize-rotate-btn')[0]);
        await fireEvent.click(screen.getByTestId('organize-apply-btn'));
        await waitFor(() => expect(screen.getByTestId('organize-status')).toHaveAttribute('data-state', 'success'));
        expect(deleted).toEqual([submitted[0].output_path]);
        expect(deleted[0]).toMatch(/\.organizing-/);
    });

    test('engines contract: a failed intermediate delete is logged and the apply still succeeds (KI-002)', async () => {
        failDelete = true;
        const warn = vi.spyOn(console, 'warn').mockImplementation(() => {});
        const cards = await renderGrid();
        await fireEvent.dragStart(cards[0]);
        await fireEvent.drop(cards[2]);
        await fireEvent.click(screen.getAllByTestId('organize-rotate-btn')[0]);
        await fireEvent.click(screen.getByTestId('organize-apply-btn'));
        await waitFor(() => expect(screen.getByTestId('organize-status')).toHaveAttribute('data-state', 'success'));
        expect(deleted).toHaveLength(1);
        expect(warn.mock.calls.flat().join(' ')).toMatch(/intermediate/i);
    });

    test('engines contract: path_exists is used; a name it reports as present is skipped', async () => {
        const fixed = new Date(2026, 10, 6, 8, 0, 0, 0);
        pathExistsAnswer = (p) => p.endsWith('080000-000.pdf');
        const p = await freeOutputPath('/docs/plan.pdf', 'organized', fixed);
        expect(p).toBe('/docs/plan.organized-20261106-080000-000-2.pdf');
    });

    test('engines contract: path_exists false still gets the safe second check (stat errors report false)', async () => {
        const fixed = new Date(2026, 10, 6, 8, 1, 0, 0);
        pathExistsAnswer = () => false;
        files.add('/docs/plan.organized-20261106-080100-000.pdf'); // readable → really taken
        const p = await freeOutputPath('/docs/plan.pdf', 'organized', fixed);
        expect(p).toBe('/docs/plan.organized-20261106-080100-000-2.pdf');
    });

    test('concurrent name requests never get the same name', async () => {
        const fixed = new Date(2026, 10, 6, 8, 2, 0, 0);
        const [a, b] = await Promise.all([
            freeOutputPath('/docs/plan.pdf', 'organized', fixed),
            freeOutputPath('/docs/plan.pdf', 'organized', fixed),
        ]);
        expect(a).not.toBe(b);
    });
});

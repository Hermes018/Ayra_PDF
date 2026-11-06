import { expect, test, vi, beforeEach, describe } from 'vitest';
import { applyOrganize, type OrganizePage } from '$lib/components/organizeApply';

describe('Organize no-deletion safety (BUG-034 bullet 3)', () => {
    let deleteCalls: string[] = [];

    beforeEach(() => {
        deleteCalls = [];
    });

    test('single-step Organize apply deletes nothing (no delete IPC called)', async () => {
        const pages: OrganizePage[] = [
            { originalIndex: 1, rotation: 90, deleted: false },
            { originalIndex: 2, rotation: 0, deleted: false },
        ];

        const mockRunJob = vi.fn(async () => {});
        const mockNameFor = vi.fn(async (_input: string, tag: string) => `/tmp/out.${tag}.pdf`);
        const mockDelete = vi.fn(async (path: string) => {
            deleteCalls.push(path);
        });

        const outcome = await applyOrganize('/tmp/in.pdf', pages, mockRunJob, mockNameFor, mockDelete);

        expect(outcome.ok).toBe(true);
        expect(mockDelete).not.toHaveBeenCalled();
        expect(deleteCalls).toEqual([]);
    });

    test('failed Organize apply deletes nothing (no delete IPC called)', async () => {
        // Multi-step organize (both delete and rotate would normally have an intermediate)
        const pages: OrganizePage[] = [
            { originalIndex: 1, rotation: 90, deleted: true },
            { originalIndex: 2, rotation: 180, deleted: false },
        ];

        const mockRunJob = vi.fn(async () => {
            throw new Error('engine failure during page processing');
        });
        const mockNameFor = vi.fn(async (_input: string, tag: string) => `/tmp/out.${tag}.pdf`);
        const mockDelete = vi.fn(async (path: string) => {
            deleteCalls.push(path);
        });

        const outcome = await applyOrganize('/tmp/in.pdf', pages, mockRunJob, mockNameFor, mockDelete);

        expect(outcome.ok).toBe(false);
        if (!outcome.ok) {
            expect(outcome.error).toContain('engine failure');
        }
        expect(mockDelete).not.toHaveBeenCalled();
        expect(deleteCalls).toEqual([]);
    });

    test('failed single-step Organize apply deletes nothing', async () => {
        const pages: OrganizePage[] = [
            { originalIndex: 1, rotation: 90, deleted: false },
        ];

        const mockRunJob = vi.fn(async () => {
            throw new Error('job failed');
        });
        const mockNameFor = vi.fn(async (_input: string, tag: string) => `/tmp/out.${tag}.pdf`);
        const mockDelete = vi.fn(async (path: string) => {
            deleteCalls.push(path);
        });

        const outcome = await applyOrganize('/tmp/in.pdf', pages, mockRunJob, mockNameFor, mockDelete);

        expect(outcome.ok).toBe(false);
        expect(mockDelete).not.toHaveBeenCalled();
        expect(deleteCalls).toEqual([]);
    });
});

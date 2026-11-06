// BUG-050 (P0): an input without a ".pdf" extension made `.replace(/\.pdf$/i, ...)` return the INPUT path, so a job
// would write over the original. Every output name now comes from the collision-safe naming (outputNames).
import { expect, test, vi, beforeEach, afterEach, describe } from 'vitest';
import { render, screen, cleanup, fireEvent, waitFor } from '@testing-library/svelte';

const runJob = vi.fn(async (..._args: unknown[]) => 'id');
vi.mock('$lib/ipc', () => ({
    IpcClient: { runJob: (...a: unknown[]) => runJob(...a), submitJob: vi.fn(async () => 'id'), onJobEvent: vi.fn(async () => () => {}), cancelJob: vi.fn(), silentSubmitsSettled: vi.fn(async () => {}), isSilentJob: () => false },
    pickFilesDialog: vi.fn(async () => []),
}));
vi.mock('@tauri-apps/api/core', () => ({
    invoke: vi.fn(async (cmd: string) => {
        if (cmd === 'path_exists') return false;
        if (cmd === 'read_file_bytes') throw new Error('No such file or directory (os error 2)');
        return null;
    }),
}));
vi.mock('$lib/components/PdfViewer.svelte', () => ({ default: () => {} }));

import { freeOutputPath } from '../studio/outputNames';
import OptimizeView from '../OptimizeView.svelte';
import EditView from '../EditView.svelte';
import { appState } from '$lib/state.svelte';

const NOW = new Date(2026, 10, 6, 12, 0, 0, 0);

describe('freeOutputPath for inputs without a .pdf extension', () => {
    test('"doc" gets a distinct .pdf output with the tag', async () => {
        const out = await freeOutputPath('/docs/doc', 'rotated', NOW, async () => false);
        expect(out).not.toBe('/docs/doc');
        expect(out).toMatch(/^\/docs\/doc\.rotated-\d{8}-\d{6}-\d{3}(-\d+)?\.pdf$/);
    });
    test('an .md output keeps its extension and is never the input', async () => {
        const out = await freeOutputPath('/docs/report.pdf', 'summary', NOW, async () => false, 'md');
        expect(out).toMatch(/^\/docs\/report\.summary-\d{8}-\d{6}-\d{3}(-\d+)?\.md$/);
        const out2 = await freeOutputPath('/docs/notes', 'summary', NOW, async () => false, 'md');
        expect(out2).not.toBe('/docs/notes');
    });
});

describe('views never submit output == input', () => {
    beforeEach(() => {
        runJob.mockClear();
        appState.clear();
        appState.addFiles(['/docs/doc']); // no extension
    });
    afterEach(() => cleanup());

    test('Optimize: Compress, Repair and OCR use fresh output paths', async () => {
        render(OptimizeView);
        for (const name of ['Compress PDF', 'Repair PDF', 'Recognize text (OCR)']) {
            await fireEvent.click(screen.getByRole('button', { name }));
            await waitFor(() => expect(runJob).toHaveBeenCalled());
            const req = runJob.mock.calls.at(-1)![0] as { input_path: string; output_path: string };
            expect(req.input_path).toBe('/docs/doc');
            expect(req.output_path).not.toBe(req.input_path);
            expect(req.output_path).toMatch(/\.pdf$/);
            await waitFor(() => expect(screen.getByRole('button', { name })).not.toHaveAttribute('aria-disabled'));
        }
    });

    test('Edit: watermark and flatten use fresh output paths', async () => {
        render(EditView);
        for (const name of ['Add watermark', 'Flatten comments']) {
            await fireEvent.click(screen.getByRole('button', { name }));
            await waitFor(() => expect(runJob).toHaveBeenCalled());
            const req = runJob.mock.calls.at(-1)![0] as { input_path: string; output_path: string };
            expect(req.output_path).not.toBe(req.input_path);
            await waitFor(() => expect(screen.getByRole('button', { name })).not.toHaveAttribute('aria-disabled'));
        }
    });
});

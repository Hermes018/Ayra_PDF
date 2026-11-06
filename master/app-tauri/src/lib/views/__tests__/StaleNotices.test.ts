// A new notice replaces the old one: it never sits beside it or hides behind it. Clear the previous notice whenever
// a new action starts. Also: ConvertToPdf's wording is input-neutral (Office files are not PDFs).
import { expect, test, vi, beforeEach, afterEach, describe } from 'vitest';
import { render, screen, cleanup, fireEvent, waitFor } from '@testing-library/svelte';

vi.mock('@tauri-apps/api/core', () => ({
    invoke: vi.fn(async (cmd: string) => {
        if (cmd === 'path_exists') return false;
        if (cmd === 'read_file_bytes') throw new Error('No such file or directory (os error 2)');
        return null;
    }),
}));

import { IpcClient } from '$lib/ipc';
import { appState } from '$lib/state.svelte';
import { resetIssuedOutputNamesForTests } from '../studio/outputNames';
import ConvertFromPdfView from '../ConvertFromPdfView.svelte';
import ConvertToPdfView from '../ConvertToPdfView.svelte';
import PipelinesView from '../PipelinesView.svelte';

let jobHandler: ((ev: unknown) => void) | null = null;
const flush = () => new Promise<void>((r) => setTimeout(r, 0));
const click = async (el: HTMLElement) => {
    await fireEvent.click(el);
    await flush();
    await flush();
};

beforeEach(() => {
    jobHandler = null;
    resetIssuedOutputNamesForTests();
    appState.clear();
    appState.addFiles(['/docs/a.pdf']);
    vi.spyOn(IpcClient, 'listPipelinePresets').mockResolvedValue([{ id: 'p', name: 'P', description: '', steps: [{ type: 'Repair' }] }] as never);
    vi.spyOn(IpcClient, 'onJobEvent').mockImplementation(async (cb) => {
        jobHandler = cb as (ev: unknown) => void;
        return () => {};
    });
});
afterEach(() => {
    cleanup();
    vi.restoreAllMocks();
});

describe('a new action replaces the previous notice', () => {
    test.each([
        ['Convert from PDF', ConvertFromPdfView, /Convert Now/i],
        ['Convert to PDF', ConvertToPdfView, /Convert Now|Convert to PDF/i],
    ])('%s: after a failed job, converting again shows only the new notice', async (_n, View, btn) => {
        let n = 0;
        vi.spyOn(IpcClient, 'submitJob').mockImplementation(async () => `job-${++n}`);
        render(View);
        await click(screen.getByRole('button', { name: btn }));
        await waitFor(() => expect(jobHandler).not.toBeNull());
        jobHandler!({ kind: 'Failed', data: { job_id: 'job-1', error_message: 'engine said no' } });
        await flush();
        expect(screen.getByTestId('convert-failure')).toBeInTheDocument();
        await click(screen.getByRole('button', { name: btn }));
        expect(screen.queryByTestId('convert-failure')).toBeNull(); // the stale failure is gone
        expect(screen.getByTestId('convert-submitted')).toBeInTheDocument(); // the new notice is visible
    });

    test('Pipelines: loading a preset clears a previous failure; running clears the previous message', async () => {
        let n = 0;
        vi.spyOn(IpcClient, 'submitJob').mockImplementation(async () => `job-${++n}`);
        render(PipelinesView);
        await screen.findAllByTestId('step-card');
        await click(screen.getByTestId('btn-run-workflow'));
        await waitFor(() => expect(jobHandler).not.toBeNull());
        jobHandler!({ kind: 'Failed', data: { job_id: 'job-1', error_message: 'engine said no' } });
        await flush();
        expect(screen.getByTestId('run-failure')).toBeInTheDocument();
        await click(screen.getByTestId('btn-run-workflow')); // a new run: the old failure goes
        expect(screen.queryByTestId('run-failure')).toBeNull();
        expect(screen.getByTestId('execution-message')).toHaveTextContent(/enqueued successfully/);
    });
});

describe('input-neutral wording (behavioural)', () => {
    test('ConvertToPdf with no file says "No file is selected." (its input can be an Office file or an image)', async () => {
        appState.clear();
        render(ConvertToPdfView);
        await click(screen.getByRole('button', { name: /Convert Now|Convert to PDF/i }));
        const alert = screen.getByRole('alert');
        expect(alert).toHaveTextContent('No file is selected.');
        expect(alert).not.toHaveTextContent(/No PDF/);
    });
    test('ConvertFromPdf with no file keeps the PDF wording (its input is always a PDF)', async () => {
        appState.clear();
        render(ConvertFromPdfView);
        await click(screen.getByRole('button', { name: /Convert Now/i }));
        expect(screen.getByRole('alert')).toHaveTextContent('No PDF is selected.');
    });
});

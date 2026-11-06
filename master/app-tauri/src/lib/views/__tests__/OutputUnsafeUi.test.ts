// BUG-052: an OUTPUT_PATH_UNSAFE refusal (the backend's backstop behind the free-name check) is shown as an
// ErrorState in plain words with a Retry that uses the next free name; a failing name lookup never leaves a screen
// stuck busy, and a double click starts one job.
import { expect, test, vi, beforeEach, afterEach, describe } from 'vitest';
import { render, screen, cleanup, fireEvent, waitFor } from '@testing-library/svelte';

let diskAlwaysTaken = false;
vi.mock('@tauri-apps/api/core', () => ({
    invoke: vi.fn(async (cmd: string) => {
        if (cmd === 'path_exists') return diskAlwaysTaken;
        if (cmd === 'read_file_bytes') {
            if (diskAlwaysTaken) return [1];
            throw new Error('No such file or directory (os error 2)');
        }
        return null;
    }),
}));

import { IpcClient } from '$lib/ipc';
import { appState } from '$lib/state.svelte';
import { resetIssuedOutputNamesForTests } from '../studio/outputNames';
import PipelinesView from '../PipelinesView.svelte';
import AiSummarizeView from '../AiSummarizeView.svelte';
import MarkdownView from '../MarkdownView.svelte';
import ConvertFromPdfView from '../ConvertFromPdfView.svelte';
import ConvertToPdfView from '../ConvertToPdfView.svelte';

type Req = Record<string, unknown>;
const submitted: Req[] = [];
let jobHandler: ((ev: unknown) => void) | null = null;
const EXISTS = 'OUTPUT_PATH_UNSAFE: a file already exists at the output path (/docs/a_x.pdf). Nothing was written.';
const flush = () => new Promise<void>((r) => setTimeout(r, 0));
const click = async (el: HTMLElement) => {
    await fireEvent.click(el);
    await flush();
    await flush();
};
const out = (r: Req) => (r.output_path ?? r.output_dir) as string;

beforeEach(() => {
    submitted.length = 0;
    diskAlwaysTaken = false;
    jobHandler = null;
    resetIssuedOutputNamesForTests();
    appState.clear();
    appState.addFiles(['/docs/a.pdf']);
    vi.spyOn(IpcClient, 'getSettings').mockResolvedValue({} as never);
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

const rejectFirstWithExists = () => {
    let n = 0;
    vi.spyOn(IpcClient, 'submitJob').mockImplementation(async (r) => {
        submitted.push(r as unknown as Req);
        if (++n === 1) throw new Error(EXISTS);
        return `job-${n}`;
    });
};

function expectPlainExistsError() {
    const alert = screen.getByRole('alert');
    expect(alert).toHaveTextContent(/A file named a_x\.pdf already exists\./);
    expect(alert).toHaveTextContent(/next free name/);
    expect(document.body.textContent).not.toContain('OUTPUT_PATH_UNSAFE');
}

describe('submit-time refusal', () => {
    test('Pipelines: ErrorState, Retry submits again with a different name', async () => {
        rejectFirstWithExists();
        render(PipelinesView);
        await screen.findAllByTestId('step-card');
        await click(screen.getByTestId('btn-run-workflow'));
        expectPlainExistsError();
        await click(screen.getByRole('button', { name: 'Retry' }));
        await waitFor(() => expect(submitted).toHaveLength(2));
        expect(out(submitted[1])).not.toBe(out(submitted[0]));
    });

    test('AiSummarize: ErrorState, Retry submits again with a different name', async () => {
        rejectFirstWithExists();
        render(AiSummarizeView);
        await click(screen.getByTestId('generate-summary-btn'));
        expectPlainExistsError();
        await click(screen.getByRole('button', { name: 'Retry' }));
        await waitFor(() => expect(submitted).toHaveLength(2));
        expect(out(submitted[1])).not.toBe(out(submitted[0]));
    });
});

describe('event-time refusal (the job fails after it was queued)', () => {
    async function failFirstJobWithExists() {
        await waitFor(() => expect(jobHandler).not.toBeNull());
        jobHandler!({ kind: 'Failed', data: { job_id: 'job-1', error_message: EXISTS } });
        await flush();
    }
    function queueAsJob1() {
        vi.spyOn(IpcClient, 'submitJob').mockImplementation(async (r) => {
            submitted.push(r as unknown as Req);
            return `job-${submitted.length}`;
        });
    }

    test('Markdown: plain ErrorState; Retry gets a new name', async () => {
        queueAsJob1();
        render(MarkdownView);
        await click(screen.getByRole('button', { name: /Convert Document/i }));
        await failFirstJobWithExists();
        expectPlainExistsError();
        await click(screen.getByRole('button', { name: 'Retry' }));
        await waitFor(() => expect(submitted).toHaveLength(2));
        expect(out(submitted[1])).not.toBe(out(submitted[0]));
    });

    test('Convert from PDF: plain ErrorState; Retry gets a new name', async () => {
        queueAsJob1();
        render(ConvertFromPdfView);
        await click(screen.getByRole('button', { name: /Convert Now/i }));
        await failFirstJobWithExists();
        expectPlainExistsError();
        await click(screen.getByRole('button', { name: 'Retry' }));
        await waitFor(() => expect(submitted).toHaveLength(2));
        expect(out(submitted[1])).not.toBe(out(submitted[0]));
    });

    test('Convert to PDF: plain ErrorState; Retry gets a new name', async () => {
        queueAsJob1();
        render(ConvertToPdfView);
        await click(screen.getByRole('button', { name: /Convert Now|Convert to PDF/i }));
        await failFirstJobWithExists();
        expectPlainExistsError();
        await click(screen.getByRole('button', { name: 'Retry' }));
        await waitFor(() => expect(submitted).toHaveLength(2));
        expect(out(submitted[1])).not.toBe(out(submitted[0]));
    });
});

describe('busy guard and failing name lookup', () => {
    test('Pipelines: a double click starts ONE job (the busy flag is set before any await)', async () => {
        vi.spyOn(IpcClient, 'submitJob').mockImplementation(async (r) => {
            submitted.push(r as unknown as Req);
            return 'job-1';
        });
        render(PipelinesView);
        await screen.findAllByTestId('step-card');
        const btn = screen.getByTestId('btn-run-workflow');
        void fireEvent.click(btn);
        void fireEvent.click(btn);
        await flush();
        await flush();
        expect(submitted).toHaveLength(1);
    });

    test('Pipelines: no free name -> an ErrorState, the Run button is usable again, no unhandled rejection', async () => {
        diskAlwaysTaken = true;
        vi.spyOn(IpcClient, 'submitJob').mockResolvedValue('job-1');
        render(PipelinesView);
        await screen.findAllByTestId('step-card');
        await click(screen.getByTestId('btn-run-workflow'));
        await waitFor(() => expect(screen.getByRole('alert')).toBeInTheDocument());
        expect(screen.getByTestId('btn-run-workflow')).not.toBeDisabled();
        expect(IpcClient.submitJob).not.toHaveBeenCalled();
    });

    test('Markdown: no free name -> Convert is not stuck on "Converting..."', async () => {
        diskAlwaysTaken = true;
        vi.spyOn(IpcClient, 'submitJob').mockResolvedValue('job-1');
        render(MarkdownView);
        await click(screen.getByRole('button', { name: /Convert Document/i }));
        await waitFor(() => expect(screen.getByRole('button', { name: /Convert Document/i })).not.toBeDisabled());
        expect(IpcClient.submitJob).not.toHaveBeenCalled();
        expect(screen.getByRole('alert')).toBeInTheDocument();
    });

    test('AiSummarize: no free name -> an error is shown and Generate works again', async () => {
        diskAlwaysTaken = true;
        vi.spyOn(IpcClient, 'submitJob').mockResolvedValue('job-1');
        render(AiSummarizeView);
        await click(screen.getByTestId('generate-summary-btn'));
        await waitFor(() => expect(screen.getByRole('alert')).toBeInTheDocument());
        expect(screen.getByTestId('generate-summary-btn')).not.toBeDisabled();
    });
});

// The status messages the Convert screens always showed (the banner was dropped by mistake): each must be visible.
describe.each([
    ['Convert from PDF', ConvertFromPdfView, /Convert Now/i],
    ['Convert to PDF', ConvertToPdfView, /Convert Now|Convert to PDF/i],
])('%s status messages are visible', (_name, View, btnName) => {
    test('success: a non-blocking notice names the output file', async () => {
        vi.spyOn(IpcClient, 'submitJob').mockImplementation(async (r) => {
            submitted.push(r as unknown as Req);
            return 'job-1';
        });
        render(View);
        await click(screen.getByRole('button', { name: btnName }));
        const notice = await screen.findByTestId('convert-submitted');
        expect(notice).toHaveAttribute('role', 'status');
        const file = out(submitted[0]).split('/').pop()!;
        expect(notice).toHaveTextContent(file);
    });

    test('a failed submit: ErrorState in plain words with Retry, no raw error text', async () => {
        vi.spyOn(IpcClient, 'submitJob').mockRejectedValue(new Error('boom: raw backend text'));
        render(View);
        await click(screen.getByRole('button', { name: btnName }));
        const alert = await screen.findByTestId('convert-status-error');
        expect(alert).toHaveTextContent(/Couldn't start/);
        expect(screen.getByRole('button', { name: 'Retry' })).toBeInTheDocument();
        expect(document.body.textContent).not.toContain('boom: raw backend text');
    });

    test('no free output name: ErrorState says so', async () => {
        diskAlwaysTaken = true;
        vi.spyOn(IpcClient, 'submitJob').mockResolvedValue('job-1');
        render(View);
        await click(screen.getByRole('button', { name: btnName }));
        expect(await screen.findByTestId('convert-status-error')).toHaveTextContent(/free output file name/);
    });
});

describe('Pipelines: a queued job that later fails', () => {
    test('the "enqueued successfully" banner is replaced by the plain ErrorState', async () => {
        vi.spyOn(IpcClient, 'submitJob').mockImplementation(async (r) => {
            submitted.push(r as unknown as Req);
            return 'job-1';
        });
        render(PipelinesView);
        await screen.findAllByTestId('step-card');
        await click(screen.getByTestId('btn-run-workflow'));
        expect(screen.getByTestId('execution-message')).toHaveTextContent(/enqueued successfully/);
        await waitFor(() => expect(jobHandler).not.toBeNull());
        jobHandler!({ kind: 'Failed', data: { job_id: 'job-1', error_message: EXISTS } });
        await flush();
        expect(screen.queryByTestId('execution-message')).toBeNull();
        expectPlainExistsError();
    });
});

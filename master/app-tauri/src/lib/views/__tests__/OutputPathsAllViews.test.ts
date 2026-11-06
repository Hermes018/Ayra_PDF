// BUG-050 / BUG-051: EVERY view that writes a file submits an output that is not the input and not an existing file,
// also for an input without a ".pdf" extension, and running twice gives two distinct outputs (the backend refuses
// an existing file, so a repeated name would fail the second run with OUTPUT_PATH_UNSAFE).
import { expect, test, vi, beforeEach, afterEach, describe } from 'vitest';
import { render, screen, cleanup, fireEvent, waitFor } from '@testing-library/svelte';

const NOT_FOUND = new Error('No such file or directory (os error 2)');
const diskFiles = new Set<string>();
vi.mock('@tauri-apps/api/core', () => ({
    invoke: vi.fn(async (cmd: string, args: { path?: string }) => {
        if (cmd === 'path_exists') return diskFiles.has(args?.path ?? '');
        if (cmd === 'read_file_bytes') {
            if (diskFiles.has(args?.path ?? '')) return [1];
            throw NOT_FOUND;
        }
        return null;
    }),
}));
vi.mock('$lib/components/PdfViewer.svelte', () => ({ default: () => {} }));
vi.mock('$lib/components/GridEditor.svelte', () => ({ default: () => {} }));

import { IpcClient, pickFilesDialog } from '$lib/ipc';
import { appState } from '$lib/state.svelte';
import { resetIssuedOutputNamesForTests } from '../studio/outputNames';
import OrganizeView from '../OrganizeView.svelte';
import FormsView from '../FormsView.svelte';
import SecurityView from '../SecurityView.svelte';
import PipelinesView from '../PipelinesView.svelte';
import AiSummarizeView from '../AiSummarizeView.svelte';
import MarkdownView from '../MarkdownView.svelte';
import ConvertFromPdfView from '../ConvertFromPdfView.svelte';
import ConvertToPdfView from '../ConvertToPdfView.svelte';

type Req = Record<string, unknown>;
const submitted: Req[] = [];
const outputOf = (r: Req) => (r.output_path ?? r.output_dir) as string;
const flush = () => new Promise<void>((r) => setTimeout(r, 0));
const click = async (el: HTMLElement) => {
    await fireEvent.click(el);
    await flush();
    await flush();
};

beforeEach(() => {
    submitted.length = 0;
    diskFiles.clear();
    resetIssuedOutputNamesForTests();
    appState.clear();
    vi.spyOn(IpcClient, 'submitJob').mockImplementation(async (r) => {
        submitted.push(r as Req);
        return `job-${submitted.length}`;
    });
    vi.spyOn(IpcClient, 'runJob').mockImplementation(async (r) => {
        submitted.push(r as unknown as Req);
        return `job-${submitted.length}`;
    });
    vi.spyOn(IpcClient, 'getSettings').mockResolvedValue({} as never);
    vi.spyOn(IpcClient, 'listPipelinePresets').mockResolvedValue([
        { id: 'p', name: 'P', description: '', steps: [{ type: 'Repair' }] },
    ] as never);
});
afterEach(() => {
    cleanup();
    vi.restoreAllMocks();
});

function expectSafeOutputs(input: string) {
    expect(submitted.length).toBeGreaterThanOrEqual(1);
    for (const r of submitted) {
        const out = outputOf(r);
        expect(out, JSON.stringify(r)).toBeTruthy();
        expect(out).not.toBe(input);
        expect(diskFiles.has(out)).toBe(false);
    }
}

const INPUTS = ['/docs/doc', '/docs/doc.pdf'];

describe.each(INPUTS)('every writer: input %s', (input) => {
    beforeEach(() => {
        appState.addFiles([input]);
    });

    test('Organize: Extract, Delete, Rotate pages, Rotate document', async () => {
        render(OrganizeView);
        for (const [label, name] of [['Pages to extract', 'Extract'], ['Pages to delete', 'Delete pages'], ['Pages to rotate', 'Rotate pages']] as const) {
            await fireEvent.input(screen.getByLabelText(label), { target: { value: '1' } });
            await click(screen.getByRole('button', { name }));
        }
        await click(screen.getByRole('button', { name: 'Rotate all pages' }));
        expect(submitted.map((r) => r.op)).toEqual(['ExtractPages', 'DeletePages', 'RotatePages', 'RotateDocument']);
        expectSafeOutputs(input);
    });

    test('Organize: Merge all', async () => {
        appState.addFiles([input + '-b']);
        render(OrganizeView);
        await click(screen.getByTestId('organize-merge-btn'));
        expect(submitted.map((r) => r.op)).toEqual(['Merge']);
        expectSafeOutputs(input);
    });

    test('Forms: flatten', async () => {
        render(FormsView);
        await click(screen.getByRole('button', { name: /Flatten All Forms/i }));
        expectSafeOutputs(input);
    });

    test('Security: protect, unlock, redact', async () => {
        render(SecurityView);
        await fireEvent.input(document.getElementById('protectPassword') as HTMLElement, { target: { value: 'pw' } });
        await click(screen.getByRole('button', { name: /Protect PDF/i }));
        await fireEvent.input(document.getElementById('unlockPassword') as HTMLElement, { target: { value: 'pw' } });
        await click(screen.getByRole('button', { name: /Remove Security/i }));
        await click(screen.getByRole('button', { name: /Apply Redaction/i }));
        expect(submitted.length).toBe(3);
        expectSafeOutputs(input);
    });

    test('Pipelines: run workflow twice gives two distinct outputs', async () => {
        render(PipelinesView);
        await screen.findAllByTestId('step-card');
        await click(screen.getByTestId('btn-run-workflow'));
        await click(screen.getByTestId('btn-run-workflow'));
        expect(submitted.map((r) => r.op)).toEqual(['RunPipeline', 'RunPipeline']);
        expect(outputOf(submitted[0])).not.toBe(outputOf(submitted[1]));
        expectSafeOutputs(input);
    });

    test('AiSummarize: extractive summary twice gives two distinct outputs', async () => {
        render(AiSummarizeView);
        await fireEvent.input(screen.getByTestId('input-path'), { target: { value: input } });
        await click(screen.getByTestId('generate-summary-btn'));
        await click(screen.getByTestId('generate-summary-btn'));
        expect(submitted).toHaveLength(2);
        expect(outputOf(submitted[0])).not.toBe(outputOf(submitted[1]));
        expectSafeOutputs(input);
    });

    test('Convert from PDF: twice gives two distinct outputs', async () => {
        render(ConvertFromPdfView);
        await click(screen.getByRole('button', { name: /Convert Now/i }));
        await click(screen.getByRole('button', { name: /Convert Now/i }));
        expect(submitted).toHaveLength(2);
        expect(outputOf(submitted[0])).not.toBe(outputOf(submitted[1]));
        expectSafeOutputs(input);
    });

    test('Convert to PDF: twice gives two distinct outputs', async () => {
        render(ConvertToPdfView);
        await click(screen.getByRole('button', { name: /Convert Now|Convert to PDF/i }));
        await click(screen.getByRole('button', { name: /Convert Now|Convert to PDF/i }));
        expect(submitted).toHaveLength(2);
        expect(outputOf(submitted[0])).not.toBe(outputOf(submitted[1]));
        expectSafeOutputs(input);
    });
});

describe('Markdown: output is never the input, also for .md and extension-less sources', () => {
    test.each(['/docs/note.md', '/docs/note', '/docs/note.pdf'])('MarkdownToPdf / PdfToMarkdown for %s', async (input) => {
        appState.addFiles([input]);
        render(MarkdownView);
        // (the button stays busy until the job completes, so one run per mount; "twice" is covered by resolveFreeOutput)
        await click(screen.getByRole('button', { name: /Convert Document/i }));
        expect(submitted).toHaveLength(1);
        expect(outputOf(submitted[0])).not.toBe(input);
        expect(diskFiles.has(outputOf(submitted[0]))).toBe(false);
    });
});

describe('Batch queue: compress outputs', () => {
    test('files added to the batch get fresh outputs, never their own path', async () => {
        const spy = vi.spyOn(IpcClient, 'submitBatch').mockImplementation(async (reqs) => {
            submitted.push(...(reqs as unknown as Req[]));
            return reqs.map((_r, i) => `b${i}`);
        });
        vi.spyOn(IpcClient, 'listJobs').mockResolvedValue([]);
        vi.mocked(pickFilesDialog);
        const mod = await import('$lib/ipc');
        vi.spyOn(mod, 'pickFilesDialog').mockResolvedValue(['/docs/one', '/docs/two.pdf']);
        const { default: BatchQueueView } = await import('../BatchQueueView.svelte');
        render(BatchQueueView);
        await click(screen.getByRole('button', { name: /Add Files|Add files|Browse/i }));
        await waitFor(() => expect(spy).toHaveBeenCalled());
        for (const r of submitted) {
            expect(outputOf(r)).not.toBe(r.input_path);
            expect(outputOf(r)).toMatch(/\.pdf$/);
        }
    });
});

// BUG-058: no console call (log, info, debug, warn, error) may contain a password, in any screen that takes one,
// whether the job succeeds or fails.
import { expect, test, vi, beforeEach, afterEach, describe } from 'vitest';
import { render, screen, cleanup, fireEvent } from '@testing-library/svelte';

vi.mock('@tauri-apps/api/core', () => ({
    invoke: vi.fn(async (cmd: string) => {
        if (cmd === 'path_exists') return false;
        if (cmd === 'read_file_bytes') throw new Error('No such file or directory (os error 2)');
        return null;
    }),
}));
vi.mock('@tauri-apps/plugin-dialog', () => ({ open: vi.fn(async () => '/certs/me.p12') }));
vi.mock('$lib/components/PdfViewer.svelte', () => ({
    default: (_a: unknown, props: { onpagecount?: (n: number) => void }) => {
        queueMicrotask(() => props?.onpagecount?.(5));
    },
}));

import { IpcClient } from '$lib/ipc';
import { appState } from '$lib/state.svelte';
import { resetIssuedOutputNamesForTests } from '../studio/outputNames';
import SecurityView from '../SecurityView.svelte';
import FormsView from '../FormsView.svelte';
import PipelinesView from '../PipelinesView.svelte';

const PW = 'hunter2-TOP-SECRET-pw';
const logged: string[] = [];
const flush = () => new Promise<void>((r) => setTimeout(r, 0));
const click = async (el: HTMLElement) => {
    await fireEvent.click(el);
    await flush();
    await flush();
};
const text = (args: unknown[]) =>
    args
        .map((a) => (a instanceof Error ? `${a.name}: ${a.message}\n${a.stack ?? ''}` : typeof a === 'string' ? a : JSON.stringify(a)))
        .join(' ');

let failJobs = false;
beforeEach(() => {
    logged.length = 0;
    failJobs = false;
    resetIssuedOutputNamesForTests();
    appState.clear();
    appState.addFiles(['/docs/a.pdf']);
    for (const m of ['log', 'info', 'debug', 'warn', 'error'] as const) {
        vi.spyOn(console, m).mockImplementation((...a: unknown[]) => {
            logged.push(text(a));
        });
    }
    vi.spyOn(IpcClient, 'runJob').mockImplementation(async () => {
        if (failJobs) throw new Error('engine said no');
        return 'job';
    });
    vi.spyOn(IpcClient, 'submitJob').mockImplementation(async () => {
        if (failJobs) throw new Error('engine said no');
        return 'job-1';
    });
    vi.spyOn(IpcClient, 'listPipelinePresets').mockResolvedValue([]);
    vi.spyOn(IpcClient, 'onJobEvent').mockResolvedValue(() => {});
});
afterEach(() => {
    cleanup();
    vi.restoreAllMocks();
});

const noLeak = () => expect(logged.filter((l) => l.includes(PW))).toEqual([]);

describe.each([[false], [true]])('failJobs=%s', (fail) => {
    test('Security: protect and unlock', async () => {
        failJobs = fail;
        render(SecurityView);
        await fireEvent.input(screen.getByTestId('protect-password'), { target: { value: PW } });
        await click(screen.getByRole('button', { name: 'Protect PDF' }));
        await fireEvent.input(screen.getByTestId('unlock-password'), { target: { value: PW } });
        await click(screen.getByRole('button', { name: 'Remove security' }));
        noLeak();
    });

    test('Forms: sign', async () => {
        failJobs = fail;
        render(FormsView);
        await click(screen.getByRole('button', { name: 'Browse' }));
        await flush();
        await fireEvent.input(screen.getByTestId('sign-password'), { target: { value: PW } });
        await click(screen.getByRole('button', { name: 'Sign document' }));
        noLeak();
    });

    test('Pipelines: a Protect step', async () => {
        failJobs = fail;
        render(PipelinesView);
        await fireEvent.change(screen.getByTestId('select-new-step'), { target: { value: 'Protect' } });
        await click(screen.getByTestId('btn-add-step'));
        await fireEvent.input(screen.getByTestId('protect-password-input'), { target: { value: PW } });
        await click(screen.getByTestId('btn-run-workflow'));
        noLeak();
    });
});

// SECURITY: a Protect step used to be created with the hard-coded password "password123" and could be run without the
// user ever seeing it. It now starts empty and a workflow with an empty Protect password is refused.
import { expect, test, vi, beforeEach, afterEach, describe } from 'vitest';
import { render, screen, cleanup, fireEvent } from '@testing-library/svelte';

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
import PipelinesView from '../PipelinesView.svelte';

const flush = () => new Promise<void>((r) => setTimeout(r, 0));
const click = async (el: HTMLElement) => {
    await fireEvent.click(el);
    await flush();
    await flush();
};

beforeEach(() => {
    resetIssuedOutputNamesForTests();
    appState.clear();
    appState.addFiles(['/docs/a.pdf']);
    vi.spyOn(IpcClient, 'listPipelinePresets').mockResolvedValue([]);
    vi.spyOn(IpcClient, 'onJobEvent').mockResolvedValue(() => {});
});
afterEach(() => {
    cleanup();
    vi.restoreAllMocks();
});

async function addProtectStep() {
    render(PipelinesView);
    await fireEvent.change(screen.getByTestId('select-new-step'), { target: { value: 'Protect' } });
    await click(screen.getByTestId('btn-add-step'));
}

describe('Pipelines Protect step', () => {
    test('a new Protect step has an EMPTY password (no hidden default)', async () => {
        await addProtectStep();
        const pw = screen.getByTestId('protect-password-input') as HTMLInputElement;
        expect(pw.value).toBe('');
        expect(pw.type).toBe('password');
        expect(pw).toHaveAttribute('autocomplete', 'off');
    });

    test('running with an empty Protect password is refused with a clear message, nothing is submitted', async () => {
        const submit = vi.spyOn(IpcClient, 'submitJob').mockResolvedValue('job-1');
        await addProtectStep();
        await click(screen.getByTestId('btn-run-workflow'));
        expect(submit).not.toHaveBeenCalled();
        expect(screen.getByTestId('execution-message')).toHaveTextContent(/password for the Protect step/i);
    });

    test('with a password typed, the workflow runs and sends exactly that password', async () => {
        const submit = vi.spyOn(IpcClient, 'submitJob').mockResolvedValue('job-1');
        await addProtectStep();
        await fireEvent.input(screen.getByTestId('protect-password-input'), { target: { value: 'my-own-pw' } });
        await click(screen.getByTestId('btn-run-workflow'));
        expect(submit).toHaveBeenCalledTimes(1);
        const req = submit.mock.calls[0][0] as { steps: Array<{ type: string; password?: string }> };
        expect(req.steps.find((s) => s.type === 'Protect')?.password).toBe('my-own-pw');
    });
});

// Number fields: a real 0 stays 0; an empty (or invalid) field means the field's old default.
import { expect, test, vi, beforeEach, afterEach, describe } from 'vitest';
import { render, screen, cleanup, fireEvent, waitFor } from '@testing-library/svelte';

const runJob = vi.fn(async (..._args: unknown[]) => 'id');
vi.mock('$lib/ipc', () => ({
    IpcClient: { runJob: (...a: unknown[]) => runJob(...a), submitJob: vi.fn(async () => 'id'), onJobEvent: vi.fn(async () => () => {}), silentSubmitsSettled: vi.fn(async () => {}), isSilentJob: () => false },
    pickFilesDialog: vi.fn(async () => []),
}));
vi.mock('@tauri-apps/api/core', () => ({
    invoke: vi.fn(async (cmd: string) => {
        if (cmd === 'path_exists') return false;
        throw new Error('No such file or directory (os error 2)');
    }),
}));
vi.mock('$lib/components/PdfViewer.svelte', () => ({ default: () => {} }));

import EditView from '../EditView.svelte';
import { parseNumberOr } from '$lib/core/parseNumber';
import { appState } from '$lib/state.svelte';

describe('parseNumberOr', () => {
    test('0 stays 0; empty, blank and invalid use the fallback; decimals and negatives parse', () => {
        expect(parseNumberOr('0', 0.5)).toBe(0);
        expect(parseNumberOr('0.0', 0.5)).toBe(0);
        expect(parseNumberOr('', 0.5)).toBe(0.5);
        expect(parseNumberOr('   ', 0.5)).toBe(0.5);
        expect(parseNumberOr('abc', 7)).toBe(7);
        expect(parseNumberOr('-45', 1)).toBe(-45);
        expect(parseNumberOr('0.25', 1)).toBe(0.25);
        expect(parseNumberOr('Infinity', 3)).toBe(3);
        // what bind:value on a number input actually yields
        expect(parseNumberOr(0, 0.5)).toBe(0);
        expect(parseNumberOr(null, 0.5)).toBe(0.5);
        expect(parseNumberOr(undefined, 0.5)).toBe(0.5);
        expect(parseNumberOr(NaN, 2)).toBe(2);
    });
});

describe('Edit view numbers', () => {
    beforeEach(() => {
        runJob.mockClear();
        appState.clear();
        appState.addFiles(['/docs/a.pdf']);
    });
    afterEach(() => cleanup());

    async function watermarkSettings() {
        await fireEvent.click(screen.getByRole('button', { name: 'Add watermark' }));
        await waitFor(() => expect(runJob).toHaveBeenCalled());
        return (runJob.mock.calls.at(-1)![0] as { settings: { opacity: number; rotation: number } }).settings;
    }

    test('opacity 0 and angle 0 are sent as 0', async () => {
        render(EditView);
        await fireEvent.input(screen.getByLabelText('Opacity'), { target: { value: '0' } });
        await fireEvent.input(screen.getByLabelText('Angle'), { target: { value: '0' } });
        const s = await watermarkSettings();
        expect(s.opacity).toBe(0);
        expect(s.rotation).toBe(0);
    });

    test('empty fields mean the old defaults (opacity 0.5, angle 45)', async () => {
        render(EditView);
        await fireEvent.input(screen.getByLabelText('Opacity'), { target: { value: '' } });
        await fireEvent.input(screen.getByLabelText('Angle'), { target: { value: '' } });
        const s = await watermarkSettings();
        expect(s.opacity).toBe(0.5);
        expect(s.rotation).toBe(45);
    });
});

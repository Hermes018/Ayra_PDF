// The fixed-position "sample highlight" is a demo, not a feature: Labs only, off by default, honest label.
import { expect, test, vi, beforeEach, afterEach, describe } from 'vitest';
import { render, screen, cleanup } from '@testing-library/svelte';
import { flushSync } from 'svelte';

vi.mock('$lib/ipc', () => ({
    IpcClient: { runJob: vi.fn(async () => 'id'), submitJob: vi.fn(async () => 'id'), onJobEvent: vi.fn(async () => () => {}), silentSubmitsSettled: vi.fn(async () => {}), isSilentJob: () => false },
    pickFilesDialog: vi.fn(async () => []),
}));
vi.mock('$lib/components/PdfViewer.svelte', () => ({ default: () => {} }));

import EditView from '../EditView.svelte';
import { appState } from '$lib/state.svelte';
import { setLabs, resetLabsGateForTests } from '$lib/components/studio/labs.svelte';

describe('Edit view sample highlight (Labs)', () => {
    beforeEach(() => {
        resetLabsGateForTests();
        appState.clear();
        appState.addFiles(['/docs/a.pdf']);
    });
    afterEach(() => {
        cleanup();
        resetLabsGateForTests();
    });

    test('hidden when Labs is off (default)', () => {
        render(EditView);
        expect(screen.queryByRole('button', { name: /sample highlight/i })).not.toBeInTheDocument();
        expect(screen.getByRole('button', { name: 'Flatten comments' })).toBeInTheDocument();
    });

    test('shown as "Sample highlight (Labs)" when Labs is on', () => {
        setLabs(true);
        flushSync();
        render(EditView);
        expect(screen.getByRole('button', { name: /Sample highlight \(Labs\)/ })).toBeInTheDocument();
    });
});

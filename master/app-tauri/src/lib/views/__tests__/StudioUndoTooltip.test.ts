import { expect, test, describe, afterEach } from 'vitest';
import { render, screen, cleanup, waitFor } from '@testing-library/svelte';
import { flushSync } from 'svelte';
import SpatialCanvas from '../studio/SpatialCanvas.svelte';
import { commands } from '$lib/core/commandEngine.svelte';

// The Undo button keeps its accessible name "Undo"; its tooltip says what will be undone.
const tip = () => {
    const b = screen.getByRole('button', { name: 'Undo' });
    return document.getElementById(b.getAttribute('aria-describedby') ?? '')?.textContent?.trim();
};

describe('Undo tooltip wiring', () => {
    afterEach(() => {
        cleanup();
    });

    test('shows "Undo: <label>" and follows commands.undoLabel; plain "Undo" when there is nothing to undo', async () => {
        commands.register('test.edit', (doc, args) => ({
            next: { rev: doc.rev + 1, pages: doc.pages },
            label: (args as { label: string }).label,
        }));
        render(SpatialCanvas);
        expect(screen.getByRole('button', { name: 'Undo' })).toHaveAttribute('aria-label', 'Undo');
        expect(tip()).toBe('Undo: Nothing to undo'); // disabled: the reason

        commands.dispatch({ kind: 'execute', command: 'test.edit', args: { label: 'Draw stroke' } });
        flushSync();
        await waitFor(() => expect(tip()).toMatch(/^Undo: Draw stroke/));

        commands.dispatch({ kind: 'execute', command: 'test.edit', args: { label: 'Move shape' } });
        flushSync();
        await waitFor(() => expect(tip()).toMatch(/^Undo: Move shape/));
    });
});

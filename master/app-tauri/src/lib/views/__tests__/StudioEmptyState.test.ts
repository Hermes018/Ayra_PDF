import { expect, test, beforeEach, afterEach, describe } from 'vitest';
import { render, screen, cleanup } from '@testing-library/svelte';
import SpatialCanvas from '../studio/SpatialCanvas.svelte';
import { appState } from '$lib/state.svelte';
import { viewport } from '$lib/core/viewportEngine.svelte';

// UI-BUG-006 (reopened 2026-11-05): with no PDF open the Studio must show a real empty
// state — no phantom "Page 1 / 2" placeholder pages and no live document-only actions.
describe('Studio empty state (UI-BUG-006)', () => {
    beforeEach(() => {
        appState.clear();
        viewport.pages = [];
    });
    afterEach(() => {
        cleanup();
    });

    test('shows "Open a PDF" instead of placeholder pages', () => {
        render(SpatialCanvas);
        expect(screen.getByTestId('studio-empty-state')).toBeInTheDocument();
        expect(screen.getByRole('button', { name: 'Open PDF…' })).toBeEnabled();
        expect(screen.queryByText(/\/\s*2/)).not.toBeInTheDocument();
        expect(viewport.pages.length).toBe(0);
    });

    test('document-only actions are disabled with a reason', () => {
        render(SpatialCanvas);
        for (const name of [/^Save/, /^Flatten/, 'Snap to drawing geometry', 'Digital drafting ruler']) {
            const btn = screen.getByRole('button', { name });
            expect(btn).toBeDisabled();
            expect(btn).toHaveAttribute('title', 'Open a PDF first');
        }
        expect(screen.getByRole('button', { name: /Edit Text/ })).toBeDisabled();
    });

    test('the Ledger stays available without a document', () => {
        render(SpatialCanvas);
        expect(screen.getByRole('button', { name: /Ledger/ })).toBeEnabled();
    });
});

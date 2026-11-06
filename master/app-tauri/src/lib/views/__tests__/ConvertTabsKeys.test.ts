// WAI-ARIA tabs keyboard pattern on the Convert tablist: one tab stop (roving tabindex), ArrowLeft/Right wrap,
// Home/End jump. The keys come through commandEngine (scoped while a tab has focus), not a local listener.
import { expect, test, vi, beforeEach, afterEach, describe } from 'vitest';
import { render, screen, cleanup, fireEvent } from '@testing-library/svelte';
import { tick } from 'svelte';

vi.mock('../ConvertFromPdfView.svelte', () => ({ default: () => {} }));
vi.mock('../ConvertToPdfView.svelte', () => ({ default: () => {} }));

import ConvertView from '../ConvertView.svelte';
import { commands } from '$lib/core/commandEngine.svelte';

let detach: () => void = () => {};
beforeEach(() => {
    detach = commands.attachKeyboard(window);
});
afterEach(() => {
    detach();
    cleanup();
});

async function press(key: string) {
    window.dispatchEvent(new KeyboardEvent('keydown', { key, bubbles: true, cancelable: true }));
    await tick();
    await tick();
}
const tabs = () => screen.getAllByRole('tab');
// jsdom's focus() does not always dispatch focusin; real browsers do. Make it explicit.
async function focusTab(i: number) {
    tabs()[i].focus();
    await fireEvent.focusIn(tabs()[i]);
}

describe('Convert tabs keyboard pattern', () => {
    test('roving tabindex: the selected tab is the only tab stop', async () => {
        render(ConvertView);
        expect(tabs().map((t) => t.tabIndex)).toEqual([0, -1]);
        await focusTab(0);
        await press('ArrowRight');
        expect(tabs().map((t) => t.tabIndex)).toEqual([-1, 0]);
    });

    test('ArrowRight / ArrowLeft move selection and focus, and wrap', async () => {
        render(ConvertView);
        await focusTab(0);
        await press('ArrowRight');
        expect(tabs()[1]).toHaveAttribute('aria-selected', 'true');
        expect(tabs()[1]).toHaveFocus();
        await press('ArrowRight'); // wraps
        expect(tabs()[0]).toHaveAttribute('aria-selected', 'true');
        expect(tabs()[0]).toHaveFocus();
        await press('ArrowLeft'); // wraps back
        expect(tabs()[1]).toHaveAttribute('aria-selected', 'true');
    });

    test('Home and End jump to the first and last tab', async () => {
        render(ConvertView);
        await focusTab(0);
        await press('End');
        expect(tabs()[1]).toHaveAttribute('aria-selected', 'true');
        await press('Home');
        expect(tabs()[0]).toHaveAttribute('aria-selected', 'true');
        expect(tabs()[0]).toHaveFocus();
    });

    test('with focus outside the tablist the arrow keys do nothing here', async () => {
        render(ConvertView);
        (document.activeElement as HTMLElement | null)?.blur();
        await press('ArrowRight');
        expect(tabs()[0]).toHaveAttribute('aria-selected', 'true');
    });
});

describe('commands.registerKeyScope', () => {
    test('a scoped handler sees keys first, consuming them stops normal dispatch, unregister removes it', () => {
        const seen: string[] = [];
        const off = commands.registerKeyScope((e) => {
            seen.push(e.key);
            return e.key === 'q';
        });
        const q = new KeyboardEvent('keydown', { key: 'q', bubbles: true, cancelable: true });
        window.dispatchEvent(q);
        expect(q.defaultPrevented).toBe(true);
        window.dispatchEvent(new KeyboardEvent('keydown', { key: 'w', bubbles: true, cancelable: true }));
        off();
        window.dispatchEvent(new KeyboardEvent('keydown', { key: 'e', bubbles: true, cancelable: true }));
        expect(seen).toEqual(['q', 'w']);
    });
});

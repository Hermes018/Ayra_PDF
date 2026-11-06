// Characterization of FaqModal keyboard behaviour (recorded against the old local window listener,
// must stay green after the move into commandEngine).
import { expect, test, beforeEach, afterEach, describe } from 'vitest';
import { render, cleanup } from '@testing-library/svelte';
import { tick } from 'svelte';
import FaqModal from './FaqModal.svelte';
import { faqStore } from '$lib/i18n/faqStore.svelte';
import { commands } from '$lib/core/commandEngine.svelte';

let detach: () => void = () => {};
beforeEach(() => {
    faqStore.close();
    detach = commands.attachKeyboard(window);
});
afterEach(() => {
    detach();
    cleanup();
    faqStore.close();
});

async function press(target: EventTarget, key: string) {
    target.dispatchEvent(new KeyboardEvent('keydown', { key, bubbles: true, cancelable: true }));
    await tick();
}

describe('FaqModal keys', () => {
    test('Escape on window closes an open FAQ', async () => {
        render(FaqModal);
        faqStore.open();
        await tick();
        await press(window, 'Escape');
        expect(faqStore.isOpen).toBe(false);
    });
    test('Escape closes from focus inside the dialog (select)', async () => {
        const { container } = render(FaqModal);
        faqStore.open();
        await tick();
        const sel = container.ownerDocument.querySelector('select');
        expect(sel).not.toBeNull();
        (sel as HTMLElement).focus();
        await press(sel as HTMLElement, 'Escape');
        expect(faqStore.isOpen).toBe(false);
    });
    test('Escape closes even when focus is in an input/textarea', async () => {
        render(FaqModal);
        faqStore.open();
        await tick();
        for (const tag of ['input', 'textarea']) {
            faqStore.open();
            await tick(); // ui/Dialog registers Escape in an effect while open
            const el = document.createElement(tag);
            document.body.appendChild(el);
            el.focus();
            await press(el, 'Escape');
            expect(faqStore.isOpen, tag).toBe(false);
            el.remove();
        }
    });
    test('other keys do not close; Escape when closed is a no-op', async () => {
        render(FaqModal);
        faqStore.open();
        await tick();
        await press(window, 'Enter');
        await press(window, 'a');
        expect(faqStore.isOpen).toBe(true);
        faqStore.close();
        await press(window, 'Escape');
        expect(faqStore.isOpen).toBe(false);
    });
    test('Escape with another dialog open closes the FAQ modal', async () => {
        let otherClosed = false;
        const unregisterOther = commands.registerEscape(() => {
            otherClosed = true;
            return true;
        });
        render(FaqModal);
        faqStore.open();
        await tick();
        expect(faqStore.isOpen).toBe(true);

        await press(window, 'Escape');
        expect(faqStore.isOpen).toBe(false);
        expect(otherClosed).toBe(true);
        unregisterOther();
    });
    test('Escape after unmount does not throw or reopen', async () => {
        const { unmount } = render(FaqModal);
        faqStore.open();
        await tick();
        expect(faqStore.isOpen).toBe(true);
        unmount();
        await tick();
        await expect(press(window, 'Escape')).resolves.not.toThrow();
        expect(faqStore.isOpen).toBe(true);
    });
});

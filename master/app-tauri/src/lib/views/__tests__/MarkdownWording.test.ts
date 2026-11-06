// The Markdown screen only claims what is true for each operation.
import { expect, test, vi, afterEach, describe } from 'vitest';
import { render, screen, cleanup, fireEvent } from '@testing-library/svelte';
import MarkdownView from '../MarkdownView.svelte';

afterEach(() => {
    cleanup();
    vi.restoreAllMocks();
});

describe('Markdown safe-writes wording', () => {
    test('says single-file conversions use a new name and that a folder batch writes into the chosen folder', () => {
        render(MarkdownView);
        const li = screen.getByText(/Safe writes:/).closest('li') as HTMLElement;
        expect(li).toHaveTextContent(/single-file conversions/i);
        expect(li).toHaveTextContent(/new name/i);
        expect(li).toHaveTextContent(/folder batch/i);
    });

    test('never claims that existing files are never overwritten across the board', () => {
        render(MarkdownView);
        expect(document.body.textContent).not.toMatch(/Existing files are never overwritten; a new name is used\./);
    });
});

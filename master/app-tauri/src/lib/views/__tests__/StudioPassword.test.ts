import { expect, test, vi, beforeEach, afterEach, describe } from 'vitest';
import { render, screen, cleanup, fireEvent, waitFor } from '@testing-library/svelte';

// BUG-011: opening an encrypted PDF must prompt for the password (pdf.js PasswordException:
// code 1 = needs password, 2 = incorrect) and open it once the right password is given.

const { submitted } = vi.hoisted(() => ({ submitted: [] as any[] }));

vi.mock('@tauri-apps/api/core', () => ({
    invoke: vi.fn(async (cmd: string, args: any) => {
        if (cmd === 'read_file_bytes') return [37, 80, 68, 70];
        // The backend cannot open an encrypted file without its password (we never pass it).
        if (cmd === 'submit_job') {
            submitted.push(args.request);
            return 'probe-1';
        }
        if (cmd === 'get_job_status') return 'Failed';
        if (cmd === 'list_jobs') return [{ job_id: 'probe-1', error_message: 'document is encrypted', error_code: null }];
        return null;
    }),
}));

import { commands } from '$lib/core/commandEngine.svelte';
import SpatialCanvas from '../studio/SpatialCanvas.svelte';
import { appState } from '$lib/state.svelte';
import { viewport } from '$lib/core/viewportEngine.svelte';

const PASSWORD = 'lpdf-test';
const attempts: Array<string | undefined> = [];

function passwordError(code: number) {
    const e = new Error(code === 1 ? 'No password given' : 'Incorrect Password') as Error & { code: number };
    e.name = 'PasswordException';
    e.code = code;
    return e;
}

function encryptedPdfjs() {
    return {
        GlobalWorkerOptions: {},
        getDocument: (src: { data: Uint8Array; password?: string }) => {
            attempts.push(src.password);
            if (src.password !== PASSWORD) {
                return { promise: Promise.reject(passwordError(src.password ? 2 : 1)), destroy: async () => {} };
            }
            return {
                promise: Promise.resolve({
                    numPages: 2,
                    getPage: async () => ({
                        getViewport: ({ scale }: { scale: number }) => ({ width: 612 * scale, height: 792 * scale }),
                        render: () => ({ promise: Promise.resolve() }),
                    }),
                    destroy: async () => {},
                }),
            };
        },
    };
}

describe('Encrypted PDF password prompt (BUG-011)', () => {
    beforeEach(() => {
        attempts.length = 0;
        submitted.length = 0;
        (window as any).pdfjsLib = encryptedPdfjs();
        appState.clear();
        viewport.pages = [];
        appState.addFiles(['/docs/encrypted.pdf']);
    });

    afterEach(() => {
        cleanup();
        delete (window as any).pdfjsLib;
        appState.clear();
        viewport.pages = [];
    });

    test('prompts, rejects a wrong password with a message, then opens with the right one', async () => {
        render(SpatialCanvas);
        const dialog = await screen.findByTestId('pdf-password-dialog');
        expect(dialog).toHaveTextContent('encrypted.pdf');
        expect(viewport.pages).toHaveLength(0);

        const input = screen.getByTestId('pdf-password-input') as HTMLInputElement;
        expect(input.type).toBe('password');
        await fireEvent.input(input, { target: { value: 'wrong' } });
        await fireEvent.click(screen.getByTestId('pdf-password-submit'));
        await waitFor(() => expect(screen.getByTestId('pdf-password-error')).toHaveTextContent(/incorrect/i));

        await fireEvent.input(screen.getByTestId('pdf-password-input'), { target: { value: PASSWORD } });
        await fireEvent.click(screen.getByTestId('pdf-password-submit'));
        await waitFor(() => expect(viewport.pages).toHaveLength(2));
        expect(screen.queryByTestId('pdf-password-dialog')).not.toBeInTheDocument();
        expect(attempts).toEqual([undefined, 'wrong', PASSWORD]);

        // An encrypted file is not "damaged": no backend probe, no damaged banner.
        await new Promise((r) => setTimeout(r, 300));
        expect(submitted.filter((r) => r.op === 'GetInfo')).toHaveLength(0);
        expect(screen.queryByText(/damaged/i)).not.toBeInTheDocument();
    });

    test('cancelling leaves a visible notice instead of a silent empty canvas', async () => {
        render(SpatialCanvas);
        await screen.findByTestId('pdf-password-dialog');
        await fireEvent.click(screen.getByTestId('pdf-password-cancel'));
        expect(screen.queryByTestId('pdf-password-dialog')).not.toBeInTheDocument();
        expect(await screen.findByTestId('studio-status-banner')).toHaveTextContent(/password/i);
    });

    // SECURITY (presentation-only move to ui/Dialog): Escape is the same as Cancel, the typed password
    // never reaches pdf.js on cancel, and the field is empty when the prompt comes back (DO-15).
    test('Escape cancels exactly like Cancel and no password attempt is made with typed text', async () => {
        const detach = commands.attachKeyboard(window);
        try {
            render(SpatialCanvas);
            await screen.findByTestId('pdf-password-dialog');
            await fireEvent.input(screen.getByTestId('pdf-password-input'), { target: { value: 'half-typed' } });
            window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true }));
            await waitFor(() => expect(screen.queryByTestId('pdf-password-dialog')).not.toBeInTheDocument());
            expect(await screen.findByTestId('studio-status-banner')).toHaveTextContent(/password/i);
            expect(attempts).toEqual([undefined]); // only the first, password-less attempt
            expect(viewport.pages).toHaveLength(0);
        } finally {
            detach();
        }
    });

    test('submitting the form (Enter in the field) opens with the typed password; the field starts empty on a retry', async () => {
        render(SpatialCanvas);
        await screen.findByTestId('pdf-password-dialog');
        await fireEvent.input(screen.getByTestId('pdf-password-input'), { target: { value: 'wrong' } });
        await fireEvent.submit(screen.getByTestId('pdf-password-input').closest('form') as HTMLFormElement);
        await waitFor(() => expect(screen.getByTestId('pdf-password-error')).toBeInTheDocument());
        expect((screen.getByTestId('pdf-password-input') as HTMLInputElement).value).toBe('');
        await fireEvent.input(screen.getByTestId('pdf-password-input'), { target: { value: PASSWORD } });
        await fireEvent.submit(screen.getByTestId('pdf-password-input').closest('form') as HTMLFormElement);
        await waitFor(() => expect(viewport.pages).toHaveLength(2));
        expect(attempts).toEqual([undefined, 'wrong', PASSWORD]);
    });

    // SECURITY: the field must not offer browser autofill / saved passwords for a document password.
    test('the password field has autocomplete="off" and is a password input', async () => {
        render(SpatialCanvas);
        await screen.findByTestId('pdf-password-dialog');
        const input = screen.getByTestId('pdf-password-input') as HTMLInputElement;
        expect(input.type).toBe('password');
        expect(input).toHaveAttribute('autocomplete', 'off');
    });

    test('a backdrop click cancels the prompt with no password attempt', async () => {
        render(SpatialCanvas);
        const dialog = await screen.findByTestId('pdf-password-dialog');
        await fireEvent.input(screen.getByTestId('pdf-password-input'), { target: { value: 'half-typed' } });
        await fireEvent.click(dialog.parentElement as HTMLElement); // the backdrop
        await waitFor(() => expect(screen.queryByTestId('pdf-password-dialog')).not.toBeInTheDocument());
        expect(await screen.findByTestId('studio-status-banner')).toHaveTextContent(/password/i);
        expect(attempts).toEqual([undefined]);
        expect(viewport.pages).toHaveLength(0);
    });
});

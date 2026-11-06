import { expect, test, vi, beforeEach, afterEach, describe } from 'vitest';
import { render, screen, cleanup, fireEvent, waitFor } from '@testing-library/svelte';

// BUG-058: the Studio password prompt must never put the typed password in a console call, even when the PDF
// library's own error text echoes it. (Derived from StudioPassword.test.ts.)
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
const attemptsEcho: string[] = [];

function passwordError(code: number) {
    const e = new Error(code === 1 ? 'No password given' : `Incorrect Password (tried: ${attemptsEcho.at(-1) ?? ''})`) as Error & { code: number };
    e.name = 'PasswordException';
    e.code = code;
    return e;
}

function encryptedPdfjs() {
    return {
        GlobalWorkerOptions: {},
        getDocument: (src: { data: Uint8Array; password?: string }) => {
            attempts.push(src.password);
            attemptsEcho.push(src.password ?? '');
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


const WRONG = 'wrong-pw-SECRET-123';
const logged: string[] = [];
const text = (args: unknown[]) =>
    args.map((a) => (a instanceof Error ? `${a.name}: ${a.message}\n${a.stack ?? ''}` : typeof a === 'string' ? a : JSON.stringify(a))).join(' ');

describe('Studio password prompt: no console leak (BUG-058)', () => {
    beforeEach(() => {
        attempts.length = 0;
        attemptsEcho.length = 0;
        submitted.length = 0;
        logged.length = 0;
        (window as any).pdfjsLib = encryptedPdfjs();
        appState.clear();
        viewport.pages = [];
        appState.addFiles(['/docs/encrypted.pdf']);
        for (const m of ['log', 'info', 'debug', 'warn', 'error'] as const) {
            vi.spyOn(console, m).mockImplementation((...a: unknown[]) => {
                logged.push(text(a));
            });
        }
    });
    afterEach(() => {
        cleanup();
        vi.restoreAllMocks();
        delete (window as any).pdfjsLib;
        appState.clear();
        viewport.pages = [];
    });

    test('control: the console spy would catch a leak (so the real assertions can fail)', () => {
        console.error(new Error(`boom ${WRONG}`));
        expect(logged.some((l) => l.includes(WRONG))).toBe(true);
    });

    test('a wrong then a right password: neither appears in any console call, though the library error echoes it', async () => {
        render(SpatialCanvas);
        await screen.findByTestId('pdf-password-dialog');
        await fireEvent.input(screen.getByTestId('pdf-password-input'), { target: { value: WRONG } });
        await fireEvent.click(screen.getByTestId('pdf-password-submit'));
        await waitFor(() => expect(screen.getByTestId('pdf-password-error')).toBeInTheDocument());
        await fireEvent.input(screen.getByTestId('pdf-password-input'), { target: { value: PASSWORD } });
        await fireEvent.click(screen.getByTestId('pdf-password-submit'));
        await waitFor(() => expect(viewport.pages).toHaveLength(2));
        expect(attempts).toEqual([undefined, WRONG, PASSWORD]);
        expect(logged.filter((l) => l.includes(WRONG) || l.includes(PASSWORD))).toEqual([]);
        // nor on the screen
        expect(document.body.textContent).not.toContain(WRONG);
    });

    test('cancelling after typing: the half-typed password is not logged either', async () => {
        render(SpatialCanvas);
        await screen.findByTestId('pdf-password-dialog');
        await fireEvent.input(screen.getByTestId('pdf-password-input'), { target: { value: WRONG } });
        await fireEvent.click(screen.getByTestId('pdf-password-cancel'));
        await screen.findByTestId('studio-status-banner');
        expect(logged.filter((l) => l.includes(WRONG))).toEqual([]);
    });
});

// Security and Forms keep their requests; presentation moved to ui/ components and the shared job runner.
import { expect, test, vi, beforeEach, afterEach, describe } from 'vitest';
import { render, screen, cleanup, fireEvent, waitFor } from '@testing-library/svelte';

vi.mock('@tauri-apps/api/core', () => ({
    invoke: vi.fn(async (cmd: string) => {
        if (cmd === 'path_exists') return false;
        if (cmd === 'read_file_bytes') throw new Error('No such file or directory (os error 2)');
        return null;
    }),
}));
vi.mock('@tauri-apps/plugin-dialog', () => ({ open: vi.fn(async () => '/certs/me.p12') }));
// The viewer reports its page count (the real one comes from pdf.js); tests use a 5-page document.
vi.mock('$lib/components/PdfViewer.svelte', () => ({
    default: (_anchor: unknown, props: { onpagecount?: (n: number) => void }) => {
        queueMicrotask(() => props?.onpagecount?.(5));
    },
}));

import { IpcClient } from '$lib/ipc';
import { appState } from '$lib/state.svelte';
import { resetIssuedOutputNamesForTests } from '../studio/outputNames';
import SecurityView from '../SecurityView.svelte';
import FormsView from '../FormsView.svelte';

type Req = Record<string, unknown>;
const calls: Req[] = [];
let failNext = false;
const flush = () => new Promise<void>((r) => setTimeout(r, 0));
const click = async (el: HTMLElement) => {
    await fireEvent.click(el);
    await flush();
    await flush();
};

beforeEach(() => {
    calls.length = 0;
    failNext = false;
    resetIssuedOutputNamesForTests();
    appState.clear();
    appState.addFiles(['/docs/a.pdf']);
    vi.spyOn(IpcClient, 'runJob').mockImplementation(async (r) => {
        calls.push(r as unknown as Req);
        if (failNext) throw new Error('engine said no');
        return 'job';
    });
});
afterEach(() => {
    cleanup();
    vi.restoreAllMocks();
});

describe('Security (SECURITY path: presentation only)', () => {
    test('password fields are password inputs with autocomplete off', () => {
        render(SecurityView);
        for (const id of ['protect-password', 'unlock-password']) {
            const el = screen.getByTestId(id) as HTMLInputElement;
            expect(el.type).toBe('password');
            expect(el).toHaveAttribute('autocomplete', 'off');
        }
    });

    test('protect sends the same request, clears the password after success', async () => {
        render(SecurityView);
        await fireEvent.input(screen.getByTestId('protect-password'), { target: { value: 'pw1' } });
        await click(screen.getByRole('button', { name: 'Protect PDF' }));
        expect(calls[0]).toMatchObject({
            op: 'ProtectDocument',
            input_path: '/docs/a.pdf',
            password: 'pw1',
            permissions: { allow_print: true, allow_copy: false, allow_edit: false },
        });
        expect(calls[0].output_path).not.toBe('/docs/a.pdf');
        expect((screen.getByTestId('protect-password') as HTMLInputElement).value).toBe('');
    });

    test('a failed protect keeps the password for the Retry and says so in plain words', async () => {
        failNext = true;
        render(SecurityView);
        await fireEvent.input(screen.getByTestId('protect-password'), { target: { value: 'pw1' } });
        await click(screen.getByRole('button', { name: 'Protect PDF' }));
        expect(screen.getByRole('alert')).toHaveTextContent("Couldn't protect the PDF.");
        expect(document.body.textContent).not.toContain('engine said no');
        expect((screen.getByTestId('protect-password') as HTMLInputElement).value).toBe('pw1');
        failNext = false;
        await click(screen.getByRole('button', { name: 'Retry' }));
        await waitFor(() => expect(calls).toHaveLength(2));
        expect(calls[1].password).toBe('pw1');
    });

    // BUG-053: pages are 1-based and checked against the real page count.
    test('the page field is 1-based ("Page", default 1) and names the real page count', async () => {
        render(SecurityView);
        const page = (await screen.findByLabelText(/^Page \(1.5\)/)) as HTMLInputElement;
        expect(page.value).toBe('1');
        expect(document.body.textContent).not.toMatch(/starts at 0|0-idx/i);
    });

    test.each(['0', '6', '', '2.5', '-1', 'abc'])('page "%s" is refused with an inline error and no job', async (value) => {
        render(SecurityView);
        const page = await screen.findByLabelText(/^Page \(1.5\)/);
        await fireEvent.input(page, { target: { value } });
        await click(screen.getByRole('button', { name: 'Apply redaction' }));
        expect(calls).toHaveLength(0);
        expect(screen.getByTestId('redact-error')).toHaveTextContent(/page from 1 to 5/i);
    });

    test('page 5 (the last page) is sent as 5, page 3 as 3: no off-by-one', async () => {
        render(SecurityView);
        const page = await screen.findByLabelText(/^Page \(1.5\)/);
        for (const n of ['5', '3']) {
            await fireEvent.input(page, { target: { value: n } });
            await click(screen.getByRole('button', { name: 'Apply redaction' }));
        }
        expect(calls.map((c) => (c as { regions: Array<{ page: number }> }).regions[0].page)).toEqual([5, 3]);
    });

    test('region values pass through exactly (non-default values), and a typed 0 stays 0', async () => {
        render(SecurityView);
        await screen.findByLabelText(/^Page \(1.5\)/);
        await fireEvent.input(screen.getByLabelText('X'), { target: { value: '12.5' } });
        await fireEvent.input(screen.getByLabelText('Y'), { target: { value: '7' } });
        await fireEvent.input(screen.getByLabelText('Width'), { target: { value: '200' } });
        await fireEvent.input(screen.getByLabelText('Height'), { target: { value: '50' } });
        await click(screen.getByRole('button', { name: 'Apply redaction' }));
        expect((calls[0] as { regions: Array<Record<string, number>> }).regions[0]).toEqual({ page: 1, x: 12.5, y: 7, width: 200, height: 50 });
        // 0 is valid for X and Y: typed over a non-default value it is sent as 0
        await fireEvent.input(screen.getByLabelText('X'), { target: { value: '9' } });
        await fireEvent.input(screen.getByLabelText('X'), { target: { value: '0' } });
        await fireEvent.input(screen.getByLabelText('Y'), { target: { value: '0' } });
        await click(screen.getByRole('button', { name: 'Apply redaction' }));
        expect((calls[1] as { regions: Array<Record<string, number>> }).regions[0]).toMatchObject({ x: 0, y: 0, width: 200, height: 50 });
    });

    test.each([['X'], ['Y'], ['Width'], ['Height']])('a blank %s is refused ("Enter all four values"), nothing is defaulted', async (label) => {
        render(SecurityView);
        await screen.findByLabelText(/^Page \(1.5\)/);
        await fireEvent.input(screen.getByLabelText(label), { target: { value: '' } });
        await click(screen.getByRole('button', { name: 'Apply redaction' }));
        expect(calls).toHaveLength(0);
        expect(screen.getByTestId('redact-error')).toHaveTextContent('Enter all four values');
    });

    test('a width or height of 0 (or less) is refused too', async () => {
        render(SecurityView);
        await screen.findByLabelText(/^Page \(1.5\)/);
        await fireEvent.input(screen.getByLabelText('Width'), { target: { value: '0' } });
        await click(screen.getByRole('button', { name: 'Apply redaction' }));
        expect(calls).toHaveLength(0);
        expect(screen.getByTestId('redact-error')).toBeInTheDocument();
    });

    // A password kept for a retry must never be applied to a different file.
    test('Security: every password field is cleared when the open file changes', async () => {
        failNext = true;
        render(SecurityView);
        await fireEvent.input(screen.getByTestId('protect-password'), { target: { value: 'pw1' } });
        await fireEvent.input(screen.getByTestId('unlock-password'), { target: { value: 'pw2' } });
        await click(screen.getByRole('button', { name: 'Protect PDF' }));
        expect((screen.getByTestId('protect-password') as HTMLInputElement).value).toBe('pw1'); // kept for Retry
        appState.addFiles(['/docs/b.pdf']);
        appState.setActiveFile('/docs/b.pdf');
        await flush();
        expect((screen.getByTestId('protect-password') as HTMLInputElement).value).toBe('');
        expect((screen.getByTestId('unlock-password') as HTMLInputElement).value).toBe('');
        expect(screen.queryByRole('button', { name: 'Retry' })).toBeNull(); // the old file's retry is gone with it
    });
});

describe('Forms', () => {
    test('sign: browse picks the certificate, the password is cleared after success', async () => {
        render(FormsView);
        await click(screen.getByRole('button', { name: 'Browse' }));
        await waitFor(() => expect((screen.getByTestId('cert-path') as HTMLInputElement).value).toBe('/certs/me.p12'));
        await fireEvent.input(screen.getByTestId('sign-password'), { target: { value: 'secret' } });
        await click(screen.getByRole('button', { name: 'Sign document' }));
        expect(calls[0]).toMatchObject({ op: 'SignDocument', p12_path: '/certs/me.p12', password: 'secret' });
        expect((screen.getByTestId('sign-password') as HTMLInputElement).value).toBe('');
    });

    test('fill form sends the field; no document shows the empty state', async () => {
        render(FormsView);
        await click(screen.getByRole('button', { name: 'Apply value' }));
        expect(calls[0]).toMatchObject({ op: 'FillForm', fields: { Text1: 'John Doe' } });
        cleanup();
        appState.clear();
        render(FormsView);
        expect(screen.getByText('No document selected')).toBeInTheDocument();
    });

    test('Forms: the signing password is cleared when the open file changes (also after a failure)', async () => {
        failNext = true;
        render(FormsView);
        await click(screen.getByRole('button', { name: 'Browse' }));
        await waitFor(() => expect((screen.getByTestId('cert-path') as HTMLInputElement).value).toBe('/certs/me.p12'));
        await fireEvent.input(screen.getByTestId('sign-password'), { target: { value: 'secret' } });
        await click(screen.getByRole('button', { name: 'Sign document' }));
        expect((screen.getByTestId('sign-password') as HTMLInputElement).value).toBe('secret');
        appState.addFiles(['/docs/b.pdf']);
        appState.setActiveFile('/docs/b.pdf');
        await flush();
        expect((screen.getByTestId('sign-password') as HTMLInputElement).value).toBe('');
    });
});

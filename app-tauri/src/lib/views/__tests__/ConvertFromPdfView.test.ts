import { expect, test, vi, beforeEach, afterEach, describe } from 'vitest';
import { render, screen, cleanup, fireEvent } from '@testing-library/svelte';
import ConvertFromPdfView from '../ConvertFromPdfView.svelte';
import ConvertView from '../ConvertView.svelte';
import { appState } from '$lib/state.svelte';
import { IpcClient } from '$lib/ipc';

describe('ConvertFromPdfView', () => {
    beforeEach(() => {
        appState.clear();
        vi.restoreAllMocks();
    });
    afterEach(() => {
        cleanup();
    });

    test('shows placeholder when no PDF is active', () => {
        render(ConvertFromPdfView);
        expect(screen.getByText(/No PDF loaded/i)).toBeInTheDocument();
    });

    test('shows active PDF and format options', () => {
        appState.addFiles(['/path/to/my_document.pdf']);
        render(ConvertFromPdfView);
        expect(screen.getByText('my_document.pdf')).toBeInTheDocument();
        expect(screen.getByText(/Convert from PDF/i)).toBeInTheDocument();
        expect(screen.getAllByText(/Word Document \(\.docx\)/i).length).toBeGreaterThan(0);
        expect(screen.getByText(/Render to Images \(PNG\)/i)).toBeInTheDocument();
        expect(screen.getByText(/Extract Embedded Images/i)).toBeInTheDocument();
        expect(screen.getByText(/HTML Document \(\.html\)/i)).toBeInTheDocument();
        expect(screen.getByText(/Plain Text \(\.txt\)/i)).toBeInTheDocument();
    });

    test('submits JobRequest when export button clicked', async () => {
        appState.addFiles(['/path/to/my_document.pdf']);
        const submitSpy = vi.spyOn(IpcClient, 'submitJob').mockResolvedValue('test-job-id-1234');
        render(ConvertFromPdfView);

        const btn = screen.getByRole('button', { name: /Convert Now/i });
        await fireEvent.click(btn);

        expect(submitSpy).toHaveBeenCalledWith(expect.objectContaining({
            op: 'PdfToDocx',
            input_path: '/path/to/my_document.pdf',
            output_path: '/path/to/my_document.docx'
        }));
    });
});

describe('ConvertView tab container', () => {
    beforeEach(() => {
        appState.clear();
        vi.restoreAllMocks();
    });
    afterEach(() => {
        cleanup();
    });

    test('renders tabs and defaults to Convert from PDF', () => {
        render(ConvertView);
        expect(screen.getByRole('button', { name: /Convert from PDF/i })).toBeInTheDocument();
        expect(screen.getByRole('button', { name: /Convert to PDF/i })).toBeInTheDocument();
    });

    test('switches tabs on click', async () => {
        appState.addFiles(['/path/to/test.docx']);
        render(ConvertView);
        const toPdfTab = screen.getByRole('button', { name: /Convert to PDF/i });
        await fireEvent.click(toPdfTab);
        expect(screen.getByRole('button', { name: /Word \(\.docx\) to PDF/i })).toBeInTheDocument();
    });
});


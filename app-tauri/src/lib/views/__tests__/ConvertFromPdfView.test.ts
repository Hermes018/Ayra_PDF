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
        expect(screen.getAllByText(/PDF to PDF\/A \(Archival\)/i).length).toBeGreaterThan(0);
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

    test('submits PdfToPdfA archival JobRequest with conformance level', async () => {
        appState.addFiles(['/path/to/my_document.pdf']);
        const submitSpy = vi.spyOn(IpcClient, 'submitJob').mockResolvedValue('test-pdfa-job-5678');
        render(ConvertFromPdfView);

        // Click PDF/A format tile
        const pdfaTile = screen.getByRole('button', { name: /PDF to PDF\/A \(Archival\)/i });
        await fireEvent.click(pdfaTile);

        // Verify conformance selector appears with PDF/A-2b recommended
        expect(screen.getByText(/PDF\/A Conformance Level/i)).toBeInTheDocument();
        expect(screen.getByText(/Recommended/i)).toBeInTheDocument();

        // Submit default (PDF/A-2b)
        const btn = screen.getByRole('button', { name: /Convert Now/i });
        await fireEvent.click(btn);

        expect(submitSpy).toHaveBeenCalledWith(expect.objectContaining({
            op: 'PdfToPdfA',
            input_path: '/path/to/my_document.pdf',
            output_path: '/path/to/my_document_pdfa.pdf',
            conformance: 'PdfA2b'
        }));
    });

    test('allows selecting PDF/A-3b conformance level', async () => {
        appState.addFiles(['/path/to/my_document.pdf']);
        const submitSpy = vi.spyOn(IpcClient, 'submitJob').mockResolvedValue('test-pdfa-job-9012');
        render(ConvertFromPdfView);

        // Click PDF/A format tile
        const pdfaTile = screen.getByRole('button', { name: /PDF to PDF\/A \(Archival\)/i });
        await fireEvent.click(pdfaTile);

        // Select PDF/A-3b radio
        const radio3b = screen.getByRole('radio', { name: /PDF\/A-3b/i });
        await fireEvent.click(radio3b);

        // Submit
        const btn = screen.getByRole('button', { name: /Convert Now/i });
        await fireEvent.click(btn);

        expect(submitSpy).toHaveBeenCalledWith(expect.objectContaining({
            op: 'PdfToPdfA',
            input_path: '/path/to/my_document.pdf',
            output_path: '/path/to/my_document_pdfa.pdf',
            conformance: 'PdfA3b'
        }));
    });

    test('submits PdfToOffice sidecar for complex Word (.docx)', async () => {
        appState.addFiles(['/path/to/my_document.pdf']);
        const submitSpy = vi.spyOn(IpcClient, 'submitJob').mockResolvedValue('test-docx-job-1111');
        render(ConvertFromPdfView);

        const wordSidecarTile = screen.getByRole('button', { name: /PDF to Word \(Complex Flowable via Sidecar\)/i });
        await fireEvent.click(wordSidecarTile);

        const btn = screen.getByRole('button', { name: /Convert Now/i });
        await fireEvent.click(btn);

        expect(submitSpy).toHaveBeenCalledWith(expect.objectContaining({
            op: 'PdfToOffice',
            input_path: '/path/to/my_document.pdf',
            output_path: '/path/to/my_document.docx',
            format: 'Docx'
        }));
    });

    test('submits PdfToOffice sidecar for Excel (.xlsx)', async () => {
        appState.addFiles(['/path/to/my_sheet.pdf']);
        const submitSpy = vi.spyOn(IpcClient, 'submitJob').mockResolvedValue('test-xlsx-job-2222');
        render(ConvertFromPdfView);

        const excelTile = screen.getByRole('button', { name: /PDF to Excel \(\.xlsx\)/i });
        await fireEvent.click(excelTile);

        const btn = screen.getByRole('button', { name: /Convert Now/i });
        await fireEvent.click(btn);

        expect(submitSpy).toHaveBeenCalledWith(expect.objectContaining({
            op: 'PdfToOffice',
            input_path: '/path/to/my_sheet.pdf',
            output_path: '/path/to/my_sheet.xlsx',
            format: 'Xlsx'
        }));
    });

    test('submits PdfToOffice sidecar for PowerPoint (.pptx)', async () => {
        appState.addFiles(['/path/to/my_slides.pdf']);
        const submitSpy = vi.spyOn(IpcClient, 'submitJob').mockResolvedValue('test-pptx-job-3333');
        render(ConvertFromPdfView);

        const pptxTile = screen.getByRole('button', { name: /PDF to PowerPoint \(\.pptx\)/i });
        await fireEvent.click(pptxTile);

        const btn = screen.getByRole('button', { name: /Convert Now/i });
        await fireEvent.click(btn);

        expect(submitSpy).toHaveBeenCalledWith(expect.objectContaining({
            op: 'PdfToOffice',
            input_path: '/path/to/my_slides.pdf',
            output_path: '/path/to/my_slides.pptx',
            format: 'Pptx'
        }));
    });

    test('submits PdfToMarkdown sidecar job request', async () => {
        appState.addFiles(['/path/to/article.pdf']);
        const submitSpy = vi.spyOn(IpcClient, 'submitJob').mockResolvedValue('test-md-job-4444');
        render(ConvertFromPdfView);

        const mdTile = screen.getByRole('button', { name: /PDF to Markdown \(\.md \+ assets\/\)/i });
        await fireEvent.click(mdTile);

        const btn = screen.getByRole('button', { name: /Convert Now/i });
        await fireEvent.click(btn);

        expect(submitSpy).toHaveBeenCalledWith(expect.objectContaining({
            op: 'PdfToMarkdown',
            input_path: '/path/to/article.pdf',
            output_path: '/path/to/article.md'
        }));
    });

    test('submits BatchMarkdownConvert from PDF view', async () => {
        appState.addFiles(['/path/to/my_folder']);
        const submitSpy = vi.spyOn(IpcClient, 'submitJob').mockResolvedValue('test-batch-from-pdf');
        render(ConvertFromPdfView);

        const batchTile = screen.getByRole('button', { name: /Batch Folder to Markdown \(\.md\)/i });
        await fireEvent.click(batchTile);

        const btn = screen.getByRole('button', { name: /Convert Now/i });
        await fireEvent.click(btn);

        expect(submitSpy).toHaveBeenCalledWith(expect.objectContaining({
            op: 'BatchMarkdownConvert',
            input_dir: '/path/to/my_folder',
            options: expect.objectContaining({
                target: 'Markdown',
                inject_front_matter: true,
                preserve_front_matter: true,
                rewrite_assets: true
            })
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


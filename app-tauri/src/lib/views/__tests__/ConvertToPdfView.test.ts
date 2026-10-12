import { expect, test, vi, beforeEach, afterEach, describe } from 'vitest';
import { render, screen, cleanup, fireEvent } from '@testing-library/svelte';
import ConvertToPdfView from '../ConvertToPdfView.svelte';
import { appState } from '$lib/state.svelte';
import { IpcClient } from '$lib/ipc';

describe('ConvertToPdfView', () => {
    beforeEach(() => {
        appState.clear();
        vi.restoreAllMocks();
    });
    afterEach(() => {
        cleanup();
    });

    test('shows placeholder when no file is active', () => {
        render(ConvertToPdfView);
        expect(screen.getByText(/No file loaded/i)).toBeInTheDocument();
    });

    test('shows active file and conversion options', () => {
        appState.addFiles(['/path/to/my_presentation.pptx']);
        render(ConvertToPdfView);
        expect(screen.getByText('my_presentation.pptx')).toBeInTheDocument();
        expect(screen.getAllByText(/Convert to PDF/i).length).toBeGreaterThan(0);
        expect(screen.getAllByText(/Word to PDF \(High-Fidelity Layout via Sidecar\)/i).length).toBeGreaterThan(0);
        expect(screen.getByText(/Excel to PDF \(\.xlsx\)/i)).toBeInTheDocument();
        expect(screen.getByText(/PowerPoint to PDF \(\.pptx\)/i)).toBeInTheDocument();
        expect(screen.getByText(/Image to PDF/i)).toBeInTheDocument();
        expect(screen.getByText(/Word \(\.docx\) to PDF \(Structural\)/i)).toBeInTheDocument();
        expect(screen.getByText(/HTML to PDF/i)).toBeInTheDocument();
    });

    test('submits OfficeToPdf JobRequest for Word high-fidelity layout by default', async () => {
        appState.addFiles(['/path/to/report.docx']);
        const submitSpy = vi.spyOn(IpcClient, 'submitJob').mockResolvedValue('test-job-id-office-1');
        render(ConvertToPdfView);

        const btn = screen.getByRole('button', { name: /Convert to PDF/i });
        await fireEvent.click(btn);

        expect(submitSpy).toHaveBeenCalledWith(expect.objectContaining({
            op: 'OfficeToPdf',
            input_path: '/path/to/report.docx',
            output_path: '/path/to/report.pdf',
            format: 'Docx'
        }));
    });

    test('submits OfficeToPdf JobRequest for Excel spreadsheet', async () => {
        appState.addFiles(['/path/to/budget.xlsx']);
        const submitSpy = vi.spyOn(IpcClient, 'submitJob').mockResolvedValue('test-job-id-office-2');
        render(ConvertToPdfView);

        const xlsxTile = screen.getByRole('button', { name: /Excel to PDF \(\.xlsx\)/i });
        await fireEvent.click(xlsxTile);

        const btn = screen.getByRole('button', { name: /Convert to PDF/i });
        await fireEvent.click(btn);

        expect(submitSpy).toHaveBeenCalledWith(expect.objectContaining({
            op: 'OfficeToPdf',
            input_path: '/path/to/budget.xlsx',
            output_path: '/path/to/budget.pdf',
            format: 'Xlsx'
        }));
    });

    test('submits OfficeToPdf JobRequest for PowerPoint presentation', async () => {
        appState.addFiles(['/path/to/slides.pptx']);
        const submitSpy = vi.spyOn(IpcClient, 'submitJob').mockResolvedValue('test-job-id-office-3');
        render(ConvertToPdfView);

        const pptxTile = screen.getByRole('button', { name: /PowerPoint to PDF \(\.pptx\)/i });
        await fireEvent.click(pptxTile);

        const btn = screen.getByRole('button', { name: /Convert to PDF/i });
        await fireEvent.click(btn);

        expect(submitSpy).toHaveBeenCalledWith(expect.objectContaining({
            op: 'OfficeToPdf',
            input_path: '/path/to/slides.pptx',
            output_path: '/path/to/slides.pdf',
            format: 'Pptx'
        }));
    });

    test('submits native ImageToPdf when selected', async () => {
        appState.addFiles(['/path/to/photo.png']);
        const submitSpy = vi.spyOn(IpcClient, 'submitJob').mockResolvedValue('test-job-id-img');
        render(ConvertToPdfView);

        const imgTile = screen.getByRole('button', { name: /Image to PDF/i });
        await fireEvent.click(imgTile);

        const btn = screen.getByRole('button', { name: /Convert to PDF/i });
        await fireEvent.click(btn);

        expect(submitSpy).toHaveBeenCalledWith(expect.objectContaining({
            op: 'ImageToPdf',
            input_path: '/path/to/photo.png',
            output_path: '/path/to/photo.pdf'
        }));
    });
});

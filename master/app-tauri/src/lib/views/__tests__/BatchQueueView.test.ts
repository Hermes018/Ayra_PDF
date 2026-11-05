import { expect, test, vi, beforeEach, afterEach, describe } from 'vitest';
import { render, screen, cleanup, fireEvent } from '@testing-library/svelte';
import BatchQueueView from '../BatchQueueView.svelte';
import { IpcClient } from '$lib/ipc';
import type { JobRecord } from '$lib/bindings/JobRecord';

const mockJobs: JobRecord[] = [
    {
        job_id: 'job-1111-active',
        title: 'Compress PDF',
        status: 'Running',
        progress: 45,
        priority: 'High',
        input_path: '/path/to/annual_report.pdf',
        output_path: '/path/to/annual_report_compressed.pdf',
        error_code: null,
        error_message: null,
        created_at_epoch_ms: 1700000000000n,
        request: {
            op: 'Compress',
            input_path: '/path/to/annual_report.pdf',
            output_path: '/path/to/annual_report_compressed.pdf',
            profile: 'Medium'
        }
    },
    {
        job_id: 'job-2222-queued',
        title: 'PDF to DOCX',
        status: 'Queued',
        progress: 0,
        priority: 'Normal',
        input_path: '/path/to/contract.pdf',
        output_path: '/path/to/contract.docx',
        error_code: null,
        error_message: null,
        created_at_epoch_ms: 1700000001000n,
        request: {
            op: 'PdfToDocx',
            input_path: '/path/to/contract.pdf',
            output_path: '/path/to/contract.docx'
        }
    },
    {
        job_id: 'job-3333-completed',
        title: 'Delete Pages',
        status: 'Completed',
        progress: 100,
        priority: 'Low',
        input_path: '/path/to/slides.pdf',
        output_path: '/path/to/slides_trimmed.pdf',
        error_code: null,
        error_message: null,
        created_at_epoch_ms: 1700000002000n,
        request: {
            op: 'DeletePages',
            input_path: '/path/to/slides.pdf',
            output_path: '/path/to/slides_trimmed.pdf',
            pages: [1, 2]
        }
    },
    {
        job_id: 'job-4444-failed',
        title: 'OCR Extract',
        status: 'Failed',
        progress: 20,
        priority: 'Urgent',
        input_path: '/path/to/scan.pdf',
        output_path: '/path/to/scan_ocr.pdf',
        error_code: 'OCR_INFERENCE_ERROR',
        error_message: 'Model execution timeout or out-of-memory.',
        created_at_epoch_ms: 1700000003000n,
        request: {
            op: 'Ocr',
            input_path: '/path/to/scan.pdf',
            output_path: '/path/to/scan_ocr.pdf',
            lang: 'eng'
        }
    }
];

describe('BatchQueueView', () => {
    beforeEach(() => {
        vi.restoreAllMocks();
        vi.spyOn(IpcClient, 'onJobEvent').mockResolvedValue(() => {});
        vi.spyOn(IpcClient, 'listJobs').mockResolvedValue([...mockJobs]);
    });

    afterEach(() => {
        cleanup();
    });

    test('renders summary header with correct counts', async () => {
        render(BatchQueueView);

        // Header and subtitle
        expect(screen.getByText('Batch Queue Management')).toBeInTheDocument();

        // Check metrics: 1 Running, 1 Queued, 1 Completed, 1 Failed
        expect(await screen.findByText('Running')).toBeInTheDocument();
        expect(screen.getByText('Queued')).toBeInTheDocument();
        expect(screen.getByText('Completed')).toBeInTheDocument();
        expect(screen.getByText('Failed')).toBeInTheDocument();

        // Cards rendered
        const cards = await screen.findAllByTestId('job-card');
        expect(cards.length).toBe(4);
    });

    test('filters queue by status tabs', async () => {
        render(BatchQueueView);
        await screen.findAllByTestId('job-card');

        // Filter: Active (Running + Queued = 2)
        const activeTab = screen.getByTestId('filter-active');
        await fireEvent.click(activeTab);
        let cards = screen.getAllByTestId('job-card');
        expect(cards.length).toBe(2);
        expect(screen.getByText('Compress PDF')).toBeInTheDocument();
        expect(screen.getByText('PDF to DOCX')).toBeInTheDocument();

        // Filter: Completed (1)
        const completedTab = screen.getByTestId('filter-completed');
        await fireEvent.click(completedTab);
        cards = screen.getAllByTestId('job-card');
        expect(cards.length).toBe(1);
        expect(screen.getByText('Delete Pages')).toBeInTheDocument();

        // Filter: Failed (1)
        const failedTab = screen.getByTestId('filter-failed');
        await fireEvent.click(failedTab);
        cards = screen.getAllByTestId('job-card');
        expect(cards.length).toBe(1);
        expect(screen.getByText('OCR Extract')).toBeInTheDocument();
        expect(screen.getByText('OCR_INFERENCE_ERROR')).toBeInTheDocument();
    });

    test('dispatches job cancellation', async () => {
        const cancelSpy = vi.spyOn(IpcClient, 'cancelJob').mockResolvedValue(true);
        render(BatchQueueView);

        const cancelBtn = await screen.findByTestId('btn-cancel-job-1111-active');
        await fireEvent.click(cancelBtn);

        expect(cancelSpy).toHaveBeenCalledWith('job-1111-active');
    });

    test('dispatches job retry for failed jobs', async () => {
        const retrySpy = vi.spyOn(IpcClient, 'retryJob').mockResolvedValue('job-new-retried-5555');
        render(BatchQueueView);

        const retryBtn = await screen.findByTestId('btn-retry-job-4444-failed');
        await fireEvent.click(retryBtn);

        expect(retrySpy).toHaveBeenCalledWith('job-4444-failed');
    });

    test('clears finished jobs', async () => {
        const clearSpy = vi.spyOn(IpcClient, 'clearFinishedJobs').mockResolvedValue(2);
        render(BatchQueueView);

        const clearBtn = await screen.findByTestId('btn-clear-finished');
        await fireEvent.click(clearBtn);

        expect(clearSpy).toHaveBeenCalled();
    });

    test('submits batch demo when clicked', async () => {
        const submitBatchSpy = vi.spyOn(IpcClient, 'submitBatch').mockResolvedValue(['id-1', 'id-2', 'id-3']);
        render(BatchQueueView);

        const batchBtn = await screen.findByTestId('btn-add-batch-demo');
        await fireEvent.click(batchBtn);

        expect(submitBatchSpy).toHaveBeenCalledWith(
            expect.arrayContaining([
                expect.objectContaining({ op: 'Compress' }),
                expect.objectContaining({ op: 'PdfToDocx' }),
                expect.objectContaining({ op: 'PdfToImages' })
            ]),
            'High'
        );
    });
});

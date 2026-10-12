import { expect, test, vi, beforeEach, afterEach, describe } from 'vitest';
import { render, screen, cleanup, fireEvent } from '@testing-library/svelte';
import PipelinesView from '../PipelinesView.svelte';
import { IpcClient } from '$lib/ipc';
import { appState } from '$lib/state.svelte';
import type { PipelinePreset } from '$lib/bindings/PipelinePreset';

const mockPresets: PipelinePreset[] = [
    {
        id: 'archive_searchable',
        name: 'Archive & Searchable (OCR + PDF/A-2b)',
        description: 'Repairs document structure, performs OCR, and exports PDF/A-2b.',
        steps: [
            { type: 'Repair' },
            { type: 'Ocr' },
            { type: 'Compress', profile: 'Medium' },
            { type: 'PdfToPdfA', conformance: 'PdfA2b' }
        ]
    },
    {
        id: 'clean_web_optimize',
        name: 'Clean & Web Optimize',
        description: 'Repairs xref, flattens comments, and compresses.',
        steps: [
            { type: 'Repair' },
            { type: 'FlattenComments' },
            { type: 'Compress', profile: 'High' }
        ]
    }
];

describe('PipelinesView', () => {
    beforeEach(() => {
        appState.clear();
        vi.restoreAllMocks();
        vi.spyOn(IpcClient, 'listPipelinePresets').mockResolvedValue([...mockPresets]);
    });

    afterEach(() => {
        cleanup();
    });

    test('renders presets and applies default preset', async () => {
        render(PipelinesView);

        expect(screen.getByText('Workflows & Tool Chaining')).toBeInTheDocument();
        expect(await screen.findByText('Archive & Searchable (OCR + PDF/A-2b)')).toBeInTheDocument();
        expect(screen.getByText('Clean & Web Optimize')).toBeInTheDocument();

        // 4 steps in default preset
        const stepCards = await screen.findAllByTestId('step-card');
        expect(stepCards.length).toBe(4);
    });

    test('switches active workflow when another preset card is clicked', async () => {
        render(PipelinesView);
        await screen.findByText('Clean & Web Optimize');

        const cleanCard = screen.getByTestId('preset-card-clean_web_optimize');
        await fireEvent.click(cleanCard);

        // 3 steps in Clean & Web Optimize
        const stepCards = screen.getAllByTestId('step-card');
        expect(stepCards.length).toBe(3);
    });

    test('adds, reorders, and removes custom steps', async () => {
        render(PipelinesView);
        await screen.findAllByTestId('step-card');

        // Select Watermark from dropdown
        const select = screen.getByTestId('select-new-step');
        await fireEvent.change(select, { target: { value: 'Watermark' } });

        const addBtn = screen.getByTestId('btn-add-step');
        await fireEvent.click(addBtn);

        // Now 5 steps
        let stepCards = screen.getAllByTestId('step-card');
        expect(stepCards.length).toBe(5);
        expect(screen.getByText('Burn-in Watermark')).toBeInTheDocument();

        // Remove the newly added watermark step (index 4)
        const removeBtn = screen.getByTestId('btn-remove-4');
        await fireEvent.click(removeBtn);
        stepCards = screen.getAllByTestId('step-card');
        expect(stepCards.length).toBe(4);

        // Move step 1 up (which moves step 1 to index 0)
        const moveUpBtn = screen.getByTestId('btn-move-up-1');
        await fireEvent.click(moveUpBtn);
        stepCards = screen.getAllByTestId('step-card');
        expect(stepCards.length).toBe(4);
    });

    test('submits RunPipeline JobRequest with active chain and paths', async () => {
        const submitSpy = vi.spyOn(IpcClient, 'submitJob').mockResolvedValue('workflow-job-9999');
        render(PipelinesView);
        await screen.findAllByTestId('step-card');

        const inputField = screen.getByTestId('input-path-field');
        const outputField = screen.getByTestId('output-path-field');

        await fireEvent.input(inputField, { target: { value: '/data/legal_doc.pdf' } });
        await fireEvent.input(outputField, { target: { value: '/data/legal_doc_processed.pdf' } });

        const runBtn = screen.getByTestId('btn-run-workflow');
        await fireEvent.click(runBtn);

        expect(submitSpy).toHaveBeenCalledWith(expect.objectContaining({
            op: 'RunPipeline',
            input_path: '/data/legal_doc.pdf',
            output_path: '/data/legal_doc_processed.pdf',
            steps: expect.arrayContaining([
                expect.objectContaining({ type: 'Repair' }),
                expect.objectContaining({ type: 'Ocr' })
            ])
        }));

        expect(await screen.findByText(/Workflow enqueued successfully/i)).toBeInTheDocument();
    });
});

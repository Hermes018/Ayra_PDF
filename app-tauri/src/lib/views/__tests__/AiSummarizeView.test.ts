import { expect, test, vi, beforeEach, afterEach, describe } from 'vitest';
import { render, screen, cleanup, fireEvent, waitFor } from '@testing-library/svelte';
import AiSummarizeView from '../AiSummarizeView.svelte';
import { IpcClient } from '$lib/ipc';
import { appState } from '$lib/state.svelte';
import type { AppSettings } from '$lib/bindings/AppSettings';

const defaultSettings: AppSettings = {
    tier_override: null,
    output_policy: 'SuffixNewFile',
    backups_enabled: true,
    language: 'en',
    theme: 'System',
    local_ai_model_path: null,
    ai_local_enabled: false,
    ai_license_acknowledged: false,
    ai_model_sha256: null,
    ai_model_license: null,
};

describe('AiSummarizeView', () => {
    beforeEach(() => {
        appState.clear();
        vi.restoreAllMocks();
        vi.spyOn(IpcClient, 'getSettings').mockResolvedValue({ ...defaultSettings });
    });

    afterEach(() => {
        cleanup();
    });

    test('renders header and mode cards with Basic Extractive active by default', () => {
        render(AiSummarizeView);

        expect(screen.getByText('AI Document Summarizer')).toBeInTheDocument();
        expect(screen.getByText('100% Offline & Private')).toBeInTheDocument();

        const extractiveCard = screen.getByTestId('mode-extractive');
        expect(extractiveCard).toHaveClass('active');
        expect(screen.getByText('Basic Extractive')).toBeInTheDocument();
        expect(screen.getByText('Default Mode')).toBeInTheDocument();

        const llmCard = screen.getByTestId('mode-local-llm');
        expect(llmCard).not.toHaveClass('active');
        expect(screen.getByText('Local LLM Mode')).toBeInTheDocument();
        expect(screen.getByText('Gated Opt-In')).toBeInTheDocument();
    });

    test('navigates to settings when clicking configure in settings link', async () => {
        const onNavigate = vi.fn();
        render(AiSummarizeView, { props: { onNavigate } });

        const settingsLink = screen.getByTestId('settings-link');
        await fireEvent.click(settingsLink);

        expect(onNavigate).toHaveBeenCalledWith('Settings');
    });

    test('allows selecting different summary lengths', async () => {
        render(AiSummarizeView);

        const shortBtn = screen.getByTestId('length-short');
        const medBtn = screen.getByTestId('length-medium');
        const detBtn = screen.getByTestId('length-detailed');

        // Medium is default
        expect(medBtn).toHaveClass('selected');
        expect(shortBtn).not.toHaveClass('selected');
        expect(detBtn).not.toHaveClass('selected');

        // Select Short
        await fireEvent.click(shortBtn);
        expect(shortBtn).toHaveClass('selected');
        expect(medBtn).not.toHaveClass('selected');

        // Select Detailed
        await fireEvent.click(detBtn);
        expect(detBtn).toHaveClass('selected');
        expect(shortBtn).not.toHaveClass('selected');
    });

    test('shows validation error when attempting to submit without input path', async () => {
        render(AiSummarizeView);

        const generateBtn = screen.getByTestId('generate-summary-btn');
        await fireEvent.click(generateBtn);

        expect(screen.getByTestId('error-message')).toHaveTextContent(
            'Please provide an input PDF file path.'
        );
    });

    test('submits SummarizeExtractive job request via IpcClient', async () => {
        const submitSpy = vi.spyOn(IpcClient, 'submitJob').mockResolvedValue('test-summary-job-42');
        render(AiSummarizeView);

        const input = screen.getByTestId('input-path');
        const output = screen.getByTestId('output-path');
        const shortBtn = screen.getByTestId('length-short');
        const generateBtn = screen.getByTestId('generate-summary-btn');

        await fireEvent.input(input, { target: { value: '/docs/quarterly_report.pdf' } });
        // Select short length
        await fireEvent.click(shortBtn);

        // Click generate
        await fireEvent.click(generateBtn);

        expect(submitSpy).toHaveBeenCalledWith({
            op: 'SummarizeExtractive',
            input_path: '/docs/quarterly_report.pdf',
            output_path: '/docs/quarterly_report_summary.md',
            length: 'Short',
        });

        expect(await screen.findByTestId('submission-status')).toHaveTextContent(
            'Summary job queued successfully (ID: test-summary-job-42)'
        );
    });

    test('synchronizes input path with appState when available', () => {
        appState.addFiles(['/test/state_doc.pdf']);
        render(AiSummarizeView);

        const input = screen.getByTestId('input-path') as HTMLInputElement;
        const output = screen.getByTestId('output-path') as HTMLInputElement;

        expect(input.value).toBe('/test/state_doc.pdf');
        expect(output.value).toBe('/test/state_doc_summary.md');
    });

    test('displays Model Ready badge and model metadata when Local AI is configured and acknowledged', async () => {
        vi.spyOn(IpcClient, 'getSettings').mockResolvedValue({
            tier_override: null,
            output_policy: 'SuffixNewFile',
            backups_enabled: true,
            language: 'en',
            theme: 'System',
            local_ai_model_path: '/models/llama-3.2-1b.gguf',
            ai_local_enabled: true,
            ai_license_acknowledged: true,
            ai_model_sha256: 'a1b2c3d4e5f60718293a4b5c6d7e8f90123456789abcdef0123456789abcdef0',
            ai_model_license: 'Apache-2.0',
        });

        render(AiSummarizeView);

        await waitFor(() => {
            expect(screen.getByTestId('llm-status-badge')).toHaveTextContent('Model Ready');
        });
        expect(screen.getByTestId('llm-model-path')).toHaveTextContent('/models/llama-3.2-1b.gguf');
        expect(screen.getByTestId('llm-model-hash')).toHaveTextContent('a1b2c3d4e5f60718...');
        expect(screen.getByText(/Licence Acknowledged \(Apache-2.0\)/i)).toBeInTheDocument();
    });
});


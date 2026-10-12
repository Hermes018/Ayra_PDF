import { expect, test, vi, beforeEach, afterEach, describe } from 'vitest';
import { render, screen, cleanup, fireEvent } from '@testing-library/svelte';
import SettingsView from '../SettingsView.svelte';
import { IpcClient } from '$lib/ipc';
import type { AppSettings } from '$lib/bindings/AppSettings';
import type { AiModelInfo } from '$lib/bindings/AiModelInfo';

const initialSettings: AppSettings = {
    tier_override: null,
    output_policy: 'SuffixNewFile',
    backups_enabled: true,
    language: 'en',
    theme: 'System',
    local_ai_model_path: null,
    ai_local_enabled: false,
};

const mockLogs: string[] = [
    '2026-10-12T18:00:00Z INFO LocalPDF Studio initialized',
    '2026-10-12T18:00:01Z INFO HardwareProfile detected: Tier Medium',
    '2026-10-12T18:00:05Z DEBUG Memory budget set to 4096 MB',
];

const mockModelInfo: AiModelInfo = {
    path: '/models/llama-3.2-1b.gguf',
    file_name: 'llama-3.2-1b.gguf',
    size_bytes: 1258291200n,
    sha256_hex: 'a1b2c3d4e5f60718293a4b5c6d7e8f90123456789abcdef0123456789abcdef0',
    format: 'GGUF',
};

describe('SettingsView', () => {
    beforeEach(() => {
        vi.restoreAllMocks();
        vi.spyOn(IpcClient, 'getSettings').mockResolvedValue({ ...initialSettings });
        vi.spyOn(IpcClient, 'saveSettings').mockImplementation(async (s) => ({ ...s }));
        vi.spyOn(IpcClient, 'getRecentLogs').mockResolvedValue([...mockLogs]);
        vi.spyOn(IpcClient, 'exportDiagnostics').mockResolvedValue('/tmp/localpdf-diagnostics.txt');
        vi.spyOn(IpcClient, 'importLocalAiModel').mockResolvedValue({ ...mockModelInfo });
    });

    afterEach(() => {
        cleanup();
    });

    test('renders settings view and loads initial settings and logs', async () => {
        render(SettingsView);

        expect(screen.getByText('Settings & Diagnostics')).toBeInTheDocument();
        expect(await screen.findByText(/Auto-Detect/i)).toBeInTheDocument();
        expect(screen.getByText(/Suffix New File/i)).toBeInTheDocument();
        expect(screen.getByText(/Enable Automatic Backups/i)).toBeInTheDocument();

        // Verify logs rendered
        expect(await screen.findByText('2026-10-12T18:00:00Z INFO LocalPDF Studio initialized')).toBeInTheDocument();
        expect(screen.getByText('2026-10-12T18:00:01Z INFO HardwareProfile detected: Tier Medium')).toBeInTheDocument();
    });

    test('allows changing tier override and theme preference then saving', async () => {
        render(SettingsView);
        await screen.findByTestId('settings-loaded');

        // Click Tier High button
        const highTierBtn = screen.getByTestId('tier-btn-High');
        await fireEvent.click(highTierBtn);

        // Click Dark Theme button
        const darkThemeBtn = screen.getByText('🌙 Dark');
        await fireEvent.click(darkThemeBtn);

        // Click Save Settings button
        const saveBtn = screen.getByText('💾 Save Settings');
        await fireEvent.click(saveBtn);

        expect(IpcClient.saveSettings).toHaveBeenCalledWith(
            expect.objectContaining({
                tier_override: 'High',
                theme: 'Dark',
            })
        );
        expect(await screen.findByText('Settings saved successfully!')).toBeInTheDocument();
    });

    test('refreshes local logs when refresh button is clicked', async () => {
        render(SettingsView);
        await screen.findByText('2026-10-12T18:00:00Z INFO LocalPDF Studio initialized');

        const newLogs = ['2026-10-12T18:05:00Z INFO New job finished'];
        vi.spyOn(IpcClient, 'getRecentLogs').mockResolvedValue(newLogs);

        const refreshBtn = screen.getByText('🔄 Refresh Logs');
        await fireEvent.click(refreshBtn);

        expect(IpcClient.getRecentLogs).toHaveBeenCalled();
        expect(await screen.findByText('2026-10-12T18:05:00Z INFO New job finished')).toBeInTheDocument();
    });

    test('exports diagnostics report via IpcClient', async () => {
        render(SettingsView);

        const exportBtn = screen.getByText('Export Diagnostics');
        await fireEvent.click(exportBtn);

        expect(IpcClient.exportDiagnostics).toHaveBeenCalledWith('localpdf-diagnostics.txt');
        expect(await screen.findByText('Diagnostics report exported to: /tmp/localpdf-diagnostics.txt')).toBeInTheDocument();
    });

    test('imports local AI model and displays verified SHA-256 and size', async () => {
        render(SettingsView);

        const modelInput = screen.getByPlaceholderText('/path/to/model.gguf or model.onnx');
        await fireEvent.input(modelInput, { target: { value: '/models/llama-3.2-1b.gguf' } });

        const importBtn = screen.getByText('Import Model');
        await fireEvent.click(importBtn);

        expect(IpcClient.importLocalAiModel).toHaveBeenCalledWith('/models/llama-3.2-1b.gguf');
        expect(await screen.findByText('📄 llama-3.2-1b.gguf')).toBeInTheDocument();
        expect(screen.getByText('GGUF')).toBeInTheDocument();
        expect(screen.getByText('a1b2c3d4e5f60718293a4b5c6d7e8f90123456789abcdef0123456789abcdef0')).toBeInTheDocument();
    });
});

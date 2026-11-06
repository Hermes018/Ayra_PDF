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
    ai_license_acknowledged: false,
    ai_model_sha256: null,
    ai_model_license: null,
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
    license_spdx: 'Apache-2.0',
    license_acknowledged: true,
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
        const darkThemeBtn = screen.getByRole('button', { name: 'Dark' });
        await fireEvent.click(darkThemeBtn);

        // Click Save Settings button
        const saveBtn = screen.getByRole('button', { name: 'Save settings' });
        await fireEvent.click(saveBtn);

        expect(IpcClient.saveSettings).toHaveBeenCalledWith(
            expect.objectContaining({
                tier_override: 'High',
                theme: 'Dark',
            })
        );
        expect(await screen.findByText('Settings saved.')).toBeInTheDocument();
    });

    test('refreshes local logs when refresh button is clicked', async () => {
        render(SettingsView);
        await screen.findByText('2026-10-12T18:00:00Z INFO LocalPDF Studio initialized');

        const newLogs = ['2026-10-12T18:05:00Z INFO New job finished'];
        vi.spyOn(IpcClient, 'getRecentLogs').mockResolvedValue(newLogs);

        const refreshBtn = screen.getByRole('button', { name: 'Refresh logs' });
        await fireEvent.click(refreshBtn);

        expect(IpcClient.getRecentLogs).toHaveBeenCalled();
        expect(await screen.findByText('2026-10-12T18:05:00Z INFO New job finished')).toBeInTheDocument();
    });

    // BUG-051 follow-up: an existing file is only replaced after the user confirms; open documents are protected.
    test('an existing diagnostics file asks before it is replaced; Replace retries with overwrite, Cancel does not', async () => {
        const spy = vi.spyOn(IpcClient, 'exportDiagnostics');
        spy.mockRejectedValueOnce(new Error('OUTPUT_PATH_UNSAFE: a file already exists at the output path (/tmp/d.txt). Nothing was written.'));
        render(SettingsView);
        await fireEvent.click(screen.getByRole('button', { name: 'Export diagnostics' }));
        const dialog = await screen.findByRole('alertdialog', { name: 'Replace the existing file?' });
        expect(dialog).toBeInTheDocument();
        expect(spy).toHaveBeenCalledTimes(1);
        await fireEvent.click(screen.getByRole('button', { name: 'Replace file' }));
        await vi.waitFor(() => expect(spy).toHaveBeenCalledTimes(2));
        expect(spy).toHaveBeenLastCalledWith('localpdf-diagnostics.txt', [], true);
    });

    test('the replace question starts with focus on Cancel (the safe default)', async () => {
        const spy = vi.spyOn(IpcClient, 'exportDiagnostics');
        spy.mockRejectedValueOnce(new Error('OUTPUT_PATH_UNSAFE: a file already exists at the output path (/tmp/d.txt). Nothing was written.'));
        render(SettingsView);
        await fireEvent.click(screen.getByRole('button', { name: 'Export diagnostics' }));
        await screen.findByRole('alertdialog', { name: 'Replace the existing file?' });
        await vi.waitFor(() => expect(screen.getByRole('button', { name: 'Cancel' })).toHaveFocus());
        // explicit, not just "first button": the destructive Replace must never become the default
        expect(screen.getByRole('button', { name: 'Cancel' })).toHaveAttribute('data-autofocus');
        expect(screen.getByRole('button', { name: 'Replace file' })).not.toHaveAttribute('data-autofocus');
    });

    test('Cancel on the replace question exports nothing', async () => {
        const spy = vi.spyOn(IpcClient, 'exportDiagnostics');
        spy.mockRejectedValueOnce(new Error('OUTPUT_PATH_UNSAFE: a file already exists at the output path (/tmp/d.txt). Nothing was written.'));
        render(SettingsView);
        await fireEvent.click(screen.getByRole('button', { name: 'Export diagnostics' }));
        await screen.findByRole('alertdialog', { name: 'Replace the existing file?' });
        await fireEvent.click(screen.getByRole('button', { name: 'Cancel' }));
        expect(screen.queryByRole('alertdialog')).toBeNull();
        expect(spy).toHaveBeenCalledTimes(1);
    });

    test('an open document path is refused with a plain message, no replace question', async () => {
        const spy = vi.spyOn(IpcClient, 'exportDiagnostics');
        spy.mockRejectedValueOnce(new Error('OUTPUT_PATH_UNSAFE: the output file would overwrite the original (/docs/a.pdf). Nothing was written.'));
        render(SettingsView);
        await fireEvent.click(screen.getByRole('button', { name: 'Export diagnostics' }));
        expect(await screen.findByText(/would overwrite the original/)).toBeInTheDocument();
        expect(screen.queryByRole('alertdialog')).toBeNull();
    });

    test('exports diagnostics report via IpcClient', async () => {
        render(SettingsView);

        const exportBtn = screen.getByRole('button', { name: 'Export diagnostics' });
        await fireEvent.click(exportBtn);

        expect(IpcClient.exportDiagnostics).toHaveBeenCalledWith('localpdf-diagnostics.txt', [], false);
        expect(await screen.findByText('Diagnostics report exported to: /tmp/localpdf-diagnostics.txt')).toBeInTheDocument();
    });

    test('imports local AI model and displays verified SHA-256 and size', async () => {
        render(SettingsView);

        const modelInput = screen.getByPlaceholderText('/path/to/model.gguf or model.onnx');
        await fireEvent.input(modelInput, { target: { value: '/models/llama-3.2-1b.gguf' } });

        const importBtn = screen.getByRole('button', { name: 'Import model' });
        await fireEvent.click(importBtn);

        expect(IpcClient.importLocalAiModel).toHaveBeenCalledWith(
            '/models/llama-3.2-1b.gguf',
            'Apache-2.0',
            false
        );
        expect(await screen.findAllByText('llama-3.2-1b.gguf', { exact: false })).not.toHaveLength(0);
        expect(screen.getByText('GGUF')).toBeInTheDocument();
        expect(screen.getByTestId('model-sha256')).toHaveTextContent(
            'a1b2c3d4e5f60718293a4b5c6d7e8f90123456789abcdef0123456789abcdef0'
        );
    });

    test('enforces F19 licence acknowledgement gating for local AI toggle', async () => {
        render(SettingsView);
        await screen.findByTestId('settings-loaded');

        const aiToggle = screen.getByTestId('ai-local-toggle');
        const ackCheckbox = screen.getByTestId('license-acknowledge-checkbox');

        // Initially disabled because licence is not acknowledged
        expect(aiToggle).toHaveAttribute('aria-disabled', 'true');
        expect(ackCheckbox).toHaveAttribute('aria-checked', 'false');

        // Acknowledge licence terms
        await fireEvent.click(ackCheckbox);
        expect(ackCheckbox).toHaveAttribute('aria-checked', 'true');
        expect(aiToggle).not.toHaveAttribute('aria-disabled');

        // Enable AI toggle
        await fireEvent.click(aiToggle);
        expect(aiToggle).toHaveAttribute('aria-checked', 'true');

        // Save settings and verify payload has ai_license_acknowledged = true
        const saveBtn = screen.getByRole('button', { name: 'Save settings' });
        await fireEvent.click(saveBtn);

        expect(IpcClient.saveSettings).toHaveBeenCalledWith(
            expect.objectContaining({
                ai_local_enabled: true,
                ai_license_acknowledged: true,
            })
        );
    });

    // BUG-059: fail closed. After a load error the defaults are on screen; saving them would overwrite the user's
    // real settings (licence included).
    test('after a load error, Save is disabled with the reason, and clicking it writes nothing', async () => {
        vi.spyOn(IpcClient, 'getSettings').mockRejectedValue(new Error('disk read failed'));
        render(SettingsView);
        await screen.findByTestId('settings-loaded');
        const save = screen.getByRole('button', { name: 'Save settings' });
        expect(save).toHaveAttribute('aria-disabled', 'true');
        const tip = document.getElementById(save.getAttribute('aria-describedby') ?? '');
        expect(tip).toHaveTextContent("Settings couldn't be loaded. Retry first.");
        await fireEvent.click(save);
        await new Promise((r) => setTimeout(r, 0));
        expect(IpcClient.saveSettings).not.toHaveBeenCalled();
    });

    test('Retry that succeeds re-enables Save, and Save then writes the loaded settings', async () => {
        const get = vi.spyOn(IpcClient, 'getSettings');
        get.mockRejectedValueOnce(new Error('disk read failed'));
        render(SettingsView);
        await screen.findByTestId('settings-loaded');
        expect(screen.getByRole('button', { name: 'Save settings' })).toHaveAttribute('aria-disabled', 'true');
        get.mockResolvedValue({ ...initialSettings, backups_enabled: false });
        await fireEvent.click(screen.getByRole('button', { name: 'Retry' }));
        await vi.waitFor(() => expect(screen.getByRole('button', { name: 'Save settings' })).not.toHaveAttribute('aria-disabled'));
        await fireEvent.click(screen.getByRole('button', { name: 'Save settings' }));
        expect(IpcClient.saveSettings).toHaveBeenCalledWith(expect.objectContaining({ backups_enabled: false }));
    });

    test('Save stays disabled until the first load has finished', async () => {
        let release!: (v: AppSettings) => void;
        vi.spyOn(IpcClient, 'getSettings').mockReturnValue(new Promise<AppSettings>((r) => (release = r)));
        render(SettingsView);
        expect(screen.getByRole('button', { name: 'Save settings' })).toHaveAttribute('aria-disabled', 'true');
        release({ ...initialSettings });
        await screen.findByTestId('settings-loaded');
        await vi.waitFor(() => expect(screen.getByRole('button', { name: 'Save settings' })).not.toHaveAttribute('aria-disabled'));
    });
    test('opening Settings keeps the theme the user is already using (no snap back to the saved value)', async () => {
        const { appState } = await import('$lib/state.svelte');
        appState.setTheme('Dark');
        render(SettingsView);
        await screen.findByTestId('settings-loaded');
        expect(appState.themePreference).toBe('Dark');
        expect(screen.getByRole('button', { name: 'Dark' })).toHaveAttribute('aria-pressed', 'true');
        appState.setTheme('System');
    });
    test('Import model is disabled with the same reason after a failed load, and its handler refuses', async () => {
        vi.spyOn(IpcClient, 'getSettings').mockRejectedValue(new Error('unreadable'));
        const { component } = render(SettingsView);
        await screen.findByTestId('settings-loaded');
        const imp = screen.getByRole('button', { name: 'Import model' });
        expect(imp).toHaveAttribute('aria-disabled', 'true');
        // The reason is attached to the Import button itself (the Save button has the same text).
        const tip = imp.getAttribute('aria-describedby');
        expect(tip).toBeTruthy();
        expect(document.getElementById(tip as string)).toHaveTextContent("Settings couldn't be loaded. Retry first.");
        await fireEvent.input(screen.getByLabelText('Local model file path'), { target: { value: '/m/x.gguf' } });
        await (component as unknown as { handleImportModel: () => Promise<void> }).handleImportModel();
        expect(IpcClient.importLocalAiModel).not.toHaveBeenCalled();
    });

    test('the save handler itself refuses while the load has failed (not only the disabled button)', async () => {
        vi.spyOn(IpcClient, 'getSettings').mockRejectedValue(new Error('unreadable'));
        const { component } = render(SettingsView);
        await screen.findByTestId('settings-loaded');
        await (component as unknown as { handleSaveSettings: () => Promise<void> }).handleSaveSettings();
        expect(IpcClient.saveSettings).not.toHaveBeenCalled();
    });
});

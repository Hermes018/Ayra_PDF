import { afterEach, beforeEach, describe, expect, test, vi } from 'vitest';
import { cleanup, fireEvent, render, screen } from '@testing-library/svelte';
import AboutView from '../AboutView.svelte';
import Sidebar from '$lib/components/Sidebar.svelte';
import { IpcClient } from '$lib/ipc';

vi.mock('@tauri-apps/api/webview', () => ({ getCurrentWebview: () => ({ onDragDropEvent: async () => () => {} }) }));

const info = {
    version: '0.2.0',
    build_hash: 'e89d55beab',
    settings_file: '/home/u/.config/localpdf/settings.toml',
    studio_prefs_file: '/home/u/.config/studio/studio_prefs.json',
    markup_library_file: '/home/u/.config/localpdf/studio_markup_library.json',
    log_dir: '/home/u/.local/state/localpdf',
};

describe('AboutView', () => {
    beforeEach(() => {
        vi.restoreAllMocks();
        vi.spyOn(IpcClient, 'getAboutInfo').mockResolvedValue({ ...info });
        vi.spyOn(IpcClient, 'getThirdPartyNotices').mockResolvedValue('# Third-party notices\nMIT licence text');
    });
    afterEach(() => cleanup());

    test('shows the version and the build hash from the app', async () => {
        render(AboutView);
        expect(await screen.findByText('0.2.0')).toBeInTheDocument();
        expect(screen.getByText('e89d55beab')).toBeInTheDocument();
    });

    test('shows "unknown" when the app was built outside git', async () => {
        vi.spyOn(IpcClient, 'getAboutInfo').mockResolvedValue({ ...info, build_hash: 'unknown' });
        render(AboutView);
        expect(await screen.findByText('unknown')).toBeInTheDocument();
    });

    test('states exactly what offline means', async () => {
        render(AboutView);
        await screen.findByText('0.2.0');
        expect(
            screen.getByText('100% offline: the app makes no network connections. Your PDFs never leave this computer.'),
        ).toBeInTheDocument();
    });

    test('says where settings and logs live, and that logs stay local', async () => {
        render(AboutView);
        expect(await screen.findByText(info.settings_file)).toBeInTheDocument();
        expect(screen.getByText(info.log_dir)).toBeInTheDocument();
        expect(screen.getByText(info.studio_prefs_file)).toBeInTheDocument();
        expect(screen.getByText(info.markup_library_file)).toBeInTheDocument();
        expect(screen.getByText(/logs stay on this computer/i)).toBeInTheDocument();
    });

    test('the licence text loads on request into a scrollable, focusable region', async () => {
        render(AboutView);
        await screen.findByText('0.2.0');
        expect(IpcClient.getThirdPartyNotices).not.toHaveBeenCalled();
        await fireEvent.click(screen.getByRole('button', { name: 'Show open-source licences' }));
        const region = await screen.findByRole('region', { name: 'Licence notices' });
        expect(region).toHaveTextContent('MIT licence text');
        expect(region).toHaveAttribute('tabindex', '0');
    });

    test('a missing licence file shows an error with Retry, and Retry loads it', async () => {
        const get = vi.spyOn(IpcClient, 'getThirdPartyNotices');
        get.mockRejectedValueOnce({ kind: 'missing', message: 'the licence notices file was not found' });
        render(AboutView);
        await screen.findByText('0.2.0');
        await fireEvent.click(screen.getByRole('button', { name: 'Show open-source licences' }));
        const alert = await screen.findByRole('alert');
        expect(alert).toHaveTextContent("Couldn't load the licence notices.");
        expect(alert).not.toHaveTextContent('/home');
        get.mockResolvedValue('# Third-party notices\nBack again');
        await fireEvent.click(screen.getByRole('button', { name: 'Retry' }));
        expect(await screen.findByRole('region', { name: 'Licence notices' })).toHaveTextContent('Back again');
        expect(screen.queryByRole('alert')).toBeNull();
    });

    test('when the app info cannot be read there is an error with Retry', async () => {
        const get = vi.spyOn(IpcClient, 'getAboutInfo');
        get.mockRejectedValueOnce(new Error('boom'));
        render(AboutView);
        const alert = await screen.findByRole('alert');
        expect(alert).toHaveTextContent("Couldn't read the app details.");
        get.mockResolvedValue({ ...info });
        await fireEvent.click(screen.getByRole('button', { name: 'Retry' }));
        expect(await screen.findByText('0.2.0')).toBeInTheDocument();
    });
});

describe('Sidebar About item', () => {
    afterEach(() => cleanup());
    test('has one About item that opens the About view', async () => {
        const { component } = render(Sidebar, { currentView: 'Studio' });
        const about = screen.getByRole('button', { name: /About/ });
        await fireEvent.click(about);
        expect(about).toHaveAttribute('aria-current', 'page');
        void component;
    });
});

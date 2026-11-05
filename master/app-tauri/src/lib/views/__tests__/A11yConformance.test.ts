import { expect, test, describe, afterEach, vi } from 'vitest';
import { render, screen, cleanup, fireEvent } from '@testing-library/svelte';
import Sidebar from '$lib/components/Sidebar.svelte';
import StatusBar from '$lib/components/StatusBar.svelte';
import ConvertView from '../ConvertView.svelte';

// Mock child views of ConvertView to isolate tab semantics
vi.mock('../ConvertFromPdfView.svelte', () => ({
    default: () => {}
}));
vi.mock('../ConvertToPdfView.svelte', () => ({
    default: () => {}
}));

describe('A11y Conformance & Semantic Landmark Suite', () => {
    afterEach(() => {
        cleanup();
    });

    test('Sidebar exposes semantic navigation landmarks and aria-current state', () => {
        render(Sidebar, { currentView: 'Organize' });

        const nav = screen.getByRole('navigation', { name: 'Primary Navigation' });
        expect(nav).toBeInTheDocument();

        const organizeBtn = screen.getByRole('button', { name: /Organize/i });
        expect(organizeBtn).toHaveAttribute('aria-current', 'page');

        const optimizeBtn = screen.getByRole('button', { name: /Optimize/i });
        expect(optimizeBtn).not.toHaveAttribute('aria-current');
    });

    test('Sidebar Settings button correctly sets aria-current', () => {
        render(Sidebar, { currentView: 'Settings' });

        const settingsBtn = screen.getByRole('button', { name: /Settings/i });
        expect(settingsBtn).toHaveAttribute('aria-current', 'page');
    });

    test('StatusBar exposes role status with live announcements', () => {
        render(StatusBar);

        const statusElement = screen.getByRole('status', { name: 'System status' });
        expect(statusElement).toBeInTheDocument();
        expect(screen.getByText('Ready')).toBeInTheDocument();
    });

    test('ConvertView implements WAI-ARIA tablist pattern with proper tab and tabpanel semantics', async () => {
        render(ConvertView);

        const tablist = screen.getByRole('tablist', { name: 'Conversion Mode' });
        expect(tablist).toBeInTheDocument();

        const tabs = screen.getAllByRole('tab');
        expect(tabs).toHaveLength(2);

        const fromPdfTab = tabs[0];
        const toPdfTab = tabs[1];

        // Initial tab state: from-pdf is selected
        expect(fromPdfTab).toHaveAttribute('aria-selected', 'true');
        expect(fromPdfTab).toHaveAttribute('aria-controls', 'panel-from-pdf');
        expect(toPdfTab).toHaveAttribute('aria-selected', 'false');

        const initialPanel = screen.getByRole('tabpanel', { name: /Convert from PDF/i });
        expect(initialPanel).toHaveAttribute('id', 'panel-from-pdf');

        // Switch tab via click
        await fireEvent.click(toPdfTab);

        expect(toPdfTab).toHaveAttribute('aria-selected', 'true');
        expect(fromPdfTab).toHaveAttribute('aria-selected', 'false');

        const switchedPanel = screen.getByRole('tabpanel', { name: /Convert to PDF/i });
        expect(switchedPanel).toHaveAttribute('id', 'panel-to-pdf');
    });
});

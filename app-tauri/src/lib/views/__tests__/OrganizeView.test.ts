import { expect, test, vi, beforeEach, afterEach, describe } from 'vitest';
import { render, screen, cleanup } from '@testing-library/svelte';
import OrganizeView from '../OrganizeView.svelte';
import { appState } from '$lib/state.svelte';

// Mock the nested PdfViewer component since it relies on Tauri IPC internally
vi.mock('$lib/components/PdfViewer.svelte', () => ({
    default: () => {
        // empty component for Svelte 5
    }
}));

describe('OrganizeView', () => {
    beforeEach(() => {
        appState.clear();
    });
    afterEach(() => {
        cleanup();
    });

    test('shows placeholder when no file is active', () => {
        render(OrganizeView);
        expect(screen.getByText('No file selected. Drag and drop a PDF.')).toBeInTheDocument();
    });

    test('shows active file path when files are added', () => {
        appState.addFiles(['/home/ayra/Documents/test.pdf']);
        render(OrganizeView);
        expect(screen.getByText('/home/ayra/Documents/test.pdf')).toBeInTheDocument();
    });
});

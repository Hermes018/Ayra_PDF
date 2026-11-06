// The floating "Help and FAQ" button duplicated the sidebar item and covered content in every view.
// It now shows only while the sidebar is collapsed (when the sidebar item is out of reach).
import { describe, it, expect, afterEach, vi } from 'vitest';
import { render, screen, cleanup } from '@testing-library/svelte';
import { createRawSnippet } from 'svelte';
import MainLayout from './MainLayout.svelte';
import { appState } from '$lib/state.svelte';

vi.mock('@tauri-apps/api/webview', () => ({ getCurrentWebview: () => ({ onDragDropEvent: async () => () => {} }) }));

const children = createRawSnippet(() => ({ render: () => '<div>content</div>' }));

afterEach(() => {
  cleanup();
  appState.sidebarCollapsed = false;
});

describe('MainLayout floating Help and FAQ', () => {
  it('is not shown while the sidebar is open', () => {
    appState.sidebarCollapsed = false;
    const { container } = render(MainLayout, { children, currentView: 'Convert' });
    expect(container.querySelector('.faq-floating-trigger')).toBeNull();
    expect(screen.getAllByRole('button', { name: 'Help and FAQ' })).toHaveLength(1);
  });
  it('is shown while the sidebar is collapsed', () => {
    appState.sidebarCollapsed = true;
    const { container } = render(MainLayout, { children, currentView: 'Convert' });
    expect(container.querySelector('.faq-floating-trigger')).not.toBeNull();
  });
});

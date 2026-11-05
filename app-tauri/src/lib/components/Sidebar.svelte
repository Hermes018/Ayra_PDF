<script lang="ts">
    import { faqStore } from '$lib/i18n/faqStore.svelte';
    import { appState } from '$lib/state.svelte';

    let { currentView = $bindable() } = $props<{
        currentView: string;
    }>();

    const menuItems = [
        { id: 'Studio', label: 'Drawboard Studio', icon: '📐' },
        { id: 'Convert', label: 'Convert', icon: '🔄' },
        { id: 'Organize', label: 'Organize', icon: '🗂️' },
        { id: 'Optimize', label: 'Optimize', icon: '⚡' },
        { id: 'Security', label: 'Security', icon: '🔒' },
        { id: 'Edit', label: 'Edit', icon: '✏️' },
        { id: 'Forms', label: 'Forms & Signatures', icon: '📝' },
        { id: 'Markdown', label: 'Markdown', icon: '📄' },
        { id: 'Summarize', label: 'AI Summarizer', icon: '✨' },
        { id: 'Queue', label: 'Batch Queue', icon: '📋' },
        { id: 'Pipelines', label: 'Workflows', icon: '🔀' },
    ];
</script>

<aside class="sidebar glass--sidebar" aria-label="Application Sidebar">
    <div class="logo">
        <div class="logo-title">
            <span class="icon" aria-hidden="true">📄</span>
            <h2>LocalPDF</h2>
        </div>
        <button
            type="button"
            class="sidebar-collapse-btn"
            onclick={() => appState.toggleSidebar()}
            aria-label="Collapse Sidebar (Ctrl+B)"
            title="Collapse Sidebar (Ctrl+B)"
        >
            ◀
        </button>
    </div>

    <nav aria-label="Primary Navigation">
        <ul>
            {#each menuItems as item}
                <li>
                    <button
                        class="sidebar__item"
                        class:active={currentView === item.id}
                        aria-current={currentView === item.id ? 'page' : undefined}
                        onclick={() => { currentView = item.id; }}
                    >
                        <span class="icon" aria-hidden="true">{item.icon}</span>
                        {item.label}
                    </button>
                </li>
            {/each}
        </ul>
    </nav>

    <div class="bottom-nav">
        <button
            type="button"
            class="sidebar__item"
            onclick={() => faqStore.open()}
            aria-label="Frequently Asked Questions and Help"
        >
            <span class="icon" aria-hidden="true">❓</span>
            Help & FAQ
        </button>
        <button
            class="sidebar__item"
            class:active={currentView === 'Settings'}
            aria-current={currentView === 'Settings' ? 'page' : undefined}
            onclick={() => { currentView = 'Settings'; }}
        >
            <span class="icon" aria-hidden="true">⚙️</span>
            Settings
        </button>
    </div>
</aside>

<style>
    .sidebar {
        width: var(--sidebar-width, 210px);
        min-width: var(--sidebar-width, 210px);
        background-color: var(--glass-bg-sidebar);
        -webkit-backdrop-filter: var(--glass-blur);
        backdrop-filter: var(--glass-blur);
        color: var(--text-primary);
        display: flex;
        flex-direction: column;
        height: 100%;
        border-right: 1px solid var(--glass-border-subtle);
        transition: width 0.2s cubic-bezier(0.16, 1, 0.3, 1), min-width 0.2s cubic-bezier(0.16, 1, 0.3, 1), opacity 0.15s ease, background-color var(--dur-med, 0.2s) ease, border-color var(--dur-med, 0.2s) ease;
        box-sizing: border-box;
    }

    .logo {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 0.5rem;
        padding: 1.25rem 1rem;
        font-size: 1.15rem;
        font-weight: 700;
        color: var(--accent-primary);
        letter-spacing: -0.01em;
    }

    .logo-title {
        display: flex;
        align-items: center;
        gap: 0.5rem;
    }

    .sidebar-collapse-btn {
        background: transparent;
        border: 1px solid transparent;
        color: var(--text-muted, var(--text-primary));
        border-radius: var(--radius-sm, 6px);
        padding: 4px 6px;
        font-size: 0.85rem;
        cursor: pointer;
        display: flex;
        align-items: center;
        justify-content: center;
        transition: background-color var(--dur-fast, 120ms) ease, color var(--dur-fast, 120ms) ease, border-color var(--dur-fast, 120ms) ease;
    }

    .sidebar-collapse-btn:hover {
        background: var(--glass-bg-hover);
        color: var(--accent-primary);
        border-color: var(--glass-border-subtle);
    }

    .logo h2 {
        margin: 0;
        font-size: inherit;
        font-weight: inherit;
    }

    nav {
        flex: 1;
        padding: 0 0.5rem;
        overflow-y: auto;
    }

    ul {
        list-style: none;
        padding: 0;
        margin: 0;
        display: flex;
        flex-direction: column;
        gap: 2px;
    }

    .sidebar__item {
        display: flex;
        align-items: center;
        gap: 0.6rem;
        width: 100%;
        padding: 0.5rem 0.75rem;
        background: transparent;
        border: none;
        color: var(--text-primary);
        font-size: 0.9rem;
        text-align: left;
        border-radius: var(--radius-md, 10px);
        cursor: pointer;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        transition: background-color var(--dur-fast, 120ms) ease, color var(--dur-fast, 120ms) ease;
    }

    .sidebar__item:hover {
        background-color: var(--glass-bg-hover);
    }

    .sidebar__item:focus-visible {
        outline: none;
        box-shadow: 0 0 0 3px var(--accent-ring);
    }

    .sidebar__item.active {
        background-color: var(--glass-bg-active);
        color: var(--accent-primary);
        font-weight: 600;
    }

    .bottom-nav {
        padding: 0.75rem 0.5rem;
        border-top: 1px solid var(--glass-border-subtle);
    }
</style>

<script lang="ts">
    let { currentView = $bindable() } = $props<{
        currentView: string;
    }>();

    const menuItems = [
        { id: 'Organize', label: 'Organize', icon: '🗂️' },
        { id: 'Optimize', label: 'Optimize', icon: '⚡' },
        { id: 'Convert', label: 'Convert', icon: '🔄' },
        { id: 'Security', label: 'Security', icon: '🔒' },
        { id: 'Edit', label: 'Edit', icon: '✏️' },
        { id: 'Forms', label: 'Forms & Signatures', icon: '📝' },
        { id: 'Markdown', label: 'Markdown', icon: '📝' },
        { id: 'Summarize', label: 'AI Summarizer', icon: '✨' },
        { id: 'Queue', label: 'Batch Queue', icon: '📋' },
        { id: 'Pipelines', label: 'Workflows', icon: '🔀' },
    ];
</script>

<aside class="sidebar" aria-label="Application Sidebar">
    <div class="logo">
        <span class="icon" aria-hidden="true">📄</span>
        <h2>LocalPDF</h2>
    </div>

    <nav aria-label="Primary Navigation">
        <ul>
            {#each menuItems as item}
                <li>
                    <button
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
        width: 250px;
        background-color: var(--surface-bg, #1e1e2e);
        color: var(--text-color, #cdd6f4);
        display: flex;
        flex-direction: column;
        height: 100%;
        border-right: 1px solid var(--border-color, #313244);
        transition: background-color 0.2s ease, border-color 0.2s ease;
    }

    .logo {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        padding: 1.5rem;
        font-size: 1.25rem;
        font-weight: bold;
        color: var(--primary-color, #cba6f7);
    }

    nav {
        flex: 1;
        padding: 0 1rem;
        overflow-y: auto;
    }

    ul {
        list-style: none;
        padding: 0;
        margin: 0;
        display: flex;
        flex-direction: column;
        gap: 0.25rem;
    }

    button {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        width: 100%;
        padding: 0.75rem 1rem;
        background: transparent;
        border: none;
        color: var(--text-color, #cdd6f4);
        font-size: 0.95rem;
        text-align: left;
        border-radius: 8px;
        cursor: pointer;
        transition: all 0.2s;
    }

    button:hover {
        background-color: var(--surface-elevated, #313244);
    }

    button:focus-visible {
        outline: 2px solid var(--primary-color, #cba6f7);
        outline-offset: 2px;
    }

    button.active {
        background-color: var(--primary-color, #89b4fa);
        color: #ffffff;
        font-weight: 600;
    }

    .bottom-nav {
        padding: 1rem;
        border-top: 1px solid var(--border-color, #313244);
    }
</style>

<script lang="ts">
    import { faqStore } from '$lib/i18n/faqStore.svelte';
    import { appState } from '$lib/state.svelte';
    import Button from '$lib/components/ui/Button.svelte';
    import IconButton from '$lib/components/ui/IconButton.svelte';

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
        { id: 'Forms', label: 'Forms and signatures', icon: '📝' },
        { id: 'Markdown', label: 'Markdown', icon: '📄' },
        { id: 'Summarize', label: 'AI summarizer', icon: '✨' },
        { id: 'Queue', label: 'Batch queue', icon: '📋' },
        { id: 'Pipelines', label: 'Workflows', icon: '🔀' },
    ];
</script>

<aside class="sidebar glass--sidebar" aria-label="Application sidebar">
    <div class="logo">
        <div class="logo-title">
            <span class="icon" aria-hidden="true">📄</span>
            <h2>LocalPDF</h2>
        </div>
        <IconButton
            icon="chevron.left"
            size="sm"
            ariaLabel="Collapse sidebar"
            shortcutCmd="app.toggleSidebar"
            onclick={() => appState.toggleSidebar()}
        />
    </div>

    <nav aria-label="Primary Navigation">
        <ul>
            {#each menuItems as item}
                <li>
                    <Button
                        variant="plain"
                        block
                        current={currentView === item.id}
                        onclick={() => { currentView = item.id; }}
                    >
                        <span class="icon" aria-hidden="true">{item.icon}</span>
                        {item.label}
                    </Button>
                </li>
            {/each}
        </ul>
    </nav>

    <div class="bottom-nav">
        <Button variant="plain" block onclick={() => faqStore.open()}>
            <span class="icon" aria-hidden="true">❓</span>
            Help and FAQ
        </Button>
        <Button variant="plain" block current={currentView === 'Settings'} onclick={() => { currentView = 'Settings'; }}>
            <span class="icon" aria-hidden="true">⚙️</span>
            Settings
        </Button>
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
        gap: var(--space-2);
        padding: var(--space-5) var(--space-4);
        font-size: var(--font-size-title2);
        font-weight: 700;
        color: var(--accent-primary);
        letter-spacing: -0.01em;
    }

    .logo-title {
        display: flex;
        align-items: center;
        gap: var(--space-2);
    }

    .logo h2 {
        margin: 0;
        font-size: inherit;
        font-weight: inherit;
    }

    nav {
        flex: 1;
        padding: 0 var(--space-2);
        overflow-y: auto;
    }

    ul {
        list-style: none;
        padding: 0;
        margin: 0;
        display: flex;
        flex-direction: column;
        gap: var(--space-1);
    }

    .bottom-nav {
        padding: var(--space-3) var(--space-2);
        border-top: 1px solid var(--glass-border-subtle);
        display: flex;
        flex-direction: column;
        gap: var(--space-1);
    }
</style>

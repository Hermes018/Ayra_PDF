<script lang="ts">
    import { type Snippet } from 'svelte';
    import Sidebar from './Sidebar.svelte';
    import StatusBar from './StatusBar.svelte';
    import Dropzone from './Dropzone.svelte';
    import ToastContainer from './ToastContainer.svelte';
    import FaqModal from './FaqModal.svelte';
    import { faqStore } from '$lib/i18n/faqStore.svelte';
    import { appState } from '$lib/state.svelte';
    import Button from '$lib/components/ui/Button.svelte';
    import IconButton from '$lib/components/ui/IconButton.svelte';

    let { children, currentView = $bindable() } = $props<{
        children: Snippet;
        currentView: string;
    }>();
</script>

<a href="#main-content" class="skip-link">Skip to main content</a>

<div class="app-container app-shell" class:sidebar-collapsed={appState.sidebarCollapsed}>
    {#if appState.sidebarCollapsed}
        <div class="sidebar-floating-toggle">
            <IconButton
                icon="chevron.right"
                ariaLabel="Expand sidebar"
                shortcutCmd="app.toggleSidebar"
                data-testid="sidebar-toggle-btn"
                onclick={() => appState.toggleSidebar()}
            />
        </div>
    {/if}

    <Sidebar bind:currentView />
    
    <div class="main-content-wrapper content-column">
        <main
            id="main-content"
            class="main-content main-view"
            class:studio-bleed={currentView === 'Studio'}
            tabindex="-1"
        >
            <div class="view-inner" class:studio-bleed-inner={currentView === 'Studio'}>
                {@render children()}
            </div>
        </main>
        
        <StatusBar />
    </div>
    
    {#if appState.sidebarCollapsed}
        <div class="faq-floating-trigger">
            <Button variant="secondary" size="sm" onclick={() => faqStore.toggle()}>Help and FAQ</Button>
        </div>
    {/if}

    <Dropzone />
    <ToastContainer />
    <FaqModal />
</div>

<style>
    .app-container {
        display: flex;
        height: 100vh;
        width: 100vw;
        background: var(--glass-canvas-gradient), var(--glass-bg-canvas);
        overflow: hidden;
    }

    .main-content-wrapper {
        flex: 1;
        display: flex;
        flex-direction: column;
        min-width: 0;
        height: 100%;
        overflow: hidden;
    }

    .main-content {
        flex: 1;
        min-width: 0;
        width: 100%;
        max-width: 100%;
        overflow-x: hidden;
        overflow-y: auto;
        padding: clamp(var(--space-4), 2.5vw, var(--space-8));
        box-sizing: border-box;
        background-color: var(--glass-bg-canvas);
        backdrop-filter: blur(24px) saturate(180%);
        -webkit-backdrop-filter: blur(24px) saturate(180%);
        color: var(--text-primary);
        outline: none;
        transition: background-color var(--dur-med, 0.2s) ease, color var(--dur-med, 0.2s) ease;
    }

    .view-inner {
        max-width: 1200px;
        margin: 0 auto;
        width: 100%;
    }

    .main-content.studio-bleed {
        padding: 0;
        overflow: hidden;
    }

    .view-inner.studio-bleed-inner {
        max-width: none;
        height: 100%;
        margin: 0;
    }

    .skip-link {
        position: absolute;
        top: -100px;
        left: 1rem;
        background: var(--accent-primary);
        color: var(--text-on-accent);
        padding: var(--space-2) var(--space-4);
        font-weight: 600;
        border-radius: var(--radius-xs, 4px);
        z-index: 99999;
        text-decoration: none;
        transition: top 0.15s ease-out;
    }

    .skip-link:focus-visible {
        top: 1rem;
        outline: 3px solid var(--accent-ring);
        outline-offset: 2px;
    }

    .faq-floating-trigger {
        position: fixed;
        bottom: calc(var(--statusbar-height, 28px) + var(--space-3));
        right: var(--space-5);
        z-index: 9000;
        box-shadow: var(--glass-shadow-sm);
        border-radius: var(--radius-md);
    }

    .sidebar-floating-toggle {
        position: fixed;
        top: var(--space-3);
        left: var(--space-3);
        z-index: 9500;
        background-color: var(--glass-bg-elevated);
        border: 1px solid var(--glass-border-specular);
        border-radius: var(--radius-md);
        box-shadow: var(--glass-shadow-sm);
    }

    :global(.app-shell.sidebar-collapsed .sidebar) {
        width: 0 !important;
        min-width: 0 !important;
        padding: 0 !important;
        border-right: none !important;
        overflow: hidden !important;
        opacity: 0 !important;
        pointer-events: none !important;
    }
</style>

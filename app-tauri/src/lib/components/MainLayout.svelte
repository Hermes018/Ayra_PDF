<script lang="ts">
    import { type Snippet } from 'svelte';
    import Sidebar from './Sidebar.svelte';
    import StatusBar from './StatusBar.svelte';
    import Dropzone from './Dropzone.svelte';
    import ToastContainer from './ToastContainer.svelte';
    import FaqModal from './FaqModal.svelte';
    import { faqStore } from '$lib/i18n/faqStore.svelte';

    let { children, currentView = $bindable() } = $props<{
        children: Snippet;
        currentView: string;
    }>();
</script>

<a href="#main-content" class="skip-link">Skip to main content</a>

<div class="app-container app-shell">
    <Sidebar bind:currentView />
    
    <div class="main-content-wrapper content-column">
        <main id="main-content" class="main-content main-view" tabindex="-1">
            <div class="view-inner">
                {@render children()}
            </div>
        </main>
        
        <StatusBar />
    </div>
    
    <button
        type="button"
        class="faq-floating-trigger"
        onclick={() => faqStore.toggle()}
        aria-label="Frequently Asked Questions and Help"
        title="Help & FAQ"
    >
        ?
    </button>

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
        padding: clamp(1rem, 2.5vw, 2rem);
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

    .skip-link {
        position: absolute;
        top: -100px;
        left: 1rem;
        background: var(--accent-primary);
        color: var(--text-on-accent);
        padding: 0.5rem 1rem;
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
        bottom: calc(var(--statusbar-height, 28px) + 0.85rem);
        right: 1.25rem;
        width: 32px;
        height: 32px;
        border-radius: 50%;
        background-color: var(--glass-bg-elevated);
        backdrop-filter: blur(20px) saturate(180%);
        -webkit-backdrop-filter: blur(20px) saturate(180%);
        border: 1px solid var(--glass-border-specular);
        color: var(--text-primary);
        font-size: 0.95rem;
        font-weight: 700;
        cursor: pointer;
        display: flex;
        align-items: center;
        justify-content: center;
        box-shadow: var(--glass-shadow-sm);
        z-index: 9000;
        transition: transform 0.18s cubic-bezier(0.34, 1.56, 0.64, 1), background-color 0.18s ease, border-color 0.18s ease, color 0.18s ease;
    }

    .faq-floating-trigger:hover {
        background-color: var(--glass-bg-hover);
        border-color: var(--accent-primary);
        color: var(--accent-primary);
        transform: scale(1.1);
    }
</style>

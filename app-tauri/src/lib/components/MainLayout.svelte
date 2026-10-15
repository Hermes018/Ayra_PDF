<script lang="ts">
    import { type Snippet } from 'svelte';
    import Sidebar from './Sidebar.svelte';
    import StatusBar from './StatusBar.svelte';
    import Dropzone from './Dropzone.svelte';

    let { children, currentView = $bindable() } = $props<{
        children: Snippet;
        currentView: string;
    }>();
</script>

<a href="#main-content" class="skip-link">Skip to main content</a>

<div class="app-container">
    <Sidebar bind:currentView />
    
    <div class="main-content-wrapper">
        <main id="main-content" class="main-content" tabindex="-1">
            {@render children()}
        </main>
        
        <StatusBar />
    </div>
    
    <Dropzone />
</div>

<style>
    :global(:root),
    :global([data-theme="dark"]) {
        --bg-color: #0f172a;
        --surface-bg: #1e293b;
        --surface-elevated: #252538;
        --text-color: #f8fafc;
        --text-muted: #94a3b8;
        --border-color: #334155;
        --border-subtle: #1e293b;
        --primary-color: #89b4fa;
        --primary-hover: #b4befe;
        --input-bg: #1e293b;
        --input-text: #f8fafc;
        --input-border: #334155;
        --option-bg: #1e293b;
        --option-text: #f8fafc;
        --shadow-color: rgba(0, 0, 0, 0.4);
    }

    :global([data-theme="light"]) {
        --bg-color: #f8fafc;
        --surface-bg: #ffffff;
        --surface-elevated: #f1f5f9;
        --text-color: #0f172a;
        --text-muted: #64748b;
        --border-color: #cbd5e1;
        --border-subtle: #e2e8f0;
        --primary-color: #2563eb;
        --primary-hover: #1d4ed8;
        --input-bg: #ffffff;
        --input-text: #0f172a;
        --input-border: #cbd5e1;
        --option-bg: #ffffff;
        --option-text: #0f172a;
        --shadow-color: rgba(0, 0, 0, 0.08);
    }

    :global(select) {
        background-color: var(--input-bg);
        color: var(--input-text);
        border: 1px solid var(--input-border);
    }

    :global(select option) {
        background-color: var(--option-bg);
        color: var(--option-text);
    }

    :global(input[type="text"]),
    :global(input[type="number"]),
    :global(input[type="password"]),
    :global(textarea) {
        background-color: var(--input-bg);
        color: var(--input-text);
        border: 1px solid var(--input-border);
    }

    :global(body) {
        margin: 0;
        padding: 0;
        font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, 'Open Sans', 'Helvetica Neue', sans-serif;
        background-color: var(--bg-color);
        color: var(--text-color);
        height: 100vh;
        overflow: hidden;
        transition: background-color 0.2s ease, color 0.2s ease;
    }

    .app-container {
        display: flex;
        height: 100vh;
        width: 100vw;
        background-color: var(--bg-color);
    }

    .main-content-wrapper {
        flex: 1;
        display: flex;
        flex-direction: column;
        min-width: 0;
    }

    .main-content {
        flex: 1;
        min-width: 0;
        overflow-x: hidden;
        overflow-y: auto;
        box-sizing: border-box;
        background-color: var(--bg-color);
        color: var(--text-color);
        outline: none;
        transition: background-color 0.2s ease, color 0.2s ease;
    }

    .skip-link {
        position: absolute;
        top: -100px;
        left: 1rem;
        background: #cba6f7;
        color: #11111b;
        padding: 0.5rem 1rem;
        font-weight: 600;
        border-radius: 4px;
        z-index: 9999;
        text-decoration: none;
        transition: top 0.15s ease-out;
    }

    .skip-link:focus-visible {
        top: 1rem;
        outline: 3px solid #89b4fa;
        outline-offset: 2px;
    }
</style>

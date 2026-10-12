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
    :global(body) {
        margin: 0;
        padding: 0;
        font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, 'Open Sans', 'Helvetica Neue', sans-serif;
        background-color: #1e1e2e;
        color: #cdd6f4;
        height: 100vh;
        overflow: hidden;
    }

    .app-container {
        display: flex;
        height: 100vh;
        width: 100vw;
    }

    .main-content-wrapper {
        flex: 1;
        display: flex;
        flex-direction: column;
        min-width: 0;
    }

    .main-content {
        flex: 1;
        overflow-y: auto;
        background-color: #181825;
        outline: none;
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

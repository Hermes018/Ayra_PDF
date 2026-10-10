<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import { getCurrentWebview } from '@tauri-apps/api/webview';
    import { appState } from '$lib/state.svelte';
    
    let isDragging = $state(false);
    let unlisten: (() => void) | undefined;

    onMount(async () => {
        unlisten = await getCurrentWebview().onDragDropEvent((event) => {
            if (event.payload.type === 'enter' || event.payload.type === 'over') {
                isDragging = true;
            } else if (event.payload.type === 'leave') {
                isDragging = false;
            } else if (event.payload.type === 'drop') {
                isDragging = false;
                const paths = event.payload.paths;
                if (Array.isArray(paths) && paths.length > 0) {
                    appState.addFiles(paths);
                }
            }
        });
    });

    onDestroy(() => {
        if (unlisten) unlisten();
    });
</script>

{#if isDragging}
    <div class="dropzone-overlay">
        <div class="drop-message">
            <span class="icon">📄</span>
            <h2>Drop PDFs here</h2>
        </div>
    </div>
{/if}

<style>
    .dropzone-overlay {
        position: fixed;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        background-color: rgba(137, 180, 250, 0.2);
        backdrop-filter: blur(4px);
        z-index: 9999;
        display: flex;
        align-items: center;
        justify-content: center;
        border: 4px dashed #89b4fa;
        pointer-events: none; /* Let events pass through to the webview */
    }

    .drop-message {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 1rem;
        background-color: #1e1e2e;
        padding: 3rem;
        border-radius: 1rem;
        box-shadow: 0 10px 25px rgba(0,0,0,0.5);
    }

    .icon {
        font-size: 4rem;
    }

    h2 {
        margin: 0;
        color: #cdd6f4;
    }
</style>

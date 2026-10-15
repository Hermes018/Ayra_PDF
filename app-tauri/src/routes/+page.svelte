<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import { listen, type UnlistenFn } from '@tauri-apps/api/event';
    import { appState } from '$lib/state.svelte';
    import { getStartupFiles } from '$lib/ipc';
    import MainLayout from '$lib/components/MainLayout.svelte';
    import OrganizeView from '$lib/views/OrganizeView.svelte';
    import OptimizeView from '$lib/views/OptimizeView.svelte';
    import ConvertView from '$lib/views/ConvertView.svelte';
    import SecurityView from '$lib/views/SecurityView.svelte';
    import EditView from '$lib/views/EditView.svelte';
    import FormsView from '$lib/views/FormsView.svelte';
    import MarkdownView from '$lib/views/MarkdownView.svelte';
    import BatchQueueView from '$lib/views/BatchQueueView.svelte';
    import PipelinesView from '$lib/views/PipelinesView.svelte';
    import SettingsView from '$lib/views/SettingsView.svelte';
    import AiSummarizeView from '$lib/views/AiSummarizeView.svelte';

    // Svelte 5 reactive state for view routing
    let currentView = $state('Organize');

    let unlistenOpenFiles: UnlistenFn | undefined;

    onMount(() => {
        getStartupFiles().then((files) => {
            if (files && files.length > 0) {
                appState.addFiles(files);
            }
        });

        listen<string[]>('lpdf://open-files', (event) => {
            if (event.payload && event.payload.length > 0) {
                appState.addFiles(event.payload);
            }
        }).then((unlisten) => {
            unlistenOpenFiles = unlisten;
        }).catch((err) => {
            console.error('Failed to listen for lpdf://open-files:', err);
        });
    });

    onDestroy(() => {
        if (unlistenOpenFiles) {
            unlistenOpenFiles();
        }
    });
</script>

<MainLayout bind:currentView>
    {#if currentView === 'Organize'}
        <OrganizeView />
    {:else if currentView === 'Optimize'}
        <OptimizeView />
    {:else if currentView === 'Convert'}
        <ConvertView />
    {:else if currentView === 'Security'}
        <SecurityView />
    {:else if currentView === 'Edit'}
        <EditView />
    {:else if currentView === 'Forms'}
        <FormsView />
    {:else if currentView === 'Markdown'}
        <MarkdownView />
    {:else if currentView === 'Summarize'}
        <AiSummarizeView onNavigate={(view) => { currentView = view; }} />
    {:else if currentView === 'Queue'}
        <BatchQueueView />
    {:else if currentView === 'Pipelines'}
        <PipelinesView />
    {:else if currentView === 'Settings'}
        <SettingsView />
    {:else}
        <div>Unknown view: {currentView}</div>
    {/if}
</MainLayout>

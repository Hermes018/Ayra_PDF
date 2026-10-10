<script lang="ts">
    import MainLayout from '$lib/components/MainLayout.svelte';
    import OrganizeView from '$lib/views/OrganizeView.svelte';
    import OptimizeView from '$lib/views/OptimizeView.svelte';
    import ConvertToPdfView from '$lib/views/ConvertToPdfView.svelte';
    import SecurityView from '$lib/views/SecurityView.svelte';
    import EditView from '$lib/views/EditView.svelte';
    import FormsView from '$lib/views/FormsView.svelte';
    import MarkdownView from '$lib/views/MarkdownView.svelte';
    import SettingsView from '$lib/views/SettingsView.svelte';

    // Svelte 5 reactive state for view routing
    let currentView = $state('Organize');
</script>

<MainLayout bind:currentView>
    {#if currentView === 'Organize'}
        <OrganizeView />
    {:else if currentView === 'Optimize'}
        <OptimizeView />
    {:else if currentView === 'Convert'}
        <ConvertToPdfView />
    {:else if currentView === 'Security'}
        <SecurityView />
    {:else if currentView === 'Edit'}
        <EditView />
    {:else if currentView === 'Forms'}
        <FormsView />
    {:else if currentView === 'Markdown'}
        <MarkdownView />
    {:else if currentView === 'Settings'}
        <SettingsView />
    {:else}
        <div>Unknown view: {currentView}</div>
    {/if}
</MainLayout>

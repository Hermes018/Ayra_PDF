<script lang="ts">
    // The marker (marker.ts) must never appear in a release frontend bundle (tools/check-no-gallery.sh).
    import { GALLERY_MARKER } from './marker';
    import { onMount } from 'svelte';
    import { theme } from '$lib/core/themeContract.svelte';
    import { commands } from '$lib/core/commandEngine.svelte';
    import { appState } from '$lib/state.svelte';
    import ToastContainer from '$lib/components/ToastContainer.svelte';
    import Button from '$lib/components/ui/Button.svelte';
    import IconButton from '$lib/components/ui/IconButton.svelte';
    import Input from '$lib/components/ui/Input.svelte';
    import Select from '$lib/components/ui/Select.svelte';
    import Toggle from '$lib/components/ui/Toggle.svelte';
    import Dialog from '$lib/components/ui/Dialog.svelte';
    import EmptyState from '$lib/components/ui/EmptyState.svelte';
    import ErrorState from '$lib/components/ui/ErrorState.svelte';
    import Progress from '$lib/components/ui/Progress.svelte';
    import SidebarSection from '$lib/components/ui/SidebarSection.svelte';

    // Early error recorder so the E2E flow also sees errors thrown while this page mounts.
    const w = window as unknown as { __galleryErrors?: string[] };
    w.__galleryErrors ??= [];
    window.addEventListener('error', (e) => w.__galleryErrors?.push(String(e.message)));
    window.addEventListener('unhandledrejection', (e) => w.__galleryErrors?.push(String(e.reason)));

    let dark = $state(new URLSearchParams(location.search).get('theme') !== 'light');
    $effect(() => {
        theme.mode = dark ? 'dark' : 'light';
        theme.applyToRoot(document.documentElement);
    });

    let name = $state('report');
    let badName = $state('');
    let mode = $state('a');
    let on = $state(true);
    let off = $state(false);
    let dialogOpen = $state(false);
    let sheetOpen = $state(false);
    let pressed = $state(false);
    let clicks = $state(0);
    const options = [{ value: 'a', label: 'Fast' }, { value: 'b', label: 'Small file' }];
    const noop = () => {};

    onMount(() => {
        // Same single keyboard entry point as the app (Dialog Escape goes through it).
        const detachKeys = commands.attachKeyboard(window);
        document.documentElement.dataset.galleryReady = '1';
        return detachKeys;
    });
</script>

<main class="gallery" data-gallery={GALLERY_MARKER}>
    <header class="bar">
        <h1>Component gallery</h1>
        <Toggle bind:checked={dark} label="Dark theme" data-testid="g-theme" />
    </header>

    <section aria-labelledby="g-buttons">
        <h2 id="g-buttons">Button</h2>
        <div class="row">
            {#each ['primary', 'secondary', 'destructive', 'plain'] as const as variant}
                <Button {variant} onclick={() => clicks++} data-testid="g-btn-{variant}">Save copy</Button>
            {/each}
        </div>
        <div class="row">
            <Button variant="primary" size="sm" onclick={noop}>Small</Button>
            <Button variant="primary" size="md" onclick={noop}>Medium</Button>
            <Button variant="primary" size="lg" onclick={noop}>Large</Button>
        </div>
        <div class="row">
            <Button variant="primary" onclick={noop} disabled disabledReason="Open a PDF to enable Save" data-testid="g-btn-disabled">Save copy</Button>
            <Button variant="secondary" onclick={noop} disabled disabledReason="Select pages to enable Delete">Delete pages</Button>
            <Button variant="primary" onclick={noop} busy>Compress PDF</Button>
        </div>
        <p class="note" data-testid="g-clicks">Activations: {clicks}</p>
    </section>

    <section aria-labelledby="g-icons">
        <h2 id="g-icons">IconButton</h2>
        <div class="row">
            <IconButton icon="undo" ariaLabel="Undo" onclick={noop} shortcutCmd="history.undo" />
            <IconButton icon="pin" ariaLabel="Pin toolbar" pressed={pressed} onclick={() => (pressed = !pressed)} />
            <IconButton icon="redo" ariaLabel="Redo" onclick={noop} disabled disabledReason="Nothing to redo" data-testid="g-icon-disabled" />
        </div>
    </section>

    <section aria-labelledby="g-fields">
        <h2 id="g-fields">Input, Select, Toggle</h2>
        <div class="row">
            <Input bind:value={name} label="File name" hint="The copy is saved next to the original." />
            <Input bind:value={badName} label="Page range" error="Enter a page number, for example 3." data-testid="g-input-error" />
            <Input value="" label="Password" type="password" disabled disabledReason="Open a PDF first" />
        </div>
        <div class="row">
            <Select bind:value={mode} label="Quality" {options} />
            <Select value="a" label="Format" {options} disabled disabledReason="Only one format is available" />
        </div>
        <div class="row">
            <Toggle bind:checked={on} label="Show grid" />
            <Toggle bind:checked={off} label="Snap to objects" />
            <Toggle checked={false} label="Labs tools" disabled disabledReason="Turn on Labs in Settings" />
        </div>
    </section>

    <section aria-labelledby="g-overlays">
        <h2 id="g-overlays">Dialog, Sheet, Toast</h2>
        <div class="row">
            <Button variant="secondary" onclick={() => (dialogOpen = true)} data-testid="g-open-dialog">Open dialog</Button>
            <Button variant="secondary" onclick={() => (sheetOpen = true)}>Open sheet</Button>
            <Button variant="secondary" onclick={() => appState.addToast('Saved report-annotated.pdf', 'success', '/home/user/report-annotated.pdf')}>Show success toast</Button>
            <Button variant="secondary" onclick={() => appState.addToast("Couldn't read page 4", 'error', 'The file may be damaged; try another copy.')}>Show error toast</Button>
        </div>
    </section>

    <section aria-labelledby="g-states">
        <h2 id="g-states">EmptyState, ErrorState, Progress</h2>
        <div class="row">
            <EmptyState title="No PDF open" body="Open a PDF to start marking it up." action={{ label: 'Open PDF', onclick: noop }} />
            <ErrorState what="Couldn't read page 4." next="The file may be damaged; try another copy." retry={noop} />
        </div>
        <div class="row">
            <Progress value={42} label="Compressing" oncancel={noop} />
            <Progress value={null} label="Reading file" oncancel={noop} />
        </div>
    </section>

    <section aria-labelledby="g-sidebar">
        <h2 id="g-sidebar">SidebarSection</h2>
        <div class="row">
            <SidebarSection title="Tools" collapsible>
                <Button variant="plain" onclick={noop}>Merge</Button>
                <Button variant="plain" onclick={noop}>Split</Button>
            </SidebarSection>
        </div>
    </section>

    <Dialog bind:open={dialogOpen} title="Delete 3 pages?" onclose={noop}>
        This removes them from the new file. The original stays unchanged.
        {#snippet actions()}
            <Button variant="secondary" onclick={() => (dialogOpen = false)} data-testid="g-dialog-cancel">Cancel</Button>
            <Button variant="destructive" onclick={() => (dialogOpen = false)}>Delete pages</Button>
        {/snippet}
    </Dialog>
    <Dialog bind:open={sheetOpen} variant="sheet" title="Export options" onclose={noop}>
        Choose how the copy is written.
        {#snippet actions()}
            <Button variant="primary" onclick={() => (sheetOpen = false)}>Save copy</Button>
        {/snippet}
    </Dialog>
    <ToastContainer />
</main>

<style>
    .gallery {
        height: 100vh;
        overflow: auto;
        padding: var(--space-6);
        display: flex;
        flex-direction: column;
        gap: var(--space-6);
        color: var(--text-primary);
        font-size: var(--font-size-body);
    }
    .bar { display: flex; align-items: center; justify-content: space-between; }
    h1 { margin: 0; font-size: var(--font-size-title1); }
    h2 { margin: 0 0 var(--space-3); font-size: var(--font-size-title3); }
    section { padding: var(--space-4); border: 1px solid var(--glass-border-subtle); border-radius: var(--radius-lg); background: var(--glass-bg-surface); }
    .row { display: flex; flex-wrap: wrap; align-items: flex-start; gap: var(--space-4); margin-bottom: var(--space-3); }
    .note { margin: 0; color: var(--text-secondary); font-size: var(--font-size-footnote); }
</style>

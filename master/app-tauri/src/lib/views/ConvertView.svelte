<script lang="ts">
    import ConvertFromPdfView from './ConvertFromPdfView.svelte';
    import ConvertToPdfView from './ConvertToPdfView.svelte';
    import Button from '$lib/components/ui/Button.svelte';

    import { onDestroy } from 'svelte';
    import { commands } from '$lib/core/commandEngine.svelte';

    type Tab = 'from-pdf' | 'to-pdf';
    const TABS: Tab[] = ['from-pdf', 'to-pdf'];
    let activeTab = $state<Tab>('from-pdf');

    // Tabs keyboard pattern: ArrowLeft/Right (wrapping), Home, End. The keys are routed through commandEngine
    // while a tab has focus (registered on focus-in, removed on focus-out); no local keydown listener.
    let offScope: (() => void) | null = null;
    function onTabKey(e: KeyboardEvent): boolean {
        const i = TABS.indexOf(activeTab);
        let next = i;
        if (e.key === 'ArrowRight') next = (i + 1) % TABS.length;
        else if (e.key === 'ArrowLeft') next = (i - 1 + TABS.length) % TABS.length;
        else if (e.key === 'Home') next = 0;
        else if (e.key === 'End') next = TABS.length - 1;
        else return false;
        activeTab = TABS[next];
        document.getElementById(`tab-${TABS[next]}`)?.focus();
        return true;
    }
    // Leaving the view while a tab has focus must not leave the scope registered.
    onDestroy(() => offScope?.());
    function tabsFocusIn() {
        offScope ??= commands.registerKeyScope(onTabKey);
    }
    function tabsFocusOut(e: FocusEvent) {
        // Moving focus between the tabs keeps the scope; leaving the tablist removes it.
        if ((e.currentTarget as HTMLElement).contains(e.relatedTarget as Node | null)) return;
        offScope?.();
        offScope = null;
    }
</script>

<div class="convert-container">
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div class="tab-bar" role="tablist" aria-label="Conversion mode" onfocusin={tabsFocusIn} onfocusout={tabsFocusOut}>
        <Button
            variant="secondary"
            role="tab"
            id="tab-from-pdf"
            controls="panel-from-pdf"
            selected={activeTab === 'from-pdf'}
            tabindex={activeTab === 'from-pdf' ? 0 : -1}
            onclick={() => { activeTab = 'from-pdf'; }}
        >
            <span aria-hidden="true">📤</span>
            <span>Convert from PDF</span>
        </Button>
        <Button
            variant="secondary"
            role="tab"
            id="tab-to-pdf"
            controls="panel-to-pdf"
            selected={activeTab === 'to-pdf'}
            tabindex={activeTab === 'to-pdf' ? 0 : -1}
            onclick={() => { activeTab = 'to-pdf'; }}
        >
            <span aria-hidden="true">📥</span>
            <span>Convert to PDF</span>
        </Button>
    </div>

    {#if activeTab === 'from-pdf'}
        <div id="panel-from-pdf" class="tab-content" role="tabpanel" aria-labelledby="tab-from-pdf">
            <ConvertFromPdfView />
        </div>
    {:else}
        <div id="panel-to-pdf" class="tab-content" role="tabpanel" aria-labelledby="tab-to-pdf">
            <ConvertToPdfView />
        </div>
    {/if}
</div>

<style>
    .convert-container {
        display: flex;
        flex-direction: column;
        height: 100%;
        width: 100%;
        background: transparent;
        color: var(--text-primary);
        overflow: hidden;
    }

    .tab-bar {
        display: flex;
        gap: var(--space-2);
        padding: var(--space-3) var(--space-6) var(--space-2);
        background: transparent;
        border-bottom: 1px solid var(--glass-border-subtle);
    }

    .tab-content {
        flex: 1;
        min-height: 0;
        overflow-y: auto;
    }
</style>

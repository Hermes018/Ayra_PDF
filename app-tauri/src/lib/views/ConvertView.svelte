<script lang="ts">
    import ConvertFromPdfView from './ConvertFromPdfView.svelte';
    import ConvertToPdfView from './ConvertToPdfView.svelte';

    let activeTab = $state<'from-pdf' | 'to-pdf'>('from-pdf');
</script>

<div class="convert-container">
    <div class="tab-bar" role="tablist" aria-label="Conversion Mode">
        <button
            type="button"
            id="tab-from-pdf"
            role="tab"
            aria-selected={activeTab === 'from-pdf'}
            aria-controls="panel-from-pdf"
            tabindex={activeTab === 'from-pdf' ? 0 : -1}
            class="tab-btn"
            class:active={activeTab === 'from-pdf'}
            onclick={() => { activeTab = 'from-pdf'; }}
        >
            <span class="tab-icon" aria-hidden="true">📤</span>
            <span class="tab-text">Convert from PDF</span>
        </button>
        <button
            type="button"
            id="tab-to-pdf"
            role="tab"
            aria-selected={activeTab === 'to-pdf'}
            aria-controls="panel-to-pdf"
            tabindex={activeTab === 'to-pdf' ? 0 : -1}
            class="tab-btn"
            class:active={activeTab === 'to-pdf'}
            onclick={() => { activeTab = 'to-pdf'; }}
        >
            <span class="tab-icon" aria-hidden="true">📥</span>
            <span class="tab-text">Convert to PDF</span>
        </button>
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
        gap: 0.5rem;
        padding: 0.75rem 1.5rem 0.5rem;
        background: transparent;
        border-bottom: 1px solid var(--glass-border-subtle);
    }

    .tab-btn {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        padding: 0.5rem 1.25rem;
        background: var(--glass-bg-surface);
        border: 1px solid var(--glass-border-subtle);
        color: var(--text-secondary);
        font-size: 0.95rem;
        font-weight: 600;
        cursor: pointer;
        transition: all var(--dur-fast, 120ms) ease;
        border-radius: var(--radius-md, 10px);
        box-shadow: inset 0 1px 0 0 var(--glass-border-specular), var(--glass-shadow-sm);
    }

    .tab-btn:hover {
        color: var(--text-primary);
        background: var(--glass-bg-hover);
        transform: translateY(-1px);
    }

    .tab-btn:focus-visible {
        outline: 2px solid var(--accent-ring);
        outline-offset: 2px;
    }

    .tab-btn.active {
        color: var(--text-on-accent);
        background: var(--accent-primary);
        border-color: var(--accent-primary);
        box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
    }

    .tab-icon {
        font-size: 1.1rem;
    }

    .tab-content {
        flex: 1;
        min-height: 0;
        overflow-y: auto;
    }
</style>


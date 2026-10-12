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
        background: #11111b;
        color: #cdd6f4;
        overflow: hidden;
    }

    .tab-bar {
        display: flex;
        gap: 0.5rem;
        padding: 0.75rem 1.5rem 0;
        background: #181825;
        border-bottom: 1px solid #313244;
    }

    .tab-btn {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        padding: 0.65rem 1.25rem;
        background: transparent;
        border: none;
        border-bottom: 2px solid transparent;
        color: #a6adc8;
        font-size: 0.95rem;
        font-weight: 600;
        cursor: pointer;
        transition: all 0.2s ease;
        border-radius: 6px 6px 0 0;
    }

    .tab-btn:hover {
        color: #cdd6f4;
        background: rgba(255, 255, 255, 0.04);
    }

    .tab-btn:focus-visible {
        outline: 2px solid #cba6f7;
        outline-offset: -2px;
    }

    .tab-btn.active {
        color: #89b4fa;
        border-bottom-color: #89b4fa;
        background: rgba(137, 180, 250, 0.08);
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


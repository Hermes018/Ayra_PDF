<script lang="ts">
    import ConvertFromPdfView from './ConvertFromPdfView.svelte';
    import ConvertToPdfView from './ConvertToPdfView.svelte';

    let activeTab = $state<'from-pdf' | 'to-pdf'>('from-pdf');
</script>

<div class="convert-container">
    <div class="tab-bar">
        <button
            type="button"
            class="tab-btn"
            class:active={activeTab === 'from-pdf'}
            onclick={() => { activeTab = 'from-pdf'; }}
        >
            <span class="tab-icon">📤</span>
            <span class="tab-text">Convert from PDF</span>
        </button>
        <button
            type="button"
            class="tab-btn"
            class:active={activeTab === 'to-pdf'}
            onclick={() => { activeTab = 'to-pdf'; }}
        >
            <span class="tab-icon">📥</span>
            <span class="tab-text">Convert to PDF</span>
        </button>
    </div>

    <div class="tab-content">
        {#if activeTab === 'from-pdf'}
            <ConvertFromPdfView />
        {:else}
            <ConvertToPdfView />
        {/if}
    </div>
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


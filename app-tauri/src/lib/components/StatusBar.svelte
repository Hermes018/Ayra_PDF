<script lang="ts">
</script>

<footer class="status-bar" role="status" aria-label="System status">
    <div class="status-left">
        <span class="icon" aria-hidden="true">🟢</span> <span>Ready</span>
    </div>
    <div class="status-right" aria-live="polite">
        <span>0 active jobs</span>
    </div>
</footer>

<style>
    .status-bar {
        height: 32px;
        background-color: var(--surface-bg, #11111b);
        color: var(--text-muted, #a6adc8);
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 0 1rem;
        font-size: 0.8rem;
        border-top: 1px solid var(--border-color, #313244);
        transition: background-color 0.2s ease, border-color 0.2s ease, color 0.2s ease;
    }

    .status-left, .status-right {
        display: flex;
        align-items: center;
        gap: 0.5rem;
    }
</style>

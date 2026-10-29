<script lang="ts">
</script>

<footer class="status-bar" role="status" aria-label="System status">
    <div class="status-left">
        <span class="status-dot" aria-hidden="true"></span> <span>Ready</span>
    </div>
    <div class="status-right" aria-live="polite">
        <span>0 active jobs</span>
    </div>
</footer>

<style>
    .status-bar {
        height: var(--statusbar-height, 28px);
        background: var(--glass-bg-sidebar);
        -webkit-backdrop-filter: var(--glass-blur);
        backdrop-filter: var(--glass-blur);
        color: var(--text-secondary);
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 0 1rem;
        font-size: 11px;
        border-top: 1px solid var(--glass-border-subtle);
        transition: background-color var(--dur-med, 0.2s) ease, border-color var(--dur-med, 0.2s) ease, color var(--dur-med, 0.2s) ease;
        box-sizing: border-box;
    }

    .status-left, .status-right {
        display: flex;
        align-items: center;
        gap: 0.5rem;
    }

    .status-dot {
        width: 7px;
        height: 7px;
        border-radius: 50%;
        background-color: var(--semantic-success);
        box-shadow: 0 0 6px var(--semantic-success);
    }
</style>

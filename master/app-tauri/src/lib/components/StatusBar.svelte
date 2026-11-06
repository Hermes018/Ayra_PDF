<script lang="ts">
</script>

<footer class="status-bar" role="status" aria-label="System status">
    <div class="status-left">
        <span class="status-dot" aria-hidden="true"></span> <span>Ready</span>
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
        padding: 0 var(--space-4);
        font-size: var(--font-size-caption);
        border-top: 1px solid var(--glass-border-subtle);
        transition: background-color var(--dur-med, 0.2s) ease, border-color var(--dur-med, 0.2s) ease, color var(--dur-med, 0.2s) ease;
        box-sizing: border-box;
    }

    .status-left {
        display: flex;
        align-items: center;
        gap: var(--space-2);
    }

    .status-dot {
        width: 7px;
        height: 7px;
        border-radius: 50%;
        background-color: var(--semantic-success);
        box-shadow: 0 0 6px var(--semantic-success);
    }
</style>

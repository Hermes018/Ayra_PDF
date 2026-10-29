<script lang="ts">
    import { fly, fade } from 'svelte/transition';
    import { backOut } from 'svelte/easing';
    import { flip } from 'svelte/animate';
    import { appState } from '$lib/state.svelte';

    const ICON: Record<string, string> = {
        success: '✓',
        error: '!',
        warning: '⚠',
        info: 'ℹ'
    };
</script>

<section class="toast-stack" aria-live="polite" aria-label="Notifications">
    {#each appState.toasts as t (t.id)}
        <div
            class="toast toast--{t.kind}"
            role={t.kind === 'error' ? 'alert' : 'status'}
            animate:flip={{ duration: 220 }}
            in:fly={{ x: 40, y: -8, duration: 420, easing: backOut }}
            out:fade={{ duration: 160 }}
            onmouseenter={() => appState.pauseToast(t.id)}
            onmouseleave={() => appState.resumeToast(t.id)}
        >
            <span class="toast__badge" aria-hidden="true">{ICON[t.kind] || 'ℹ'}</span>
            <div class="toast__body">
                <div class="toast__title">{t.message}</div>
                {#if t.detail}
                    <div class="toast__detail" title={t.detail}>{t.detail}</div>
                {/if}
            </div>
            <div class="toast__actions">
                <button
                    class="toast__btn"
                    aria-label="Clear notification"
                    onclick={() => appState.dismissToast(t.id)}
                >
                    Clear
                </button>
            </div>
        </div>
    {/each}
</section>

<style>
    .toast-stack {
        position: fixed;
        top: 16px;
        right: 16px;
        z-index: 9999;
        display: flex;
        flex-direction: column;
        gap: 10px;
        width: min(380px, calc(100vw - 32px));
        pointer-events: none;
    }

    .toast {
        pointer-events: auto;
        display: grid;
        grid-template-columns: auto 1fr;
        grid-template-areas: "badge body" "badge actions";
        column-gap: 12px;
        row-gap: 8px;
        padding: 12px 14px;
        border-radius: var(--radius-xl, 20px);
        background: var(--glass-bg-elevated, rgba(58, 58, 68, 0.85));
        -webkit-backdrop-filter: var(--glass-blur-toast, blur(20px));
        backdrop-filter: var(--glass-blur-toast, blur(20px));
        border: 1px solid var(--glass-border-subtle, rgba(255, 255, 255, 0.1));
        box-shadow: inset 0 1px 0 0 var(--glass-border-specular, rgba(255, 255, 255, 0.2)), var(--glass-shadow-lg, 0 12px 40px rgba(0, 0, 0, 0.4));
        color: var(--text-primary);
        min-width: 0;
    }

    .toast__badge {
        grid-area: badge;
        width: 28px;
        height: 28px;
        border-radius: 50%;
        display: grid;
        place-items: center;
        font-weight: 700;
        font-size: 14px;
        color: #fff;
        box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.35);
    }

    .toast--success .toast__badge { background: var(--semantic-success, #34C759); }
    .toast--error   .toast__badge { background: var(--semantic-danger, #FF3B30); }
    .toast--warning .toast__badge { background: var(--semantic-warning, #FF9F0A); }
    .toast--info    .toast__badge { background: var(--accent-primary, #0A84FF); }

    .toast__body {
        grid-area: body;
        min-width: 0;
    }

    .toast__title {
        font-weight: 600;
        font-size: 13px;
        line-height: 1.3;
    }

    .toast__detail {
        margin-top: 3px;
        font-family: var(--font-mono, monospace);
        font-size: 11.5px;
        color: var(--text-secondary, rgba(235, 235, 245, 0.7));
        overflow-wrap: anywhere;
        display: -webkit-box;
        -webkit-line-clamp: 3;
        line-clamp: 3;
        -webkit-box-orient: vertical;
        overflow: hidden;
    }

    .toast__actions {
        grid-area: actions;
        display: flex;
        gap: 6px;
        justify-content: flex-end;
    }

    .toast__btn {
        font: inherit;
        font-size: 12px;
        font-weight: 500;
        padding: 3px 10px;
        border-radius: var(--radius-pill, 999px);
        border: 1px solid var(--glass-border-subtle, rgba(255, 255, 255, 0.15));
        background: var(--glass-bg-input, rgba(118, 118, 128, 0.24));
        color: var(--text-primary);
        cursor: pointer;
        transition: background-color 0.15s ease;
    }

    .toast__btn:hover {
        background: var(--glass-bg-hover, rgba(255, 255, 255, 0.15));
    }
</style>

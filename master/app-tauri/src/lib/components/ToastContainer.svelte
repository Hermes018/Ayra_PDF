<script lang="ts">
    import { fly, fade } from 'svelte/transition';
    import { backOut } from 'svelte/easing';
    import { flip } from 'svelte/animate';
    import { appState } from '$lib/state.svelte';
    import Button from '$lib/components/ui/Button.svelte';

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
                <Button variant="plain" size="sm" onclick={() => appState.dismissToast(t.id)}>Clear</Button>
            </div>
        </div>
    {/each}
</section>

<style>
    .toast-stack {
        position: fixed;
        top: var(--space-4);
        right: var(--space-4);
        z-index: var(--z-toast);
        display: flex;
        flex-direction: column;
        gap: var(--space-2);
        width: min(380px, calc(100vw - var(--space-8)));
        pointer-events: none;
    }

    .toast {
        pointer-events: auto;
        display: grid;
        grid-template-columns: auto 1fr;
        grid-template-areas: "badge body" "badge actions";
        column-gap: var(--space-3);
        row-gap: var(--space-2);
        padding: var(--space-3);
        border-radius: var(--radius-xl);
        background: var(--glass-bg-elevated);
        -webkit-backdrop-filter: var(--glass-blur-toast);
        backdrop-filter: var(--glass-blur-toast);
        border: 1px solid var(--glass-border-subtle);
        box-shadow: inset 0 1px 0 0 var(--glass-border-specular), var(--glass-shadow-lg);
        color: var(--text-primary);
        min-width: 0;
    }

    .toast__badge {
        grid-area: badge;
        width: var(--control-height-md);
        height: var(--control-height-md);
        border-radius: var(--radius-pill);
        display: grid;
        place-items: center;
        font-weight: 700;
        font-size: var(--font-size-body);
        color: var(--text-on-accent);
        box-shadow: inset 0 1px 0 var(--glass-border-specular);
    }

    .toast--success .toast__badge { background: var(--semantic-success); }
    .toast--error   .toast__badge { background: var(--semantic-danger); }
    .toast--warning .toast__badge { background: var(--semantic-warning); }
    .toast--info    .toast__badge { background: var(--accent-primary); }

    .toast__body {
        grid-area: body;
        min-width: 0;
    }

    .toast__title {
        font-weight: 600;
        font-size: var(--font-size-body);
        line-height: 1.3;
    }

    .toast__detail {
        margin-top: var(--space-1);
        font-family: var(--font-mono);
        font-size: var(--font-size-caption);
        color: var(--text-secondary);
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
        gap: var(--space-2);
        justify-content: flex-end;
    }
</style>

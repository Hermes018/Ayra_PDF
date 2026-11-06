<script lang="ts">
    import { faqStore, SUPPORTED_LANGUAGES, type SupportedFaqLang } from '$lib/i18n/faqStore.svelte';
    import For from '$lib/components/For.svelte';
    import Dialog from '$lib/components/ui/Dialog.svelte';
    import Button from '$lib/components/ui/Button.svelte';
    import Input from '$lib/components/ui/Input.svelte';
    import Select from '$lib/components/ui/Select.svelte';
    import EmptyState from '$lib/components/ui/EmptyState.svelte';
    import Progress from '$lib/components/ui/Progress.svelte';

    let expandedId = $state<string | null>(null);

    function toggleExpand(id: string) {
        expandedId = expandedId === id ? null : id;
    }

    // The language Select binds a local string; a change loads that catalog.
    const LANG_OPTIONS = SUPPORTED_LANGUAGES.map((l) => ({ value: l.code, label: l.label }));
    let lang = $state<string>(faqStore.currentLang);
    $effect(() => {
        if (lang !== faqStore.currentLang) faqStore.loadLanguage(lang as SupportedFaqLang);
    });
    // Escape, focus trap and focus return come from ui/Dialog (Escape goes through commandEngine).
</script>

<Dialog
    bind:open={faqStore.isOpen}
    title={faqStore.catalog?.title || 'Frequently asked questions'}
    onclose={() => faqStore.close()}
    class="faq-dialog"
>
    <div class="faq-controls">
        <Select label="Language" bind:value={lang} options={LANG_OPTIONS} />
        <Input
            type="search"
            label="Search"
            placeholder={faqStore.catalog?.searchPlaceholder || 'Search features...'}
            bind:value={faqStore.searchQuery}
        />
    </div>

    <div class="faq-body">
        {#if faqStore.isLoading}
            <Progress value={null} label="Loading help topics" />
        {:else if faqStore.filteredItems.length === 0}
            <EmptyState
                title="No matching topics"
                body={faqStore.catalog?.emptySearch || 'No matching features found.'}
            />
        {:else}
            <div class="faq-list">
                <For items={faqStore.filteredItems}>
                    {#snippet children(item)}
                        <article class="faq-item">
                            <Button
                                variant="plain"
                                block
                                expanded={expandedId === item.id}
                                onclick={() => toggleExpand(item.id)}
                            >
                                <span class="feature-tag">{item.feature}</span>
                                <span class="question-text">{item.question}</span>
                            </Button>
                            {#if expandedId === item.id}
                                <div class="faq-details">
                                    <p class="answer-text">{item.answer}</p>
                                    <div class="detail-section">
                                        <div class="detail-block">
                                            <span class="detail-label">Usage:</span>
                                            <span class="detail-value">{item.usage}</span>
                                        </div>
                                        <div class="detail-block">
                                            <span class="detail-label">Outputs:</span>
                                            <span class="detail-value">{item.outputs}</span>
                                        </div>
                                        <div class="detail-block">
                                            <span class="detail-label">Offline boundaries:</span>
                                            <span class="detail-value">{item.offlineBoundaries}</span>
                                        </div>
                                    </div>
                                </div>
                            {/if}
                        </article>
                    {/snippet}
                </For>
            </div>
        {/if}
    </div>

    {#snippet actions()}
        <span class="offline-note">100% local and offline. No network activity.</span>
        <Button variant="primary" onclick={() => faqStore.close()}>Close</Button>
    {/snippet}
</Dialog>

<style>
    :global(.lp-dialog.faq-dialog) {
        width: min(840px, 100%);
        max-height: 86vh;
    }

    .faq-controls {
        display: grid;
        grid-template-columns: 1fr 2fr;
        gap: var(--space-3);
        margin-bottom: var(--space-3);
    }

    .faq-list {
        display: flex;
        flex-direction: column;
        gap: var(--space-2);
    }

    .faq-item {
        border: 1px solid var(--glass-border-subtle);
        border-radius: var(--radius-md);
        background: var(--glass-bg-surface);
    }

    .feature-tag {
        font-size: var(--font-size-caption);
        font-weight: 600;
        color: var(--accent-primary);
        background: var(--glass-bg-active);
        border-radius: var(--radius-pill);
        padding: var(--space-1) var(--space-2);
    }

    .question-text {
        font-size: var(--font-size-body);
        text-align: left;
        white-space: normal;
    }

    .faq-details {
        padding: 0 var(--space-4) var(--space-4);
        display: flex;
        flex-direction: column;
        gap: var(--space-3);
    }

    .answer-text {
        margin: 0;
        font-size: var(--font-size-body);
        color: var(--text-secondary);
        line-height: 1.5;
    }

    .detail-section {
        display: flex;
        flex-direction: column;
        gap: var(--space-2);
        padding: var(--space-3);
        border-radius: var(--radius-sm);
        background: var(--glass-bg-input);
    }

    .detail-block {
        display: flex;
        gap: var(--space-2);
        font-size: var(--font-size-footnote);
    }

    .detail-label {
        font-weight: 600;
        color: var(--text-secondary);
        white-space: nowrap;
    }

    .detail-value {
        color: var(--text-primary);
    }

    .offline-note {
        margin-right: auto;
        align-self: center;
        font-size: var(--font-size-footnote);
        color: var(--text-secondary);
    }
</style>

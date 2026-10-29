<script lang="ts">
    import { faqStore, SUPPORTED_LANGUAGES, type SupportedFaqLang } from '$lib/i18n/faqStore.svelte';
    import For from '$lib/components/For.svelte';

    let expandedId = $state<string | null>(null);

    function toggleExpand(id: string) {
        expandedId = expandedId === id ? null : id;
    }

    function handleKeydown(event: KeyboardEvent) {
        if (event.key === 'Escape' && faqStore.isOpen) {
            faqStore.close();
        }
    }

    function onBackdropClick(event: MouseEvent) {
        if (event.target === event.currentTarget) {
            faqStore.close();
        }
    }

    function onLanguageChange(event: Event) {
        const target = event.target as HTMLSelectElement;
        faqStore.loadLanguage(target.value as SupportedFaqLang);
    }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if faqStore.isOpen}
    <div
        class="faq-backdrop"
        onclick={onBackdropClick}
        onkeydown={(e) => { if (e.key === 'Enter') onBackdropClick(e as unknown as MouseEvent); }}
        role="presentation"
    >
        <div
            class="faq-card"
            role="dialog"
            aria-modal="true"
            aria-labelledby="faq-modal-title"
        >
            <header class="faq-header">
                <div class="header-main">
                    <div class="title-group">
                        <span class="header-icon" aria-hidden="true">📖</span>
                        <h2 id="faq-modal-title">{faqStore.catalog?.title || 'Frequently Asked Questions'}</h2>
                    </div>
                    <div class="header-actions">
                        <div class="lang-picker-wrapper">
                            <label for="faq-lang-select" class="sr-only">Language</label>
                            <select
                                id="faq-lang-select"
                                class="lang-select"
                                value={faqStore.currentLang}
                                onchange={onLanguageChange}
                            >
                                <For items={SUPPORTED_LANGUAGES}>
                                    {#snippet children(lang)}
                                        <option value={lang.code}>{lang.label}</option>
                                    {/snippet}
                                </For>
                            </select>
                        </div>
                        <button
                            type="button"
                            class="close-btn"
                            onclick={() => faqStore.close()}
                            aria-label="Close FAQ dialog"
                        >
                            ✕
                        </button>
                    </div>
                </div>

                <div class="search-bar-row">
                    <input
                        type="search"
                        class="search-input"
                        placeholder={faqStore.catalog?.searchPlaceholder || 'Search features...'}
                        bind:value={faqStore.searchQuery}
                    />
                </div>
            </header>

            <div class="faq-body">
                {#if faqStore.isLoading}
                    <div class="loading-state">
                        <div class="spinner" aria-hidden="true"></div>
                        <p>Loading catalog...</p>
                    </div>
                {:else if faqStore.filteredItems.length === 0}
                    <div class="empty-state">
                        <p>{faqStore.catalog?.emptySearch || 'No matching features found.'}</p>
                    </div>
                {:else}
                    <div class="faq-list">
                        <For items={faqStore.filteredItems}>
                            {#snippet children(item)}
                                <article class="faq-item" class:expanded={expandedId === item.id}>
                                    <button
                                        type="button"
                                        class="faq-summary-btn"
                                        onclick={() => toggleExpand(item.id)}
                                        aria-expanded={expandedId === item.id}
                                    >
                                        <div class="summary-content">
                                            <span class="feature-tag">{item.feature}</span>
                                            <span class="question-text">{item.question}</span>
                                        </div>
                                        <span class="chevron" aria-hidden="true">
                                            {expandedId === item.id ? '▲' : '▼'}
                                        </span>
                                    </button>

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
                                                <div class="detail-block highlight">
                                                    <span class="detail-label">Offline Boundaries:</span>
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

            <footer class="faq-footer">
                <span class="offline-badge">
                    <span class="badge-dot" aria-hidden="true"></span>
                    100% Local & Offline • Zero Network Activity
                </span>
                <button
                    type="button"
                    class="done-btn"
                    onclick={() => faqStore.close()}
                >
                    Close
                </button>
            </footer>
        </div>
    </div>
{/if}

<style>
    .faq-backdrop {
        position: fixed;
        inset: 0;
        background-color: rgba(0, 0, 0, 0.48);
        backdrop-filter: blur(12px);
        -webkit-backdrop-filter: blur(12px);
        z-index: 9999;
        display: flex;
        align-items: center;
        justify-content: center;
        padding: 1.5rem;
    }

    .faq-card {
        background-color: var(--glass-bg-elevated);
        backdrop-filter: blur(28px) saturate(190%);
        -webkit-backdrop-filter: blur(28px) saturate(190%);
        border: 1px solid var(--glass-border-specular);
        border-radius: var(--radius-xl, 20px);
        box-shadow: var(--glass-shadow-lg);
        width: 100%;
        max-width: 840px;
        max-height: 86vh;
        display: flex;
        flex-direction: column;
        overflow: hidden;
        color: var(--text-primary);
        animation: fadeIn 0.2s cubic-bezier(0.16, 1, 0.3, 1);
    }

    @keyframes fadeIn {
        from {
            opacity: 0;
            transform: scale(0.97) translateY(8px);
        }
        to {
            opacity: 1;
            transform: scale(1) translateY(0);
        }
    }

    .faq-header {
        padding: 1.5rem 1.75rem 1rem 1.75rem;
        border-bottom: 1px solid var(--glass-border-subtle);
        display: flex;
        flex-direction: column;
        gap: 1rem;
    }

    .header-main {
        display: flex;
        justify-content: space-between;
        align-items: center;
        gap: 1rem;
    }

    .title-group {
        display: flex;
        align-items: center;
        gap: 0.75rem;
    }

    .header-icon {
        font-size: 1.5rem;
    }

    h2 {
        font-size: 1.35rem;
        font-weight: 700;
        margin: 0;
        color: var(--text-primary);
        letter-spacing: -0.01em;
    }

    .header-actions {
        display: flex;
        align-items: center;
        gap: 0.75rem;
    }

    .lang-select {
        background-color: var(--glass-bg-input);
        color: var(--text-primary);
        border: 1px solid var(--glass-border-subtle);
        border-radius: var(--radius-md, 10px);
        padding: 0.4rem 0.8rem;
        font-size: 0.88rem;
        font-weight: 500;
        cursor: pointer;
        outline: none;
        transition: border-color 0.2s ease;
    }

    .lang-select:focus {
        border-color: var(--accent-primary);
    }

    .lang-select option {
        background-color: var(--option-bg);
        color: var(--option-fg);
    }

    .close-btn {
        background: transparent;
        border: none;
        color: var(--text-secondary);
        font-size: 1.1rem;
        width: 32px;
        height: 32px;
        border-radius: 50%;
        display: flex;
        align-items: center;
        justify-content: center;
        cursor: pointer;
        transition: all 0.15s ease;
    }

    .close-btn:hover {
        background-color: var(--glass-bg-hover);
        color: var(--text-primary);
    }

    .search-input {
        width: 100%;
        background-color: var(--glass-bg-input);
        color: var(--text-primary);
        border: 1px solid var(--glass-border-subtle);
        border-radius: var(--radius-md, 10px);
        padding: 0.65rem 1rem;
        font-size: 0.92rem;
        outline: none;
        box-sizing: border-box;
        transition: border-color 0.2s ease;
    }

    .search-input:focus {
        border-color: var(--accent-primary);
    }

    .faq-body {
        flex: 1;
        overflow-y: auto;
        padding: 1.25rem 1.75rem;
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
    }

    .faq-list {
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
    }

    .faq-item {
        background-color: var(--glass-bg-surface);
        border: 1px solid var(--glass-border-subtle);
        border-radius: var(--radius-lg, 14px);
        overflow: hidden;
        transition: all 0.2s ease;
    }

    .faq-item:hover {
        border-color: var(--glass-border-specular);
    }

    .faq-item.expanded {
        border-color: var(--accent-primary);
        background-color: var(--glass-bg-surface);
        box-shadow: var(--glass-shadow-sm);
    }

    .faq-summary-btn {
        width: 100%;
        background: transparent;
        border: none;
        padding: 1rem 1.25rem;
        text-align: left;
        cursor: pointer;
        display: flex;
        justify-content: space-between;
        align-items: center;
        gap: 1rem;
        color: inherit;
    }

    .summary-content {
        display: flex;
        align-items: center;
        gap: 0.85rem;
        flex: 1;
    }

    .feature-tag {
        font-size: 0.75rem;
        font-weight: 700;
        text-transform: uppercase;
        letter-spacing: 0.04em;
        background-color: var(--glass-bg-input);
        color: var(--accent-primary);
        padding: 0.25rem 0.6rem;
        border-radius: var(--radius-xs, 6px);
        white-space: nowrap;
    }

    .question-text {
        font-size: 0.95rem;
        font-weight: 600;
        color: var(--text-primary);
        line-height: 1.35;
    }

    .chevron {
        font-size: 0.75rem;
        color: var(--text-tertiary);
        transition: transform 0.2s ease;
    }

    .faq-details {
        padding: 0 1.25rem 1.25rem 1.25rem;
        display: flex;
        flex-direction: column;
        gap: 1rem;
        border-top: 1px solid var(--glass-border-subtle);
        animation: slideDown 0.18s ease;
    }

    @keyframes slideDown {
        from {
            opacity: 0;
            transform: translateY(-4px);
        }
        to {
            opacity: 1;
            transform: translateY(0);
        }
    }

    .answer-text {
        font-size: 0.92rem;
        line-height: 1.5;
        color: var(--text-secondary);
        margin: 1rem 0 0 0;
    }

    .detail-section {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
        background-color: var(--glass-bg-input);
        padding: 0.85rem 1rem;
        border-radius: var(--radius-md, 10px);
        border: 1px solid var(--glass-border-subtle);
    }

    .detail-block {
        display: flex;
        flex-direction: column;
        gap: 0.2rem;
        font-size: 0.84rem;
    }

    .detail-block.highlight {
        margin-top: 0.25rem;
        padding-top: 0.45rem;
        border-top: 1px dashed var(--glass-border-subtle);
    }

    .detail-label {
        font-weight: 700;
        color: var(--text-primary);
        font-size: 0.78rem;
        text-transform: uppercase;
        letter-spacing: 0.03em;
    }

    .detail-block.highlight .detail-label {
        color: var(--semantic-success);
    }

    .detail-value {
        color: var(--text-secondary);
        line-height: 1.4;
    }

    .faq-footer {
        padding: 1rem 1.75rem;
        border-top: 1px solid var(--glass-border-subtle);
        display: flex;
        justify-content: space-between;
        align-items: center;
        gap: 1rem;
        background-color: var(--glass-bg-surface);
    }

    .offline-badge {
        display: inline-flex;
        align-items: center;
        gap: 0.5rem;
        font-size: 0.82rem;
        font-weight: 500;
        color: var(--semantic-success);
    }

    .badge-dot {
        width: 8px;
        height: 8px;
        border-radius: 50%;
        background-color: var(--semantic-success);
    }

    .done-btn {
        background-color: var(--accent-primary);
        color: var(--text-on-accent);
        border: none;
        border-radius: var(--radius-md, 10px);
        padding: 0.5rem 1.25rem;
        font-size: 0.88rem;
        font-weight: 600;
        cursor: pointer;
        transition: opacity 0.15s ease;
    }

    .done-btn:hover {
        opacity: 0.9;
    }

    .loading-state,
    .empty-state {
        padding: 3rem 1rem;
        text-align: center;
        color: var(--text-secondary);
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 0.75rem;
    }

    .spinner {
        width: 24px;
        height: 24px;
        border: 2px solid var(--text-tertiary);
        border-top-color: var(--accent-primary);
        border-radius: 50%;
        animation: spin 0.8s linear infinite;
    }

    @keyframes spin {
        to {
            transform: rotate(360deg);
        }
    }

    .sr-only {
        position: absolute;
        width: 1px;
        height: 1px;
        padding: 0;
        margin: -1px;
        overflow: hidden;
        clip: rect(0, 0, 0, 0);
        white-space: nowrap;
        border: 0;
    }
</style>
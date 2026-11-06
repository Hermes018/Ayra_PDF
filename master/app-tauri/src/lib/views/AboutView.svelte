<script lang="ts">
    import { onMount } from 'svelte';
    import { IpcClient, type AboutInfo } from '$lib/ipc';
    import Button from '$lib/components/ui/Button.svelte';
    import ErrorState from '$lib/components/ui/ErrorState.svelte';

    let info = $state<AboutInfo | null>(null);
    let infoFailed = $state(false);

    // The licence text is large, so it is only fetched when the user asks for it.
    let licencesShown = $state(false);
    let licenceText = $state<string | null>(null);
    let licenceFailed = $state(false);
    let licenceLoading = $state(false);

    async function loadInfo() {
        infoFailed = false;
        try {
            info = await IpcClient.getAboutInfo();
        } catch (e) {
            console.error('Failed to read the app details:', e);
            infoFailed = true;
        }
    }

    async function loadLicences() {
        licencesShown = true;
        licenceFailed = false;
        licenceLoading = true;
        try {
            licenceText = await IpcClient.getThirdPartyNotices();
        } catch (e) {
            console.error('Failed to load the licence notices:', e);
            licenceText = null;
            licenceFailed = true;
        } finally {
            licenceLoading = false;
        }
    }

    onMount(loadInfo);
</script>

<div class="about-view">
    <div class="header">
        <h2>About LocalPDF Studio</h2>
        <p class="subtitle">Version, where your files live, and open-source licences</p>
    </div>

    {#if infoFailed}
        <ErrorState what="Couldn't read the app details." next="Try again." retry={loadInfo} />
    {/if}

    <div class="about-grid">
        <section class="card" aria-labelledby="about-version-title">
            <h3 id="about-version-title">This build</h3>
            <dl class="facts">
                <dt>Version</dt>
                <dd data-testid="about-version">{info ? info.version : '…'}</dd>
                <dt>Build</dt>
                <dd data-testid="about-build"><code>{info ? info.build_hash : '…'}</code></dd>
            </dl>
        </section>

        <section class="card" aria-labelledby="about-offline-title">
            <h3 id="about-offline-title">Privacy</h3>
            <p class="statement">100% offline: the app makes no network connections. Your PDFs never leave this computer.</p>
        </section>

        <section class="card" aria-labelledby="about-where-title">
            <h3 id="about-where-title">Where things live</h3>
            <dl class="facts">
                <dt>Settings file</dt>
                <dd><code>{info ? info.settings_file : '…'}</code></dd>
                <dt>Studio preferences</dt>
                <dd><code>{info ? info.studio_prefs_file : '…'}</code></dd>
                <dt>Markup library</dt>
                <dd><code>{info ? info.markup_library_file : '…'}</code></dd>
                <dt>Log folder</dt>
                <dd><code>{info ? info.log_dir : '…'}</code></dd>
            </dl>
            <p class="note">Logs stay on this computer. Nothing is uploaded.</p>
        </section>
    </div>

    <section class="card" aria-labelledby="about-licences-title">
        <h3 id="about-licences-title">Open-source licences</h3>
        <p class="note">LocalPDF Studio is built on open-source software. The full notices are bundled with the app.</p>
        {#if !licencesShown}
            <div>
                <Button variant="secondary" onclick={loadLicences}>Show open-source licences</Button>
            </div>
        {:else if licenceFailed}
            <ErrorState what="Couldn't load the licence notices." next="Try again." retry={loadLicences} />
        {:else if licenceLoading}
            <p class="note" role="status">Loading the licence notices…</p>
        {:else if licenceText !== null}
            <div class="licence-region" role="region" aria-label="Licence notices" tabindex="0">
                <pre>{licenceText}</pre>
            </div>
        {/if}
    </section>
</div>

<style>
    .about-view {
        display: flex;
        flex-direction: column;
        gap: var(--space-4);
        padding: var(--space-4);
        color: var(--text-primary);
    }

    .header h2 {
        margin: 0;
        font-size: var(--font-size-title1);
        color: var(--accent-primary);
    }

    .subtitle,
    .note {
        margin: var(--space-1) 0 0;
        color: var(--text-secondary);
        font-size: var(--font-size-footnote);
    }

    .about-grid {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
        align-items: start;
        gap: var(--space-4);
    }

    .card {
        background: var(--glass-bg-surface);
        border: 1px solid var(--glass-border-subtle);
        border-radius: var(--radius-lg);
        padding: var(--space-4);
        display: flex;
        flex-direction: column;
        gap: var(--space-3);
        min-width: 0;
    }

    .card h3 {
        margin: 0;
        font-size: var(--font-size-title3);
    }

    .statement {
        margin: 0;
        font-size: var(--font-size-body);
    }

    .facts {
        display: grid;
        grid-template-columns: max-content 1fr;
        gap: var(--space-2) var(--space-4);
        margin: 0;
    }

    .facts dt {
        color: var(--text-secondary);
        font-size: var(--font-size-footnote);
    }

    .facts dd {
        margin: 0;
        min-width: 0;
    }

    code {
        font-family: var(--font-mono);
        font-size: var(--font-size-caption);
        word-break: break-all;
    }

    .licence-region {
        max-height: 24rem;
        overflow: auto;
        padding: var(--space-3);
        border-radius: var(--radius-md);
        background: var(--glass-bg-input);
    }

    .licence-region pre {
        margin: 0;
        white-space: pre-wrap;
        word-break: break-word;
        font-family: var(--font-mono);
        font-size: var(--font-size-caption);
    }
</style>

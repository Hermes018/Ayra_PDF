<script lang="ts">
    import StudioIcon from '$lib/components/studio/StudioIcon.svelte';
    import { onMount, onDestroy } from 'svelte';
    import { IpcClient, pickFilesDialog } from '$lib/ipc';
    import For from '$lib/components/For.svelte';
    import Button from '$lib/components/ui/Button.svelte';
    import EmptyState from '$lib/components/ui/EmptyState.svelte';
    import { friendlyJobError } from '$lib/core/jobError';
    import { SAMPLE_BATCH_ID, SAMPLE_BATCH_LABS, LABS_BADGE, labsVisible } from '$lib/components/studio/labs.svelte';
    import { appState } from '$lib/state.svelte';
    import { freeOutputPath, siblingOutputPath } from '$lib/views/studio/outputNames';
    import type { JobRecord } from '$lib/bindings/JobRecord';
    import type { JobEvent } from '$lib/bindings/JobEvent';
    import type { JobPriority } from '$lib/bindings/JobPriority';
    import type { JobRequest } from '$lib/bindings/JobRequest';
    import type { UnlistenFn } from '@tauri-apps/api/event';

    // Svelte 5 runes for queue state
    let jobs = $state<JobRecord[]>([]);
    let currentFilter = $state<'All' | 'Active' | 'Completed' | 'Failed'>('All');
    let loading = $state(false);
    let actionMessage = $state<string | null>(null);
    let unlisten: UnlistenFn | null = null;

    // Derived counts
    let activeJobs = $derived(jobs.filter(j => j.status === 'Running' || j.status === 'Queued'));
    let runningCount = $derived(jobs.filter(j => j.status === 'Running').length);
    let queuedCount = $derived(jobs.filter(j => j.status === 'Queued').length);
    let completedCount = $derived(jobs.filter(j => j.status === 'Completed').length);
    let failedCount = $derived(jobs.filter(j => j.status === 'Failed').length);
    let totalCount = $derived(jobs.length);

    // Filtered jobs list
    let filteredJobs = $derived(
        jobs.filter(job => {
            if (currentFilter === 'All') return true;
            if (currentFilter === 'Active') return job.status === 'Running' || job.status === 'Queued';
            if (currentFilter === 'Completed') return job.status === 'Completed';
            if (currentFilter === 'Failed') return job.status === 'Failed' || job.status === 'Cancelled';
            return true;
        })
    );

    async function loadQueue() {
        try {
            loading = true;
            jobs = await IpcClient.listJobs();
        } catch (e) {
            console.error('Failed to load queue:', e);
        } finally {
            loading = false;
        }
    }

    onMount(() => {
        loadQueue();

        IpcClient.onJobEvent((event: JobEvent) => {
            handleJobEvent(event);
        }).then(fn => {
            unlisten = fn;
        }).catch(err => {
            console.error('Failed to register job event listener:', err);
        });
    });

    onDestroy(() => {
        if (unlisten) {
            unlisten();
        }
    });

    function handleJobEvent(event: JobEvent) {
        if ('kind' in event) {
            const kind = (event as any).kind;
            const data = (event as any).data;

            if (kind === 'Progress') {
                const target = jobs.find(j => j.job_id === data.job_id);
                if (target) {
                    target.progress = data.percent;
                    target.status = 'Running';
                }
                return;
            }

            if (kind === 'Completed') {
                const target = jobs.find(j => j.job_id === data.job_id);
                if (target) {
                    target.status = 'Completed';
                    target.progress = 100;
                    if (data.output_path) target.output_path = data.output_path;
                } else {
                    loadQueue();
                }
                return;
            }

            if (kind === 'Failed') {
                const target = jobs.find(j => j.job_id === data.job_id);
                if (target) {
                    target.status = 'Failed';
                    target.error_code = data.error_code;
                    target.error_message = data.error_message;
                } else {
                    loadQueue();
                }
                return;
            }

            if (kind === 'Cancelled') {
                const target = jobs.find(j => j.job_id === data.job_id);
                if (target) {
                    target.status = 'Cancelled';
                } else {
                    loadQueue();
                }
                return;
            }

            if (kind === 'Started') {
                const target = jobs.find(j => j.job_id === data.job_id);
                if (target) {
                    target.status = 'Running';
                } else {
                    loadQueue();
                }
                return;
            }

            if (kind === 'Queued') {
                loadQueue();
                return;
            }
        }
        // Fallback refresh
        loadQueue();
    }

    async function handleCancelJob(jobId: string) {
        try {
            await IpcClient.cancelJob(jobId);
            actionMessage = `Job ${jobId.slice(0, 8)} cancelled.`;
            await loadQueue();
        } catch (e) {
            console.error('Failed to cancel job:', e);
            actionMessage = "Couldn't cancel the job. Refresh the queue and try again.";
        }
    }

    async function handleRetryJob(jobId: string) {
        try {
            const newId = await IpcClient.retryJob(jobId);
            actionMessage = `Job re-enqueued as ${newId.slice(0, 8)}.`;
            await loadQueue();
        } catch (e) {
            console.error('Failed to retry job:', e);
            actionMessage = "Couldn't retry the job. Refresh the queue and try again.";
        }
    }

    async function handleClearFinished() {
        try {
            const count = await IpcClient.clearFinishedJobs();
            actionMessage = `Cleared ${count} finished jobs.`;
            await loadQueue();
        } catch (e) {
            console.error('Failed to clear finished jobs:', e);
            actionMessage = "Couldn't clear the finished jobs. Try again.";
        }
    }

    async function handleCancelAllActive() {
        const active = activeJobs;
        if (active.length === 0) return;
        try {
            for (const j of active) {
                await IpcClient.cancelJob(j.job_id);
            }
            actionMessage = `Cancelled ${active.length} active jobs.`;
            await loadQueue();
        } catch (e) {
            console.error('Failed to cancel active jobs:', e);
            actionMessage = "Couldn't cancel all active jobs. Refresh the queue and try again.";
        }
    }

    async function handleAddBatchDemo() {
        const demoRequests: JobRequest[] = [
            {
                op: 'Compress',
                input_path: '/home/ayra/docs/annual_report_2025.pdf',
                output_path: '/home/ayra/docs/annual_report_2025_compressed.pdf',
                profile: 'Medium'
            },
            {
                op: 'PdfToDocx',
                input_path: '/home/ayra/docs/contracts_batch_part1.pdf',
                output_path: '/home/ayra/docs/contracts_batch_part1.docx'
            },
            {
                op: 'PdfToImages',
                input_path: '/home/ayra/docs/presentation_slides.pdf',
                output_path: '/home/ayra/docs/presentation_slides_images'
            }
        ];

        try {
            const ids = await IpcClient.submitBatch(demoRequests, 'High');
            actionMessage = `Submitted the sample batch with ${ids.length} jobs.`;
            await loadQueue();
        } catch (e) {
            console.error('Failed to dispatch the sample batch:', e);
            actionMessage = "Couldn't submit the sample batch. Try again.";
        }
    }

    async function handleAddFiles() {
        try {
            const files = await pickFilesDialog(true, ['pdf']);
            if (!files || files.length === 0) return;

            appState.addFiles(files);

            // Collision-safe outputs (never the input, never an existing file; BUG-050).
            const requests: JobRequest[] = await Promise.all(files.map(async (file): Promise<JobRequest> => ({
                op: 'Compress',
                input_path: file,
                output_path: await freeOutputPath(file, 'compressed'),
                profile: 'Medium'
            })));

            const ids = await IpcClient.submitBatch(requests, 'Normal');
            actionMessage = `Enqueued ${ids.length} batch job(s) for selected files.`;
            await loadQueue();
        } catch (e) {
            console.error('Failed to add files to batch queue:', e);
            actionMessage = "Couldn't add the files. Try again; if it keeps failing, check that the files exist.";
        }
    }

    function formatFilename(filePath: string | null): string {
        if (!filePath) return '—';
        const parts = filePath.split('/');
        return parts[parts.length - 1] || filePath;
    }

    function getPriorityClass(priority: JobPriority): string {
        switch (priority) {
            case 'Urgent': return 'badge-urgent';
            case 'High': return 'badge-high';
            case 'Normal': return 'badge-normal';
            case 'Low': return 'badge-low';
            default: return 'badge-normal';
        }
    }

    function getStatusClass(status: string): string {
        switch (status) {
            case 'Running': return 'status-running';
            case 'Queued': return 'status-queued';
            case 'Completed': return 'status-completed';
            case 'Failed': return 'status-failed';
            case 'Cancelled': return 'status-cancelled';
            default: return '';
        }
    }
</script>

<div class="batch-queue-container" data-testid="batch-queue-view">
    <!-- View Header -->
    <header class="header">
        <div class="header-title">
            <div>
                <h2>Batch queue</h2>
                <p class="subtitle">Inspect, monitor, and control multi-file background jobs</p>
            </div>
        </div>
        <div class="header-actions">
            <Button variant="secondary" disabled={loading} disabledReason="Refreshing the queue" onclick={loadQueue}>Refresh</Button>
            <Button variant="primary" data-testid="btn-add-batch-files" title="Choose PDF files to compress in the background" onclick={handleAddFiles}>Add files…</Button>
            {#if labsVisible(SAMPLE_BATCH_ID)}
                <Button variant="secondary" data-testid="btn-add-batch-demo" ariaLabel={SAMPLE_BATCH_LABS.label} title={`${SAMPLE_BATCH_LABS.label}: ${SAMPLE_BATCH_LABS.note}`} onclick={handleAddBatchDemo}>
                    {SAMPLE_BATCH_LABS.name} <span class="labs-badge">{LABS_BADGE}</span>
                </Button>
            {/if}
        </div>
    </header>

    <!-- Notification Banner -->
    {#if actionMessage}
        <div class="alert-banner">
            <span>{actionMessage}</span>
            <Button variant="plain" size="sm" ariaLabel="Dismiss this message" title="Dismiss this message" onclick={() => { actionMessage = null; }}><StudioIcon name="close" size={14} /></Button>
        </div>
    {/if}

    <!-- Summary Metrics Grid -->
    <section class="summary-grid" data-testid="summary-header">
        <div class="summary-card active">
            <div class="card-icon"><StudioIcon name="bolt" size={22} /></div>
            <div class="card-info">
                <span class="card-value">{runningCount}</span>
                <span class="card-label">Running</span>
            </div>
        </div>
        <div class="summary-card queued">
            <div class="card-icon"><StudioIcon name="clock" size={22} /></div>
            <div class="card-info">
                <span class="card-value">{queuedCount}</span>
                <span class="card-label">Queued</span>
            </div>
        </div>
        <div class="summary-card completed">
            <div class="card-icon"><StudioIcon name="check" size={22} /></div>
            <div class="card-info">
                <span class="card-value">{completedCount}</span>
                <span class="card-label">Completed</span>
            </div>
        </div>
        <div class="summary-card failed">
            <div class="card-icon"><StudioIcon name="close" size={22} /></div>
            <div class="card-info">
                <span class="card-value">{failedCount}</span>
                <span class="card-label">Failed</span>
            </div>
        </div>
    </section>

    <!-- Controls & Filter Bar -->
    <section class="controls-bar">
        <div class="filter-tabs">
            <Button variant="plain" size="sm" pressed={currentFilter === 'All'} data-testid="filter-all" onclick={() => { currentFilter = 'All'; }}>All ({totalCount})</Button>
            <Button variant="plain" size="sm" pressed={currentFilter === 'Active'} data-testid="filter-active" onclick={() => { currentFilter = 'Active'; }}>Active ({runningCount + queuedCount})</Button>
            <Button variant="plain" size="sm" pressed={currentFilter === 'Completed'} data-testid="filter-completed" onclick={() => { currentFilter = 'Completed'; }}>Completed ({completedCount})</Button>
            <Button variant="plain" size="sm" pressed={currentFilter === 'Failed'} data-testid="filter-failed" onclick={() => { currentFilter = 'Failed'; }}>Failed ({failedCount})</Button>
        </div>

        <div class="bulk-actions">
            <Button variant="destructive" size="sm" disabled={activeJobs.length === 0} disabledReason="No jobs are running or queued" data-testid="btn-cancel-all-active" onclick={handleCancelAllActive}>Cancel all active</Button>
            <Button variant="secondary" size="sm" disabled={completedCount === 0 && failedCount === 0} disabledReason="There are no finished jobs to clear" data-testid="btn-clear-finished" onclick={handleClearFinished}>Clear finished</Button>
        </div>
    </section>

    <!-- Queue List -->
    <main class="queue-list">
        {#if filteredJobs.length === 0}
            <div data-testid="empty-queue-message">
                <EmptyState
                    title={`No jobs match the ${currentFilter} filter`}
                    body="Choose PDF files to compress in the background."
                    action={{ label: 'Add files…', onclick: handleAddFiles }}
                />
            </div>
        {:else}
            <For items={filteredJobs}>
                {#snippet children(job)}
                <article class="job-card" data-testid="job-card" id={`job-${job.job_id}`}>
                    <div class="job-card-header">
                        <div class="job-title-group">
                            <span class="job-title">{job.title}</span>
                            <span class="badge {getPriorityClass(job.priority)}">{job.priority} priority</span>
                        </div>
                        <span class="status-pill {getStatusClass(job.status)}" data-testid="status-badge">
                            {job.status}
                        </span>
                    </div>

                    <div class="job-files-row">
                        <div class="file-item">
                            <span class="file-label">Input:</span>
                            <span class="file-name" title={job.input_path ?? ''}>{formatFilename(job.input_path)}</span>
                        </div>
                        {#if job.output_path}
                            <span class="arrow"><StudioIcon name="arrow.right" size={16} /></span>
                            <div class="file-item">
                                <span class="file-label">Output:</span>
                                <span class="file-name" title={job.output_path}>{formatFilename(job.output_path)}</span>
                            </div>
                        {/if}
                    </div>

                    <!-- Progress Bar -->
                    <div class="progress-container">
                        <div class="progress-bar-bg">
                            <div
                                class="progress-bar-fill {getStatusClass(job.status)}"
                                style="width: {job.progress}%"
                                role="progressbar"
                                aria-valuenow={job.progress}
                                aria-valuemin="0"
                                aria-valuemax="100"
                            ></div>
                        </div>
                        <span class="progress-label">{job.progress}%</span>
                    </div>

                    <!-- Diagnostics for Failures -->
                    {#if job.status === 'Failed' && (job.error_code || job.error_message)}
                        <div class="diagnostics-box" data-testid="diagnostics-box" role="alert" aria-live="assertive">
                            <div class="diag-header">
                                <span class="diag-badge">{job.error_code || 'ERROR'}</span>
                                <span class="diag-title">Details</span>
                            </div>
                            <p class="diag-message">{friendlyJobError(job.error_message ?? '', { what: job.error_message || 'The job failed.', next: '' }).what}</p>
                        </div>
                    {/if}

                    <!-- Actions -->
                    <div class="job-actions-row">
                        <span class="job-id-text">ID: {job.job_id.slice(0, 12)}...</span>
                        <div class="btn-group">
                            {#if job.status === 'Queued' || job.status === 'Running'}
                                <Button variant="destructive" size="sm" data-testid={`btn-cancel-${job.job_id}`} onclick={() => handleCancelJob(job.job_id)}>Cancel</Button>
                            {/if}
                            {#if job.status === 'Failed' || job.status === 'Cancelled'}
                                <Button variant="secondary" size="sm" data-testid={`btn-retry-${job.job_id}`} onclick={() => handleRetryJob(job.job_id)}>Retry job</Button>
                            {/if}
                        </div>
                    </div>
                </article>
                {/snippet}
            </For>
        {/if}
    </main>
</div>

<style>
    .batch-queue-container {
        padding: var(--space-6);
        display: flex;
        flex-direction: column;
        gap: var(--space-6);
        color: var(--text-primary);
        max-width: 1200px;
        margin: 0 auto;
        width: 100%;
    }

    .header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        flex-wrap: wrap;
        gap: var(--space-4);
    }

    .header-title {
        display: flex;
        align-items: center;
        gap: var(--space-4);
    }

    .header-title h2 {
        font-size: var(--font-size-title1);
        margin: 0;
        color: var(--accent-primary);
    }

    .subtitle {
        margin: var(--space-1) 0 0;
        font-size: var(--font-size-body);
        color: var(--text-secondary);
    }

    .header-actions {
        display: flex;
        gap: var(--space-3);
        flex-wrap: wrap;
    }

    .alert-banner {
        display: flex;
        justify-content: space-between;
        align-items: center;
        background: var(--glass-bg-surface);
        border-left: 4px solid var(--accent-primary);
        border: 1px solid var(--glass-border-subtle);
        padding: var(--space-3) var(--space-4);
        border-radius: var(--radius-sm, 8px);
        font-size: var(--font-size-body);
        color: var(--text-primary);
    }


    /* Summary Metrics Grid */
    .summary-grid {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(min(100%, 200px), 1fr));
        gap: var(--space-4);
    }

    .summary-card {
        background: var(--glass-bg-surface);
        -webkit-backdrop-filter: var(--glass-blur);
        backdrop-filter: var(--glass-blur);
        border: 1px solid var(--glass-border-subtle);
        border-radius: var(--radius-lg, 14px);
        box-shadow: inset 0 1px 0 0 var(--glass-border-specular), var(--glass-shadow-sm);
        padding: var(--space-5);
        display: flex;
        align-items: center;
        gap: var(--space-4);
        transition: transform var(--dur-fast, 120ms) ease;
    }

    .summary-card:hover {
        transform: translateY(-2px);
    }

    .summary-card.active { border-color: var(--accent-primary); }
    .summary-card.queued { border-color: var(--semantic-warning); }
    .summary-card.completed { border-color: var(--semantic-success); }
    .summary-card.failed { border-color: var(--semantic-danger); }

    .card-icon {
        color: var(--text-secondary);
        width: 48px;
        height: 48px;
        border-radius: var(--radius-md, 10px);
        display: flex;
        align-items: center;
        justify-content: center;
        background: var(--glass-bg-input);
    }

    .card-info {
        display: flex;
        flex-direction: column;
    }

    .card-value {
        font-size: var(--font-size-title1);
        font-weight: 700;
        color: var(--text-primary);
    }

    .card-label {
        font-size: var(--font-size-body);
        color: var(--text-secondary);
        text-transform: uppercase;
        letter-spacing: 0.5px;
    }

    /* Controls & Filter Bar */
    .controls-bar {
        display: flex;
        justify-content: space-between;
        align-items: center;
        flex-wrap: wrap;
        gap: var(--space-4);
        background: var(--glass-bg-surface);
        -webkit-backdrop-filter: var(--glass-blur);
        backdrop-filter: var(--glass-blur);
        padding: var(--space-3) var(--space-4);
        border-radius: var(--radius-lg, 14px);
        border: 1px solid var(--glass-border-subtle);
    }

    .filter-tabs {
        display: flex;
        gap: var(--space-2);
        flex-wrap: wrap;
    }




    .bulk-actions {
        display: flex;
        gap: var(--space-3);
        flex-wrap: wrap;
    }

    /* Buttons */


    .btn-primary:hover:not(:disabled) {
        background: var(--accent-primary-hover);
    }

    .btn-secondary:hover:not(:disabled) {
        background: var(--glass-bg-hover);
    }

    .btn-outline:hover:not(:disabled) {
        background: var(--glass-bg-hover);
    }

    .btn-outline-danger:hover:not(:disabled) {
        background: color-mix(in srgb, var(--semantic-danger) 15%, transparent);
    }


    .btn-danger:hover:not(:disabled) {
        filter: brightness(1.1);
    }

    .btn-retry:hover:not(:disabled) {
        filter: brightness(1.1);
    }

    /* Queue List */
    .queue-list {
        display: flex;
        flex-direction: column;
        gap: var(--space-4);
    }

    .empty-state {
        background: var(--glass-bg-surface);
        -webkit-backdrop-filter: var(--glass-blur);
        backdrop-filter: var(--glass-blur);
        border: 2px dashed var(--glass-border-subtle);
        border-radius: var(--radius-lg, 14px);
        padding: calc(var(--space-8) * 2) var(--space-8);
        text-align: center;
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: var(--space-4);
    }

    .empty-icon {
        font-size: calc(var(--font-size-title1) * 2.5);
    }

    .empty-state h3 {
        margin: 0;
        font-size: var(--font-size-title1);
        color: var(--text-primary);
    }

    .empty-state p {
        margin: 0;
        color: var(--text-secondary);
        max-width: 450px;
    }

    /* Job Card */
    .job-card {
        background: var(--glass-bg-surface);
        -webkit-backdrop-filter: var(--glass-blur);
        backdrop-filter: var(--glass-blur);
        border: 1px solid var(--glass-border-subtle);
        border-radius: var(--radius-lg, 14px);
        padding: var(--space-5);
        display: flex;
        flex-direction: column;
        gap: var(--space-3);
        box-shadow: inset 0 1px 0 0 var(--glass-border-specular), var(--glass-shadow-sm);
        transition: border-color var(--dur-fast, 120ms) ease;
    }

    .job-card:hover {
        border-color: var(--accent-primary);
    }

    .job-card-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        flex-wrap: wrap;
        gap: var(--space-2);
    }

    .job-title-group {
        display: flex;
        align-items: center;
        gap: var(--space-3);
    }

    .job-title {
        font-size: var(--font-size-title2);
        font-weight: 700;
        color: var(--text-primary);
    }

    .badge {
        font-size: var(--font-size-footnote);
        font-weight: 600;
        padding: var(--space-1) var(--space-2);
        border-radius: var(--radius-xs, 4px);
        text-transform: uppercase;
    }

    .badge-urgent { background: var(--semantic-danger); color: var(--text-on-accent); }
    .badge-high { background: var(--semantic-warning); color: var(--text-on-accent); }
    .badge-normal { background: var(--accent-primary); color: var(--text-on-accent); }
    .badge-low { background: var(--glass-bg-input); color: var(--text-secondary); }

    .status-pill {
        font-size: var(--font-size-footnote);
        font-weight: 600;
        padding: var(--space-1) var(--space-3);
        border-radius: var(--radius-pill, 999px);
        text-transform: uppercase;
        letter-spacing: 0.5px;
    }

    .status-running { background: var(--accent-primary); color: var(--text-on-accent); }
    .status-queued { background: var(--semantic-warning); color: var(--text-on-accent); }
    .status-completed { background: var(--semantic-success); color: var(--text-on-accent); }
    .status-failed { background: var(--semantic-danger); color: var(--text-on-accent); }
    .status-cancelled { background: var(--glass-bg-input); color: var(--text-tertiary); }

    .job-files-row {
        display: flex;
        align-items: center;
        gap: var(--space-3);
        font-size: var(--font-size-body);
        background: var(--glass-bg-input);
        padding: var(--space-2) var(--space-3);
        border-radius: var(--radius-sm, 8px);
        overflow: hidden;
    }

    .file-item {
        display: flex;
        gap: var(--space-2);
        align-items: center;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }

    .file-label {
        color: var(--text-secondary);
        font-weight: 600;
    }

    .file-name {
        color: var(--text-primary);
        font-family: var(--font-mono, monospace);
    }

    .arrow {
        color: var(--accent-primary);
        font-weight: bold;
    }

    /* Progress */
    .progress-container {
        display: flex;
        align-items: center;
        gap: var(--space-3);
    }

    .progress-bar-bg {
        flex: 1;
        height: 8px;
        background: var(--glass-bg-input);
        border-radius: 4px;
        overflow: hidden;
    }

    .progress-bar-fill {
        height: 100%;
        transition: width 0.3s ease;
    }

    .progress-bar-fill.status-running {
        background: linear-gradient(90deg, var(--accent-primary), var(--accent-primary-hover));
    }
    .progress-bar-fill.status-completed {
        background: var(--semantic-success);
    }
    .progress-bar-fill.status-failed {
        background: var(--semantic-danger);
    }
    .progress-bar-fill.status-queued {
        background: var(--semantic-warning);
    }
    .progress-bar-fill.status-cancelled {
        background: var(--glass-bg-input);
    }

    .progress-label {
        font-size: var(--font-size-footnote);
        font-family: var(--font-mono, monospace);
        color: var(--text-secondary);
        min-width: 38px;
        text-align: right;
    }

    /* Diagnostics */
    .diagnostics-box {
        background: color-mix(in srgb, var(--semantic-danger) 12%, transparent);
        border: 1px solid var(--semantic-danger);
        border-radius: var(--radius-sm, 8px);
        padding: var(--space-3);
        font-size: var(--font-size-body);
    }

    .diag-header {
        display: flex;
        align-items: center;
        gap: var(--space-2);
        margin-bottom: var(--space-1);
    }

    .diag-badge {
        background: var(--semantic-danger);
        color: var(--text-on-accent);
        font-size: var(--font-size-footnote);
        font-weight: bold;
        padding: var(--space-1) var(--space-2);
        border-radius: 3px;
        font-family: var(--font-mono, monospace);
    }

    .diag-title {
        color: var(--semantic-danger);
        font-weight: 600;
    }

    .diag-message {
        margin: 0;
        color: var(--text-primary);
        font-family: var(--font-mono, monospace);
        font-size: var(--font-size-footnote);
    }

    /* Card Actions */
    .job-actions-row {
        display: flex;
        justify-content: space-between;
        align-items: center;
        border-top: 1px solid var(--glass-border-subtle);
        padding-top: var(--space-2);
    }

    .job-id-text {
        font-size: var(--font-size-footnote);
        color: var(--text-tertiary);
        font-family: var(--font-mono, monospace);
    }

    .btn-group {
        display: flex;
        gap: var(--space-2);
    }
</style>

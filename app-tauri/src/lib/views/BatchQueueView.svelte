<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import { IpcClient } from '$lib/ipc';
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
            actionMessage = `Failed to cancel job: ${e}`;
        }
    }

    async function handleRetryJob(jobId: string) {
        try {
            const newId = await IpcClient.retryJob(jobId);
            actionMessage = `Job re-enqueued as ${newId.slice(0, 8)}.`;
            await loadQueue();
        } catch (e) {
            actionMessage = `Failed to retry job: ${e}`;
        }
    }

    async function handleClearFinished() {
        try {
            const count = await IpcClient.clearFinishedJobs();
            actionMessage = `Cleared ${count} finished jobs.`;
            await loadQueue();
        } catch (e) {
            actionMessage = `Failed to clear finished jobs: ${e}`;
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
            actionMessage = `Error cancelling active jobs: ${e}`;
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
            actionMessage = `Submitted batch demo with ${ids.length} jobs.`;
            await loadQueue();
        } catch (e) {
            actionMessage = `Failed to dispatch batch: ${e}`;
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
            <span class="icon">📋</span>
            <div>
                <h2>Batch Queue Management</h2>
                <p class="subtitle">Inspect, monitor, prioritize, and control multi-file background operations</p>
            </div>
        </div>
        <div class="header-actions">
            <button class="btn btn-secondary" onclick={loadQueue} disabled={loading} title="Refresh Queue">
                🔄 Refresh
            </button>
            <button class="btn btn-primary" onclick={handleAddBatchDemo} data-testid="btn-add-batch-demo">
                ⚡ Submit Batch Demo / Add Files
            </button>
        </div>
    </header>

    <!-- Notification Banner -->
    {#if actionMessage}
        <div class="alert-banner">
            <span>{actionMessage}</span>
            <button class="close-alert" onclick={() => { actionMessage = null; }}>×</button>
        </div>
    {/if}

    <!-- Summary Metrics Grid -->
    <section class="summary-grid" data-testid="summary-header">
        <div class="summary-card active">
            <div class="card-icon">⚡</div>
            <div class="card-info">
                <span class="card-value">{runningCount}</span>
                <span class="card-label">Running</span>
            </div>
        </div>
        <div class="summary-card queued">
            <div class="card-icon">⏳</div>
            <div class="card-info">
                <span class="card-value">{queuedCount}</span>
                <span class="card-label">Queued</span>
            </div>
        </div>
        <div class="summary-card completed">
            <div class="card-icon">✓</div>
            <div class="card-info">
                <span class="card-value">{completedCount}</span>
                <span class="card-label">Completed</span>
            </div>
        </div>
        <div class="summary-card failed">
            <div class="card-icon">✕</div>
            <div class="card-info">
                <span class="card-value">{failedCount}</span>
                <span class="card-label">Failed</span>
            </div>
        </div>
    </section>

    <!-- Controls & Filter Bar -->
    <section class="controls-bar">
        <div class="filter-tabs">
            <button
                class="tab-btn"
                class:active={currentFilter === 'All'}
                onclick={() => { currentFilter = 'All'; }}
                data-testid="filter-all"
            >
                All ({totalCount})
            </button>
            <button
                class="tab-btn"
                class:active={currentFilter === 'Active'}
                onclick={() => { currentFilter = 'Active'; }}
                data-testid="filter-active"
            >
                Active ({runningCount + queuedCount})
            </button>
            <button
                class="tab-btn"
                class:active={currentFilter === 'Completed'}
                onclick={() => { currentFilter = 'Completed'; }}
                data-testid="filter-completed"
            >
                Completed ({completedCount})
            </button>
            <button
                class="tab-btn"
                class:active={currentFilter === 'Failed'}
                onclick={() => { currentFilter = 'Failed'; }}
                data-testid="filter-failed"
            >
                Failed ({failedCount})
            </button>
        </div>

        <div class="bulk-actions">
            <button
                class="btn btn-outline-danger"
                onclick={handleCancelAllActive}
                disabled={activeJobs.length === 0}
                data-testid="btn-cancel-all-active"
            >
                Cancel All Active
            </button>
            <button
                class="btn btn-outline"
                onclick={handleClearFinished}
                disabled={completedCount === 0 && failedCount === 0}
                data-testid="btn-clear-finished"
            >
                Clear Finished
            </button>
        </div>
    </section>

    <!-- Queue List -->
    <main class="queue-list">
        {#if filteredJobs.length === 0}
            <div class="empty-state" data-testid="empty-queue-message">
                <div class="empty-icon">📭</div>
                <h3>No jobs matching '{currentFilter}' filter</h3>
                <p>Submit a new batch operation or click below to simulate batch PDF conversions.</p>
                <button class="btn btn-primary" onclick={handleAddBatchDemo}>
                    Enqueue Sample Batch Jobs
                </button>
            </div>
        {:else}
            {#each filteredJobs as job (job.job_id)}
                <article class="job-card" data-testid="job-card" id={`job-${job.job_id}`}>
                    <div class="job-card-header">
                        <div class="job-title-group">
                            <span class="job-title">{job.title}</span>
                            <span class="badge {getPriorityClass(job.priority)}">{job.priority} Priority</span>
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
                            <span class="arrow">➔</span>
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
                        <div class="diagnostics-box" data-testid="diagnostics-box">
                            <div class="diag-header">
                                <span class="diag-badge">{job.error_code || 'ERROR'}</span>
                                <span class="diag-title">Diagnostics</span>
                            </div>
                            <p class="diag-message">{job.error_message || 'Job execution encountered an unhandled error.'}</p>
                        </div>
                    {/if}

                    <!-- Actions -->
                    <div class="job-actions-row">
                        <span class="job-id-text">ID: {job.job_id.slice(0, 12)}...</span>
                        <div class="btn-group">
                            {#if job.status === 'Queued' || job.status === 'Running'}
                                <button
                                    class="btn btn-sm btn-danger"
                                    onclick={() => handleCancelJob(job.job_id)}
                                    data-testid={`btn-cancel-${job.job_id}`}
                                >
                                    Cancel
                                </button>
                            {/if}
                            {#if job.status === 'Failed' || job.status === 'Cancelled'}
                                <button
                                    class="btn btn-sm btn-retry"
                                    onclick={() => handleRetryJob(job.job_id)}
                                    data-testid={`btn-retry-${job.job_id}`}
                                >
                                    🔄 Retry Job
                                </button>
                            {/if}
                        </div>
                    </div>
                </article>
            {/each}
        {/if}
    </main>
</div>

<style>
    .batch-queue-container {
        padding: 2rem;
        display: flex;
        flex-direction: column;
        gap: 1.5rem;
        color: #cdd6f4;
        max-width: 1200px;
        margin: 0 auto;
        width: 100%;
    }

    .header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        flex-wrap: wrap;
        gap: 1rem;
    }

    .header-title {
        display: flex;
        align-items: center;
        gap: 1rem;
    }

    .header-title .icon {
        font-size: 2.2rem;
    }

    .header-title h2 {
        font-size: 1.75rem;
        margin: 0;
        font-weight: 700;
        color: #f5e0dc;
    }

    .subtitle {
        margin: 0.25rem 0 0;
        font-size: 0.9rem;
        color: #a6adc8;
    }

    .header-actions {
        display: flex;
        gap: 0.75rem;
    }

    .alert-banner {
        display: flex;
        justify-content: space-between;
        align-items: center;
        background: #313244;
        border-left: 4px solid #89b4fa;
        padding: 0.75rem 1rem;
        border-radius: 6px;
        font-size: 0.9rem;
        color: #cdd6f4;
    }

    .close-alert {
        background: none;
        border: none;
        color: #a6adc8;
        font-size: 1.25rem;
        cursor: pointer;
    }

    /* Summary Metrics Grid */
    .summary-grid {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
        gap: 1rem;
    }

    .summary-card {
        background: #1e1e2e;
        border: 1px solid #313244;
        border-radius: 12px;
        padding: 1.25rem;
        display: flex;
        align-items: center;
        gap: 1rem;
        transition: transform 0.2s, box-shadow 0.2s;
    }

    .summary-card:hover {
        transform: translateY(-2px);
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.25);
    }

    .summary-card.active { border-color: #89b4fa; }
    .summary-card.queued { border-color: #f9e2af; }
    .summary-card.completed { border-color: #a6e3a1; }
    .summary-card.failed { border-color: #f38ba8; }

    .card-icon {
        font-size: 1.8rem;
        width: 48px;
        height: 48px;
        border-radius: 10px;
        display: flex;
        align-items: center;
        justify-content: center;
        background: #181825;
    }

    .card-info {
        display: flex;
        flex-direction: column;
    }

    .card-value {
        font-size: 1.6rem;
        font-weight: 700;
        color: #cdd6f4;
    }

    .card-label {
        font-size: 0.85rem;
        color: #a6adc8;
        text-transform: uppercase;
        letter-spacing: 0.5px;
    }

    /* Controls & Filter Bar */
    .controls-bar {
        display: flex;
        justify-content: space-between;
        align-items: center;
        flex-wrap: wrap;
        gap: 1rem;
        background: #181825;
        padding: 0.75rem 1rem;
        border-radius: 10px;
        border: 1px solid #313244;
    }

    .filter-tabs {
        display: flex;
        gap: 0.5rem;
    }

    .tab-btn {
        background: transparent;
        border: 1px solid transparent;
        color: #a6adc8;
        padding: 0.45rem 0.9rem;
        border-radius: 6px;
        cursor: pointer;
        font-size: 0.9rem;
        font-weight: 500;
        transition: all 0.2s;
    }

    .tab-btn:hover {
        color: #cdd6f4;
        background: #313244;
    }

    .tab-btn.active {
        color: #11111b;
        background: #cba6f7;
        font-weight: 600;
    }

    .bulk-actions {
        display: flex;
        gap: 0.5rem;
    }

    /* Buttons */
    .btn {
        padding: 0.5rem 1rem;
        border-radius: 6px;
        font-size: 0.9rem;
        font-weight: 600;
        cursor: pointer;
        border: 1px solid transparent;
        transition: all 0.2s;
        display: inline-flex;
        align-items: center;
        gap: 0.4rem;
    }

    .btn:disabled {
        opacity: 0.45;
        cursor: not-allowed;
    }

    .btn-primary {
        background: #89b4fa;
        color: #11111b;
    }
    .btn-primary:hover:not(:disabled) {
        background: #b4befe;
    }

    .btn-secondary {
        background: #313244;
        color: #cdd6f4;
    }
    .btn-secondary:hover:not(:disabled) {
        background: #45475a;
    }

    .btn-outline {
        background: transparent;
        border-color: #45475a;
        color: #cdd6f4;
    }
    .btn-outline:hover:not(:disabled) {
        background: #313244;
    }

    .btn-outline-danger {
        background: transparent;
        border-color: #f38ba8;
        color: #f38ba8;
    }
    .btn-outline-danger:hover:not(:disabled) {
        background: rgba(243, 139, 168, 0.15);
    }

    .btn-sm {
        padding: 0.3rem 0.75rem;
        font-size: 0.8rem;
    }

    .btn-danger {
        background: #f38ba8;
        color: #11111b;
    }
    .btn-danger:hover:not(:disabled) {
        background: #eba0ac;
    }

    .btn-retry {
        background: #f9e2af;
        color: #11111b;
    }
    .btn-retry:hover:not(:disabled) {
        background: #f5c2e7;
    }

    /* Queue List */
    .queue-list {
        display: flex;
        flex-direction: column;
        gap: 1rem;
    }

    .empty-state {
        background: #1e1e2e;
        border: 2px dashed #313244;
        border-radius: 12px;
        padding: 4rem 2rem;
        text-align: center;
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 1rem;
    }

    .empty-icon {
        font-size: 3.5rem;
    }

    .empty-state h3 {
        margin: 0;
        font-size: 1.3rem;
        color: #cdd6f4;
    }

    .empty-state p {
        margin: 0;
        color: #a6adc8;
        max-width: 450px;
    }

    /* Job Card */
    .job-card {
        background: #1e1e2e;
        border: 1px solid #313244;
        border-radius: 10px;
        padding: 1.25rem;
        display: flex;
        flex-direction: column;
        gap: 0.85rem;
        box-shadow: 0 2px 6px rgba(0, 0, 0, 0.2);
        transition: border-color 0.2s;
    }

    .job-card:hover {
        border-color: #45475a;
    }

    .job-card-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    .job-title-group {
        display: flex;
        align-items: center;
        gap: 0.75rem;
    }

    .job-title {
        font-size: 1.1rem;
        font-weight: 700;
        color: #f5e0dc;
    }

    .badge {
        font-size: 0.75rem;
        font-weight: 600;
        padding: 0.2rem 0.5rem;
        border-radius: 4px;
        text-transform: uppercase;
    }

    .badge-urgent { background: #f38ba8; color: #11111b; }
    .badge-high { background: #fab387; color: #11111b; }
    .badge-normal { background: #89b4fa; color: #11111b; }
    .badge-low { background: #6c7086; color: #cdd6f4; }

    .status-pill {
        font-size: 0.8rem;
        font-weight: 600;
        padding: 0.3rem 0.7rem;
        border-radius: 12px;
        text-transform: uppercase;
        letter-spacing: 0.5px;
    }

    .status-running { background: #89b4fa; color: #11111b; }
    .status-queued { background: #f9e2af; color: #11111b; }
    .status-completed { background: #a6e3a1; color: #11111b; }
    .status-failed { background: #f38ba8; color: #11111b; }
    .status-cancelled { background: #585b70; color: #cdd6f4; }

    .job-files-row {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        font-size: 0.85rem;
        background: #181825;
        padding: 0.6rem 0.85rem;
        border-radius: 6px;
        overflow: hidden;
    }

    .file-item {
        display: flex;
        gap: 0.4rem;
        align-items: center;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }

    .file-label {
        color: #a6adc8;
        font-weight: 600;
    }

    .file-name {
        color: #bac2de;
        font-family: monospace;
    }

    .arrow {
        color: #cba6f7;
        font-weight: bold;
    }

    /* Progress */
    .progress-container {
        display: flex;
        align-items: center;
        gap: 0.75rem;
    }

    .progress-bar-bg {
        flex: 1;
        height: 8px;
        background: #313244;
        border-radius: 4px;
        overflow: hidden;
    }

    .progress-bar-fill {
        height: 100%;
        transition: width 0.3s ease;
    }

    .progress-bar-fill.status-running {
        background: linear-gradient(90deg, #89b4fa, #cba6f7);
    }
    .progress-bar-fill.status-completed {
        background: #a6e3a1;
    }
    .progress-bar-fill.status-failed {
        background: #f38ba8;
    }
    .progress-bar-fill.status-queued {
        background: #f9e2af;
    }
    .progress-bar-fill.status-cancelled {
        background: #585b70;
    }

    .progress-label {
        font-size: 0.8rem;
        font-family: monospace;
        color: #a6adc8;
        min-width: 38px;
        text-align: right;
    }

    /* Diagnostics */
    .diagnostics-box {
        background: #241926;
        border: 1px solid #f38ba8;
        border-radius: 6px;
        padding: 0.75rem;
        font-size: 0.85rem;
    }

    .diag-header {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        margin-bottom: 0.35rem;
    }

    .diag-badge {
        background: #f38ba8;
        color: #11111b;
        font-size: 0.7rem;
        font-weight: bold;
        padding: 0.15rem 0.4rem;
        border-radius: 3px;
        font-family: monospace;
    }

    .diag-title {
        color: #eba0ac;
        font-weight: 600;
    }

    .diag-message {
        margin: 0;
        color: #cdd6f4;
        font-family: monospace;
        font-size: 0.8rem;
    }

    /* Card Actions */
    .job-actions-row {
        display: flex;
        justify-content: space-between;
        align-items: center;
        border-top: 1px solid #313244;
        padding-top: 0.6rem;
    }

    .job-id-text {
        font-size: 0.75rem;
        color: #6c7086;
        font-family: monospace;
    }

    .btn-group {
        display: flex;
        gap: 0.5rem;
    }
</style>

import { invoke } from '@tauri-apps/api/core';
import type { JobRequest } from './bindings/JobRequest';
import type { JobStatus } from './bindings/JobStatus';
import type { JobPriority } from './bindings/JobPriority';
import type { JobRecord } from './bindings/JobRecord';
import type { PipelinePreset } from './bindings/PipelinePreset';
import type { PipelineStep } from './bindings/PipelineStep';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { JobEvent } from './bindings/JobEvent';

export class IpcClient {
    /**
     * Submit a job to the Rust backend
     */
    static async submitJob(request: JobRequest): Promise<string> {
        return await invoke<string>('submit_job', { request });
    }

    /**
     * Submit a batch of jobs with optional priority
     */
    static async submitBatch(requests: JobRequest[], priority?: JobPriority): Promise<string[]> {
        return await invoke<string[]>('submit_batch', { requests, priority: priority ?? null });
    }

    /**
     * Cancel an ongoing job
     */
    static async cancelJob(jobId: string): Promise<boolean> {
        return await invoke<boolean>('cancel_job', { jobId });
    }

    /**
     * Check the status of a job
     */
    static async getJobStatus(jobId: string): Promise<JobStatus | null> {
        return await invoke<JobStatus | null>('get_job_status', { jobId });
    }

    /**
     * List all tracked jobs in the queue
     */
    static async listJobs(): Promise<JobRecord[]> {
        return await invoke<JobRecord[]>('list_jobs');
    }

    /**
     * Clear all finished (completed, failed, cancelled) jobs
     */
    static async clearFinishedJobs(): Promise<number> {
        return await invoke<number>('clear_finished_jobs');
    }

    /**
     * Re-enqueue a failed or cancelled job
     */
    static async retryJob(jobId: string): Promise<string> {
        return await invoke<string>('retry_job', { jobId });
    }

    /**
     * List built-in pipeline presets
     */
    static async listPipelinePresets(): Promise<PipelinePreset[]> {
        return await invoke<PipelinePreset[]>('list_pipeline_presets');
    }

    /**
     * Listen for job events emitted by the backend
     */
    static async onJobEvent(handler: (event: JobEvent) => void): Promise<UnlistenFn> {
        return await listen<JobEvent>('job-event', (event) => {
            handler(event.payload);
        });
    }
}



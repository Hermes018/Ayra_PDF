import { invoke } from '@tauri-apps/api/core';
import type { JobRequest } from './bindings/JobRequest';
import type { JobStatus } from './bindings/JobStatus';
import type { JobPriority } from './bindings/JobPriority';
import type { JobRecord } from './bindings/JobRecord';
import type { PipelinePreset } from './bindings/PipelinePreset';
import type { PipelineStep } from './bindings/PipelineStep';
import type { AppSettings } from './bindings/AppSettings';
import type { AiModelInfo } from './bindings/AiModelInfo';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { JobEvent } from './bindings/JobEvent';

export interface ConvertPdfPayload {
    inputPath: string;
    outputPath: string;
    format?: string;
    nativeTextOnly?: boolean;
}

export interface ExtractTextPayload {
    inputPath: string;
    outputPath: string;
    nativeTextOnly?: boolean;
}

export class IpcClient {
    /**
     * Submit a job to the Rust backend
     */
    static async submitJob(request: JobRequest): Promise<string> {
        return await invoke<string>('submit_job', { request });
    }

    /**
     * Submit a job and wait until it reaches a terminal state (BUG-014). `submit_job` returns as
     * soon as the job is queued, so a caller that chains jobs (output of one = input of the next)
     * or reports success must wait. Resolves with the job id on `Completed`; rejects with the
     * job's error message on `Failed` / `Cancelled` / unknown job.
     *
     * Never waits forever (review r2): `signal` cancels the job (cancel_job) and rejects with
     * "cancelled"; if neither status nor progress changes for `stallMs` the job is cancelled and
     * the promise rejects with a "no progress" error. Long jobs that keep reporting progress are
     * not cut off (DON'T-7: no caps). `silent` keeps the global "Operation …" toasts away.
     */
    static async runJob(
        request: JobRequest,
        opts: { pollMs?: number; stallMs?: number; signal?: AbortSignal; silent?: boolean } = {}
    ): Promise<string> {
        const pollMs = opts.pollMs ?? 100;
        const stallMs = opts.stallMs ?? 120_000;
        const jobId = opts.silent ? await IpcClient.submitSilentJob(request) : await IpcClient.submitJob(request);
        // The no-progress timer starts when the job leaves the queue (review r3): time spent
        // waiting behind other jobs is not "no progress".
        let lastChange: number | null = null;
        let lastKey = '';
        for (;;) {
            if (opts.signal?.aborted) {
                await IpcClient.cancelJob(jobId).catch(() => false);
                throw new Error('Cancelled by you; no file was written by the unfinished step.');
            }
            const status = await IpcClient.getJobStatus(jobId);
            if (status === 'Completed') return jobId;
            if (status === 'Failed' || status === 'Cancelled' || status === null) {
                let detail = status === null ? 'job not found' : status.toLowerCase();
                try {
                    const rec = (await IpcClient.listJobs()).find((j) => j.job_id === jobId);
                    if (rec?.error_message) detail = rec.error_message;
                    else if (rec?.error_code) detail = rec.error_code;
                } catch {
                    // keep the status-based detail
                }
                throw new Error(detail);
            }
            let progress = '';
            try {
                const rec = (await IpcClient.listJobs()).find((j) => j.job_id === jobId);
                progress = rec ? String(rec.progress) : '';
            } catch {
                // progress is optional
            }
            const key = `${status}:${progress}`;
            if (status === 'Queued') {
                lastChange = null;
            } else if (key !== lastKey || lastChange === null) {
                lastKey = key;
                lastChange = Date.now();
            } else if (Date.now() - lastChange >= stallMs) {
                await IpcClient.cancelJob(jobId).catch(() => false);
                throw new Error(
                    `The job made no progress for ${stallMs / 1000} s and was stopped. Try again, or check the file.`
                );
            }
            await new Promise((resolve) => setTimeout(resolve, pollMs));
        }
    }

    /* ---------- Internal (silent) jobs: no global "Operation …" toasts (review r2) ---------- */
    static #silentJobs = new Set<string>();
    static #silentSubmitsInFlight = 0;
    static #settledWaiters: Array<() => void> = [];

    /** Submit a job the user did not start (e.g. the open-time GetInfo probe). */
    static async submitSilentJob(request: JobRequest): Promise<string> {
        IpcClient.#silentSubmitsInFlight++;
        try {
            const id = await IpcClient.submitJob(request);
            IpcClient.#silentJobs.add(id);
            return id;
        } finally {
            IpcClient.#silentSubmitsInFlight--;
            if (IpcClient.#silentSubmitsInFlight === 0) {
                IpcClient.#settledWaiters.splice(0).forEach((w) => w());
            }
        }
    }

    /** Resolves once no silent submission is in flight (an event may arrive before its id is known). */
    static silentSubmitsSettled(): Promise<void> {
        if (IpcClient.#silentSubmitsInFlight === 0) return Promise.resolve();
        return new Promise((resolve) => IpcClient.#settledWaiters.push(resolve));
    }

    static isSilentJob(jobId: string): boolean {
        return IpcClient.#silentJobs.has(jobId);
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
     * Fetch active application settings
     */
    static async getSettings(): Promise<AppSettings> {
        return await invoke<AppSettings>('get_settings');
    }

    /**
     * Save application settings atomically
     */
    static async saveSettings(settings: AppSettings): Promise<AppSettings> {
        return await invoke<AppSettings>('save_settings', { settings });
    }

    /**
     * Fetch recent log lines from local rotating log
     */
    static async getRecentLogs(maxLines?: number): Promise<string[]> {
        return await invoke<string[]>('get_recent_logs', { maxLines: maxLines ?? null });
    }

    /**
     * Export sanitized local diagnostics report
     */
    static async exportDiagnostics(outputPath: string, protectedPaths: string[] = [], overwrite = false): Promise<string> {
        return await invoke<string>('export_diagnostics', { outputPath, protectedPaths, overwrite });
    }

    /**
     * Validate and import local AI model (.gguf, .onnx, etc.) with license acknowledgement
     */
    static async importLocalAiModel(
        modelPath: string,
        licenseSpdx?: string,
        acknowledgeLicense?: boolean
    ): Promise<AiModelInfo> {
        return await invoke<AiModelInfo>('import_local_ai_model', {
            modelPath,
            licenseSpdx: licenseSpdx ?? null,
            acknowledgeLicense: acknowledgeLicense ?? false,
        });
    }

    /**
     * Listen for job events emitted by the backend
     */
    static async onJobEvent(handler: (event: JobEvent) => void): Promise<UnlistenFn> {
        return await listen<JobEvent>('job-event', (event) => {
            handler(event.payload);
        });
    }

    /**
     * BUG-018: ask the backend engine (pdfcraft, lopdf fallback = F7 degraded mode) whether a
     * document opens cleanly. Runs a GetInfo job; the job's Completed event carries `degraded`.
     */
    static async probeDocument(
        path: string,
        opts: { pollMs?: number; timeoutMs?: number } = {}
    ): Promise<{ ok: true; degraded: boolean; timedOut?: boolean } | { ok: false; error: string; errorCode: string | null }> {
        const pollMs = opts.pollMs ?? 100;
        const timeoutMs = opts.timeoutMs ?? 60_000;
        // Held in an object: TypeScript does not track assignments made inside the listener.
        const seen: { degraded: boolean | null; failure: string | null; code: string | null } = {
            degraded: null,
            failure: null,
            code: null,
        };
        let jobId: string | null = null;
        const early: JobEvent[] = [];
        const take = (ev: JobEvent) => {
            if (ev.data.job_id !== jobId) return;
            if (ev.kind === 'Completed') seen.degraded = ev.data.degraded;
            if (ev.kind === 'Failed') {
                seen.failure = ev.data.error_message || ev.data.error_code;
                seen.code = ev.data.error_code || null;
            }
        };
        const unlisten = await IpcClient.onJobEvent((ev) => (jobId === null ? early.push(ev) : take(ev)));
        const started = Date.now();
        try {
            jobId = await IpcClient.submitSilentJob({ op: 'GetInfo', input_path: path });
            early.splice(0).forEach(take);
            for (;;) {
                const status = await IpcClient.getJobStatus(jobId);
                if (status === 'Completed') {
                    // The Completed event may land just after the status flips.
                    if (seen.degraded === null) await new Promise((r) => setTimeout(r, pollMs));
                    return { ok: true, degraded: seen.degraded === true };
                }
                if (status === 'Failed' || status === 'Cancelled' || status === null) {
                    if (!seen.failure || !seen.code) {
                        const rec = (await IpcClient.listJobs()).find((j) => j.job_id === jobId);
                        seen.failure = seen.failure || rec?.error_message || rec?.error_code || (status ? status.toLowerCase() : 'job not found');
                        seen.code = seen.code || rec?.error_code || null;
                    }
                    return { ok: false, error: seen.failure ?? 'unknown error', errorCode: seen.code };
                }
                if (Date.now() - started >= timeoutMs) {
                    // Review r3: never hang. Give up quietly (no false "damaged" notice) and stop the job.
                    await IpcClient.cancelJob(jobId).catch(() => false);
                    return { ok: true, degraded: false, timedOut: true };
                }
                await new Promise((r) => setTimeout(r, pollMs));
            }
        } finally {
            unlisten();
        }
    }

    /**
     * Open native OS file picker dialog
     */
    static async pickFilesDialog(multiple = false, extensions?: string[]): Promise<string[]> {
        return await invoke<string[]>('pick_files_dialog', {
            multiple,
            extensions: extensions ?? null,
        });
    }

    /**
     * Open native OS folder picker dialog
     */
    static async pickFolderDialog(): Promise<string | null> {
        return await invoke<string | null>('pick_folder_dialog');
    }

    /**
     * Open native OS save file dialog
     */
    static async saveFileDialog(defaultName?: string): Promise<string | null> {
        return await invoke<string | null>('save_file_dialog', {
            defaultName: defaultName ?? null,
        });
    }

    /**
     * Convert PDF with native text extraction option
     */
    static async convertPdf(payload: ConvertPdfPayload): Promise<string> {
        return await invoke<string>('convert_pdf', {
            payload: {
                input_path: payload.inputPath,
                output_path: payload.outputPath,
                format: payload.format ?? null,
                native_text_only: payload.nativeTextOnly ?? null,
            }
        });
    }

    /**
     * Extract text with native_text_only option
     */
    static async extractText(payload: ExtractTextPayload): Promise<string> {
        return await invoke<string>('extract_text', {
            inputPath: payload.inputPath,
            outputPath: payload.outputPath,
            nativeTextOnly: payload.nativeTextOnly ?? null,
        });
    }
}

export async function convertPdf(payload: ConvertPdfPayload): Promise<string> {
    return IpcClient.convertPdf(payload);
}

export async function extractText(payload: ExtractTextPayload): Promise<string> {
    return IpcClient.extractText(payload);
}

export async function pickFilesDialog(multiple = false, extensions?: string[]): Promise<string[]> {
    return IpcClient.pickFilesDialog(multiple, extensions);
}

export async function pickFolderDialog(): Promise<string | null> {
    return IpcClient.pickFolderDialog();
}

export async function saveFileDialog(defaultName?: string): Promise<string | null> {
    return IpcClient.saveFileDialog(defaultName);
}

export async function getStartupFiles(): Promise<string[]> {
  try {
    return await invoke<string[]>('get_startup_files');
  } catch {
    return [];
  }
}

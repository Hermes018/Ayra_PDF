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
    static async exportDiagnostics(outputPath: string): Promise<string> {
        return await invoke<string>('export_diagnostics', { outputPath });
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

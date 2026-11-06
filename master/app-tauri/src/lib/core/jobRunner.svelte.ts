// One running job at a time for a screen: waits for it (existing IPC job signals), exposes a label for Progress,
// an abort for Cancel, and a plain-words error with Retry. Technical detail goes to the console, not the UI.
import { IpcClient } from '$lib/ipc';
import type { JobRequest } from '$lib/bindings/JobRequest';
import { friendlyJobError } from './jobError';

/** A request, or a builder that makes a fresh one on every attempt (so a Retry can pick the next free output name). */
export type JobRequestSource = JobRequest | (() => Promise<JobRequest>);

export interface JobError {
  what: string;
  next: string;
  retry: () => void;
}

export class JobRunner {
  busyLabel = $state<string | null>(null);
  error = $state<JobError | null>(null);
  #abort: AbortController | null = null;

  cancel(): void {
    this.#abort?.abort();
  }

  async run(label: string, what: string, next: string, source: JobRequestSource): Promise<void> {
    if (this.busyLabel) return; // checked and set before any await: a double click starts one job
    this.error = null;
    this.busyLabel = label;
    this.#abort = new AbortController();
    const signal = this.#abort.signal;
    try {
      const req = typeof source === 'function' ? await source() : source;
      await IpcClient.runJob(req, { signal });
    } catch (e) {
      if (!signal.aborted) {
        console.error(`${what}`, e);
        const f = friendlyJobError(e instanceof Error ? e.message : String(e), { what, next });
        this.error = { what: f.what, next: f.next, retry: () => void this.run(label, what, next, source) };
      }
    } finally {
      this.busyLabel = null;
      this.#abort = null;
    }
  }
}

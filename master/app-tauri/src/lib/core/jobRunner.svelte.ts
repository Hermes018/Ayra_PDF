// One running job at a time for a screen: waits for it (existing IPC job signals), exposes a label for Progress,
// an abort for Cancel, and a plain-words error with Retry. Technical detail goes to the console, not the UI.
import { IpcClient } from '$lib/ipc';
import type { JobRequest } from '$lib/bindings/JobRequest';

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

  async run(label: string, what: string, next: string, req: JobRequest): Promise<void> {
    if (this.busyLabel) return;
    this.error = null;
    this.busyLabel = label;
    this.#abort = new AbortController();
    const signal = this.#abort.signal;
    try {
      await IpcClient.runJob(req, { signal });
    } catch (e) {
      if (!signal.aborted) {
        console.error(`${what} (${req.op})`, e);
        this.error = { what, next, retry: () => void this.run(label, what, next, req) };
      }
    } finally {
      this.busyLabel = null;
      this.#abort = null;
    }
  }
}

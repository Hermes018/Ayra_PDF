import { describe, it, expect, vi, beforeEach } from 'vitest';

const runJob = vi.fn();
vi.mock('$lib/ipc', () => ({ IpcClient: { runJob: (...a: unknown[]) => runJob(...a) } }));

import { JobRunner } from './jobRunner.svelte';

const REQ = { op: 'Repair', input_path: '/a.pdf', output_path: '/a_repaired.pdf' } as never;

describe('JobRunner', () => {
  beforeEach(() => {
    runJob.mockReset();
  });

  it('runs one job: busyLabel while running, cleared after; no error', async () => {
    let release!: () => void;
    runJob.mockReturnValue(new Promise<void>((r) => (release = r)));
    const r = new JobRunner();
    const p = r.run('Repairing', "Couldn't repair the file.", 'Try again.', REQ);
    expect(r.busyLabel).toBe('Repairing');
    release();
    await p;
    expect(r.busyLabel).toBeNull();
    expect(r.error).toBeNull();
  });

  it('a failure sets a plain-words error with a working retry', async () => {
    runJob.mockRejectedValueOnce(new Error('engine said no'));
    const r = new JobRunner();
    await r.run('Repairing', "Couldn't repair the file.", 'Check the file, then try again.', REQ);
    expect(r.error).toMatchObject({ what: "Couldn't repair the file.", next: 'Check the file, then try again.' });
    expect(JSON.stringify(r.error)).not.toContain('engine said no'); // details go to the log, not the UI
    runJob.mockResolvedValueOnce('id');
    r.error!.retry();
    await vi.waitFor(() => expect(r.error).toBeNull());
    expect(runJob).toHaveBeenCalledTimes(2);
  });

  it('cancel aborts the job and shows no error', async () => {
    runJob.mockImplementation((_req: unknown, opts: { signal: AbortSignal }) =>
      new Promise((_res, rej) => opts.signal.addEventListener('abort', () => rej(new Error('Cancelled by you')))));
    const r = new JobRunner();
    const p = r.run('Compressing', "Couldn't compress.", 'Try again.', REQ);
    r.cancel();
    await p;
    expect(r.error).toBeNull();
    expect(r.busyLabel).toBeNull();
  });

  it('refuses a second job while one is running', async () => {
    runJob.mockReturnValue(new Promise(() => {}));
    const r = new JobRunner();
    void r.run('One', 'x', 'y', REQ);
    await r.run('Two', 'x', 'y', REQ);
    expect(runJob).toHaveBeenCalledTimes(1);
  });

  it('an OUTPUT_PATH_UNSAFE refusal shows the plain-words error; Retry rebuilds the request (a new name)', async () => {
    runJob.mockRejectedValueOnce(new Error('OUTPUT_PATH_UNSAFE: a file already exists at the output path (/a_repaired.pdf). Nothing was written.'));
    const r = new JobRunner();
    let n = 0;
    const build = async () => ({ op: 'Repair', input_path: '/a.pdf', output_path: `/a_repaired-${++n}.pdf` }) as never;
    await r.run('Repairing', "Couldn't repair the file.", 'Try again.', build);
    expect(r.error?.what).toBe('A file named a_repaired.pdf already exists.');
    runJob.mockResolvedValueOnce('id');
    r.error!.retry();
    await vi.waitFor(() => expect(runJob).toHaveBeenCalledTimes(2));
    const outs = runJob.mock.calls.map((c) => (c[0] as { output_path: string }).output_path);
    expect(outs).toEqual(['/a_repaired-1.pdf', '/a_repaired-2.pdf']);
  });

  it('a throwing request builder is an error, not an unhandled rejection, and busyLabel resets', async () => {
    const r = new JobRunner();
    await r.run('Repairing', "Couldn't repair the file.", 'Try again.', async () => {
      throw new Error('no free name');
    });
    expect(r.busyLabel).toBeNull();
    expect(r.error?.what).toBe("Couldn't repair the file.");
  });
});

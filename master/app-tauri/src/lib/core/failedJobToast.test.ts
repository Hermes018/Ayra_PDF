// The global "Operation failed" toast (every screen that fires a job without its own error UI) shows plain words for
// an OUTPUT_PATH_UNSAFE refusal, not the raw backend string.
import { describe, it, expect, vi, beforeEach } from 'vitest';

// The app state registers its listener while it is imported, before the rest of this file runs.
const held = vi.hoisted(() => ({ handler: null as ((ev: unknown) => void) | null }));
vi.mock('$lib/ipc', () => ({
  IpcClient: {
    onJobEvent: vi.fn(async (cb: (ev: unknown) => void) => {
      held.handler = cb;
      return () => {};
    }),
    silentSubmitsSettled: vi.fn(async () => {}),
    isSilentJob: () => false,
  },
}));

import { appState } from '$lib/state.svelte';

describe('failed job toast', () => {
  beforeEach(() => {
    appState.clearToasts();
  });

  async function fail(message: string) {
    await vi.waitFor(() => expect(held.handler).not.toBeNull());
    held.handler!({ kind: 'Failed', data: { job_id: 'j1', error_message: message } });
    await new Promise((r) => setTimeout(r, 0));
    await new Promise((r) => setTimeout(r, 0));
  }

  it('an existing output file: plain words, next step, no raw code', async () => {
    await fail('OUTPUT_PATH_UNSAFE: a file already exists at the output path (/docs/a_x.pdf). Nothing was written.');
    const t = appState.toasts[0];
    expect(t.kind).toBe('error');
    expect(t.message).toBe('A file named a_x.pdf already exists.'); // the failing file name is kept
    expect(t.detail).toMatch(/Nothing was written/);
    expect(t.detail).not.toMatch(/Try again|next free name/i); // a toast has no retry: no promise of one
    expect(JSON.stringify(appState.toasts)).not.toContain('OUTPUT_PATH_UNSAFE');
  });

  it('other failures keep the generic title and the detail', async () => {
    await fail('engine said no');
    expect(appState.toasts[0].message).toBe('Operation Failed');
    expect(appState.toasts[0].detail).toBe('engine said no');
  });
});

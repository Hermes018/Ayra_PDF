// "Export takeoff (CSV)" gives each export its own free name (the backend refuses an existing file) and says so
// honestly when the backend refuses.
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, cleanup, fireEvent, waitFor } from '@testing-library/svelte';

const calls: Array<{ outputPath: string }> = [];
let refuse = false;
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(async (cmd: string, args: { outputPath?: string; path?: string }) => {
    if (cmd === 'studio_export_takeoff_csv') {
      if (refuse) throw 'OUTPUT_PATH_UNSAFE: a file already exists at the output path (x). Nothing was written.';
      calls.push({ outputPath: args.outputPath ?? '' });
      return null;
    }
    if (cmd === 'path_exists') return false;
    if (cmd === 'read_file_bytes') throw new Error('No such file or directory (os error 2)');
    return null;
  }),
}));

import AnnotationLedger from './AnnotationLedger.svelte';
import { inserts } from '$lib/core/insertEngine.svelte';
import { resetIssuedOutputNamesForTests } from '$lib/views/studio/outputNames';

const notes: string[] = [];
beforeEach(() => {
  calls.length = 0;
  notes.length = 0;
  refuse = false;
  resetIssuedOutputNamesForTests();
  inserts.ledgerOpen = true;
});
afterEach(() => {
  cleanup();
  inserts.ledgerOpen = false;
});

describe('ledger takeoff export', () => {
  it('two exports write two distinct, collision-safe CSV names (never the PDF)', async () => {
    render(AnnotationLedger, { activePdfPath: '/docs/plan.pdf', onNotice: (m: string) => notes.push(m) });
    const btn = () => screen.getByRole('button', { name: 'Export takeoff (CSV)' });
    await fireEvent.click(btn());
    await waitFor(() => expect(calls).toHaveLength(1));
    await fireEvent.click(btn());
    await waitFor(() => expect(calls).toHaveLength(2));
    expect(calls[0].outputPath).not.toBe(calls[1].outputPath);
    for (const c of calls) {
      expect(c.outputPath).toMatch(/^\/docs\/plan\.takeoff-\d{8}-\d{6}-\d{3}(-\d+)?\.csv$/);
    }
  });

  it('a backend refusal is reported, not shown as an export', async () => {
    refuse = true;
    render(AnnotationLedger, { activePdfPath: '/docs/plan.pdf', onNotice: (m: string) => notes.push(m) });
    await fireEvent.click(screen.getByRole('button', { name: 'Export takeoff (CSV)' }));
    await waitFor(() => expect(notes.length).toBeGreaterThan(0));
    expect(notes.join(' ')).not.toMatch(/^Exported/);
    expect(notes.join(' ')).toMatch(/Couldn't export the takeoff/);
  });
});

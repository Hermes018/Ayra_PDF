// The takeoff CSV export must not report success when the backend refused the path (it used to swallow every error).
import { describe, it, expect, vi } from 'vitest';

const invoke = vi.fn();
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...a: unknown[]) => invoke(...a) }));

import { inserts } from './insertEngine.svelte';

describe('exportTakeoffCsv', () => {
  it('passes the protected (open) paths and the overwrite flag to the command', async () => {
    invoke.mockResolvedValue(undefined);
    await inserts.exportTakeoffCsv('/docs/a.takeoff-1.csv', ['/docs/a.pdf']);
    expect(invoke).toHaveBeenCalledWith('studio_export_takeoff_csv', expect.objectContaining({
      outputPath: '/docs/a.takeoff-1.csv',
      protectedPaths: ['/docs/a.pdf'],
    }));
  });
  it('a refused path (OUTPUT_PATH_UNSAFE) is thrown, not swallowed', async () => {
    invoke.mockRejectedValueOnce('OUTPUT_PATH_UNSAFE: a file already exists at the output path (/x.csv). Nothing was written.');
    await expect(inserts.exportTakeoffCsv('/x.csv', [])).rejects.toThrow(/OUTPUT_PATH_UNSAFE/);
  });
  it('other errors stay non-fatal (pure frontend runs without a backend)', async () => {
    invoke.mockRejectedValueOnce(new Error('no backend'));
    await expect(inserts.exportTakeoffCsv('/x.csv', [])).resolves.toBeDefined();
  });
});

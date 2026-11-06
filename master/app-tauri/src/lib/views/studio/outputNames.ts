/**
 * BUG-021: collision-safe names for Studio and Organize outputs (protocol DO-5: write a NEW file
 * by default; the backend's atomic_write REPLACES an existing target, so we must never pick one).
 *
 * `x.pdf` → `x.annotated-20261106-054800-123.pdf`. Saving again (also from a saved copy) gives a
 * fresh name instead of overwriting the previous output, earlier suffixes are not chained, names
 * issued in this run are never reused, and `freeOutputPath` also checks the disk.
 */
import { invoke } from '@tauri-apps/api/core';

/** Any short lowercase word naming what the output is (annotated, organized, rotated, compressed, ...). */
export type OutputTag = string;

const SUFFIX_CHAIN = /(\.(annotated|flattened|edited|organized|organizing)(-\d{8}-\d{6}-\d{3}(-\d+)?)?)+$/i;
const issued = new Set<string>();
/** Candidates tried before giving up (each one is a distinct name; this only bounds a broken disk check). */
const MAX_CANDIDATES = 1000;

function pad(n: number, width = 2): string {
  return String(n).padStart(width, '0');
}

function stamp(d: Date): string {
  return (
    `${d.getFullYear()}${pad(d.getMonth() + 1)}${pad(d.getDate())}-` +
    `${pad(d.getHours())}${pad(d.getMinutes())}${pad(d.getSeconds())}-${pad(d.getMilliseconds(), 3)}`
  );
}

function candidates(inputPath: string, tag: OutputTag, now: Date, ext = 'pdf'): string[] {
  const hasPdfExt = /\.pdf$/i.test(inputPath);
  // An input without a .pdf extension keeps its whole name as the stem (the output is never the input).
  const stem = (hasPdfExt ? inputPath.slice(0, -4) : inputPath).replace(SUFFIX_CHAIN, '');
  const base = `${stem}.${tag}-${stamp(now)}`;
  const out = [`${base}.${ext}`];
  for (let n = 2; n <= MAX_CANDIDATES; n++) out.push(`${base}-${n}.${ext}`);
  return out;
}

/** Name only (no disk check). Prefer `freeOutputPath` before writing. */
export function newOutputPath(inputPath: string, tag: OutputTag, now: Date = new Date(), ext = 'pdf'): string {
  for (const c of candidates(inputPath, tag, now, ext)) {
    if (!issued.has(c) && c !== inputPath) {
      issued.add(c);
      return c;
    }
  }
  throw new Error(`No free output name for ${inputPath}`);
}

/** The pre-path_exists check: read_file_bytes fails fast with "not found" for a free name. Any
 *  other error (permissions, …) counts as "taken", so we never risk overwriting. */
async function readProbeExists(path: string): Promise<boolean> {
  try {
    await invoke('read_file_bytes', { path });
    return true;
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e);
    return !/no such file|not found|os error 2\b|cannot find/i.test(msg);
  }
}

/**
 * Whether a path exists. Uses the backend's `path_exists` (stat only). It reports `false` for stat
 * errors too, so a `false` is double-checked with the read probe (an unreadable path counts as
 * taken). If the command is unavailable, the read probe alone decides.
 */
export async function pathExistsOnDisk(path: string): Promise<boolean> {
  let answer: unknown = null;
  try {
    answer = await invoke('path_exists', { path });
  } catch {
    answer = null;
  }
  if (answer === true) return true;
  return readProbeExists(path);
}

/** A new output name that was not issued in this run and does not exist on disk. Names are
 *  reserved before the disk check, so concurrent callers never get the same name. */
export async function freeOutputPath(
  inputPath: string,
  tag: OutputTag,
  now: Date = new Date(),
  exists: (path: string) => Promise<boolean> = pathExistsOnDisk,
  ext = 'pdf'
): Promise<string> {
  for (const c of candidates(inputPath, tag, now, ext)) {
    if (issued.has(c) || c === inputPath) continue;
    issued.add(c); // reserve
    if (await exists(c)) continue; // taken on disk: stays reserved, never offered again
    return c;
  }
  throw new Error(`Could not find a free output file name next to ${inputPath}.`);
}

/**
 * A readable sibling name for fields the user can edit (summary, translation, workflow output): `x.pdf` +
 * `_summary.md` → `x_summary.md`; an input without `.pdf` keeps its whole name (`doc` → `doc_summary.md`), so the
 * result is never the input. The backend also refuses an output that is the input or an existing file.
 */
export function siblingOutputPath(inputPath: string, suffix: string): string {
  const stem = /\.pdf$/i.test(inputPath) ? inputPath.slice(0, -4) : inputPath;
  return `${stem}${suffix}`;
}

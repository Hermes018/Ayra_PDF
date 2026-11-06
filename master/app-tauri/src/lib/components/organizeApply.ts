/**
 * BUG-014: Organize Grid "Apply" — plan and run the page jobs in order, waiting for each one to
 * finish before the next uses its output, and report the real outcome.
 *
 * Before: jobs were only *submitted* (submit_job returns at once), so the rotate step ran on a
 * file that did not exist yet, failed in the background, and the UI still said "success".
 */
import type { JobRequest } from '$lib/bindings/JobRequest';
import { invoke } from '@tauri-apps/api/core';
import { freeOutputPath, type OutputTag } from '$lib/views/studio/outputNames';

/** Engines contract: removes a `<stem>.organizing-<token>.pdf` this session wrote. */
async function deleteIntermediateViaBackend(path: string): Promise<void> {
  await invoke('delete_organize_intermediate', { path });
}

export interface OrganizePage {
  /** 1-based page number in the source file. */
  originalIndex: number;
  /** Clockwise degrees: 0, 90, 180 or 270. */
  rotation: number;
  deleted: boolean;
}

export interface OrganizeStep {
  label: string;
  request: JobRequest;
}

export type OrganizeOutcome =
  | { ok: true; outputPath: string | null; steps: string[] }
  | { ok: false; failedStep: string | null; error: string; completedSteps: string[] };

/** Runs one job to completion. Resolves on success; rejects with the job's error otherwise. */
export type RunJob = (request: JobRequest) => Promise<void>;

/** Normalise clockwise degrees to the backend's -90 / 90 / 180. */
function normaliseDegrees(rotation: number): number {
  const deg = ((rotation % 360) + 360) % 360;
  return deg > 180 ? deg - 360 : deg;
}

export interface OrganizeNames {
  /** Only used when pages are both reordered/deleted AND rotated. */
  intermediate: string;
  final: string;
}

/**
 * Builds the ordered job list (at most two jobs, at most one intermediate file):
 * 1. reorder/delete → ExtractPages / DeletePages;
 * 2. all rotations in ONE RunPipeline job (its per-step files live in a backend stage dir that the
 *    backend removes). Throws a user-facing Error for impossible requests.
 */
export function planOrganizeJobs(filePath: string, pages: OrganizePage[], names: OrganizeNames): OrganizeStep[] {
  const kept = pages.filter((p) => !p.deleted);
  if (kept.length === 0) {
    throw new Error('Cannot delete every page: keep at least one.');
  }
  // Rotations refer to positions in the file produced by the page step.
  const byDegree = new Map<number, number[]>();
  kept.forEach((p, newIndex) => {
    const deg = normaliseDegrees(p.rotation);
    if (deg === 0) return;
    byDegree.set(deg, [...(byDegree.get(deg) ?? []), newIndex + 1]);
  });
  const hasRotation = byDegree.size > 0;

  const steps: OrganizeStep[] = [];
  let current = filePath;
  const isReordered = pages.some((p, i) => p.originalIndex !== i + 1);
  const hasDeleted = pages.some((p) => p.deleted);
  const pageOut = hasRotation ? names.intermediate : names.final;
  if (hasDeleted && !isReordered) {
    steps.push({
      label: `Delete ${pages.length - kept.length} page(s)`,
      request: {
        op: 'DeletePages',
        input_path: current,
        output_path: pageOut,
        pages: pages.filter((p) => p.deleted).map((p) => p.originalIndex),
      },
    });
    current = pageOut;
  } else if (isReordered) {
    steps.push({
      label: hasDeleted ? 'Reorder and delete pages' : 'Reorder pages',
      request: {
        op: 'ExtractPages',
        input_path: current,
        output_path: pageOut,
        pages: kept.map((p) => p.originalIndex),
      },
    });
    current = pageOut;
  }

  if (hasRotation) {
    const rot = [...byDegree].map(([degrees, pgs]) => ({ degrees, pages: pgs }));
    steps.push({
      label: rot.map((r) => `Rotate page(s) ${r.pages.join(', ')} by ${r.degrees}°`).join('; '),
      request: {
        op: 'RunPipeline',
        input_path: current,
        output_path: names.final,
        steps: rot.map((r) => ({ type: 'Rotate' as const, degrees: r.degrees, pages: r.pages })),
        preset_name: null,
      },
    });
  }
  return steps;
}

/**
 * Runs the steps strictly in order; stops at the first failure and says which step failed.
 * Output names are new, collision-safe files (never the input, never an existing file: DO-5).
 */
export async function applyOrganize(
  filePath: string,
  pages: OrganizePage[],
  runJob: RunJob,
  nameFor: (input: string, tag: OutputTag) => Promise<string> = (input, tag) => freeOutputPath(input, tag),
  deleteIntermediate: (path: string) => Promise<void> = deleteIntermediateViaBackend
): Promise<OrganizeOutcome> {
  let steps: OrganizeStep[];
  try {
    const names = { intermediate: '', final: '' };
    // Plan once with placeholders to learn whether an intermediate is needed.
    const probe = planOrganizeJobs(filePath, pages, { intermediate: 'intermediate.pdf', final: 'final.pdf' });
    names.final = await nameFor(filePath, 'organized');
    if (probe.length > 1) names.intermediate = await nameFor(filePath, 'organizing');
    steps = planOrganizeJobs(filePath, pages, names);
  } catch (e) {
    return { ok: false, failedStep: null, error: errorText(e), completedSteps: [] };
  }
  const done: string[] = [];
  for (const step of steps) {
    try {
      await runJob(step.request);
    } catch (e) {
      return { ok: false, failedStep: step.label, error: errorText(e), completedSteps: done };
    }
    done.push(step.label);
  }
  // The one intermediate (reorder/delete AND rotate) is no longer needed. A refusal never fails
  // the apply: it is logged and the file stays (KNOWN_ISSUES KI-002).
  if (steps.length > 1) {
    const intermediate = outputOf(steps[0].request);
    if (intermediate) {
      try {
        await deleteIntermediate(intermediate);
      } catch (e) {
        console.warn(`Organize: could not delete the intermediate file ${intermediate} (KI-002):`, e);
      }
    }
  }
  const last = steps.at(-1);
  return { ok: true, outputPath: last ? outputOf(last.request) : null, steps: done };
}

function outputOf(req: JobRequest): string | null {
  return 'output_path' in req && typeof req.output_path === 'string' ? req.output_path : null;
}

function errorText(e: unknown): string {
  if (e instanceof Error) return e.message;
  if (typeof e === 'string') return e;
  return String(e);
}

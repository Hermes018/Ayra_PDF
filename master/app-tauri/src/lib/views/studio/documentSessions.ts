/**
 * Document store — the ONLY owner of per-document Studio state (review r4, structural).
 *
 * The Studio engines (ink, measure, viewport calibrations, snap, insert/selection state) are
 * singletons that hold ONE document: the live one. Canvas components never stash, reset or merge
 * engine state themselves; they call `activate(path)` and follow the token it returns:
 *
 *   activate(B):  stash live A (only if A is restored; an unrestored A keeps what was added to it
 *                 as "pending") → reset ALL engines → seq++ → live = B (not restored) → caller loads B.
 *   completeRestore(token, …): if the token is still current, put B's session / saved markups (+ its
 *                 pending additions) into the engines → B is restored → drawing and saving allowed.
 *
 * A canvas that mounts while the requested document is already live and restored (Studio →
 * Organize → Studio) gets mode 'render': the engines are already right, it only paints pages.
 * Every async continuation checks `isCurrent(token)` after each await and aborts if it changed.
 * Undo history is not kept per document: it is cleared on every activate (known limit).
 */
import { ink, type ActiveStroke } from '$lib/core/inkEngine.svelte';
import { measureEngine } from '$lib/core/measureEngine.svelte';
import { viewport } from '$lib/core/viewportEngine.svelte';
import { snapEngine } from '$lib/core/snapEngine.svelte';
import { selection } from '$lib/core/selectionEngine.svelte';
import { inserts } from '$lib/core/insertEngine.svelte';
import { commands } from '$lib/core/commandEngine.svelte';
import { shapes } from '$lib/core/shapeAndHatchEngine.svelte';

export interface DocumentSession {
  strokes: ActiveStroke[];
  measurements: typeof measureEngine.measurements;
  scales: typeof measureEngine.scales;
  calibrations: typeof viewport.calibrations;
  view: { zoom: number; panX: number; panY: number } | null;
}

export interface LoadToken {
  path: string;
  seq: number;
}

const sessions = new Map<string, DocumentSession>();
/** What was added to a document that was left before it was restored (e.g. an early insert). */
const pending = new Map<string, { strokes: ActiveStroke[]; measurements: DocumentSession['measurements'] }>();
const baselines = new Map<string, string>();
let livePath: string | null = null;
let liveRestored = false;
/** Why the live document is not restored (accurate Save refusals, review r5). */
export type LiveState = 'none' | 'loading' | 'restored' | 'failed' | 'locked' | 'hostile';
let liveState: LiveState = 'none';
/** Page count of the live document's layout at restore (Save refuses if the layout differs). */
let livePageCount = 0;
/** Closed documents: never stashed again (review r5 BLOCKER). Reopening clears the mark. */
const closed = new Set<string>();
/** How long a follower pane (Labs split view) waits for the live document before retrying. */
export const FOLLOWER_WAIT = { ms: 30_000 };
let seq = 0;
let liveViewId = 'main-canvas';
const readyWaiters = new Map<string, Array<(ok: boolean) => void>>();
const probed = new Set<string>();
/** Documents whose saved page scales were applied (a late or abandoned load leaves this unset). */
const scalesLoaded = new Set<string>();

/** Default wait for a document to become ready (whenDocumentLive). */
export const DOCUMENT_READY_TIMEOUT_MS = 30_000;

function signature(strokes: readonly ActiveStroke[], measurements: DocumentSession['measurements']): string {
  const s = strokes.map((st) => {
    const a = st.points[0];
    const b = st.points[st.points.length - 1];
    return `${st.id}:${st.points.length}:${a?.x},${a?.y}:${b?.x},${b?.y}:${st.text ?? ''}:${st.color}:${st.widthPt}`;
  });
  const m = measurements.map((me) => `${me.id}:${me.points.length}:${me.calibratedValue}`);
  return `${s.join('|')}#${m.join('|')}`;
}

function liveSignature(): string {
  return signature(ink.completedStrokes, measureEngine.measurements);
}

function mergeById<T extends { id: string }>(...lists: T[][]): T[] {
  const seen = new Set<string>();
  const out: T[] = [];
  for (const list of lists) {
    for (const item of list) {
      if (seen.has(item.id)) continue;
      seen.add(item.id);
      out.push(item);
    }
  }
  return out;
}

/**
 * Every reactive engine field, classified (review r5). `perDocument` fields are reset on every
 * document switch; `app` fields are user/app preferences kept across documents. EngineReset.test.ts
 * fails if a new engine field is not listed here.
 */
export const ENGINE_STATE: Record<string, { perDocument: string[]; app: string[]; derived?: string[] }> = {
  ink: { perDocument: ['currentStroke', 'completedStrokes', 'laserPoints'], app: [] },
  inserts: {
    perDocument: [
      'latexModalOpen',
      'pendingInsertPt',
      'signatureModalOpen',
      'audioModalOpen',
      'isRecordingAudio',
      'audioDurationSec',
      'audioNotes',
      'pastePagesModalOpen',
      'detectedRegions',
    ],
    app: [
      'pageOverlay',
      'ledgerOpen',
      'latexDraft',
      'signatureMode',
      'signatureTypedName',
      'signatureThreshold',
      'liveWaveform',
      'pasteRangeSpec',
      'activeAuthor',
      'authorFilter',
      'workspaceDocs',
      'watchFolderPath',
      'ingestedWatchFiles',
      'markupLibrary',
    ],
  },
  measureEngine: {
    perDocument: ['scales', 'measurements', 'activeDraft', 'calibrationModal', 'ruler'],
    app: ['endpointStyle', 'tapeMode'],
    derived: ['totalsByUnit'],
  },
  shapes: { perDocument: ['activeShape', 'previewPoint', 'editingTextBox'], app: [] },
  selection: {
    perDocument: ['selectedIds', 'marqueeRect', 'lassoPoints'],
    app: ['copiedStyle', 'formatPainterActive'],
    derived: ['hasSelection', 'selectedStrokes', 'selectionBounds'],
  },
  snapEngine: { perDocument: ['activeSnap', 'pageSegments'], app: ['enabled', 'gridSnapEnabled', 'gridStepPt'] },
  viewport: { perDocument: ['calibrations', 'pages'], app: ['views', 'defaultCalibration'] },
  tools: {
    perDocument: [],
    app: ['activeToolId', 'presets', 'favorites'],
    derived: ['currentToolId', 'activePreset', 'activeToolLabel', 'supportsStrokeProperties', 'supportsEraserMode', 'active', 'activeTool'],
  },
  commands: {
    perDocument: [],
    app: ['doc', 'historyLimit', 'shortcuts', 'lastError', 'spaceSpringActive'],
    derived: ['canUndo', 'canRedo', 'undoLabel'],
  },
};

/** Reset EVERY per-document engine field listed in ENGINE_STATE (and the undo history). */
function resetAllEngines(): void {
  ink.cancelStroke();
  ink.completedStrokes = [];
  ink.laserPoints = [];
  inserts.resetDocumentState();
  measureEngine.resetDocumentState();
  shapes.activeShape = null;
  shapes.previewPoint = null;
  shapes.editingTextBox = null;
  selection.selectedIds = [];
  selection.marqueeRect = null;
  selection.lassoPoints = [];
  snapEngine.pageSegments = {};
  snapEngine.clearSnap();
  viewport.calibrations = {};
  viewport.pages = [];
  commands.load(commands.doc); // undo must never cross documents
}

function stashLive(): void {
  if (!livePath || closed.has(livePath)) return; // a closed document is never stored again
  if (liveRestored) {
    const v = viewport.views[liveViewId];
    sessions.set(livePath, {
      strokes: [...ink.completedStrokes],
      measurements: [...measureEngine.measurements],
      scales: { ...measureEngine.scales },
      calibrations: { ...viewport.calibrations },
      view: v ? { zoom: v.zoom, panX: v.panX, panY: v.panY } : null,
    });
  } else if (ink.completedStrokes.length > 0 || measureEngine.measurements.length > 0) {
    // Left before its saved markups were restored: keep what was added meanwhile for its restore.
    const prev = pending.get(livePath);
    pending.set(livePath, {
      strokes: mergeById(prev?.strokes ?? [], [...ink.completedStrokes]),
      measurements: mergeById(prev?.measurements ?? [], [...measureEngine.measurements]),
    });
  }
}

/* ---------- Lifecycle ---------- */

/**
 * Make `path` the live document. Returns the token every async step must check, and whether the
 * caller must load + restore ('load') or only paint pages because the engines already hold the
 * restored document ('render').
 */
export function activate(path: string, viewId = 'main-canvas'): LoadToken & { mode: 'load' | 'render' } {
  liveViewId = viewId;
  closed.delete(path); // (re)opening
  if (livePath === path) {
    if (liveRestored) return { path, seq, mode: 'render' };
    seq++; // an earlier load of this document was abandoned (e.g. its canvas unmounted): restart it
    liveState = 'loading';
    return { path, seq, mode: 'load' };
  }
  stashLive();
  resetAllEngines();
  seq++;
  livePath = path;
  liveRestored = false;
  liveState = 'loading';
  return { path, seq, mode: 'load' };
}

/** No document is shown any more (last tab closed). */
export function deactivate(): void {
  if (livePath === null) return;
  stashLive();
  resetAllEngines();
  seq++;
  livePath = null;
  liveRestored = false;
  liveState = 'none';
}

/**
 * Close `path` through the store (review r5 BLOCKER): it is marked closed BEFORE anything else
 * runs, its stored state is dropped, and if it is live the engines are reset WITHOUT stashing.
 * Reopening it later shows only what is on disk.
 */
export function closeDocument(path: string): void {
  closed.add(path);
  sessions.delete(path);
  pending.delete(path);
  baselines.delete(path);
  scalesLoaded.delete(path);
  if (livePath === path) {
    resetAllEngines();
    seq++;
    livePath = null;
    liveRestored = false;
    liveState = 'none';
  }
}

export function isCurrent(token: LoadToken): boolean {
  return token.seq === seq && token.path === livePath;
}

export function currentToken(): LoadToken | null {
  return livePath ? { path: livePath, seq } : null;
}

/** The stored session for `path`, if it was shown and left before in this run. */
export function documentSession(path: string): DocumentSession | undefined {
  return sessions.get(path);
}

/**
 * Put the document's markups into the engines (after its pages are laid out). `fromDisk` is what
 * studio_load_annotations returned (ignored when a session exists). Pending additions and anything
 * already in the engines are kept. Returns false (and changes nothing) if the token is stale.
 */
export function completeRestore(token: LoadToken, fromDisk: ActiveStroke[] | null, pageCount = viewport.pages.length): boolean {
  if (!isCurrent(token)) return false;
  const path = token.path;
  const session = sessions.get(path);
  const early = pending.get(path);
  pending.delete(path);
  if (session) {
    ink.completedStrokes = mergeById(session.strokes, early?.strokes ?? [], ink.completedStrokes);
    measureEngine.measurements = mergeById(session.measurements, early?.measurements ?? [], measureEngine.measurements);
    measureEngine.scales = { ...session.scales };
    viewport.calibrations = { ...session.calibrations };
    const v = viewport.views[liveViewId];
    if (v && session.view) viewport.upsertView({ ...v, ...session.view });
  } else {
    const disk = fromDisk ?? [];
    baselines.set(path, signature(disk, []));
    ink.completedStrokes = mergeById(disk, early?.strokes ?? [], ink.completedStrokes);
    measureEngine.measurements = mergeById(early?.measurements ?? [], measureEngine.measurements);
  }
  liveRestored = true;
  liveState = 'restored';
  livePageCount = pageCount;
  for (const [p, waiters] of [...readyWaiters]) {
    readyWaiters.delete(p);
    waiters.forEach((w) => w(p === path));
  }
  return true;
}

/** A load attempt failed (`failed`), was cancelled at the password prompt (`locked`), or the engine
 *  refused the file (`hostile`): its waiters get false. */
export function failLoad(token: LoadToken, reason: 'failed' | 'locked' | 'hostile' = 'failed'): void {
  if (isCurrent(token)) {
    if (reason === 'hostile' || liveRestored) {
      // The file must not stay open: drop whatever was restored for it (nothing is stored).
      resetAllEngines();
      liveRestored = false;
    }
    liveState = reason;
  }
  const waiters = readyWaiters.get(token.path) ?? [];
  readyWaiters.delete(token.path);
  waiters.forEach((w) => w(false));
}

/** Close a document: drop its stored state (after the Save/Discard/Cancel prompt). */
export function forgetDocumentSession(path: string): void {
  sessions.delete(path);
  pending.delete(path);
  baselines.delete(path);
  scalesLoaded.delete(path);
}

export function liveDocumentState(): LiveState {
  return liveState;
}

/** True when the live layout is the one the markups were restored into (safe page geometry). */
export function liveGeometryOk(): boolean {
  return liveRestored && livePageCount > 0 && viewport.pages.length === livePageCount;
}

/** True when `path` is live and its markups are in the engines (drawing and saving allowed). */
export function isDocumentRestored(path: string): boolean {
  return livePath === path && liveRestored;
}

/**
 * Resolves true when `path` is ready (restored); false when another document becomes ready first
 * (superseded), its load fails or is cancelled, or after `timeoutMs`. Never hangs.
 */
export function whenDocumentLive(path: string, timeoutMs = DOCUMENT_READY_TIMEOUT_MS): Promise<boolean> {
  if (isDocumentRestored(path)) return Promise.resolve(true);
  return new Promise((resolve) => {
    let done = false;
    const finish = (ok: boolean) => {
      if (done) return;
      done = true;
      clearTimeout(timer);
      const list = (readyWaiters.get(path) ?? []).filter((w) => w !== finish);
      if (list.length > 0) readyWaiters.set(path, list);
      else readyWaiters.delete(path);
      resolve(ok);
    };
    const timer = setTimeout(() => finish(false), timeoutMs);
    readyWaiters.set(path, [...(readyWaiters.get(path) ?? []), finish]);
  });
}

/* ---------- Unsaved-changes tracking (F11 close guard) ---------- */

/** Signature of what is in the engines now, for `path` (null when `path` is not restored). */
export function documentSnapshot(path: string): string | null {
  return isDocumentRestored(path) ? liveSignature() : null;
}

/** Record `snapshot` (from documentSnapshot) as `path`'s saved state, i.e. what a Save wrote. */
export function markDocumentSavedAs(path: string, snapshot: string): void {
  baselines.set(path, snapshot);
}

/** True when the document has markups or measurements that are not saved. */
export function isDocumentDirty(path: string): boolean {
  const early = pending.get(path);
  if (early && (early.strokes.length > 0 || early.measurements.length > 0)) return true;
  if (livePath === path && !liveRestored && (ink.completedStrokes.length > 0 || measureEngine.measurements.length > 0)) {
    return true; // added while loading
  }
  const session = sessions.get(path);
  let current: string | null = null;
  if (isDocumentRestored(path)) current = liveSignature();
  else if (session) current = signature(session.strokes, session.measurements);
  if (current === null) return false; // never restored in this run and nothing added: nothing to lose
  return current !== (baselines.get(path) ?? signature([], []));
}

/* ---------- Exactly one saver, owned by the store ---------- */

let saveImpl: ((path: string) => Promise<boolean>) | null = null;

/** Wired once by studioSave.ts at module load, never by canvas panes. */
export function setStoreSaver(fn: (path: string) => Promise<boolean>): void {
  saveImpl = fn;
}

/** Save `path` (it must be the live, restored document). Resolves false when it could not be saved. */
export async function saveDocument(path: string): Promise<boolean> {
  return saveImpl ? saveImpl(path) : false;
}

/** True once `path`'s saved page scales were applied to it (so a session's scales then win). */
export function areScalesLoaded(path: string): boolean {
  return scalesLoaded.has(path);
}

export function markScalesLoaded(token: LoadToken): void {
  if (isCurrent(token)) scalesLoaded.add(token.path);
}

/** True the first time a document is opened in this run (the backend health probe runs once). */
export function shouldProbeDocument(path: string): boolean {
  if (probed.has(path)) return false;
  probed.add(path);
  return true;
}

/** Test helper: forget every document (the engines are not touched). */
export function clearDocumentSessions(): void {
  sessions.clear();
  pending.clear();
  baselines.clear();
  probed.clear();
  scalesLoaded.clear();
  closed.clear();
  livePath = null;
  liveRestored = false;
  liveState = 'none';
  livePageCount = 0;
  seq++;
  for (const [p, waiters] of [...readyWaiters]) {
    readyWaiters.delete(p);
    waiters.forEach((w) => w(false));
  }
}

/** Test helper for the waiter unit tests: simulate a finished (ok) or failed load of `path`. */
export function notifyDocumentLoaded(path: string, ok: boolean): void {
  if (!ok) {
    failLoad({ path, seq });
    return;
  }
  if (livePath !== path) {
    seq++;
    livePath = path;
  }
  liveRestored = false;
  completeRestore({ path, seq }, null);
}

// Inserts (stamps, signatures, notes, paste-to-pages…) wait until the live document is restored.
inserts.insertGuard = () => livePath === null || liveRestored;

/**
 * Labs (experimental) Studio features — PROTOTYPE_SCOPE.md: hidden and unusable unless the user
 * turns on Settings → Labs (`appState.labsEnabled`, OFF by default). This module is the ONE gate:
 * components ask `labsVisible()` / `isLabsOn()`, select tools through `selectToolIfAllowed()`,
 * and `attachLabsGate()` (mounted once by routes/+page.svelte) keeps shortcuts and the active tool
 * consistent with the setting.
 *
 * The copy below is deliberately honest about what the code does
 * (DRAWBOARD_FEATURE_CHECKLIST 2.16 and 8.10 are PARTIAL).
 */
import { appState } from '$lib/state.svelte';
import { tools } from '$lib/core/toolEngine.svelte';
import { commands } from '$lib/core/commandEngine.svelte';
import { inserts } from '$lib/core/insertEngine.svelte';

/** Feature id for the ledger's ink "shape-to-character guess" action (8.10). */
export const INK_GUESS_ID = 'labs.inkGuess';

/** Tool ids (and feature ids) that only exist when Labs is on. */
/** Feature id for the Studio split view (review r4: cut from the default UI). */
export const SPLIT_VIEW_ID = 'studio.splitView';

/** Edit view: adds a fixed example highlight (a demo, not a real highlighter). */
export const SAMPLE_HIGHLIGHT_ID = 'labs.sampleHighlight';

/** Batch queue: submits fixed sample jobs on made-up file paths (a demo, not a feature). */
export const SAMPLE_BATCH_ID = 'labs.sampleBatch';

export const LABS_IDS: readonly string[] = ['insert.latex', INK_GUESS_ID, SPLIT_VIEW_ID, SAMPLE_HIGHLIGHT_ID, SAMPLE_BATCH_ID];

export const SAMPLE_BATCH_LABS = {
  name: 'Sample batch',
  label: 'Sample batch (Labs)',
  note: 'Queues three example jobs on made-up file paths so the queue has something to show. They will fail because the files do not exist.',
} as const;

export const SAMPLE_HIGHLIGHT_LABS = {
  name: 'Sample highlight',
  label: 'Sample highlight (Labs)',
  note: 'Adds a highlight in a fixed box near the top of page 1. It does not highlight real text; use Drawboard Studio for that.',
} as const;

/** Studio split view: two panes of the SAME document (a second document opens as a tab). */
export const SPLIT_VIEW_LABS = {
  name: 'Split view',
  label: 'Split view (experimental)',
  note: 'Shows the current document in two panes side by side. Draw in the left pane; the right pane repaints the same document. Other documents open as tabs.',
} as const;

export const LABS_BADGE = 'Labs';

/** 2.16 — `insert.latex` (`compileLatexToVector`). */
export const MATH_TEXT_LABS = {
  name: 'Math text',
  label: 'Math text (Labs)',
  note:
    'Converts \\frac, \\sqrt, Greek letters and ^/_ into Unicode text and places it as a text box. ' +
    'It is not a TeX renderer.',
  inputLabel: 'Math text source (LaTeX-style)',
  insertAction: 'Insert math text',
} as const;

/**
 * Every character `recognizeHandwritingStrokesToText` (core/insertEngine.svelte.ts) can emit for a
 * stroke without text. LabsGating.test.ts checks this list against the classifier itself.
 */
export const INK_GUESS_CHARS: readonly string[] = ['O', 'I', '-', 'L', 'A', 'e', '.'];

const quotedChars = INK_GUESS_CHARS.map((c) => `"${c}"`);

/** 8.10 — `convertSelectedInkToText` / `recognizeHandwritingStrokesToText`. */
export const INK_GUESS_LABS = {
  name: 'Shape-to-character guess',
  label: 'Shape-to-character guess (Labs)',
  note:
    'Replaces the selected ink strokes (or the last stroke, if none is selected) with one text box ' +
    'reading "Ink Note: " plus one guessed character per stroke: ' +
    `${quotedChars.slice(0, -1).join(', ')} or ${quotedChars[quotedChars.length - 1]}, ` +
    'picked from the stroke’s shape and point count. A stroke that already has text keeps that text. ' +
    'It does not recognise handwriting.',
} as const;

const LABS_TOOL_COPY: Record<string, { label: string; note: string }> = {
  'insert.latex': MATH_TEXT_LABS,
  [INK_GUESS_ID]: INK_GUESS_LABS,
  [SAMPLE_HIGHLIGHT_ID]: SAMPLE_HIGHLIGHT_LABS,
  [SAMPLE_BATCH_ID]: SAMPLE_BATCH_LABS,
};

/** Normalise preset ids (`preset.x`) and shortcut commands (`tool.x`) to the bare tool id. */
function baseId(id: string): string {
  let out = id;
  if (out.startsWith('tool.')) out = out.slice('tool.'.length);
  if (out.startsWith('preset.')) out = out.slice('preset.'.length);
  return out;
}

/** The Labs setting. Reactive. */
export function isLabsOn(): boolean {
  return appState.labsEnabled;
}

export function isLabsTool(id: string): boolean {
  return LABS_IDS.includes(baseId(id));
}

/** True when the tool / preset / `tool.*` command / Labs feature may be shown or used. Reactive. */
export function labsVisible(id: string): boolean {
  return isLabsOn() || !isLabsTool(id);
}

/** Select a tool unless it is a Labs tool and Labs is off. Returns whether it was selected. */
export function selectToolIfAllowed(id: string): boolean {
  if (!labsVisible(id)) return false;
  tools.selectTool(id);
  return true;
}

/** Honest display label for a tool; falls back to the caller's label for non-Labs tools. */
export function studioToolLabel(id: string, fallback: string): string {
  return LABS_TOOL_COPY[baseId(id)]?.label ?? fallback;
}

/** "Label: what it really does" tooltip for a Labs tool; `undefined` for every other tool. */
export function labsToolNote(id: string): string | undefined {
  const copy = LABS_TOOL_COPY[baseId(id)];
  return copy ? `${copy.label}: ${copy.note}` : undefined;
}

/* ---------- Gate enforcement ---------- */

/** Shortcuts bound to Labs tools, unbound while Labs is off so their chords do nothing. */
let parkedShortcuts: Record<string, string> = {};

/**
 * Make engine state agree with the setting. Idempotent; call after anything that may bring a
 * Labs tool back (toggling Labs, restoring saved prefs). `attachLabsGate()` also runs it reactively.
 */
export function enforceLabsGate(): void {
  if (isLabsOn()) {
    const parked = Object.entries(parkedShortcuts);
    if (parked.length === 0) return;
    parkedShortcuts = {};
    for (const [chord, cmd] of parked) {
      // Do not overwrite a chord the user rebound to something else meanwhile.
      if (!(chord in commands.shortcuts)) commands.bind(chord, cmd);
    }
    return;
  }
  for (const [chord, cmd] of Object.entries(commands.shortcuts)) {
    if (isLabsTool(cmd)) {
      parkedShortcuts = { ...parkedShortcuts, [chord]: cmd };
      commands.unbind(chord);
    }
  }
  if (isLabsTool(tools.currentToolId)) {
    tools.selectTool('pan');
  }
  if (inserts.latexModalOpen) {
    inserts.latexModalOpen = false;
  }
}

/** Shortcuts to persist: the live map plus any Labs bindings parked while Labs is off. */
export function shortcutsForSave(): Record<string, string> {
  return { ...parkedShortcuts, ...commands.shortcuts };
}

/** Turn Labs on/off and apply it at once. */
export function setLabs(on: boolean): void {
  appState.setLabsEnabled(on);
  enforceLabsGate();
}

/** Install the reactive gate (once, at the app root). Returns a detach function. */
export function attachLabsGate(): () => void {
  return $effect.root(() => {
    $effect(() => {
      // Tracks appState.labsEnabled, commands.shortcuts and tools.currentToolId.
      enforceLabsGate();
    });
  });
}

/** Test helper: forget parked shortcuts. */
export function resetLabsGateForTests(): void {
  parkedShortcuts = {};
}

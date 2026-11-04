// app-tauri/src/lib/core/commandEngine.svelte.ts
import type { ControlMessage } from '$lib/bindings/ControlMessage';
import type { SceneNode } from '$lib/bindings/SceneNode';

export interface DocSnapshot {
  readonly rev: number;
  readonly pages: ReadonlyArray<ReadonlyMap<string, SceneNode>>;
}

export interface CommandResult {
  next: DocSnapshot;
  label: string;
  coalesceKey?: string;
}

export type CommandHandler = (doc: DocSnapshot, args: unknown) => CommandResult | { error: string };
export type Chord = string; // canonical: "Ctrl+Alt+Shift+Meta+Key"

const HISTORY_LIMIT_DEFAULT = 500; // user-configurable via prefs; older entries are dropped from memory, not the file

export function canonicalChord(e: Pick<KeyboardEvent, 'ctrlKey' | 'altKey' | 'shiftKey' | 'metaKey' | 'key'>): Chord {
  const key = e.key.length === 1 ? e.key.toUpperCase() : e.key;
  return [e.ctrlKey && 'Ctrl', e.altKey && 'Alt', e.shiftKey && 'Shift', e.metaKey && 'Meta', key]
    .filter(Boolean)
    .join('+');
}

export function withPage(doc: DocSnapshot, page: number, edit: (m: Map<string, SceneNode>) => void): DocSnapshot {
  const src = doc.pages[page];
  if (!src) return doc;
  const copy = new Map(src);
  edit(copy);
  const pages = doc.pages.slice();
  pages[page] = copy;
  return Object.freeze({ rev: doc.rev + 1, pages: Object.freeze(pages) });
}

export type PointerSink = ((m: Extract<ControlMessage, { kind: 'pointer' }>) => void) | null;
export type RenderSink = ((m: Extract<ControlMessage, { kind: 'render_page' }>) => void) | null;

export class CommandBusAndShortcutEngine {
  doc = $state.raw<DocSnapshot>(Object.freeze({ rev: 0, pages: Object.freeze([]) }));
  #past = $state.raw<ReadonlyArray<{ doc: DocSnapshot; label: string; key?: string }>>([]);
  #future = $state.raw<ReadonlyArray<{ doc: DocSnapshot; label: string }>>([]);
  historyLimit = $state(HISTORY_LIMIT_DEFAULT);
  shortcuts = $state<Record<Chord, string>>({});
  lastError = $state<string | null>(null);
  readonly canUndo = $derived(this.#past.length > 0);
  readonly canRedo = $derived(this.#future.length > 0);
  readonly undoLabel = $derived(this.#past.at(-1)?.label ?? null);
  #handlers = new Map<string, CommandHandler>();
  #pointerSink: PointerSink = null;
  #renderSink: RenderSink = null;
  readonly journal: ControlMessage[] = []; // headless replay/verification trace (cleared per session)

  register(id: string, h: CommandHandler): void {
    this.#handlers.set(id, h);
  }

  setPointerSink(f: PointerSink): void {
    this.#pointerSink = f;
  }

  setRenderSink(f: RenderSink): void {
    this.#renderSink = f;
  }

  load(doc: DocSnapshot): void {
    this.doc = doc;
    this.#past = [];
    this.#future = [];
  }

  dispatch(msg: ControlMessage): boolean {
    this.journal.push(msg);
    switch (msg.kind) {
      case 'execute':
        return this.#execute(msg.command, msg.args);
      case 'pointer':
        this.#pointerSink?.(msg);
        return this.#pointerSink !== null;
      case 'ui.key': {
        if (msg.phase !== 'down') return false;
        const cmd = this.shortcuts[msg.chord];
        return cmd ? this.#execute(cmd, null) : false;
      }
      case 'render_page':
        this.#renderSink?.(msg);
        return this.#renderSink !== null;
    }
  }

  #execute(command: string, args: unknown): boolean {
    if (command === 'history.undo') return this.undo();
    if (command === 'history.redo') return this.redo();
    const h = this.#handlers.get(command);
    if (!h) {
      this.lastError = `unknown_command:${command}`;
      return false;
    }
    const r = h(this.doc, args);
    if ('error' in r) {
      this.lastError = r.error;
      return false;
    }
    if (r.next === this.doc) return true; // no-op: nothing recorded
    const top = this.#past.at(-1);
    const past = r.coalesceKey && top?.key === r.coalesceKey
      ? this.#past
      : [...this.#past, { doc: this.doc, label: r.label, key: r.coalesceKey }];
    this.#past = past.length > this.historyLimit ? past.slice(past.length - this.historyLimit) : past;
    this.#future = [];
    this.doc = r.next;
    this.lastError = null;
    return true;
  }

  undo(): boolean {
    const top = this.#past.at(-1);
    if (!top) return false;
    this.#future = [...this.#future, { doc: this.doc, label: top.label }];
    this.#past = this.#past.slice(0, -1);
    this.doc = top.doc;
    return true;
  }

  redo(): boolean {
    const top = this.#future.at(-1);
    if (!top) return false;
    this.#past = [...this.#past, { doc: this.doc, label: top.label }];
    this.#future = this.#future.slice(0, -1);
    this.doc = top.doc;
    return true;
  }

  bind(chord: Chord, command: string): { conflict: string | null } {
    const conflict = this.shortcuts[chord] ?? null;
    this.shortcuts = { ...this.shortcuts, [chord]: command };
    return { conflict };
  }

  unbind(chord: Chord): void {
    const { [chord]: _d, ...rest } = this.shortcuts;
    this.shortcuts = rest;
  }

  /** Single global keyboard entry point (protocol-lint forbids keydown listeners elsewhere). */
  attachKeyboard(target: Window): () => void {
    const on = (e: KeyboardEvent): void => {
      const el = e.target as HTMLElement | null;
      if (el && (el.isContentEditable || el.tagName === 'INPUT' || el.tagName === 'TEXTAREA')) return;
      if (this.dispatch({ kind: 'ui.key', chord: canonicalChord(e), phase: 'down' })) e.preventDefault();
    };
    target.addEventListener('keydown', on);
    return () => target.removeEventListener('keydown', on);
  }
}

export const commands = new CommandBusAndShortcutEngine();

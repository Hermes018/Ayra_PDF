// app-tauri/src/lib/core/commandEngine.svelte.ts
import type { ControlMessage } from '$lib/bindings/ControlMessage';
import type { SceneNode } from '$lib/bindings/SceneNode';
import defaultShortcuts from './defaultShortcuts.json';
import { tools, type ToolPreset } from './toolEngine.svelte';

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

export type StudioControlMessage =
  | ControlMessage
  | { kind: 'tool.select'; toolId?: string; id?: string }
  | { kind: 'style.apply'; patch?: Partial<ToolPreset>; [key: string]: unknown };

export class CommandBusAndShortcutEngine {
  doc = $state.raw<DocSnapshot>(Object.freeze({ rev: 0, pages: Object.freeze([]) }));
  #past = $state.raw<ReadonlyArray<{ doc: DocSnapshot; label: string; key?: string }>>([]);
  #future = $state.raw<ReadonlyArray<{ doc: DocSnapshot; label: string }>>([]);
  historyLimit = $state(HISTORY_LIMIT_DEFAULT);
  shortcuts = $state<Record<Chord, string>>({ ...(defaultShortcuts as Record<Chord, string>) });
  lastError = $state<string | null>(null);
  readonly canUndo = $derived(this.#past.length > 0);
  readonly canRedo = $derived(this.#future.length > 0);
  readonly undoLabel = $derived(this.#past.at(-1)?.label ?? null);
  #handlers = new Map<string, CommandHandler>();
  #pointerSink: PointerSink = null;
  #renderSink: RenderSink = null;
  #chordCaptureResolver: ((chord: Chord) => void) | null = null;
  onRadialToggle: (() => void) | null = null;
  onRadialKey: ((key: string) => boolean) | null = null;
  spaceSpringActive = $state(false);
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

  dispatch(msg: StudioControlMessage): boolean {
    this.journal.push(msg as ControlMessage);
    if (msg.kind === 'tool.select') {
      const tid = msg.toolId ?? msg.id;
      if (tid) tools.selectTool(tid);
      return true;
    }
    if (msg.kind === 'style.apply') {
      const patch = msg.patch ?? (msg as Partial<ToolPreset>);
      tools.updateActivePreset(patch);
      return true;
    }

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
    return false;
  }

  #execute(command: string, args: unknown): boolean {
    if (command === 'history.undo') return this.undo();
    if (command === 'history.redo') return this.redo();
    if (command === 'radial.open') {
      this.onRadialToggle?.();
      return true;
    }
    if (command === 'tool.select') {
      const tid = typeof args === 'string'
        ? args
        : ((args as { toolId?: string; id?: string })?.toolId ?? (args as { id?: string })?.id);
      if (tid) tools.selectTool(tid);
      return true;
    }
    if (command.startsWith('tool.')) {
      const tid = command.replace(/^tool\./, '');
      tools.selectTool(tid);
      return true;
    }
    if (command === 'style.apply') {
      const patch = (args as { patch?: Partial<ToolPreset> })?.patch ?? (args as Partial<ToolPreset>);
      tools.updateActivePreset(patch);
      return true;
    }
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

  resetShortcuts(): void {
    this.shortcuts = { ...(defaultShortcuts as Record<Chord, string>) };
  }

  captureNextChord(): Promise<Chord> {
    return new Promise((resolve) => {
      this.#chordCaptureResolver = resolve;
    });
  }

  cancelCaptureChord(): void {
    this.#chordCaptureResolver = null;
  }

  /** Single global keyboard entry point (protocol-lint forbids keydown listeners elsewhere). */
  attachKeyboard(target: Window): () => void {
    const on = (e: KeyboardEvent): void => {
      if (this.#chordCaptureResolver) {
        e.preventDefault();
        e.stopPropagation();
        const chord = canonicalChord(e);
        const resolver = this.#chordCaptureResolver;
        this.#chordCaptureResolver = null;
        resolver(chord);
        return;
      }
      const el = e.target as HTMLElement | null;
      if (el && (el.isContentEditable || el.tagName === 'INPUT' || el.tagName === 'TEXTAREA')) return;
      if (this.onRadialKey?.(e.key)) {
        e.preventDefault();
        return;
      }
      if (e.code === 'Space' && !this.spaceSpringActive) {
        this.spaceSpringActive = true;
        tools.springLoad('preset.pan');
        e.preventDefault();
        return;
      }
      if (this.dispatch({ kind: 'ui.key', chord: canonicalChord(e), phase: 'down' })) e.preventDefault();
    };
    const off = (e: KeyboardEvent): void => {
      if (e.code === 'Space' && this.spaceSpringActive) {
        this.spaceSpringActive = false;
        tools.springLoad(null);
      }
    };
    target.addEventListener('keydown', on);
    target.addEventListener('keyup', off);
    return () => {
      target.removeEventListener('keydown', on);
      target.removeEventListener('keyup', off);
    };
  }
}

export const commands = new CommandBusAndShortcutEngine();


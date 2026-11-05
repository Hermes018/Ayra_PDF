# DRAWBOARD STUDIO MASTER PLAN — Drawboard-Class Spatial PDF Editor + ArtCraft Engine Federation

- **Phase:** 14.0 (planning). **ADR:** `docs/adr/ADR-012-drawboard-artcraft-spatial-studio.md` (PROPOSED).
- **Paused track:** Windows W2–W4 (resume at `../windows/docs/WINDOWS_MASTER_PLAN.md` §Phase W2 from commit `ee1d552`).
- **Executor:** Gemini 3.8 Flash (or any agent) following `AGENTS.md` boot → this file, one phase at a time.
- **Binding protocol:** `protocol/02_RULES_DOS_DONTS.md`, `protocol/03_FAILSAFES.md`. If this plan conflicts with them, the protocol wins.

> [!IMPORTANT]
> **Entry gate for D1:** (a) the user approves ADR-012; (b) the D1–D6 tools are appended to `protocol/07_FEATURE_MAP.md` and `logs/PLAN_LOG.md` with that approval; (c) the `storytold/*` licence + API audit (§9 D1 step 0) is recorded in `ENGINE_NOTES.md`. Never call an upstream function that has not been confirmed against vendored source (DO #2). Where this plan names an upstream API, read it as *intent*. The code blocks below are self-contained, own-crate implementations that need no upstream symbol.

---

## SECTION 1 — Centralized Global Controllers & Universal Control Protocol (Anti-Hardcoding Core)

Directory: `app-tauri/src/lib/core/`. These four modules are the **only** owners of theme, tool state, coordinates, and history. Views import them, read `$derived` values, and send intents through `commandEngine.dispatch()`.

### 1.1 `themeContract.svelte.ts` — GlobalThemeContract

```ts
// app-tauri/src/lib/core/themeContract.svelte.ts
// The ONLY file permitted to contain colour literals (protocol-lint rule `no-hardcoded-colour`).
export type ThemeMode = 'system' | 'light' | 'dark';
export type ResolvedTheme = 'light' | 'dark';
export type TokenName =
  | '--glass-bg' | '--glass-border' | '--glass-blur' | '--glass-shadow' | '--glass-text'
  | '--canvas-bg' | '--canvas-page-shadow' | '--canvas-grid-major' | '--canvas-grid-minor' | '--canvas-selection'
  | '--snap-indicator-endpoint' | '--snap-indicator-midpoint' | '--snap-indicator-intersection'
  | '--snap-indicator-perpendicular' | '--snap-indicator-grid' | '--snap-indicator-size'
  | '--laser-core' | '--laser-glow' | '--measure-line' | '--measure-label-bg' | '--measure-label-text'
  | '--accent' | '--danger' | '--focus-ring' | '--motion-duration';
type TokenTable = Readonly<Record<TokenName, string>>;

const TOKENS: Readonly<Record<ResolvedTheme, TokenTable>> = Object.freeze({
  light: Object.freeze({
    '--glass-bg': 'rgba(255,255,255,0.62)', '--glass-border': 'rgba(0,0,0,0.08)', '--glass-blur': '24px',
    '--glass-shadow': '0 8px 32px rgba(0,0,0,0.12)', '--glass-text': '#1d1d1f',
    '--canvas-bg': '#e9e9ee', '--canvas-page-shadow': 'rgba(0,0,0,0.18)', '--canvas-grid-major': 'rgba(0,0,0,0.14)',
    '--canvas-grid-minor': 'rgba(0,0,0,0.06)', '--canvas-selection': 'rgba(0,122,255,0.22)',
    '--snap-indicator-endpoint': '#ff3b30', '--snap-indicator-midpoint': '#34c759', '--snap-indicator-intersection': '#ff9500',
    '--snap-indicator-perpendicular': '#af52de', '--snap-indicator-grid': '#8e8e93', '--snap-indicator-size': '10px',
    '--laser-core': '#ffffff', '--laser-glow': '#ff2d55', '--measure-line': '#0a84ff',
    '--measure-label-bg': 'rgba(255,255,255,0.9)', '--measure-label-text': '#1d1d1f',
    '--accent': '#007aff', '--danger': '#ff3b30', '--focus-ring': '#0a84ff', '--motion-duration': '180ms',
  }),
  dark: Object.freeze({
    '--glass-bg': 'rgba(30,30,32,0.58)', '--glass-border': 'rgba(255,255,255,0.10)', '--glass-blur': '24px',
    '--glass-shadow': '0 8px 32px rgba(0,0,0,0.45)', '--glass-text': '#f5f5f7',
    '--canvas-bg': '#1c1c1e', '--canvas-page-shadow': 'rgba(0,0,0,0.6)', '--canvas-grid-major': 'rgba(255,255,255,0.16)',
    '--canvas-grid-minor': 'rgba(255,255,255,0.06)', '--canvas-selection': 'rgba(10,132,255,0.28)',
    '--snap-indicator-endpoint': '#ff453a', '--snap-indicator-midpoint': '#30d158', '--snap-indicator-intersection': '#ff9f0a',
    '--snap-indicator-perpendicular': '#bf5af2', '--snap-indicator-grid': '#98989d', '--snap-indicator-size': '10px',
    '--laser-core': '#ffffff', '--laser-glow': '#ff375f', '--measure-line': '#64d2ff',
    '--measure-label-bg': 'rgba(28,28,30,0.92)', '--measure-label-text': '#f5f5f7',
    '--accent': '#0a84ff', '--danger': '#ff453a', '--focus-ring': '#64d2ff', '--motion-duration': '180ms',
  }),
});

class GlobalThemeContract {
  mode = $state<ThemeMode>('system');
  #systemDark = $state(false);
  #reducedMotion = $state(false);
  readonly resolved: ResolvedTheme = $derived(this.mode === 'system' ? (this.#systemDark ? 'dark' : 'light') : this.mode);
  readonly reducedMotion: boolean = $derived(this.#reducedMotion);
  readonly tokens: TokenTable = $derived(TOKENS[this.resolved]);

  /** Canvas-2D painters cannot read CSS vars cheaply; they read the same table here. */
  token(name: TokenName): string { return this.tokens[name]; }

  /** Mount once from App.svelte. Returns a disposer. */
  attach(root: HTMLElement): () => void {
    const dark = window.matchMedia('(prefers-color-scheme: dark)');
    const motion = window.matchMedia('(prefers-reduced-motion: reduce)');
    this.#systemDark = dark.matches;
    this.#reducedMotion = motion.matches;
    const onDark = (e: MediaQueryListEvent): void => { this.#systemDark = e.matches; };
    const onMotion = (e: MediaQueryListEvent): void => { this.#reducedMotion = e.matches; };
    dark.addEventListener('change', onDark);
    motion.addEventListener('change', onMotion);
    const stop = $effect.root(() => {
      $effect(() => {
        const table = this.tokens;
        for (const key of Object.keys(table) as TokenName[]) root.style.setProperty(key, table[key]);
        root.style.setProperty('--motion-duration', this.reducedMotion ? '0ms' : table['--motion-duration']);
        root.dataset.theme = this.resolved;
      });
    });
    return () => { dark.removeEventListener('change', onDark); motion.removeEventListener('change', onMotion); stop(); };
  }
}
export const theme = new GlobalThemeContract();
```

### 1.2 `toolEngine.svelte.ts` — ToolStateEngine

```ts
// app-tauri/src/lib/core/toolEngine.svelte.ts
import type { HatchPatternId } from '$lib/bindings/HatchPatternId';
export type ToolId =
  | 'pan' | 'select.marquee' | 'select.text' | 'select.lasso' | 'select.object' | 'laser'
  | 'pen.a' | 'pen.b' | 'highlighter' | 'eraser.whole' | 'eraser.partial'
  | 'review.textEdit' | 'review.highlight' | 'review.underline' | 'review.squiggly' | 'review.strikeout' | 'review.redact'
  | 'shape.line' | 'shape.polyline' | 'shape.rect' | 'shape.ellipse' | 'shape.polygon' | 'shape.cloud' | 'shape.arrow'
  | 'hatch.pick' | 'measure.length' | 'measure.polylength' | 'measure.rectArea' | 'measure.polyArea'
  | 'measure.protractor' | 'measure.ruler' | 'measure.calibrate'
  | 'insert.signature' | 'insert.audio' | 'insert.latex' | 'insert.callout' | 'insert.note' | 'insert.image'
  | 'insert.link' | 'insert.library' | 'snapshot';
/** Cubic Bézier pressure curve through (0,0),(x1,y1),(x2,y2),(1,1) — texturecraft-style stamp curve. */
export interface PressureCurve { x1: number; y1: number; x2: number; y2: number }
export interface StrokeStyle { colorToken: string | null; colorHex: string; widthPt: number; opacity: number;
  dash: readonly number[]; lineCap: 'butt' | 'round' | 'square'; startEnd: LineEnding; endEnd: LineEnding }
export type LineEnding = 'none' | 'open' | 'closed' | 'circle' | 'square' | 'diamond' | 'slash' | 'butt';
export interface FillStyle { kind: 'none' | 'solid' | 'hatch'; colorHex: string; opacity: number;
  hatch: HatchPatternId; hatchScale: number; hatchAngleDeg: number }
export interface FontStyle { family: string; sizePt: number; weight: number; italic: boolean;
  leading: number; trackingEm: number; colorHex: string }
export interface ToolPreset { id: string; tool: ToolId; label: string; stroke: StrokeStyle; fill: FillStyle;
  font: FontStyle; pressure: PressureCurve; tiltShading: number; smartShapes: boolean }

const clamp01 = (v: number): number => (v < 0 ? 0 : v > 1 ? 1 : v);
/** Evaluate y for given x on the curve by bisection on t (x(t) is monotone for x1,x2 ∈ [0,1]). */
export function evalPressure(c: PressureCurve, p: number): number {
  const x = clamp01(p);
  const bx = (t: number): number => 3 * (1 - t) * (1 - t) * t * c.x1 + 3 * (1 - t) * t * t * c.x2 + t * t * t;
  const by = (t: number): number => 3 * (1 - t) * (1 - t) * t * c.y1 + 3 * (1 - t) * t * t * c.y2 + t * t * t;
  let lo = 0, hi = 1;
  for (let i = 0; i < 24; i++) { const mid = (lo + hi) / 2; if (bx(mid) < x) lo = mid; else hi = mid; }
  return clamp01(by((lo + hi) / 2));
}

class ToolStateEngine {
  presets = $state<ToolPreset[]>([]);
  favorites = $state<string[]>([]);
  activePresetId = $state<string | null>(null);
  /** Tool held temporarily (e.g. Space → pan, stylus barrel → eraser); restored on release. */
  #springLoaded = $state<string | null>(null);
  readonly active: ToolPreset | null = $derived(
    this.presets.find((p) => p.id === (this.#springLoaded ?? this.activePresetId)) ?? null);
  readonly activeTool: ToolId = $derived(this.active?.tool ?? 'pan');

  load(presets: readonly ToolPreset[], favorites: readonly string[]): void {
    this.presets = presets.map((p) => structuredClone(p));
    this.favorites = favorites.filter((id) => presets.some((p) => p.id === id));
    this.activePresetId = this.presets[0]?.id ?? null;
  }
  select(presetId: string): boolean {
    if (!this.presets.some((p) => p.id === presetId)) return false;
    this.activePresetId = presetId; return true;
  }
  springLoad(presetId: string | null): void { this.#springLoaded = presetId; }
  patch(presetId: string, patch: Partial<Omit<ToolPreset, 'id' | 'tool'>>): void {
    this.presets = this.presets.map((p) => (p.id === presetId ? { ...p, ...patch } : p));
  }
  toggleFavorite(presetId: string): void {
    this.favorites = this.favorites.includes(presetId)
      ? this.favorites.filter((f) => f !== presetId) : [...this.favorites, presetId];
  }
  /** Stroke width at a pressure sample, tilt-shaded (pencil-side shading reduces width up to tiltShading). */
  widthAt(pressure: number, tiltX: number, tiltY: number): number {
    const a = this.active; if (!a) return 1;
    const tilt = Math.min(1, Math.hypot(tiltX, tiltY) / 90);
    return a.stroke.widthPt * Math.max(0.05, evalPressure(a.pressure, pressure)) * (1 + a.tiltShading * tilt);
  }
}
export const tools = new ToolStateEngine();
```

Default presets are **data**, shipped as `app-tauri/src/lib/core/defaultPresets.json` (D1). The user's copy is persisted via `prefs_save("tool_presets", json)` (Rust, atomic write per F1). The engine contains no literal style values.

### 1.3 `viewportEngine.svelte.ts` — ViewportCoordinateEngine

`DOMMatrix` is used at the render edge. The core math is a pure `Affine` type so vitest can run it in the `node` environment.

```ts
// app-tauri/src/lib/core/viewportEngine.svelte.ts
export type RealUnit = 'mm' | 'cm' | 'm' | 'in' | 'ft';
export const MM_PER_UNIT: Readonly<Record<RealUnit, number>> = Object.freeze({ mm: 1, cm: 10, m: 1000, in: 25.4, ft: 304.8 });
export const MM_PER_PT = 25.4 / 72;
export type Rotation = 0 | 90 | 180 | 270;
export interface Pt { x: number; y: number }

/** Row-vector affine [a b c d e f] identical to DOMMatrix/Canvas setTransform semantics. */
export class Affine {
  constructor(readonly a = 1, readonly b = 0, readonly c = 0, readonly d = 1, readonly e = 0, readonly f = 0) {}
  multiply(m: Affine): Affine { // this · m  (apply m first, then this)
    return new Affine(this.a * m.a + this.c * m.b, this.b * m.a + this.d * m.b,
      this.a * m.c + this.c * m.d, this.b * m.c + this.d * m.d,
      this.a * m.e + this.c * m.f + this.e, this.b * m.e + this.d * m.f + this.f);
  }
  invert(): Affine | null {
    const det = this.a * this.d - this.b * this.c;
    if (Math.abs(det) < 1e-12) return null;
    return new Affine(this.d / det, -this.b / det, -this.c / det, this.a / det,
      (this.c * this.f - this.d * this.e) / det, (this.b * this.e - this.a * this.f) / det);
  }
  apply(p: Pt): Pt { return { x: this.a * p.x + this.c * p.y + this.e, y: this.b * p.x + this.d * p.y + this.f }; }
  static translate(x: number, y: number): Affine { return new Affine(1, 0, 0, 1, x, y); }
  static scale(s: number): Affine { return new Affine(s, 0, 0, s, 0, 0); }
  static rotate(deg: Rotation): Affine {
    const [cos, sin] = deg === 0 ? [1, 0] : deg === 90 ? [0, 1] : deg === 180 ? [-1, 0] : [0, -1];
    return new Affine(cos, sin, -sin, cos, 0, 0);
  }
  toDOMMatrix(): DOMMatrix { return new DOMMatrix([this.a, this.b, this.c, this.d, this.e, this.f]); }
}

export interface Calibration { unit: RealUnit; /** real-unit length per 1 PDF pt */ unitsPerPt: number; precision: number }
export interface PageGeom { widthPt: number; heightPt: number; /** page origin in world (CSS px @ zoom 1) */ worldX: number; worldY: number; rotation: Rotation }
export interface ViewState { id: string; zoom: number; panX: number; panY: number; linkGroup: string | null; dpr: number }

export const ZOOM_MIN = 0.05, ZOOM_MAX = 64;
const PT_TO_CSS = 96 / 72; // 1 pt at zoom 1 = 1.333 CSS px

class ViewportCoordinateEngine {
  views = $state<Record<string, ViewState>>({});
  pages = $state<PageGeom[]>([]);
  calibrations = $state<Record<number, Calibration>>({}); // keyed by page index
  defaultCalibration = $state<Calibration>({ unit: 'mm', unitsPerPt: MM_PER_PT, precision: 2 });

  upsertView(v: ViewState): void { this.views = { ...this.views, [v.id]: v }; }
  closeView(id: string): void { const { [id]: _drop, ...rest } = this.views; this.views = rest; }

  /** world(CSS px) → screen(CSS px) for a view. */
  worldToScreen(viewId: string): Affine {
    const v = this.views[viewId]; if (!v) return new Affine();
    return Affine.translate(v.panX, v.panY).multiply(Affine.scale(v.zoom));
  }
  /** PDF user space (pt, origin bottom-left, y up) → screen CSS px. */
  pdfToScreen(viewId: string, page: number): Affine {
    const g = this.pages[page]; if (!g) return new Affine();
    const flip = new Affine(PT_TO_CSS, 0, 0, -PT_TO_CSS, 0, g.heightPt * PT_TO_CSS);
    const w = g.widthPt * PT_TO_CSS, h = g.heightPt * PT_TO_CSS;
    const rot = Affine.rotate(g.rotation);
    const recenter = g.rotation === 90 ? Affine.translate(h, 0) : g.rotation === 180 ? Affine.translate(w, h)
      : g.rotation === 270 ? Affine.translate(0, w) : new Affine();
    return this.worldToScreen(viewId).multiply(Affine.translate(g.worldX, g.worldY))
      .multiply(recenter).multiply(rot).multiply(flip);
  }
  screenToPdf(viewId: string, page: number, p: Pt): Pt | null {
    const inv = this.pdfToScreen(viewId, page).invert(); return inv ? inv.apply(p) : null;
  }
  calibrationFor(page: number): Calibration { return this.calibrations[page] ?? this.defaultCalibration; }
  ptToReal(page: number, lengthPt: number): number { return lengthPt * this.calibrationFor(page).unitsPerPt; }
  ptAreaToReal(page: number, areaPt2: number): number { const k = this.calibrationFor(page).unitsPerPt; return areaPt2 * k * k; }
  /** Two-point calibration: user states that segment (a,b) in pt measures `realLength` `unit`. */
  calibrate(page: number, a: Pt, b: Pt, realLength: number, unit: RealUnit, precision: number): boolean {
    const d = Math.hypot(b.x - a.x, b.y - a.y);
    if (!(d > 1e-6) || !(realLength > 0) || !Number.isFinite(realLength)) return false;
    this.calibrations = { ...this.calibrations, [page]: { unit, unitsPerPt: realLength / d, precision } };
    return true;
  }
  convert(value: number, from: RealUnit, to: RealUnit): number { return (value * MM_PER_UNIT[from]) / MM_PER_UNIT[to]; }

  /** Zoom about a screen anchor; propagates to linked split-views. */
  zoomAt(viewId: string, factor: number, anchor: Pt): void {
    const v = this.views[viewId]; if (!v) return;
    const zoom = Math.min(ZOOM_MAX, Math.max(ZOOM_MIN, v.zoom * factor));
    const k = zoom / v.zoom;
    this.#apply(v, { zoom, panX: anchor.x - (anchor.x - v.panX) * k, panY: anchor.y - (anchor.y - v.panY) * k });
  }
  panBy(viewId: string, dx: number, dy: number): void {
    const v = this.views[viewId]; if (v) this.#apply(v, { panX: v.panX + dx, panY: v.panY + dy });
  }
  #apply(src: ViewState, patch: Pick<ViewState, 'panX' | 'panY'> & Partial<Pick<ViewState, 'zoom'>>): void {
    const next = { ...this.views };
    for (const v of Object.values(next)) {
      if (v.id === src.id || (src.linkGroup !== null && v.linkGroup === src.linkGroup)) next[v.id] = { ...v, ...patch };
    }
    this.views = next;
  }
}
export const viewport = new ViewportCoordinateEngine();
```

**Multi-window sync:** detached windows are created with the Tauri v2 `WebviewWindow` API (verify the exact constructor against the pinned `@tauri-apps/api` before use). Sync runs over targeted events (`emitTo(label, 'viewport:sync', ViewState)`), never `emit_all` (DON'T #17). The capability file adds only `core:window:allow-create` and `core:event:allow-emit-to` for labels `split-*`.

### 1.4 `commandEngine.svelte.ts` — CommandBusAndShortcutEngine

Copy-on-write history: the document is an immutable `DocSnapshot` whose pages are frozen `ReadonlyMap`s. A commit copies only the touched page maps, so its cost is O(changed pages). Undo and redo are O(1) pointer swaps between snapshot roots.

```ts
// app-tauri/src/lib/core/commandEngine.svelte.ts
import type { ControlMessage } from '$lib/bindings/ControlMessage';
import type { SceneNode } from '$lib/bindings/SceneNode';
export interface DocSnapshot { readonly rev: number; readonly pages: ReadonlyArray<ReadonlyMap<string, SceneNode>> }
export interface CommandResult { next: DocSnapshot; label: string; coalesceKey?: string }
export type CommandHandler = (doc: DocSnapshot, args: unknown) => CommandResult | { error: string };
export type Chord = string; // canonical: "Ctrl+Alt+Shift+Meta+Key"

const HISTORY_LIMIT_DEFAULT = 500; // user-configurable via prefs; older entries are dropped from memory, not the file

export function canonicalChord(e: Pick<KeyboardEvent, 'ctrlKey' | 'altKey' | 'shiftKey' | 'metaKey' | 'key'>): Chord {
  const key = e.key.length === 1 ? e.key.toUpperCase() : e.key;
  return [e.ctrlKey && 'Ctrl', e.altKey && 'Alt', e.shiftKey && 'Shift', e.metaKey && 'Meta', key].filter(Boolean).join('+');
}
export function withPage(doc: DocSnapshot, page: number, edit: (m: Map<string, SceneNode>) => void): DocSnapshot {
  const src = doc.pages[page]; if (!src) return doc;
  const copy = new Map(src); edit(copy);
  const pages = doc.pages.slice(); pages[page] = copy;
  return Object.freeze({ rev: doc.rev + 1, pages: Object.freeze(pages) });
}

class CommandBusAndShortcutEngine {
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
  #pointerSink: ((m: Extract<ControlMessage, { kind: 'pointer' }>) => void) | null = null;
  #renderSink: ((m: Extract<ControlMessage, { kind: 'render_page' }>) => void) | null = null;
  readonly journal: ControlMessage[] = []; // headless replay/verification trace (cleared per session)

  register(id: string, h: CommandHandler): void { this.#handlers.set(id, h); }
  setPointerSink(f: typeof this.#pointerSink): void { this.#pointerSink = f; }
  setRenderSink(f: typeof this.#renderSink): void { this.#renderSink = f; }
  load(doc: DocSnapshot): void { this.doc = doc; this.#past = []; this.#future = []; }

  dispatch(msg: ControlMessage): boolean {
    this.journal.push(msg);
    switch (msg.kind) {
      case 'execute': return this.#execute(msg.command, msg.args);
      case 'pointer': this.#pointerSink?.(msg); return this.#pointerSink !== null;
      case 'ui.key': { if (msg.phase !== 'down') return false; const cmd = this.shortcuts[msg.chord];
        return cmd ? this.#execute(cmd, null) : false; }
      case 'render_page': this.#renderSink?.(msg); return this.#renderSink !== null;
    }
  }
  #execute(command: string, args: unknown): boolean {
    if (command === 'history.undo') return this.undo();
    if (command === 'history.redo') return this.redo();
    const h = this.#handlers.get(command);
    if (!h) { this.lastError = `unknown_command:${command}`; return false; }
    const r = h(this.doc, args);
    if ('error' in r) { this.lastError = r.error; return false; }
    if (r.next === this.doc) return true; // no-op: nothing recorded
    const top = this.#past.at(-1);
    const past = r.coalesceKey && top?.key === r.coalesceKey ? this.#past
      : [...this.#past, { doc: this.doc, label: r.label, key: r.coalesceKey }];
    this.#past = past.length > this.historyLimit ? past.slice(past.length - this.historyLimit) : past;
    this.#future = []; this.doc = r.next; this.lastError = null; return true;
  }
  undo(): boolean { const top = this.#past.at(-1); if (!top) return false;
    this.#future = [...this.#future, { doc: this.doc, label: top.label }]; this.#past = this.#past.slice(0, -1); this.doc = top.doc; return true; }
  redo(): boolean { const top = this.#future.at(-1); if (!top) return false;
    this.#past = [...this.#past, { doc: this.doc, label: top.label }]; this.#future = this.#future.slice(0, -1); this.doc = top.doc; return true; }

  bind(chord: Chord, command: string): { conflict: string | null } {
    const conflict = this.shortcuts[chord] ?? null; this.shortcuts = { ...this.shortcuts, [chord]: command }; return { conflict };
  }
  unbind(chord: Chord): void { const { [chord]: _d, ...rest } = this.shortcuts; this.shortcuts = rest; }
  /** Single global keyboard entry point (protocol-lint forbids keydown listeners elsewhere). */
  attachKeyboard(target: Window): () => void {
    const on = (e: KeyboardEvent): void => {
      const el = e.target as HTMLElement | null;
      if (el && (el.isContentEditable || el.tagName === 'INPUT' || el.tagName === 'TEXTAREA')) return;
      if (this.dispatch({ kind: 'ui.key', chord: canonicalChord(e), phase: 'down' })) e.preventDefault();
    };
    target.addEventListener('keydown', on); return () => target.removeEventListener('keydown', on);
  }
}
export const commands = new CommandBusAndShortcutEngine();
```

### 1.5 Universal control protocol (`crates/lpdf-control-bus`)

```rust
// crates/lpdf-control-bus/src/lib.rs
#![forbid(unsafe_code)]
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind")]
#[ts(export, export_to = "../../app-tauri/src/lib/bindings/")]
pub enum ControlMessage {
    #[serde(rename = "execute")]
    Execute { command: String, args: serde_json::Value },
    #[serde(rename = "pointer")]
    Pointer(PointerSample),
    #[serde(rename = "ui.key")]
    UiKey { chord: String, phase: KeyPhase },
    #[serde(rename = "render_page")]
    RenderPage { doc_id: String, page: u32, scale: f32, view_id: String },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")] #[ts(export, export_to = "../../app-tauri/src/lib/bindings/")]
pub enum KeyPhase { Down, Up }
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")] #[ts(export, export_to = "../../app-tauri/src/lib/bindings/")]
pub enum PointerPhase { Down, Move, Up, Cancel, Hover }
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")] #[ts(export, export_to = "../../app-tauri/src/lib/bindings/")]
pub enum PointerKind { Mouse, Pen, Touch }
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")] #[ts(export, export_to = "../../app-tauri/src/lib/bindings/")]
pub struct PointerSample {
    pub phase: PointerPhase, pub x: f64, pub y: f64, pub pressure: f32, pub tilt_x: f32, pub tilt_y: f32,
    pub twist: f32, pub pointer_type: PointerKind, pub buttons: u16, pub view_id: String, pub t_ms: f64,
}
#[derive(Debug, thiserror::Error, Serialize, TS)]
#[serde(tag = "code", content = "detail")] #[ts(export, export_to = "../../app-tauri/src/lib/bindings/")]
pub enum BusError {
    #[error("unknown command {0}")] UnknownCommand(String),
    #[error("invalid arguments for {command}: {reason}")] InvalidArgs { command: String, reason: String },
    #[error("replay mismatch at line {line}")] ReplayMismatch { line: u32 },
    #[error("malformed message at line {line}: {reason}")] Malformed { line: u32, reason: String },
}
pub trait Executor { fn handle(&mut self, msg: &ControlMessage) -> Result<serde_json::Value, BusError>; }

/// Headless replay: each JSONL line is `{"msg": ControlMessage, "expect": <json|null>}`.
pub fn replay<E: Executor>(exec: &mut E, script: &str) -> Result<u32, BusError> {
    #[derive(Deserialize)] struct Step { msg: ControlMessage, expect: Option<serde_json::Value> }
    let mut n = 0u32;
    for (i, raw) in script.lines().enumerate() {
        let line = u32::try_from(i + 1).unwrap_or(u32::MAX);
        if raw.trim().is_empty() || raw.trim_start().starts_with("//") { continue; }
        let step: Step = serde_json::from_str(raw).map_err(|e| BusError::Malformed { line, reason: e.to_string() })?;
        let out = exec.handle(&step.msg)?;
        if let Some(exp) = step.expect { if exp != out { return Err(BusError::ReplayMismatch { line }); } }
        n += 1;
    }
    Ok(n)
}
```

---

## SECTION 2 — Drawboard-Class Spatial UI, Dockable Toolbar & 8-Wedge Radial Menu

### 2.1 Infinite spatial canvas & split view
- `views/studio/SpatialCanvas.svelte`: one `<div class="world">` with `transform: viewport.worldToScreen(id).toDOMMatrix().toString()`. This is the only allowed CSS transform source. Pages are laid out vertically with a `16pt` gap (layout constant in `viewport.layoutPages()`).
- **Tiling:** each page is split into 512×512 device-px tiles at the quantised zoom levels `2^k/4`. A tile renders through `render_page` → worker job (pdfcraft/hayro raster). It is cached in an LRU with a memory budget from the ADR-002 tier (`tier.tileCacheBytes`), and off-screen tiles are evicted. Visibility comes from an `IntersectionObserver` per page (handles >1000 pages). Missing tiles show the previous zoom level scaled until the sharp tile arrives (no blank flash).
- **Overlay layer:** one `<canvas>` per visible page for vector markup. It repaints dirty rects in `requestAnimationFrame`. Live ink is drawn on a separate "wet" canvas (desynchronized context `getContext('2d', { desynchronized: true })` when supported) for low latency.
- **Split view:** `views/studio/SplitLayout.svelte` hosts 1–4 `SpatialCanvas` panes, `horizontal | vertical | grid`, with a draggable divider. Each pane has its own `ViewState`. The "link" toggle sets a shared `linkGroup` for synchronized pan/zoom, which is used to compare revisions. "Detach" moves a pane into a new `WebviewWindow` labelled `split-<uuid>`.

### 2.2 Floating & dockable markup toolbar
- `components/studio/MarkupToolbar.svelte`: frosted-glass (`background: var(--glass-bg); backdrop-filter: blur(var(--glass-blur))`). Dock state is `'float' | 'top' | 'left' | 'right'` with a float position `{x,y}`. **Magnetic snap:** on drag end, if the distance to an edge is under `dockThresholdPx` (pref, default 32), it docks to that edge, animated over `var(--motion-duration)`. Position is saved in prefs.
- `FavoritesBar.svelte` renders `tools.favorites` and supports drag-reorder.
- `PropertyPill.svelte` floats near the selection and edits colour, width, opacity, font, and hatch through `commands.dispatch({kind:'execute', command:'style.apply', args})`, so style edits are undoable.

### 2.3 `RadialMenu.svelte` — 8 sectors + flyout rings
- **Summon:** right-click, pen barrel button, long-press (500 ms, touch), or the `radial.open` shortcut. Opens centred at the pointer and is clamped inside the viewport.
- **Geometry:** inner radius `r0 = 36px`, wedge ring `r1 = 112px`, flyout ring `r2 = 176px`. Sector `i ∈ 0..7` spans `[-90° + 45°·i − 22.5°, −90° + 45°·i + 22.5°)`, so sector 0 points up. Hit-test: `θ = atan2(dy,dx)`, `i = floor(((θ·180/π + 90 + 22.5) mod 360) / 45)`, `ρ = hypot`. If `ρ < r0`, cancel. If `r0 ≤ ρ < r1`, select the wedge. If `r1 ≤ ρ < r2`, select a flyout item within the wedge's arc, which is divided evenly among its items.
- **Interaction:** marking-menu style. Press-drag-release selects; a click opens the menu as sticky. Keys `1–8` pick a wedge, arrows move, and Enter/Esc confirm or cancel. ARIA: `role="menu"` with `menuitem` children.
- **Wedges (data-driven from `core/radialLayout.json`, user-customisable):**

| # | Wedge | Flyout |
|---|---|---|
| 1 | Pan / Hand | — |
| 2 | Select | Marquee, Text, Lasso, Object Edit, Laser Pointer |
| 3 | Pen preset A | user pen presets |
| 4 | Pen preset B | user pen presets |
| 5 | Highlighter | presets |
| 6 | Eraser | Whole, Partial |
| 7 | Review & Edit | Inline Text Edit, Highlight, Underline, Squiggly, Strikethrough, Redact |
| 8 | Suite | Shapes, Hatch, Measure, Insert, Templates (each opens a second flyout ring) |

---

## SECTION 3 — Vector Network Ink, Smart Recognition, Partial Eraser & Procedural Hatching

### 3.1 Pressure ink pipeline
1. `pointerdown` on the overlay calls `setPointerCapture`. On `pointermove`, `e.getCoalescedEvents()` is iterated (with `[e]` as a fallback), and each sample becomes a `ControlMessage::Pointer` in screen px. `getPredictedEvents()` drives the wet layer only.
2. Each sample is converted to PDF pt via `viewport.screenToPdf`, and its width comes from `tools.widthAt(pressure, tiltX, tiltY)`. Mouse input reports pressure 0.5, which is treated as constant width.
3. Smoothing uses a one-euro filter (`minCutoff=1.0, beta=0.007, dCutoff=1.0`; values are tool-preset data) before samples go into the stroke.
4. Stroke model (`lpdf-vector-ink::VectorStroke`): a centreline of `(x,y,w)` samples. On commit, RDP simplification runs with ε = 0.25 pt, then the outline is built with Catmull-Rom → cubic Bézier for offsets ±w/2, with round caps. It is serialised as `/Ink` (centreline `/InkList` for interoperability) plus an appearance stream `/AP /N` containing the variable-width filled outline, so other viewers show the same look.
5. The highlighter uses multiply blend (`/BM /Multiply` in the AP ExtGState), a flat chisel cap, and optional snap-to-text-line when it begins over text.

### 3.2 Smart ink recognition (350 ms dwell)
When the pointer stays down and moves less than 2 px for 350 ms (both values are preset data), the classifier runs on the RDP-simplified points (`ε = 0.02 × bbox diagonal`):
- **Line:** total-least-squares fit. If the max orthogonal deviation is under 3% of the length → line.
- **Closed test:** endpoint gap under 15% of perimeter. Non-closed with ≥3 vertices → polyline.
- **Circle/ellipse:** Kasa algebraic circle fit. If the radial residual std divided by r is under 0.08 → circle, otherwise run PCA. If the axis ratio is above 1.15 and the ellipse residual is under 0.1 → ellipse.
- **Polygon:** vertex count after RDP. 3 → triangle. 4 → if all corner angles are within 90°±12°, a rectangle, and if additionally the aspect is within 10%, a square. 6 → hexagon (regularised when the side-length CV is under 0.15).
- The replacement is shown as a preview. Lifting the pen commits it as `shape.*`, and it stays undoable back to the raw ink (a two-step transaction).

### 3.3 Whole vs partial eraser
- **Whole:** `rstar::RTree<NodeEnvelope>` per page in Rust (and mirrored as a flat-bush in TS for hover). The eraser capsule (previous → current sample, radius r) queries candidate envelopes, then an exact segment–capsule distance check runs. Each hit node is removed in a coalesced transaction (`coalesceKey = eraser:<strokeId>`).
- **Partial (figcraft pattern):** for ink strokes, the centreline polyline is split where segments enter or exit the swept capsule (by solving the segment/circle quadratic and the segment/rectangle clip). Pieces outside the capsule become new strokes that inherit the style. For closed filled shapes, `i_overlay` computes the boolean difference `shape − capsule polygon (16-gon)`.

### 3.4 Shapes, clouds, pick-point hatching
- Supported shapes: line, arrow, polyline, rectangle, ellipse, polygon, and cloud. Clouds are arcs of chord `cloudArcPt` along the polygon edges, written as `/Polygon /IT /PolygonCloud /BE <</S /C /I 1>>`. Line endings map to the PDF `/LE` names.
- **Pick-point hatch (cadcraft):** the user clicks inside a region. Rust collects vector segments from the page content plus annotations within the view bounds, builds a planar arrangement (snapping endpoints within 0.5 pt, splitting at intersections), and finds the smallest face containing the pick point (leftmost-turn face walk). Faces fully inside it become **islands**, which are holes. The result is a polygon with holes that gets the hatch fill.
- **Procedural patterns (materialcraft-derived, own implementation):** `concrete` (stippled dots + triangles with seeded hash RNG), `timber` (ring-offset sine bands), `tiling` (square grid), `steel` (45° double lines), `masonry` (running bond), `earth` (ANSI-style dashes + dots), and `insulation` (sinusoidal batt curve). Each is `fn tile(scale, angle) -> Vec<PathSeg>` clipped to the region and emitted as a Tiling Pattern (`/PatternType 1`) so that it prints as vectors.

### 3.5 Laser pointer
Ephemeral and never persisted or put in history. A ring buffer of `{x,y,t}` is drawn as a Catmull-Rom spline. Alpha is `1 − (now − t)/1800ms`, with a glow made from `--laser-glow` plus `shadowBlur`. Under reduced motion, it shows only a dot.

---

## SECTION 4 — True Structural PDF Editing & Manipulation (`pdfcraft` + `designcraft`)

### 4.1 Inline text & typography editor
1. **Extract:** a worker job reads page text runs (font, size, Tm/CTM, glyph advances, ToUnicode) through pdfcraft's text extraction (API verified in D4 step 0). It groups runs into lines (baseline within 0.3·size) and lines into paragraphs (leading consistent ±20%, same font family).
2. **Edit:** a `contenteditable` overlay is positioned through `viewport.pdfToScreen`. Its styling comes from `tools.active.font`. The user edits text, font, size, kerning/tracking, leading, colour, and opacity.
3. **Reflow:** `lpdf-layout-edit::knuth_plass` (§D4 code) lays out the paragraph inside the original column width, measuring glyph widths from the embedded or substituted font metrics.
4. **Write back:** the original paragraph's text-showing operators are removed (operator-range deletion within `BT…ET`), and new `BT /Fx size Tf … TJ ET` is emitted. The font is subset-embedded when it is a system font (licence `fsType` checked; installable/editable embedding only, otherwise refuse with a notice). Output is a new file through atomic write + verify (F1, F6).
5. **Degrade honestly:** Type3 fonts or fonts without a ToUnicode map → "This text can't be edited in place; replace with a new text box?" (DON'T #5).

### 4.2 Object manipulation
- Selection covers native image XObjects, form XObjects, path groups (contiguous `q…Q` blocks), and annotations. Handles: 8 resize handles plus rotation. Rotation steps by 1° by default and snaps to 15° with Shift (figcraft convention). Group/Ungroup is stored as an overlay group id for annotations, or wrapped into a Form XObject for native content. Align L/C/R/T/M/B and distribute run on selection bounding boxes. Flip H/V pre-multiplies by `[-1 0 0 1 2cx 0]` / `[1 0 0 -1 0 2cy]`. All geometry edits rewrite the object's `cm` matrix (native content) or `/Rect` + AP `/Matrix` (annotations).
- **Snapshot tool:** a marquee region is rasterized at the chosen DPI through `render_page`, then cropped, flipped, or rotated and placed as a new image XObject or copied to the clipboard.

### 4.3 Permanent redaction & OCR deskew
- Redaction uses pdfcraft's native `pdfcraft_redact` (DO #23). Glyphs whose bounding boxes intersect the marks are removed from the content streams, image pixels are overwritten inside the image streams, and intersecting annotations and metadata text are removed. Then the **F25 verify pass** runs: re-extract text and confirm none of the redacted strings or areas remain before the rename. Black boxes alone are banned (DON'T #47).
- OCR uses `pdfcraft_ocr` (ocrs/rten, F23). **Deskew:** estimate the skew angle with a projection-profile variance maximisation over −15°…+15° in 0.1° steps on a 150 dpi binarised render, then apply the rotation to the page image XObject (resample) before OCR. The text layer goes in as invisible text (`3 Tr`).

### 4.4 Document assembly
The canvas page strip supports drag-to-reorder, multi-select, and merge (drop PDFs between pages), plus split at a marker, extract, rotate, and crop (crop box drawn on canvas, F27 clamp to MediaBox). Each action is one `execute` command and is undoable as a page-order snapshot. Applying it runs the existing pdfcraft organize jobs.

---

## SECTION 5 — CAD Calibration, Content Snapping & Measurement Suite (`cadcraft`)

### 5.1 OSNAP engine (Rust `lpdf-cad-measure::snap`)
- **Extraction:** a worker job walks page content paths, flattens curves (tolerance 0.1 pt via `kurbo` flatten), and keeps line segments in page pt. They are indexed in `rstar::RTree<Seg>` per page and cached by `(doc hash, page)`.
- **Query:** cursor `p` (pt) and `tol = snapPx / (zoom·96/72)`. Candidates within tol: endpoints, midpoints, pairwise intersections of segments in the tol box, perpendicular foot from the active anchor, nearest-on-segment, and grid (`round(p/grid)·grid`).
- **Ranking:** `score = dist / tol + priority`, where priority is endpoint 0.0, intersection 0.05, midpoint 0.1, perpendicular 0.15, nearest 0.4, grid 0.5. The lowest score wins. The UI draws the glyph colour `--snap-indicator-<kind>`.

### 5.2 Scale calibration
Two-point pick (with snapping) → enter the real length and unit → `viewport.calibrate(page, a, b, len, unit, precision)`. The scope can be the page, the document, or a viewport region (rectangle → its own `/VP` entry). It is persisted to the PDF as `/VP [<< /BBox … /Measure << /Type /Measure /Subtype /RL /R (1 in = 10 ft) /X [...] /D [...] /A [...] >> >>]` so other viewers honour it.

### 5.3 Measurement tools
| Tool | Math (pt → real via calibration k = unitsPerPt) | PDF output |
|---|---|---|
| Length | `k·‖b−a‖` | `/Line` + `/IT /LineDimension` + `/Measure` |
| Polylength | `k·Σ‖pᵢ₊₁−pᵢ‖` | `/PolyLine /IT /PolyLineDimension` |
| Rect area | `k²·w·h`, perimeter `2k(w+h)` | `/Polygon /IT /PolygonDimension` |
| Polygon area | shoelace `k²·½|Σ(xᵢyᵢ₊₁−xᵢ₊₁yᵢ)|`. Self-intersecting → split via `i_overlay` union and sum faces | `/Polygon /IT /PolygonDimension` |
| Protractor | angle at vertex `atan2(‖u×v‖, u·v)` in degrees (0–180), reflex toggle | `/PolyLine` 3-pt + label |
| Ruler | draggable/rotatable widget; edges snap pencil strokes to the ruler line while held | not persisted |

Dimensions are **associative**: each stores the snap-source ids (`SnapRef { node_id, kind, param }`). When the source moves, `measure.reassociate` recomputes the dimension in the same transaction.

---

## SECTION 6 — Insertions, LaTeX, Audio Notes, Markup Library & Overlays

- **Signatures:** three modes. (1) Stylus draw: vector ink, normalised. (2) Typed name with a locally bundled script font (OFL, vetted). (3) Webcam capture of a paper signature: `getUserMedia` (local only, explicit permission), then grayscale, then Otsu threshold, then remove the background to alpha, then optionally vectorise by marching squares + RDP. A signature photographed on a phone can be brought in as a local image file; there is no network QR transfer. Signatures are stored encrypted in the app data dir (F18 hygiene). Cryptographic signing remains `pdfcraft_sign`.
- **Audio notes:** `MediaRecorder` (codec chosen from `isTypeSupported`) → PCM decode via `AudioContext.decodeAudioData` → written as a `/Sound` annotation with a `/Sound` stream (`/R` sample rate, `/C` channels, `/B 16`, `/E /Signed`). The playback popover shows a waveform (peaks precomputed per 512 samples) and a moving marker.
- **LaTeX:** MathJax 3 `tex-svg` vendored offline (Apache-2.0) runs in a dedicated Web Worker. The SVG output string is parsed with `DOMParser` (never `{@html}`), its `<path d>` and `<use>` elements are resolved into absolute path data, and the result is converted into PDF path operators inside a Form XObject stamp. The LaTeX source is stored in the annotation `/Contents` + private `/LPDF:TeX` key so it can be edited again.
- **Callouts** (multileader: one text box, N leaders with arrowheads, `/FreeText /IT /FreeTextCallout` plus extra `/Line` leaders grouped), **sticky notes** (`/Text`), **bookmarks tree** (`/Outlines` editing via pdfcraft), **images** (drag-drop local files only, F17 path checks), and **links** (`/Link` with `/GoTo` internal or `/URI` external; external links never open automatically, they show a confirm first).
- **Reusable Markup Library:** saved node groups (JSON of `SceneNode[]`, normalised to their bbox origin) stored in the app data dir and placed by drag. **Advanced Paste:** paste the clipboard nodes at identical pt coordinates on a page-range expression (`1-5,8,odd`), as one transaction.
- **Templates & overlays:** grid, ruled, dot, isometric (30°), and engineering (major/minor). These are non-destructive *master overlay layers* (designcraft master-page model) that render only in the view, toggled per document. They are optionally "burned in" as an optional-content group (`/OCG`, off for print by default). Custom PDF templates can be inserted as new pages.

---

## SECTION 7 — Annotation History, Quantity Takeoff, Tags & Local AI

- **Annotation History panel:** a virtualised table over `commands.doc` with columns: type, page, author, created/modified, colour, label, measured value, unit, and tags. It is sortable, groupable (by page/type/colour/tag), and filterable, and clicking a row zooms to the annotation.
- **Takeoff export:** worker job writing **CSV** (RFC 4180, UTF-8 BOM opt-in), **XLSX** (`rust_xlsxwriter`, with one sheet per group plus a totals row using native `SUM` formulas), and **DXF** (R12 ASCII written by our own `lpdf-cad-measure::dxf_write`: `LINE`/`LWPOLYLINE`/`TEXT` per measurement, scaled to calibrated units). All are atomic new files.
- **Tags:** a local catalogue (SQLite is *not* added; we use a JSON index in the app data dir, written atomically) mapping document hash → tags. A filter chip bar sits on the home screen.
- **Local inbox watch-folder:** this replaces cloud email-to-upload. The user picks a folder (explicit scope, no wildcards, DON'T #11), and `notify` (or a 2 s poll fallback if no acceptable licence version exists) detects new `*.pdf` files. They are listed in an "Inbox" panel and are never auto-modified.
- **Inline AI actions** (Summarize, Rephrase, Define, Fix Grammar, Translate across the app's 6 languages) appear on the selection toolbar. They run behind the `ai-local` flag (default OFF, DO #17) through the ADR-004 Candle `lpdf-ai-worker` (F19), using user-imported model files only. With AI off, the tiles are greyed out, and Define falls back to an offline dictionary if one is installed (otherwise hidden). Results are inserted only on user confirm as a new text box or replacement (undoable).

---

## SECTION 8 — Rust Crate Topology & Type-Safe IPC Bridge

```
crates/
  lpdf-control-bus/   ControlMessage, PointerSample, BusError, Executor, replay()   [serde, serde_json, ts-rs, thiserror]
  lpdf-vector-ink/    VectorStroke, smoothing, outline, recognize, erase, pattern   [kurbo, i_overlay, rstar]
  lpdf-cad-measure/   snap (OSNAP), calibrate, measure, hatch (arrangement), dxf_read, dxf_write, takeoff  [rstar, kurbo, i_overlay, rust_xlsxwriter]
  lpdf-layout-edit/   text_runs, paragraph grouping, knuth_plass, font metrics, content rewrite  [pdfcraft (vendored)]
app-tauri/src-tauri/src/commands/studio.rs   thin #[tauri::command] wrappers (<10 ms, DO #8) → job queue
```

All four crates are **pure** (DO #3): no Tauri imports, `#![forbid(unsafe_code)]`, `thiserror` errors with stable codes, and no `unwrap/expect` outside `#[cfg(test)]`.

**Exported `ts-rs` types (written to `app-tauri/src/lib/bindings/`):**

```rust
// crates/lpdf-vector-ink/src/model.rs
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)] #[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../app-tauri/src/lib/bindings/")]
pub struct InkSample { pub x: f64, pub y: f64, pub w: f64, pub t_ms: f64 }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)] #[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../app-tauri/src/lib/bindings/")]
pub enum HatchPatternId { Concrete, Timber, Tiling, Steel, Masonry, Earth, Insulation, Solid }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)] #[serde(tag = "type", rename_all = "camelCase")]
#[ts(export, export_to = "../../app-tauri/src/lib/bindings/")]
pub enum SceneGeometry {
    Ink { samples: Vec<InkSample> },
    Line { a: [f64; 2], b: [f64; 2] },
    Polyline { points: Vec<[f64; 2]>, closed: bool },
    Rect { x: f64, y: f64, w: f64, h: f64, rotation_deg: f64 },
    Ellipse { cx: f64, cy: f64, rx: f64, ry: f64, rotation_deg: f64 },
    Cloud { points: Vec<[f64; 2]>, arc_pt: f64 },
    Hatch { outer: Vec<[f64; 2]>, holes: Vec<Vec<[f64; 2]>>, pattern: HatchPatternId, scale: f64, angle_deg: f64 },
    Text { x: f64, y: f64, w: f64, h: f64, text: String },
    Stamp { x: f64, y: f64, w: f64, h: f64, xobject_ref: String },
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)] #[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../app-tauri/src/lib/bindings/")]
pub struct SceneStyle { pub stroke_hex: String, pub stroke_width_pt: f64, pub opacity: f64, pub fill_hex: Option<String>,
    pub dash: Vec<f64>, pub start_ending: String, pub end_ending: String, pub blend_multiply: bool }
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)] #[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../app-tauri/src/lib/bindings/")]
pub struct SceneNode { pub id: String, pub page: u32, pub geometry: SceneGeometry, pub style: SceneStyle,
    pub group: Option<String>, pub measure: Option<MeasureValue>, pub tags: Vec<String>,
    pub author: String, pub created_ms: f64, pub modified_ms: f64 }
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)] #[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../app-tauri/src/lib/bindings/")]
pub struct MeasureValue { pub kind: String, pub value: f64, pub unit: String, pub perimeter: Option<f64>, pub sources: Vec<SnapRef> }
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)] #[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../app-tauri/src/lib/bindings/")]
pub struct SnapRef { pub node_id: Option<String>, pub kind: SnapKind, pub param: f64 }
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)] #[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../app-tauri/src/lib/bindings/")]
pub enum SnapKind { Endpoint, Midpoint, Intersection, Perpendicular, Nearest, Grid }
```

`ts-rs` maps `f64` → `number`, `Option<T>` → `T | null`, and `serde_json::Value` → `JsonValue` (exported). A vitest type test (`bindings.test-d.ts`) asserts that no binding contains `any` (`expectTypeOf<…>().not.toBeAny()`). The `cargo xtask protocol-lint` rule `no-any-bindings` greps `bindings/*.ts` for `: any`.

---

## SECTION 9 — Phased Execution Roadmap (D1 → D6)

### 9.0 Shared runbook blocks (referenced by every phase)

**B — Pre-phase backup & git state check** (replace `Dn` with the phase id)
```bash
git pull --recurse-submodules
git status --porcelain            # MUST be empty; otherwise stop and ask
git log --oneline -1              # record as PRE_SHA in PROGRESS_LOG
git tag -a pre-Dn -m "pre-phase Dn backup"
git bundle create ../backups/localpdf-pre-Dn.bundle --all
```

**G — Automated validation gate** (every line must pass; a red gate stops the phase, DON'T #16)
```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo xtask protocol-lint
cargo xtask feature-map-check
cargo deny check licenses bans advisories
(cd app-tauri && pnpm check && pnpm vitest run)
cargo xtask control-check tests/control/Dn/*.jsonl     # headless control-protocol replay (feature control-headless)
cargo tauri build --bundles rpm
sudo dnf install -y "./target/release/bundle/rpm/LocalPDF Studio-0.1.0-1.x86_64.rpm"
timeout 20 localpdf-studio; echo "EXIT_CODE=$?"        # MUST print EXIT_CODE=124 (alive until killed)
unshare -rn timeout 20 localpdf-studio; echo "OFFLINE_EXIT_CODE=$?"  # F9: MUST be 124
```

**R — Rollback failsafe** (F15; never `reset --hard` or force-push without approval, DON'T #14)
```bash
# Uncommitted work-in-progress:
git stash push -u -m "Dn-failed-$(date +%s)"
# Already-committed phase commits:
git revert --no-edit pre-Dn..HEAD
# Then: ERROR_LOG entry (symptom, 3 attempts, hypothesis), CHECKPOINT -> "Dn ROLLED BACK", ask the user (F14).
```

**L — Log update & commit**
```bash
# 1) Append PROGRESS_LOG entry (Changes / Gate / Tests / Pre-commit checklist 1-11 / Protocol compliance / Risks / Commit)
# 2) Overwrite logs/CHECKPOINT.md (phase, last green tag, risks, next 3 actions, ADRs)
git add -A && git commit -m "<conventional message from the phase>"
git tag -a phase-Dn-green -m "Dn gate green"
git push origin main && git push --tags
```

---

### Phase D1 — Global Controllers, Control Bus, Spatial Canvas Shell, Pan/Zoom, Split/Multi-Window

**1. Backup:** block **B** with `Dn=D1`.

**Step 0 — Licence/API audit (hard gate):**
```bash
for r in cadcraft designcraft figcraft materialcraft texturecraft craft-anything; do
  git submodule add https://github.com/storytold/$r vendor/$r   # network needed: run with explicit user approval (dev-time only)
  (cd vendor/$r && git rev-parse HEAD && ls LICENSE* NOTICE* 2>/dev/null && head -n 5 LICENSE*)
done
cargo deny check licenses
```
For each repo, record in `ENGINE_NOTES.md`: pinned SHA, licence, the modules we take, and the verified function signatures. Any non-MIT/Apache result → stop and ask. Vendored submodules are **reference only**: we do not add them as workspace members (`Cargo.toml` `members` stays `crates/*`), and the code we adopt is ported into our crates with attribution headers.

**2. File-by-file spec**
| File | Content |
|---|---|
| `app-tauri/src/lib/core/themeContract.svelte.ts` | §1.1 verbatim |
| `app-tauri/src/lib/core/toolEngine.svelte.ts` | §1.2 verbatim |
| `app-tauri/src/lib/core/viewportEngine.svelte.ts` | §1.3 verbatim |
| `app-tauri/src/lib/core/commandEngine.svelte.ts` | §1.4 verbatim |
| `app-tauri/src/lib/core/defaultPresets.json` | presets for every `ToolId` used by the radial wedges (pen.a 0.75 pt black, pen.b 1.5 pt red, highlighter 12 pt yellow 0.4 multiply, etc.) |
| `crates/lpdf-control-bus/{Cargo.toml,src/lib.rs}` | §1.5 verbatim; deps `serde`, `serde_json`, `thiserror`, `ts-rs` (vet + record) |
| `crates/lpdf-control-bus/tests/replay.rs` | `EchoExecutor` test + malformed-line test + mismatch test |
| `app-tauri/src-tauri/src/commands/studio.rs` | `control_dispatch`, `prefs_save`, `prefs_load` (below) |
| `app-tauri/src/lib/views/studio/SpatialCanvas.svelte`, `SplitLayout.svelte` | §2.1 |
| `app-tauri/src/lib/core/*.test.ts` | vitest: Affine invert round-trip (1e-9), rotation 0/90/180/270 corners, calibrate guards, zoomAt anchor-fixed, undo/redo/coalesce, chord canonicalisation, pressure curve monotone |
| `xtask/src/lint/no_hardcode.rs` | anti-hardcoding lint (below) |
| `xtask/src/control_check.rs` | runs `lpdf_control_bus::replay` against an in-process executor over `tests/control/D1/*.jsonl` |
| `app-tauri/src-tauri/capabilities/studio-split.json` | `windows: ["split-*"]`, permissions `core:window:allow-create`, `core:event:allow-emit-to` only |

```rust
// app-tauri/src-tauri/src/commands/studio.rs
use lpdf_control_bus::{BusError, ControlMessage};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

#[derive(Debug, thiserror::Error, serde::Serialize)]
#[serde(tag = "code", content = "detail")]
pub enum PrefsError {
    #[error("invalid preference key")] InvalidKey,
    #[error("preference storage unavailable")] Storage(String),
    #[error("preference payload is not valid JSON")] InvalidJson,
}
fn prefs_path(app: &AppHandle, key: &str) -> Result<PathBuf, PrefsError> {
    if key.is_empty() || key.len() > 64 || !key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err(PrefsError::InvalidKey);
    }
    let dir = app.path().app_config_dir().map_err(|e| PrefsError::Storage(e.to_string()))?;
    Ok(dir.join("prefs").join(format!("{key}.json")))
}
#[tauri::command]
pub async fn prefs_save(app: AppHandle, key: String, json: String) -> Result<(), PrefsError> {
    serde_json::from_str::<serde_json::Value>(&json).map_err(|_| PrefsError::InvalidJson)?;
    let path = prefs_path(&app, &key)?;
    tauri::async_runtime::spawn_blocking(move || lpdf_fs_safety::atomic_write(&path, json.as_bytes()))
        .await.map_err(|e| PrefsError::Storage(e.to_string()))?
        .map_err(|e| PrefsError::Storage(e.to_string()))
}
#[tauri::command]
pub async fn prefs_load(app: AppHandle, key: String) -> Result<Option<String>, PrefsError> {
    let path = prefs_path(&app, &key)?;
    match tokio::fs::read_to_string(&path).await {
        Ok(s) => Ok(Some(s)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(PrefsError::Storage(e.to_string())),
    }
}
#[tauri::command]
pub fn control_validate(msg: ControlMessage) -> Result<ControlMessage, BusError> { Ok(msg) } // schema round-trip used by headless checks
```
> Before writing: confirm that the exact name/signature of `lpdf_fs_safety::atomic_write` exists in `crates/lpdf-fs-safety/src/lib.rs` (DO #2), and adapt the call to the real function. Do not invent it.

```rust
// xtask/src/lint/no_hardcode.rs — anti-hardcoding law (ADR-012 Decision 2)
use std::path::Path;
pub struct Violation { pub file: String, pub line: usize, pub rule: &'static str }
const SCOPED: [&str; 3] = ["app-tauri/src/lib/views", "app-tauri/src/lib/components", "app-tauri/src/lib/tools"];
pub fn scan_file(rel: &str, text: &str) -> Vec<Violation> {
    let mut out = Vec::new();
    if !SCOPED.iter().any(|s| rel.starts_with(s)) { return out; }
    for (i, line) in text.lines().enumerate() {
        let l = line.trim_start();
        if l.starts_with("//") || l.starts_with("/*") || l.starts_with('*') { continue; }
        let mut push = |rule| out.push(Violation { file: rel.to_owned(), line: i + 1, rule });
        if has_hex_colour(l) || l.contains("rgb(") || l.contains("rgba(") || l.contains("hsl(") { push("no-hardcoded-colour"); }
        if (l.contains("matrix(") || l.contains("scale(") || l.contains("translate(")) && !l.contains("viewport.") { push("no-local-transform"); }
        if l.contains("undoStack") || l.contains("redoStack") { push("no-local-history"); }
        if l.contains("'keydown'") || l.contains("\"keydown\"") || l.contains("onkeydown") { push("no-local-keydown"); }
    }
    out
}
fn has_hex_colour(l: &str) -> bool {
    let b = l.as_bytes();
    b.iter().enumerate().any(|(i, &c)| c == b'#' && {
        let n = b[i + 1..].iter().take_while(|x| x.is_ascii_hexdigit()).count();
        (n == 3 || n == 4 || n == 6 || n == 8) && b.get(i + 1 + n).map_or(true, |x| !x.is_ascii_alphanumeric())
    })
}
pub fn scan_tree(root: &Path) -> std::io::Result<Vec<Violation>> {
    let mut v = Vec::new();
    for dir in SCOPED { walk(&root.join(dir), root, &mut v)?; }
    Ok(v)
}
fn walk(d: &Path, root: &Path, v: &mut Vec<Violation>) -> std::io::Result<()> {
    if !d.exists() { return Ok(()); }
    for e in std::fs::read_dir(d)? {
        let p = e?.path();
        if p.is_dir() { walk(&p, root, v)?; continue; }
        let ok = p.extension().and_then(|x| x.to_str()).is_some_and(|x| matches!(x, "svelte" | "ts" | "css"));
        if ok { let rel = p.strip_prefix(root).map(|r| r.to_string_lossy().replace('\\', "/")).unwrap_or_default();
                v.extend(scan_file(&rel, &std::fs::read_to_string(&p)?)); }
    }
    Ok(())
}
```
Note on the lint: `onkeydown` is allowed on `<input>` elements inside the components that own them (e.g. a Property Pill numeric field), through an allow-list comment `// lint-allow: no-local-keydown (field-local)`. The scanner skips a line when the preceding line carries that marker; add that check when implementing.

`tests/control/D1/viewport.jsonl` (example):
```json
{"msg":{"kind":"execute","command":"view.create","args":{"id":"main","zoom":1,"panX":0,"panY":0,"linkGroup":null,"dpr":1}},"expect":{"ok":true}}
{"msg":{"kind":"execute","command":"view.zoomAt","args":{"id":"main","factor":2,"x":100,"y":100}},"expect":{"zoom":2,"panX":-100,"panY":-100}}
{"msg":{"kind":"ui.key","chord":"Ctrl+Z","phase":"down"},"expect":{"handled":false}}
```

**3. Gate:** block **G** (`Dn=D1`), plus the vitest suites above and `cargo test -p lpdf-control-bus`.
**4. Rollback:** block **R**.
**5. Log & commit:** block **L**. Commit `feat(studio-d1): global theme/tool/viewport/command engines, control bus, spatial canvas + split view`.

---

### Phase D2 — Dockable Toolbar, Property Pill, Favorites, 8-Wedge Radial Menu, Custom Shortcuts

**1. Backup:** **B** (`D2`).
**2. Files:**
| File | Content |
|---|---|
| `components/studio/MarkupToolbar.svelte` | §2.2 (dock state machine below) |
| `components/studio/FavoritesBar.svelte`, `PropertyPill.svelte` | §2.2; all edits through `commands.dispatch` |
| `components/studio/RadialMenu.svelte` + `core/radialGeometry.ts` + `core/radialLayout.json` | §2.3; geometry below |
| `views/settings/ShortcutsEditor.svelte` | lists `commands.shortcuts`, record-chord capture (uses `commandEngine.captureNextChord()`), conflict warning, reset to defaults, persisted with `prefs_save("shortcuts")` |
| `core/defaultShortcuts.json` | `V` select, `H` pan, `P` pen.a, `Shift+P` pen.b, `I` highlighter, `E` eraser.partial, `Shift+E` eraser.whole, `L` laser, `M` measure.length, `Ctrl+Z` undo, `Ctrl+Shift+Z`/`Ctrl+Y` redo, `Space` (hold) pan, `/` radial.open |

```ts
// app-tauri/src/lib/core/radialGeometry.ts
export const RADIAL = Object.freeze({ r0: 36, r1: 112, r2: 176, sectors: 8 });
export type RadialHit = { zone: 'cancel' } | { zone: 'wedge'; sector: number } | { zone: 'flyout'; sector: number; item: number } | { zone: 'outside' };
export function radialHit(dx: number, dy: number, flyoutCounts: readonly number[]): RadialHit {
  const rho = Math.hypot(dx, dy);
  if (rho < RADIAL.r0) return { zone: 'cancel' };
  if (rho >= RADIAL.r2) return { zone: 'outside' };
  const deg = (Math.atan2(dy, dx) * 180) / Math.PI;           // screen y-down: -90 = up
  const norm = (((deg + 90 + 22.5) % 360) + 360) % 360;
  const sector = Math.floor(norm / 45) % RADIAL.sectors;
  if (rho < RADIAL.r1) return { zone: 'wedge', sector };
  const n = flyoutCounts[sector] ?? 0;
  if (n === 0) return { zone: 'wedge', sector };
  const within = norm - sector * 45;                          // 0..45 across the wedge arc
  return { zone: 'flyout', sector, item: Math.min(n - 1, Math.floor((within / 45) * n)) };
}
export function clampCenter(x: number, y: number, w: number, h: number): { x: number; y: number } {
  const r = RADIAL.r2; return { x: Math.min(w - r, Math.max(r, x)), y: Math.min(h - r, Math.max(r, y)) };
}
```
```ts
// dock snapping (inside MarkupToolbar.svelte <script>)
type Dock = 'float' | 'top' | 'left' | 'right';
function resolveDock(x: number, y: number, w: number, vw: number, threshold: number): Dock {
  if (y <= threshold) return 'top';
  if (x <= threshold) return 'left';
  if (vw - (x + w) <= threshold) return 'right';
  return 'float';
}
```
Tests: `radialGeometry.test.ts` covers all 8 sector centres, boundary angles at ±22.5°, cancel/outside zones, and flyout item partition. `tests/control/D2/radial.jsonl` contains pointer down/move/up sequences selecting wedge 6 flyout item 1 → `tools.activeTool === 'eraser.partial'`.

**3. Gate:** **G** (`D2`). **4. Rollback:** **R**. **5. Log & commit:** **L**, `feat(studio-d2): dockable glass toolbar, property pill, favorites, 8-wedge radial menu, custom shortcuts`.

---

### Phase D3 — Pressure Ink, Highlighter, Laser, Smart Shape Recognition, Whole/Partial Eraser

**1. Backup:** **B** (`D3`).
**2. Files:** `crates/lpdf-vector-ink/src/{lib.rs,model.rs,smooth.rs,simplify.rs,recognize.rs,erase.rs,outline.rs}`, `app-tauri/src/lib/tools/{inkTool.svelte.ts,laserTool.svelte.ts,eraserTool.svelte.ts}`, `app-tauri/src/lib/tools/oneEuro.ts`, and the IPC `ink_commit` (stroke → `/Ink` + AP via pdfcraft annotation API, verified in step 0).

```rust
// crates/lpdf-vector-ink/src/simplify.rs
pub fn rdp(points: &[[f64; 2]], eps: f64) -> Vec<[f64; 2]> {
    if points.len() < 3 { return points.to_vec(); }
    let mut keep = vec![false; points.len()];
    let (first, last) = (0usize, points.len() - 1);
    keep[first] = true; keep[last] = true;
    let mut stack = vec![(first, last)];
    while let Some((s, e)) = stack.pop() {
        let (mut idx, mut dmax) = (s, 0.0);
        for i in s + 1..e { let d = seg_dist(points[i], points[s], points[e]); if d > dmax { dmax = d; idx = i; } }
        if dmax > eps { keep[idx] = true; stack.push((s, idx)); stack.push((idx, e)); }
    }
    points.iter().zip(keep).filter_map(|(p, k)| k.then_some(*p)).collect()
}
pub fn seg_dist(p: [f64; 2], a: [f64; 2], b: [f64; 2]) -> f64 {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let l2 = dx * dx + dy * dy;
    let t = if l2 > 0.0 { (((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / l2).clamp(0.0, 1.0) } else { 0.0 };
    ((p[0] - a[0] - t * dx).powi(2) + (p[1] - a[1] - t * dy).powi(2)).sqrt()
}
```
```rust
// crates/lpdf-vector-ink/src/recognize.rs
use crate::simplify::{rdp, seg_dist};
#[derive(Debug, Clone, PartialEq, serde::Serialize, ts_rs::TS)]
#[serde(tag = "shape", rename_all = "camelCase")] #[ts(export, export_to = "../../app-tauri/src/lib/bindings/")]
pub enum Recognized {
    Line { a: [f64; 2], b: [f64; 2] }, Polyline { points: Vec<[f64; 2]> },
    Triangle { points: Vec<[f64; 2]> }, Rectangle { points: Vec<[f64; 2]> }, Square { points: Vec<[f64; 2]> },
    Hexagon { points: Vec<[f64; 2]> }, Circle { cx: f64, cy: f64, r: f64 },
    Ellipse { cx: f64, cy: f64, rx: f64, ry: f64, rotation_deg: f64 }, None,
}
pub fn recognize(raw: &[[f64; 2]]) -> Recognized {
    if raw.len() < 2 { return Recognized::None; }
    let (minx, miny, maxx, maxy) = raw.iter().fold((f64::MAX, f64::MAX, f64::MIN, f64::MIN),
        |(a, b, c, d), p| (a.min(p[0]), b.min(p[1]), c.max(p[0]), d.max(p[1])));
    let diag = ((maxx - minx).powi(2) + (maxy - miny).powi(2)).sqrt();
    if diag < 1e-6 { return Recognized::None; }
    let (first, last) = (raw[0], raw[raw.len() - 1]);
    let len: f64 = raw.windows(2).map(|w| dist(w[0], w[1])).sum();
    let maxdev = raw.iter().map(|p| seg_dist(*p, first, last)).fold(0.0, f64::max);
    if maxdev < 0.03 * dist(first, last).max(1e-9) { return Recognized::Line { a: first, b: last }; }
    let closed = dist(first, last) < 0.15 * len;
    if closed {
        let (cx, cy, r, resid) = kasa_circle(raw);
        if r > 0.0 && resid / r < 0.08 { return Recognized::Circle { cx, cy, r }; }
        let (ex, ey, rx, ry, rot, eres) = pca_ellipse(raw);
        if rx / ry.max(1e-9) > 1.15 && eres < 0.10 { return Recognized::Ellipse { cx: ex, cy: ey, rx, ry, rotation_deg: rot }; }
    }
    let mut pts = rdp(raw, 0.02 * diag);
    if closed && pts.len() > 2 && dist(pts[0], pts[pts.len() - 1]) < 0.15 * len { pts.pop(); }
    if !closed { return if pts.len() >= 3 { Recognized::Polyline { points: pts } } else { Recognized::Line { a: first, b: last } }; }
    match pts.len() {
        3 => Recognized::Triangle { points: pts },
        4 if corners_near_right(&pts, 12.0) => {
            let (a, b) = (dist(pts[0], pts[1]), dist(pts[1], pts[2]));
            if (a - b).abs() / a.max(b) < 0.10 { Recognized::Square { points: pts } } else { Recognized::Rectangle { points: pts } }
        }
        6 => Recognized::Hexagon { points: pts },
        _ => Recognized::Polyline { points: pts },
    }
}
fn dist(a: [f64; 2], b: [f64; 2]) -> f64 { ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt() }
fn corners_near_right(p: &[[f64; 2]], tol_deg: f64) -> bool {
    (0..p.len()).all(|i| {
        let (a, b, c) = (p[(i + p.len() - 1) % p.len()], p[i], p[(i + 1) % p.len()]);
        let (u, v) = ([a[0] - b[0], a[1] - b[1]], [c[0] - b[0], c[1] - b[1]]);
        let ang = (u[0] * v[1] - u[1] * v[0]).abs().atan2(u[0] * v[0] + u[1] * v[1]).to_degrees();
        (ang - 90.0).abs() <= tol_deg
    })
}
/// Kåsa least-squares circle: solve [Σx² Σxy Σx; Σxy Σy² Σy; Σx Σy n]·[D E F]ᵀ = −[Σx(x²+y²); Σy(x²+y²); Σ(x²+y²)].
fn kasa_circle(p: &[[f64; 2]]) -> (f64, f64, f64, f64) {
    let n = p.len() as f64;
    let (mut sx, mut sy, mut sxx, mut syy, mut sxy, mut sxz, mut syz, mut sz) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    for q in p { let (x, y) = (q[0], q[1]); let z = x * x + y * y;
        sx += x; sy += y; sxx += x * x; syy += y * y; sxy += x * y; sxz += x * z; syz += y * z; sz += z; }
    let m = [[sxx, sxy, sx], [sxy, syy, sy], [sx, sy, n]];
    let Some([d, e, f]) = solve3(m, [-sxz, -syz, -sz]) else { return (0.0, 0.0, 0.0, f64::MAX) };
    let (cx, cy) = (-d / 2.0, -e / 2.0);
    let r2 = cx * cx + cy * cy - f; if r2 <= 0.0 { return (0.0, 0.0, 0.0, f64::MAX); }
    let r = r2.sqrt();
    let var = p.iter().map(|q| (dist(*q, [cx, cy]) - r).powi(2)).sum::<f64>() / n;
    (cx, cy, r, var.sqrt())
}
fn solve3(m: [[f64; 3]; 3], b: [f64; 3]) -> Option<[f64; 3]> {
    let det = |m: [[f64; 3]; 3]| m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1]) - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0]) + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0]);
    let d = det(m); if d.abs() < 1e-12 { return None; }
    let mut out = [0.0; 3];
    for (k, o) in out.iter_mut().enumerate() { let mut mk = m; for r in 0..3 { mk[r][k] = b[r]; } *o = det(mk) / d; }
    Some(out)
}
/// PCA ellipse: axes from covariance eigenvectors, radii = √2·σ; residual = mean |(x'/rx)²+(y'/ry)²−1|.
fn pca_ellipse(p: &[[f64; 2]]) -> (f64, f64, f64, f64, f64, f64) {
    let n = p.len() as f64;
    let (cx, cy) = (p.iter().map(|q| q[0]).sum::<f64>() / n, p.iter().map(|q| q[1]).sum::<f64>() / n);
    let (mut a, mut b, mut c) = (0.0, 0.0, 0.0);
    for q in p { let (x, y) = (q[0] - cx, q[1] - cy); a += x * x; b += x * y; c += y * y; }
    a /= n; b /= n; c /= n;
    let tr = a + c; let disc = ((a - c).powi(2) / 4.0 + b * b).sqrt();
    let (l1, l2) = (tr / 2.0 + disc, (tr / 2.0 - disc).max(1e-12));
    let theta = 0.5 * (2.0 * b).atan2(a - c);
    let (rx, ry) = ((2.0 * l1).sqrt(), (2.0 * l2).sqrt());
    let (cs, sn) = (theta.cos(), theta.sin());
    let res = p.iter().map(|q| { let (x, y) = (q[0] - cx, q[1] - cy);
        let (u, v) = (x * cs + y * sn, -x * sn + y * cs); ((u / rx).powi(2) + (v / ry).powi(2) - 1.0).abs() }).sum::<f64>() / n;
    (cx, cy, rx, ry, theta.to_degrees(), res)
}
```
```rust
// crates/lpdf-vector-ink/src/erase.rs — partial erase of a centreline by a swept capsule (a→b, radius r)
use crate::model::InkSample;
fn inside(p: (f64, f64), a: (f64, f64), b: (f64, f64), r: f64) -> bool {
    crate::simplify::seg_dist([p.0, p.1], [a.0, a.1], [b.0, b.1]) <= r
}
/// Returns surviving pieces (each ≥ 2 samples). Segments are subdivided at `step` so boundaries are accurate to step/2.
pub fn partial_erase(stroke: &[InkSample], a: (f64, f64), b: (f64, f64), r: f64, step: f64) -> Vec<Vec<InkSample>> {
    let mut pieces: Vec<Vec<InkSample>> = Vec::new();
    let mut cur: Vec<InkSample> = Vec::new();
    let step = step.max(1e-3);
    for w in stroke.windows(2) {
        let (p, q) = (&w[0], &w[1]);
        let n = ((((q.x - p.x).powi(2) + (q.y - p.y).powi(2)).sqrt() / step).ceil() as usize).max(1);
        for k in 0..n {
            let t = k as f64 / n as f64;
            let s = InkSample { x: p.x + (q.x - p.x) * t, y: p.y + (q.y - p.y) * t, w: p.w + (q.w - p.w) * t, t_ms: p.t_ms + (q.t_ms - p.t_ms) * t };
            if inside((s.x, s.y), a, b, r + s.w / 2.0) { if cur.len() >= 2 { pieces.push(std::mem::take(&mut cur)); } else { cur.clear(); } }
            else { cur.push(s); }
        }
    }
    if let Some(l) = stroke.last() { if !inside((l.x, l.y), a, b, r + l.w / 2.0) { cur.push(l.clone()); } }
    if cur.len() >= 2 { pieces.push(cur); }
    pieces
}
```
Tests: unit tests for each `Recognized` variant using synthetic noisy shapes (seeded noise ±1.5%); a proptest that `rdp` output is a subsequence that keeps both endpoints; a proptest that `partial_erase` with r=0 returns one piece equal in arc length (±step); a hostile-input test (NaN/inf samples are rejected upstream by `InkSample::validate` returning `InkError::NonFinite`). Ink latency check: the vitest perf harness asserts wet-layer draw per coalesced batch stays under 4 ms on the CI tier.

**3. Gate:** **G** (`D3`). **4. Rollback:** **R**. **5. Log & commit:** **L**, `feat(studio-d3): pressure vector ink, highlighter, laser, smart shape recognition, whole/partial eraser`.

---

### Phase D4 — Inline Text/Object Editing, Shapes, Hatching, Align/Flip/Group, Snapshot

**1. Backup:** **B** (`D4`). **Step 0:** read pdfcraft's text-extraction, content-stream editing, and font-embedding modules at `75360d1` and record the verified signatures in `ENGINE_NOTES.md`. Without them, stop.
**2. Files:** `crates/lpdf-layout-edit/src/{lib.rs,runs.rs,paragraph.rs,knuth_plass.rs,rewrite.rs,error.rs}`, `crates/lpdf-vector-ink/src/pattern.rs`, `crates/lpdf-cad-measure/src/hatch.rs`, `app-tauri/src/lib/tools/{textEditTool.svelte.ts,objectTool.svelte.ts,shapeTools.svelte.ts,hatchTool.svelte.ts,snapshotTool.svelte.ts}`, and commands `layout_extract_paragraphs`, `layout_apply_edit`, `object_transform`, `hatch_pick_region` (all worker jobs).

```rust
// crates/lpdf-layout-edit/src/knuth_plass.rs — total-fit line breaking (Knuth & Plass 1981)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Item { Box { w: f64 }, Glue { w: f64, stretch: f64, shrink: f64 }, Penalty { w: f64, cost: f64, flagged: bool } }
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum BreakError { #[error("no feasible breaks within tolerance")] Infeasible, #[error("line width must be positive")] BadWidth }
const INF_PENALTY: f64 = 10_000.0;
#[derive(Clone, Copy)] struct Node { pos: usize, line: usize, demerits: f64, prev: Option<usize>, w: f64, y: f64, z: f64 }
/// Returns break indices (into `items`) for each line end. `tolerance` = max adjustment ratio (e.g. 2.0; retry with 10.0).
pub fn breaks(items: &[Item], line_width: f64, tolerance: f64) -> Result<Vec<usize>, BreakError> {
    if !(line_width > 0.0) { return Err(BreakError::BadWidth); }
    let mut nodes = vec![Node { pos: 0, line: 0, demerits: 0.0, prev: None, w: 0.0, y: 0.0, z: 0.0 }];
    let mut active: Vec<usize> = vec![0];
    let (mut sw, mut sy, mut sz) = (0.0, 0.0, 0.0);
    for (i, it) in items.iter().enumerate() {
        let legal = match it { Item::Penalty { cost, .. } => *cost < INF_PENALTY,
            Item::Glue { .. } => i > 0 && matches!(items[i - 1], Item::Box { .. }), Item::Box { .. } => false };
        if legal {
            let (pw, pcost, pflag) = match it { Item::Penalty { w, cost, flagged } => (*w, *cost, *flagged), _ => (0.0, 0.0, false) };
            let mut best: Option<(f64, usize)> = None;
            active.retain(|&a| {
                let n = nodes[a];
                let width = sw - n.w + pw;
                let r = if width < line_width { let st = sy - n.y; if st > 0.0 { (line_width - width) / st } else { f64::INFINITY } }
                    else if width > line_width { let sh = sz - n.z; if sh > 0.0 { (line_width - width) / sh } else { f64::NEG_INFINITY } } else { 0.0 };
                let forced = matches!(it, Item::Penalty { cost, .. } if *cost <= -INF_PENALTY);
                if r < -1.0 || forced { /* deactivate after considering */ }
                if r >= -1.0 && r <= tolerance {
                    let bad = 100.0 * r.abs().powi(3);
                    let base = (1.0 + bad).powi(2);
                    let mut d = if pcost >= 0.0 { base + pcost * pcost } else if pcost > -INF_PENALTY { base - pcost * pcost } else { base };
                    if pflag { if let Some(Item::Penalty { flagged: true, .. }) = items.get(n.pos) { d += 3000.0; } }
                    let total = n.demerits + d;
                    if best.map_or(true, |(bd, _)| total < bd) { best = Some((total, a)); }
                }
                !(r < -1.0 || forced)
            });
            if let Some((d, a)) = best {
                let (mut aw, mut ay, mut az) = (sw, sy, sz);
                for nx in &items[i..] { match nx { Item::Glue { w, stretch, shrink } => { aw += w; ay += stretch; az += shrink; }
                    Item::Box { .. } => break, Item::Penalty { cost, .. } if *cost <= -INF_PENALTY && nx != it => break, _ => {} } }
                nodes.push(Node { pos: i, line: nodes[a].line + 1, demerits: d, prev: Some(a), w: aw, y: ay, z: az });
                active.push(nodes.len() - 1);
            }
            if active.is_empty() { return Err(BreakError::Infeasible); }
        }
        match it { Item::Box { w } => sw += w, Item::Glue { w, stretch, shrink } => { sw += w; sy += stretch; sz += shrink; }, Item::Penalty { .. } => {} }
    }
    let end = active.iter().copied().filter(|&a| nodes[a].pos + 1 == items.len() || nodes[a].pos == items.len() - 1)
        .min_by(|&a, &b| nodes[a].demerits.total_cmp(&nodes[b].demerits)).ok_or(BreakError::Infeasible)?;
    let mut out = Vec::new(); let mut cur = Some(end);
    while let Some(c) = cur { if nodes[c].prev.is_some() { out.push(nodes[c].pos); } cur = nodes[c].prev; }
    out.reverse(); Ok(out)
}
```
Paragraph items are built by `paragraph::to_items(text, metrics, size, tracking)`. Each word becomes `Box{w}`, each inter-word space becomes `Glue{w: space, stretch: space/2, shrink: space/3}`, and the paragraph ends with `Glue{0, ∞-proxy 1e9, 0}, Penalty{0, -INF, true}`. Hyphenation is out of scope for D4 (penalties only at spaces). The caller tries `tolerance=2.0`, then `10.0`, then falls back to greedy fill and shows a "loose line" notice.

Tests: golden paragraphs (equal-width boxes → known breaks), the forced final break, the infeasible single over-wide box returns `Infeasible`, and a proptest that breaks are strictly increasing and legal.

```rust
// crates/lpdf-vector-ink/src/pattern.rs — procedural hatch tile generators (unit tile 0..1, scaled/rotated by caller)
use crate::model::HatchPatternId;
pub type Seg = ([f64; 2], [f64; 2]);
fn hash01(i: u32) -> f64 { let mut x = i.wrapping_mul(0x9E37_79B9) ^ 0x85EB_CA6B; x ^= x >> 16; x = x.wrapping_mul(0x7FEB_352D); x ^= x >> 15; f64::from(x) / f64::from(u32::MAX) }
pub fn unit_tile(p: HatchPatternId) -> Vec<Seg> {
    match p {
        HatchPatternId::Steel => vec![([0.0, 0.0], [1.0, 1.0]), ([0.0, 0.5], [0.5, 1.0]), ([0.5, 0.0], [1.0, 0.5])],
        HatchPatternId::Tiling => vec![([0.0, 0.0], [1.0, 0.0]), ([0.0, 0.0], [0.0, 1.0])],
        HatchPatternId::Masonry => vec![([0.0, 0.0], [1.0, 0.0]), ([0.0, 0.5], [1.0, 0.5]), ([0.5, 0.0], [0.5, 0.5]), ([0.0, 0.5], [0.0, 1.0])],
        HatchPatternId::Earth => vec![([0.05, 0.2], [0.45, 0.2]), ([0.55, 0.7], [0.95, 0.7]), ([0.7, 0.2], [0.72, 0.2]), ([0.2, 0.7], [0.22, 0.7])],
        HatchPatternId::Insulation => (0..16).map(|k| { let (t0, t1) = (k as f64 / 16.0, (k + 1) as f64 / 16.0);
            let f = |t: f64| [t, 0.5 + 0.4 * (t * std::f64::consts::TAU).sin()]; (f(t0), f(t1)) }).collect(),
        HatchPatternId::Timber => (0..3).flat_map(|band| (0..12).map(move |k| { let (t0, t1) = (k as f64 / 12.0, (k + 1) as f64 / 12.0);
            let y = 0.2 + 0.3 * band as f64; let f = |t: f64| [t, y + 0.05 * (t * std::f64::consts::TAU * 2.0).sin()]; (f(t0), f(t1)) })).collect(),
        HatchPatternId::Concrete => (0..10u32).flat_map(|i| { let (x, y, s) = (hash01(i * 3), hash01(i * 3 + 1), 0.04 + 0.05 * hash01(i * 3 + 2));
            if i % 3 == 0 { vec![([x, y], [x + s, y]), ([x + s, y], [x + s / 2.0, y + s]), ([x + s / 2.0, y + s], [x, y])] }
            else { vec![([x, y], [x + 0.01, y])] } }).collect(),
        HatchPatternId::Solid => Vec::new(),
    }
}
```
Pick-point hatch (`hatch.rs`): planar arrangement → face walk → islands, as described in §3.4. Tests cover a square with an inner square island (area = outer − inner), an open boundary (returns `HatchError::OpenBoundary` with the gap location, never a silent fill), and a nested 3-level island parity check.

Object ops: `object_transform(doc, page, target: ObjectRef, m: [f64; 6])` pre-multiplies the target's `cm` matrix. Flip, rotate, align, and distribute compute `m` in TS (`core/geometryOps.ts`, unit-tested) and dispatch one command per selection. Group is overlay-only for annotations. "Group native objects" wraps them in a Form XObject (verified pdfcraft API).

**3. Gate:** **G** (`D4`) plus the corpus test: edit 20 corpus paragraphs → re-extract text equals the edited text; F6 verify passes; untouched pages hash-equal.
**4. Rollback:** **R**. **5. Log & commit:** **L**, `feat(studio-d4): inline text reflow editing, object transform/align/flip/group, shapes, procedural + pick-point hatching, snapshot`.

---

### Phase D5 — OSNAP, Scale Calibration, Ruler, Protractor, Length/Polylength/Area

**1. Backup:** **B** (`D5`). Vet `rstar`, `kurbo`, `i_overlay`, `dxf` (if used) per file 08.
**2. Files:** `crates/lpdf-cad-measure/src/{lib.rs,snap.rs,measure.rs,calibrate.rs,dxf_read.rs,error.rs}`, `app-tauri/src/lib/tools/{measureTools.svelte.ts,calibrateTool.svelte.ts,rulerWidget.svelte,protractorTool.svelte.ts}`, `components/studio/SnapIndicator.svelte`, commands `snap_index_page`, `snap_query`, `measure_commit`.

```rust
// crates/lpdf-cad-measure/src/snap.rs
use rstar::{RTree, RTreeObject, AABB, PointDistance};
use lpdf_vector_ink::model::SnapKind;
#[derive(Debug, Clone, Copy, PartialEq)] pub struct Seg { pub a: [f64; 2], pub b: [f64; 2], pub id: u32 }
impl RTreeObject for Seg { type Envelope = AABB<[f64; 2]>;
    fn envelope(&self) -> Self::Envelope { AABB::from_corners(self.a, self.b) } }
impl PointDistance for Seg { fn distance_2(&self, p: &[f64; 2]) -> f64 { crate::measure::seg_dist(*p, self.a, self.b).powi(2) } }
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)] pub struct SnapHit { pub kind: SnapKind, pub p: [f64; 2], pub seg: Option<u32>, pub score: f64 }
pub struct SnapIndex { tree: RTree<Seg> }
impl SnapIndex {
    pub fn build(segs: Vec<Seg>) -> Self { Self { tree: RTree::bulk_load(segs) } }
    pub fn query(&self, p: [f64; 2], tol: f64, anchor: Option<[f64; 2]>, grid: Option<f64>) -> Option<SnapHit> {
        if !(tol > 0.0) || !p[0].is_finite() || !p[1].is_finite() { return None; }
        let near: Vec<&Seg> = self.tree.locate_in_envelope_intersecting(&AABB::from_corners([p[0] - tol, p[1] - tol], [p[0] + tol, p[1] + tol])).collect();
        let mut best: Option<SnapHit> = None;
        let mut offer = |kind: SnapKind, q: [f64; 2], seg: Option<u32>, prio: f64| {
            let d = ((q[0] - p[0]).powi(2) + (q[1] - p[1]).powi(2)).sqrt();
            if d <= tol { let score = d / tol + prio; if best.map_or(true, |b| score < b.score) { best = Some(SnapHit { kind, p: q, seg, score }); } }
        };
        for s in &near {
            offer(SnapKind::Endpoint, s.a, Some(s.id), 0.0); offer(SnapKind::Endpoint, s.b, Some(s.id), 0.0);
            offer(SnapKind::Midpoint, [(s.a[0] + s.b[0]) / 2.0, (s.a[1] + s.b[1]) / 2.0], Some(s.id), 0.1);
            if let Some(an) = anchor { if let Some(f) = foot(an, s.a, s.b) { offer(SnapKind::Perpendicular, f, Some(s.id), 0.15); } }
            if let Some(f) = foot(p, s.a, s.b) { offer(SnapKind::Nearest, f, Some(s.id), 0.4); }
        }
        for (i, s) in near.iter().enumerate() { for t in &near[i + 1..] {
            if let Some(x) = intersect(s, t) { offer(SnapKind::Intersection, x, Some(s.id), 0.05); } } }
        if let Some(g) = grid.filter(|g| *g > 0.0) { offer(SnapKind::Grid, [(p[0] / g).round() * g, (p[1] / g).round() * g], None, 0.5); }
        best
    }
}
fn foot(p: [f64; 2], a: [f64; 2], b: [f64; 2]) -> Option<[f64; 2]> {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]); let l2 = dx * dx + dy * dy; if l2 <= 0.0 { return None; }
    let t = ((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / l2; (0.0..=1.0).contains(&t).then(|| [a[0] + t * dx, a[1] + t * dy])
}
fn intersect(s: &Seg, t: &Seg) -> Option<[f64; 2]> {
    let (r, q) = ([s.b[0] - s.a[0], s.b[1] - s.a[1]], [t.b[0] - t.a[0], t.b[1] - t.a[1]]);
    let den = r[0] * q[1] - r[1] * q[0]; if den.abs() < 1e-12 { return None; }
    let w = [t.a[0] - s.a[0], t.a[1] - s.a[1]];
    let (u, v) = ((w[0] * q[1] - w[1] * q[0]) / den, (w[0] * r[1] - w[1] * r[0]) / den);
    ((0.0..=1.0).contains(&u) && (0.0..=1.0).contains(&v)).then(|| [s.a[0] + u * r[0], s.a[1] + u * r[1]])
}
```
```rust
// crates/lpdf-cad-measure/src/measure.rs
pub fn seg_dist(p: [f64; 2], a: [f64; 2], b: [f64; 2]) -> f64 { lpdf_vector_ink::simplify::seg_dist(p, a, b) }
pub fn polylength(pts: &[[f64; 2]], k: f64) -> f64 { k * pts.windows(2).map(|w| ((w[1][0] - w[0][0]).powi(2) + (w[1][1] - w[0][1]).powi(2)).sqrt()).sum::<f64>() }
pub fn polygon_area(pts: &[[f64; 2]], k: f64) -> f64 {
    let n = pts.len(); if n < 3 { return 0.0; }
    let s: f64 = (0..n).map(|i| { let (a, b) = (pts[i], pts[(i + 1) % n]); a[0] * b[1] - b[0] * a[1] }).sum();
    k * k * s.abs() / 2.0
}
pub fn angle_deg(a: [f64; 2], vtx: [f64; 2], b: [f64; 2]) -> f64 {
    let (u, v) = ([a[0] - vtx[0], a[1] - vtx[1]], [b[0] - vtx[0], b[1] - vtx[1]]);
    (u[0] * v[1] - u[1] * v[0]).abs().atan2(u[0] * v[0] + u[1] * v[1]).to_degrees()
}
pub fn format_measure(value: f64, unit: &str, precision: usize, area: bool) -> String {
    if area { format!("{value:.precision$} {unit}²") } else { format!("{value:.precision$} {unit}") }
}
```
Self-intersecting polygons: `polygon_area` is used only after `i_overlay` simplifies the ring into non-overlapping faces (a "self-intersecting" badge is shown).

Tests: snap ranking (endpoint beats nearest at equal distance; intersection of an X), proptests that area is invariant under rotation/translation and that `polylength(k)` is linear in k, the calibration round trip (calibrate 100 pt = 1 m → measure 50 pt = 0.5 m), and the `/Measure` dictionary round trip through pdfcraft write → read.

**3. Gate:** **G** (`D5`) plus a snap query perf test (p99 < 1 ms on a 100k-segment page, `cargo bench` smoke).
**4. Rollback:** **R**. **5. Log & commit:** **L**, `feat(studio-d5): OSNAP content snapping, scale calibration, ruler, protractor, length/polylength/area measurement`.

---

### Phase D6 — Insert Suite, Overlays/Templates, Annotation History + Takeoff Export, Inline AI

**1. Backup:** **B** (`D6`). Vet `rust_xlsxwriter`, `notify` (licence pin), MathJax 3 (vendored to `app-tauri/static/vendor/mathjax/`, SHA-256 recorded), and the OFL script font.
**2. Files:** `tools/{signatureTool,audioNoteTool,latexTool,calloutTool,noteTool,imageTool,linkTool,libraryTool}.svelte.ts`, `workers/latex.worker.ts`, `core/svgPathToPdf.ts`, `components/studio/{AnnotationHistory.svelte,OverlayPicker.svelte,MarkupLibrary.svelte,AdvancedPasteDialog.svelte,AiSelectionBar.svelte}`, `crates/lpdf-cad-measure/src/{takeoff.rs,dxf_write.rs}`, and commands `takeoff_export`, `inbox_watch_set`, `sound_annot_commit`.

```rust
// crates/lpdf-cad-measure/src/takeoff.rs
use lpdf_vector_ink::model::SceneNode;
#[derive(Debug, thiserror::Error)] pub enum TakeoffError {
    #[error("io: {0}")] Io(#[from] std::io::Error), #[error("xlsx: {0}")] Xlsx(String) }
pub struct Row { pub page: u32, pub kind: String, pub label: String, pub value: f64, pub unit: String, pub perimeter: Option<f64>, pub tags: String }
pub fn rows(nodes: &[SceneNode]) -> Vec<Row> {
    nodes.iter().filter_map(|n| n.measure.as_ref().map(|m| Row { page: n.page + 1, kind: m.kind.clone(),
        label: match &n.geometry { lpdf_vector_ink::model::SceneGeometry::Text { text, .. } => text.clone(), _ => String::new() },
        value: m.value, unit: m.unit.clone(), perimeter: m.perimeter, tags: n.tags.join(";") })).collect()
}
fn csv_field(s: &str) -> String {
    let needs = s.contains([',', '"', '\n', '\r']) || s.starts_with(['=', '+', '-', '@']); // also blocks CSV formula injection
    let body = if s.starts_with(['=', '+', '-', '@']) { format!("'{s}") } else { s.to_owned() };
    if needs { format!("\"{}\"", body.replace('"', "\"\"")) } else { body }
}
pub fn to_csv(rows: &[Row]) -> String {
    let mut out = String::from("page,kind,label,value,unit,perimeter,tags\r\n");
    for r in rows { out.push_str(&format!("{},{},{},{},{},{},{}\r\n", r.page, csv_field(&r.kind), csv_field(&r.label), r.value,
        csv_field(&r.unit), r.perimeter.map(|p| p.to_string()).unwrap_or_default(), csv_field(&r.tags))); }
    out
}
pub fn to_xlsx(rows: &[Row]) -> Result<Vec<u8>, TakeoffError> {
    use rust_xlsxwriter::{Workbook, Formula};
    let mut wb = Workbook::new(); let ws = wb.add_worksheet();
    let x = |e: rust_xlsxwriter::XlsxError| TakeoffError::Xlsx(e.to_string());
    for (c, h) in ["Page", "Kind", "Label", "Value", "Unit", "Perimeter", "Tags"].iter().enumerate() { ws.write_string(0, c as u16, *h).map_err(x)?; }
    for (i, r) in rows.iter().enumerate() { let row = (i + 1) as u32;
        ws.write_number(row, 0, f64::from(r.page)).map_err(x)?; ws.write_string(row, 1, &r.kind).map_err(x)?;
        ws.write_string(row, 2, &r.label).map_err(x)?; ws.write_number(row, 3, r.value).map_err(x)?;
        ws.write_string(row, 4, &r.unit).map_err(x)?;
        if let Some(p) = r.perimeter { ws.write_number(row, 5, p).map_err(x)?; }
        ws.write_string(row, 6, &r.tags).map_err(x)?; }
    let total = (rows.len() + 1) as u32;
    ws.write_string(total, 2, "Total").map_err(x)?;
    ws.write_formula(total, 3, Formula::new(format!("=SUM(D2:D{})", rows.len() + 1))).map_err(x)?;
    wb.save_to_buffer().map_err(x)
}
```
> Verify the `rust_xlsxwriter` method names against the vetted version before use (DO #2). In the real code, the per-unit totals use `SUMIF` on the Unit column, so mixed units are never summed together.

```rust
// crates/lpdf-cad-measure/src/dxf_write.rs — minimal R12 ASCII DXF (LINE / LWPOLYLINE-as-POLYLINE / TEXT)
pub enum DxfEnt<'a> { Line([f64; 2], [f64; 2]), Poly(&'a [[f64; 2]], bool), Text([f64; 2], f64, &'a str) }
pub fn write_r12(ents: &[DxfEnt], scale: f64) -> String {
    let mut s = String::from("0\nSECTION\n2\nENTITIES\n");
    for e in ents { match e {
        DxfEnt::Line(a, b) => s.push_str(&format!("0\nLINE\n8\nLPDF\n10\n{}\n20\n{}\n30\n0\n11\n{}\n21\n{}\n31\n0\n", a[0] * scale, a[1] * scale, b[0] * scale, b[1] * scale)),
        DxfEnt::Poly(pts, closed) => { s.push_str(&format!("0\nPOLYLINE\n8\nLPDF\n66\n1\n70\n{}\n", i32::from(*closed)));
            for p in *pts { s.push_str(&format!("0\nVERTEX\n8\nLPDF\n10\n{}\n20\n{}\n30\n0\n", p[0] * scale, p[1] * scale)); }
            s.push_str("0\nSEQEND\n"); }
        DxfEnt::Text(p, h, t) => s.push_str(&format!("0\nTEXT\n8\nLPDF\n10\n{}\n20\n{}\n30\n0\n40\n{}\n1\n{}\n", p[0] * scale, p[1] * scale, h * scale,
            t.replace(['\n', '\r'], " "))),
    } }
    s.push_str("0\nENDSEC\n0\nEOF\n"); s
}
```
```ts
// app-tauri/src/lib/core/svgPathToPdf.ts — absolute SVG path commands → PDF path operators (y-flip applied by caller matrix)
export function svgPathToPdfOps(d: string): string {
  const tok = d.match(/[MLHVCQZmlhvcqz]|-?\d*\.?\d+(?:e[-+]?\d+)?/g) ?? [];
  let i = 0, cx = 0, cy = 0, sx = 0, sy = 0, cmd = ''; const out: string[] = [];
  const num = (): number => Number(tok[i++]);
  const f = (n: number): string => (Math.round(n * 1000) / 1000).toString();
  while (i < tok.length) {
    if (/[A-Za-z]/.test(tok[i] ?? '')) cmd = tok[i++] ?? '';
    const rel = cmd === cmd.toLowerCase(); const ox = rel ? cx : 0, oy = rel ? cy : 0;
    switch (cmd.toUpperCase()) {
      case 'M': cx = ox + num(); cy = oy + num(); sx = cx; sy = cy; out.push(`${f(cx)} ${f(cy)} m`); cmd = rel ? 'l' : 'L'; break;
      case 'L': cx = ox + num(); cy = oy + num(); out.push(`${f(cx)} ${f(cy)} l`); break;
      case 'H': cx = (rel ? cx : 0) + num(); out.push(`${f(cx)} ${f(cy)} l`); break;
      case 'V': cy = (rel ? cy : 0) + num(); out.push(`${f(cx)} ${f(cy)} l`); break;
      case 'C': { const x1 = ox + num(), y1 = oy + num(), x2 = ox + num(), y2 = oy + num(); cx = ox + num(); cy = oy + num();
        out.push(`${f(x1)} ${f(y1)} ${f(x2)} ${f(y2)} ${f(cx)} ${f(cy)} c`); break; }
      case 'Q': { const qx = ox + num(), qy = oy + num(), ex = ox + num(), ey = oy + num();
        out.push(`${f(cx + (2 / 3) * (qx - cx))} ${f(cy + (2 / 3) * (qy - cy))} ${f(ex + (2 / 3) * (qx - ex))} ${f(ey + (2 / 3) * (qy - ey))} ${f(ex)} ${f(ey)} c`);
        cx = ex; cy = ey; break; }
      case 'Z': out.push('h'); cx = sx; cy = sy; break;
      default: return out.join('\n'); // unknown command: stop; caller validates via round-trip test and surfaces LatexError::UnsupportedPath
    }
  }
  return out.join('\n');
}
```
(MathJax SVG output uses only M/L/H/V/C/Q/Z plus `<use>` and `transform`. The `S`/`T`/`A` commands are rejected with a typed error rather than approximated silently. A test asserts that 200 corpus equations convert with zero rejections.)

Audio, signatures, overlays, Advanced Paste, the history table, the watch-folder, and AI follow §6–§7. AI tiles stay greyed out with `ai-local` off. Tests cover: the CSV formula-injection case (`=HYPERLINK(...)` is quoted and prefixed), XLSX opened by `calamine` in tests (dev-dependency only, licence vetted), DXF re-parse with the D5 reader, the `/Sound` stream round trip, Advanced Paste over `1-3,odd` producing the exact pt coordinates, and AI flag off → zero AI worker spawn (process count assertion).

**3. Gate:** **G** (`D6`) plus the offline e2e under `unshare -n` that covers LaTeX insertion and takeoff export.
**4. Rollback:** **R**. **5. Log & commit:** **L**, `feat(studio-d6): insert suite (LaTeX, audio, signature, callouts, library, advanced paste), overlays, annotation history + CSV/XLSX/DXF takeoff, inline local AI`.

---

### 9.7 Phase-exit self-audit (every phase, file 04)
Diff the phase against `02` + `03` line by line, then write the PROGRESS_LOG "Protocol compliance" entry. Expected failsafes per phase: D1 F1/F11/F13; D3 F10/F11; D4 F1/F6/F7/F25/F26/F27; D5 F4/F27; D6 F1/F5/F9/F18/F19/F20. Missing evidence means the phase is not compliant.

### 9.8 Resuming the Windows track
After D6 (or whenever the user asks): `git checkout main && git pull --recurse-submodules`, set CHECKPOINT to "Phase W2", and execute `docs/WINDOWS_MASTER_PLAN.md` §W2–W4. The D-phase engines are cross-platform. W2 vibrancy must source its colours from `themeContract` (`--glass-*`) per ADR-012 Decision 2.

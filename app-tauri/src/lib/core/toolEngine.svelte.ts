// app-tauri/src/lib/core/toolEngine.svelte.ts
import type { HatchPatternId } from '$lib/bindings/HatchPatternId';
import defaultPresetsData from './defaultPresets.json';

export type ToolId =
  | 'pan'
  | 'select'
  | 'pen.a'
  | 'pen.b'
  | 'highlighter'
  | 'eraser.partial'
  | 'eraser.whole'
  | 'laser'
  | 'text.review'
  | 'shape'
  | 'measure';

export const TOOL_LABELS: Record<ToolId, string> = {
  'pan': 'Pan Canvas',
  'select': 'Select Object',
  'pen.a': 'Pen A',
  'pen.b': 'Pen B',
  'highlighter': 'Highlighter',
  'eraser.partial': 'Partial Eraser',
  'eraser.whole': 'Whole Eraser',
  'laser': 'Laser Pointer',
  'text.review': 'Text Review',
  'shape': 'Shape Tool',
  'measure': 'Measurement Suite'
};

/** Cubic Bézier pressure curve through (0,0),(x1,y1),(x2,y2),(1,1) — texturecraft-style stamp curve. */
export interface PressureCurve { x1: number; y1: number; x2: number; y2: number }

export interface StrokeStyle {
  colorToken: string | null;
  colorHex: string;
  widthPt: number;
  opacity: number;
  dash: readonly number[];
  lineCap: 'butt' | 'round' | 'square';
  startEnd: LineEnding;
  endEnd: LineEnding;
}

export type LineEnding = 'none' | 'open' | 'closed' | 'circle' | 'square' | 'diamond' | 'slash' | 'butt';

export interface FillStyle {
  kind: 'none' | 'solid' | 'hatch';
  colorHex: string;
  opacity: number;
  hatch: HatchPatternId;
  hatchScale: number;
  hatchAngleDeg: number;
}

export interface FontStyle {
  family: string;
  sizePt: number;
  weight: number;
  italic: boolean;
  leading: number;
  trackingEm: number;
  colorHex: string;
}

export interface ToolPreset {
  id: string;
  tool: ToolId;
  label: string;
  stroke: StrokeStyle;
  fill: FillStyle;
  font: FontStyle;
  pressure: PressureCurve;
  tiltShading: number;
  smartShapes: boolean;
}

const clamp01 = (v: number): number => (v < 0 ? 0 : v > 1 ? 1 : v);

/** Evaluate y for given x on the curve by bisection on t (x(t) is monotone for x1,x2 in [0,1]). */
export function evalPressure(c: PressureCurve, p: number): number {
  const x = clamp01(p);
  const bx = (t: number): number => 3 * (1 - t) * (1 - t) * t * c.x1 + 3 * (1 - t) * t * t * c.x2 + t * t * t;
  const by = (t: number): number => 3 * (1 - t) * (1 - t) * t * c.y1 + 3 * (1 - t) * t * t * c.y2 + t * t * t;
  let lo = 0, hi = 1;
  for (let i = 0; i < 24; i++) {
    const mid = (lo + hi) / 2;
    if (bx(mid) < x) lo = mid; else hi = mid;
  }
  return clamp01(by((lo + hi) / 2));
}

function normalizeToolId(raw: string): ToolId {
  if (raw === 'preset.pan' || raw === 'pan') return 'pan';
  if (raw === 'preset.pen.a' || raw === 'pen.a') return 'pen.a';
  if (raw === 'preset.pen.b' || raw === 'pen.b') return 'pen.b';
  if (raw === 'preset.highlighter' || raw === 'highlighter') return 'highlighter';
  if (raw === 'preset.eraser.partial' || raw === 'eraser.partial') return 'eraser.partial';
  if (raw === 'preset.eraser.whole' || raw === 'eraser.whole' || raw === 'eraser') return 'eraser.whole';
  if (raw === 'laser') return 'laser';
  if (raw.startsWith('select')) return 'select';
  if (raw.startsWith('review') || raw === 'text.review') return 'text.review';
  if (raw.startsWith('shape')) return 'shape';
  if (raw.startsWith('measure')) return 'measure';
  return (raw in TOOL_LABELS ? raw : 'pan') as ToolId;
}

function createFallbackPreset(id: ToolId): ToolPreset {
  const isHighlighter = id === 'highlighter';
  return {
    id: `preset.${id}`,
    tool: id,
    label: TOOL_LABELS[id] ?? id,
    stroke: {
      colorToken: isHighlighter ? '--snap-indicator-intersection' : (id === 'pen.b' ? '--danger' : '--glass-text'),
      colorHex: isHighlighter ? '#ffd60a' : (id === 'pen.b' ? '#ff3b30' : '#1d1d1f'),
      widthPt: isHighlighter ? 12 : (id === 'pen.b' ? 1.5 : 0.75),
      opacity: isHighlighter ? 0.4 : 1.0,
      dash: [],
      lineCap: isHighlighter ? 'square' : 'round',
      startEnd: 'none',
      endEnd: 'none',
    },
    fill: {
      kind: 'none',
      colorHex: '#1d1d1f',
      opacity: 0,
      hatch: 'solid',
      hatchScale: 1,
      hatchAngleDeg: 0,
    },
    font: {
      family: 'Inter',
      sizePt: 12,
      weight: 400,
      italic: false,
      leading: 1.2,
      trackingEm: 0,
      colorHex: '#1d1d1f',
    },
    pressure: { x1: 0.25, y1: 0.1, x2: 0.25, y2: 1 },
    tiltShading: id === 'pen.a' ? 0.1 : 0.2,
    smartShapes: id === 'pen.a' || id === 'pen.b',
  };
}

function buildInitialPresets(): Record<string, ToolPreset> {
  const record: Record<string, ToolPreset> = {};
  for (const p of defaultPresetsData as unknown as ToolPreset[]) {
    const tid = normalizeToolId(p.tool);
    const item: ToolPreset = {
      ...p,
      tool: tid,
      label: TOOL_LABELS[tid] ?? p.label
    };
    record[p.id] = item;
    record[tid] = item;
  }
  // Ensure all tool IDs have a preset mapping
  const allTools: ToolId[] = [
    'pan', 'select', 'pen.a', 'pen.b', 'highlighter',
    'eraser.partial', 'eraser.whole', 'laser', 'text.review', 'shape', 'measure'
  ];
  for (const t of allTools) {
    if (!record[t]) {
      const fb = createFallbackPreset(t);
      record[t] = fb;
      record[fb.id] = fb;
    }
  }
  return record;
}

export class ToolStateEngine {
  activeToolId = $state<ToolId>('pan');
  presets = $state<Record<string, ToolPreset>>(buildInitialPresets());
  favorites = $state<string[]>(['preset.pen.a', 'preset.highlighter', 'preset.eraser.partial']);
  #springLoaded = $state<ToolId | null>(null);

  readonly currentToolId = $derived<ToolId>(this.#springLoaded ?? this.activeToolId);
  readonly activePreset = $derived<ToolPreset | null>(this.presets[this.currentToolId] ?? null);
  readonly activeToolLabel = $derived<string>(TOOL_LABELS[this.currentToolId] ?? this.currentToolId);
  readonly supportsStrokeProperties = $derived<boolean>(
    ['pen.a', 'pen.b', 'highlighter', 'shape', 'text.review'].includes(this.currentToolId)
  );
  readonly supportsEraserMode = $derived<boolean>(
    this.currentToolId === 'eraser.partial' || this.currentToolId === 'eraser.whole'
  );

  // Backward compatibility getters
  readonly active = $derived<ToolPreset | null>(this.activePreset);
  readonly activeTool = $derived<ToolId>(this.currentToolId);

  selectTool(id: ToolId | string): void {
    const norm = normalizeToolId(id);
    this.activeToolId = norm;
    if (!this.presets[norm]) {
      const fb = createFallbackPreset(norm);
      this.presets[norm] = fb;
      this.presets[fb.id] = fb;
    }
  }

  updateActivePreset(patch: Partial<ToolPreset>): void {
    const current = this.presets[this.currentToolId];
    if (current) {
      const updated: ToolPreset = { ...current, ...patch };
      this.presets[this.currentToolId] = updated;
      this.presets[updated.id] = updated;
    }
  }

  // Preset ID-based compatibility methods
  select(presetIdOrTool: string): boolean {
    this.selectTool(presetIdOrTool);
    return true;
  }

  patch(presetId: string, patch: Partial<Omit<ToolPreset, 'id' | 'tool'>>): void {
    const target = this.presets[presetId] ?? this.presets[this.currentToolId];
    if (target) {
      const updated: ToolPreset = { ...target, ...patch };
      this.presets[target.id] = updated;
      this.presets[target.tool] = updated;
    }
  }

  springLoad(presetIdOrTool: string | null): void {
    this.#springLoaded = presetIdOrTool ? normalizeToolId(presetIdOrTool) : null;
  }

  toggleFavorite(presetId: string): void {
    this.favorites = this.favorites.includes(presetId)
      ? this.favorites.filter((f) => f !== presetId)
      : [...this.favorites, presetId];
  }

  load(presetsList: readonly ToolPreset[], favorites: readonly string[]): void {
    const record: Record<string, ToolPreset> = {};
    for (const p of presetsList) {
      const tid = normalizeToolId(p.tool);
      const item: ToolPreset = { ...p, tool: tid };
      record[p.id] = item;
      record[tid] = item;
    }
    this.presets = record;
    this.favorites = [...favorites];
    if (presetsList[0]) {
      this.selectTool(presetsList[0].tool);
    }
  }

  /** Stroke width at a pressure sample, tilt-shaded. */
  widthAt(pressure: number, tiltX: number, tiltY: number): number {
    const a = this.activePreset;
    if (!a) return 1;
    const tilt = Math.min(1, Math.hypot(tiltX, tiltY) / 90);
    return a.stroke.widthPt * Math.max(0.05, evalPressure(a.pressure, pressure)) * (1 + a.tiltShading * tilt);
  }
}

export const tools = new ToolStateEngine();

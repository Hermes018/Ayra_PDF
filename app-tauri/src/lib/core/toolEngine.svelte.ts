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

/** Evaluate y for given x on the curve by bisection on t (x(t) is monotone for x1,x2 ∈ [0,1]). */
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

export class ToolStateEngine {
  presets = $state<ToolPreset[]>([]);
  favorites = $state<string[]>([]);
  activePresetId = $state<string | null>(null);
  /** Tool held temporarily (e.g. Space → pan, stylus barrel → eraser); restored on release. */
  #springLoaded = $state<string | null>(null);
  readonly active: ToolPreset | null = $derived(
    this.presets.find((p) => p.id === (this.#springLoaded ?? this.activePresetId)) ?? null
  );
  readonly activeTool: ToolId = $derived(this.active?.tool ?? 'pan');

  load(presets: readonly ToolPreset[], favorites: readonly string[]): void {
    this.presets = presets.map((p) => structuredClone(p));
    this.favorites = favorites.filter((id) => presets.some((p) => p.id === id));
    this.activePresetId = this.presets[0]?.id ?? null;
  }

  select(presetId: string): boolean {
    if (!this.presets.some((p) => p.id === presetId)) return false;
    this.activePresetId = presetId;
    return true;
  }

  springLoad(presetId: string | null): void {
    this.#springLoaded = presetId;
  }

  patch(presetId: string, patch: Partial<Omit<ToolPreset, 'id' | 'tool'>>): void {
    this.presets = this.presets.map((p) => (p.id === presetId ? { ...p, ...patch } : p));
  }

  toggleFavorite(presetId: string): void {
    this.favorites = this.favorites.includes(presetId)
      ? this.favorites.filter((f) => f !== presetId)
      : [...this.favorites, presetId];
  }

  /** Stroke width at a pressure sample, tilt-shaded (pencil-side shading reduces width up to tiltShading). */
  widthAt(pressure: number, tiltX: number, tiltY: number): number {
    const a = this.active;
    if (!a) return 1;
    const tilt = Math.min(1, Math.hypot(tiltX, tiltY) / 90);
    return a.stroke.widthPt * Math.max(0.05, evalPressure(a.pressure, pressure)) * (1 + a.tiltShading * tilt);
  }
}

export const tools = new ToolStateEngine();

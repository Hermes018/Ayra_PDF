// app-tauri/src/lib/core/toolEngine.svelte.ts
import type { HatchPatternId } from '$lib/bindings/HatchPatternId';
import defaultPresetsData from './defaultPresets.json';
import { theme } from './themeContract.svelte';

export type HatchPatternExtendedId =
  | HatchPatternId
  | 'none'
  | 'diagonal-steel'
  | 'crosshatch';

export type LineStyleId = 'solid' | 'dashed' | 'dotted' | 'dash-dot';

export type ToolId =
  | 'pan'
  | 'select'
  | 'select.marquee'
  | 'select.lasso'
  | 'select.text'
  | 'select.object'
  | 'select.snapshot'
  | 'pen'
  | 'pen.a'
  | 'pen.b'
  | 'highlighter'
  | 'eraser.partial'
  | 'eraser.whole'
  | 'laser'
  | 'text.review'
  | 'review.textEdit'
  | 'review.highlight'
  | 'review.underline'
  | 'review.squiggly'
  | 'review.squiggle'
  | 'review.strikeout'
  | 'review.strikethrough'
  | 'review.redact'
  | 'shape'
  | 'shape.line'
  | 'shape.arrow'
  | 'shape.polyline'
  | 'shape.polygon'
  | 'shape.rect'
  | 'shape.ellipse'
  | 'shape.cloud'
  | 'hatch.region'
  | 'hatch.area'
  | 'hatch.pick'
  | 'measure'
  | 'measure.calibrate'
  | 'measure.length'
  | 'measure.polylength'
  | 'measure.perimeter'
  | 'measure.rectArea'
  | 'measure.area_rect'
  | 'measure.polyArea'
  | 'measure.area_polygon'
  | 'measure.protractor'
  | 'measure.ruler'
  | 'measure.tape'
  | 'measure.count'
  | 'callout.single'
  | 'callout.multi'
  | 'insert.text'
  | 'insert.textBox'
  | 'insert.note'
  | 'insert.audio'
  | 'insert.stamp'
  | 'insert.signature'
  | 'insert.image'
  | 'insert.latex'
  | 'snapshot';

export const TOOL_LABELS: Record<ToolId, string> = {
  'pan': 'Pan Canvas',
  'select': 'Select Object',
  'select.marquee': 'Marquee Select',
  'select.lasso': 'Lasso Select',
  'select.text': 'Select Text',
  'select.object': 'Select Object',
  'select.snapshot': 'Snapshot Region',
  'pen': 'Pen',
  'pen.a': 'Pen A',
  'pen.b': 'Pen B',
  'highlighter': 'Highlighter',
  'eraser.partial': 'Partial Eraser',
  'eraser.whole': 'Whole Eraser',
  'laser': 'Laser Pointer',
  'text.review': 'Text Review',
  'review.textEdit': 'Inline Text Edit',
  'review.highlight': 'Highlight Text',
  'review.underline': 'Underline Text',
  'review.squiggly': 'Squiggly Text',
  'review.squiggle': 'Squiggle Text',
  'review.strikeout': 'Strikeout Text',
  'review.strikethrough': 'Strikethrough Text',
  'review.redact': 'Redact Region',
  'shape': 'Shape Tool',
  'shape.line': 'Line Tool',
  'shape.arrow': 'Arrow Tool',
  'shape.polyline': 'Polyline Tool',
  'shape.polygon': 'Polygon Tool',
  'shape.rect': 'Rectangle Tool',
  'shape.ellipse': 'Ellipse Tool',
  'shape.cloud': 'Revision Cloud',
  'hatch.region': 'Hatch Region',
  'hatch.area': 'Procedural Hatch Area',
  'hatch.pick': 'Pick-Point Hatch',
  'measure': 'Measurement Suite',
  'measure.calibrate': 'Calibrate Scale',
  'measure.length': 'Measure Length',
  'measure.polylength': 'Measure Polylength',
  'measure.perimeter': 'Measure Perimeter',
  'measure.rectArea': 'Measure Rect Area',
  'measure.area_rect': 'Rectangle Area',
  'measure.polyArea': 'Measure Poly Area',
  'measure.area_polygon': 'Polygon Area',
  'measure.protractor': 'Protractor Angle',
  'measure.ruler': 'Digital Ruler',
  'measure.tape': 'Transient Tape Measure',
  'measure.count': 'Count Takeoff',
  'callout.single': 'Callout Box',
  'callout.multi': 'Multileader Callout',
  'insert.text': 'Rich Text Box',
  'insert.textBox': 'Free Text Box',
  'insert.note': 'Sticky Note',
  'insert.audio': 'Voice Audio Note',
  'insert.stamp': 'Stamp Markup',
  'insert.signature': 'Signature Pad',
  'insert.image': 'Insert Image',
  'insert.latex': 'Math text (Labs)',
  'snapshot': 'Snapshot Region'
};

/** Cubic Bézier pressure curve through (0,0),(x1,y1),(x2,y2),(1,1) — texturecraft-style stamp curve. */
export interface PressureCurve { x1: number; y1: number; x2: number; y2: number }

export interface StrokeStyle {
  colorToken: string | null;
  colorHex: string;
  widthPt: number;
  opacity: number;
  dash: readonly number[];
  lineStyle?: LineStyleId;
  lineCap: 'butt' | 'round' | 'square';
  startEnd: LineEnding;
  endEnd: LineEnding;
}

export type LineEnding =
  | 'none'
  | 'open'
  | 'open-arrow'
  | 'closed'
  | 'closed-arrow'
  | 'circle'
  | 'square'
  | 'diamond'
  | 'slash'
  | 'butt';

export interface FillStyle {
  kind: 'none' | 'solid' | 'hatch';
  colorHex: string;
  opacity: number;
  hatch: HatchPatternExtendedId;
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
  if (raw === 'preset.pan') return 'pan';
  if (raw === 'preset.pen.a') return 'pen.a';
  if (raw === 'preset.pen.b') return 'pen.b';
  if (raw === 'preset.highlighter') return 'highlighter';
  if (raw === 'preset.eraser.partial') return 'eraser.partial';
  if (raw === 'preset.eraser.whole' || raw === 'eraser') return 'eraser.whole';
  if (raw === 'hatch.area') return 'hatch.region';
  if (raw === 'measure.area_rect') return 'measure.rectArea';
  if (raw === 'measure.area_polygon') return 'measure.polyArea';
  if (raw === 'measure.tape') return 'measure.tape';
  if (raw in TOOL_LABELS) return raw as ToolId;
  return 'pan';
}

function createFallbackPreset(id: ToolId): ToolPreset {
  const isHighlighter = id === 'highlighter' || id === 'review.highlight';
  const strokeToken = isHighlighter
    ? '--snap-indicator-intersection'
    : (id === 'pen.b' || id.startsWith('measure') ? '--danger' : '--glass-text');
  const resolvedStrokeColor = theme.token(strokeToken);
  const resolvedBaseText = theme.token('--glass-text');

  return {
    id: `preset.${id}`,
    tool: id,
    label: TOOL_LABELS[id] ?? id,
    stroke: {
      colorToken: strokeToken,
      colorHex: resolvedStrokeColor,
      widthPt: isHighlighter ? 12 : (id === 'pen.b' ? 1.5 : 0.75),
      opacity: isHighlighter ? 0.4 : 1.0,
      dash: [],
      lineCap: isHighlighter ? 'square' : 'round',
      startEnd: 'none',
      endEnd: id === 'shape.arrow' ? 'open' : 'none',
    },
    fill: {
      kind: id.startsWith('hatch') ? 'hatch' : 'none',
      colorHex: resolvedBaseText,
      opacity: id.startsWith('hatch') ? 0.25 : 0,
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
      colorHex: resolvedBaseText,
    },
    pressure: { x1: 0.25, y1: 0.1, x2: 0.25, y2: 1 },
    tiltShading: id === 'pen.a' ? 0.1 : 0.2,
    smartShapes: id === 'pen.a' || id === 'pen.b' || id === 'pen',
  };
}

function buildInitialPresets(): Record<string, ToolPreset> {
  const record: Record<string, ToolPreset> = {};
  for (const p of defaultPresetsData as unknown as ToolPreset[]) {
    const tid = normalizeToolId(p.id.startsWith('preset.') ? p.id : p.tool);
    const item: ToolPreset = {
      ...p,
      tool: tid,
      label: TOOL_LABELS[tid] ?? p.label
    };
    record[p.id] = item;
    record[tid] = item;
  }
  for (const t of Object.keys(TOOL_LABELS) as ToolId[]) {
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
    this.currentToolId.startsWith('pen') ||
    this.currentToolId === 'highlighter' ||
    this.currentToolId.startsWith('shape') ||
    this.currentToolId.startsWith('hatch') ||
    this.currentToolId.startsWith('review') ||
    this.currentToolId === 'text.review' ||
    this.currentToolId.startsWith('insert.text') ||
    this.currentToolId.startsWith('callout') ||
    this.currentToolId.startsWith('measure')
  );
  readonly supportsEraserMode = $derived<boolean>(
    this.currentToolId === 'eraser.partial' || this.currentToolId === 'eraser.whole'
  );

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

  pinFavorite(toolIdOrPresetId: string): void {
    const tid = normalizeToolId(toolIdOrPresetId);
    let p = this.presets[tid] ?? this.presets[toolIdOrPresetId];
    if (!p) {
      p = createFallbackPreset(tid);
      this.presets[p.id] = p;
      this.presets[tid] = p;
    }
    this.toggleFavorite(p.id);
  }

  isFavorite(toolIdOrPresetId: string): boolean {
    const tid = normalizeToolId(toolIdOrPresetId);
    const p = this.presets[tid] ?? this.presets[toolIdOrPresetId];
    if (!p) {
      return this.favorites.includes(toolIdOrPresetId) || this.favorites.includes(`preset.${toolIdOrPresetId}`);
    }
    return this.favorites.includes(p.id) || this.favorites.includes(toolIdOrPresetId);
  }

  load(presetsList: readonly ToolPreset[], favorites: readonly string[]): void {
    const record: Record<string, ToolPreset> = { ...buildInitialPresets() };
    for (const p of presetsList) {
      const tid = normalizeToolId(p.id.startsWith('preset.') ? p.id : p.tool);
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

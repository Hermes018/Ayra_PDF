// app-tauri/src/lib/core/measureEngine.svelte.ts
// Phase D5 — CAD Calibration, Measurement Takeoff & Digital Ruler Engine

import { theme } from './themeContract.svelte';
import { commands } from './commandEngine.svelte';

export type CadUnit = 'pt' | 'mm' | 'cm' | 'm' | 'in' | 'ft';

export interface PageScaleConfig {
  pageIndex: number;
  ptDistance: number;
  realDistance: number;
  unit: CadUnit;
  precision: 0 | 1 | 2 | 3;
  scalePresetLabel?: string;
  depthValue: number;
}

export type MeasurementKind =
  | 'length'
  | 'polylength'
  | 'area.rect'
  | 'area.polygon'
  | 'perimeter'
  | 'protractor'
  | 'tape';

export interface MeasurementRecord {
  id: string;
  pageIndex: number;
  kind: MeasurementKind;
  points: { x: number; y: number }[];
  rawPtValue: number;
  calibratedValue: number;
  perimeterCalibrated?: number;
  volumeCalibrated?: number;
  unitLabel: string;
  formattedLabel: string;
  strokeColor: string;
  strokeWidthPt: number;
  endpointStyle: 'tick' | 'open-arrow' | 'closed-arrow' | 'circle';
}

export interface DigitalRulerState {
  visible: boolean;
  xPt: number;
  yPt: number;
  angleDeg: number;
  lengthPt: number;
}

export interface ArchitecturalScalePreset {
  id: string;
  label: string;
  ptDistance: number;
  realDistance: number;
  unit: CadUnit;
}

/**
 * Standard architectural and metric engineering scale presets.
 * Recall: 1 inch = 72 PDF points = 25.4 mm.
 */
export const ARCHITECTURAL_SCALE_PRESETS: readonly ArchitecturalScalePreset[] = Object.freeze([
  { id: '1:1', label: '1:1 (Full Scale mm)', ptDistance: 72, realDistance: 25.4, unit: 'mm' },
  { id: '1:10', label: '1:10 (Metric cm)', ptDistance: 72, realDistance: 25.4, unit: 'cm' },
  { id: '1:50', label: '1:50 (Architectural m)', ptDistance: 72, realDistance: 1.27, unit: 'm' },
  { id: '1:100', label: '1:100 (Architectural m)', ptDistance: 72, realDistance: 2.54, unit: 'm' },
  { id: '1:200', label: '1:200 (Site Plan m)', ptDistance: 72, realDistance: 5.08, unit: 'm' },
  { id: '1:500', label: '1:500 (Master Plan m)', ptDistance: 72, realDistance: 12.7, unit: 'm' },
  { id: '1/4"=1\'-0"', label: '1/4" = 1\'-0" (Imperial ft)', ptDistance: 18, realDistance: 1.0, unit: 'ft' },
  { id: '1/8"=1\'-0"', label: '1/8" = 1\'-0" (Imperial ft)', ptDistance: 9, realDistance: 1.0, unit: 'ft' },
]);

/** Create default uncalibrated scale (`72 pt = 25.4 mm`). */
export function createDefaultPageScale(pageIndex = 0): PageScaleConfig {
  return {
    pageIndex,
    ptDistance: 72,
    realDistance: 25.4,
    unit: 'mm',
    precision: 2,
    scalePresetLabel: '1:1 (Default)',
    depthValue: 0,
  };
}

/** Create a PageScaleConfig from an architectural preset ID. */
export function createScaleFromPreset(
  pageIndex: number,
  presetId: string,
  precision: 0 | 1 | 2 | 3 = 2,
  depthValue = 0
): PageScaleConfig {
  const found = ARCHITECTURAL_SCALE_PRESETS.find(
    (p) => p.id === presetId || p.label === presetId
  );
  if (!found) {
    return createDefaultPageScale(pageIndex);
  }
  return {
    pageIndex,
    ptDistance: found.ptDistance,
    realDistance: found.realDistance,
    unit: found.unit,
    precision,
    scalePresetLabel: found.label,
    depthValue,
  };
}

/**
 * Unit conversion ratio: calibrated real units per 1 PDF point (`pt`).
 * ratio = realDistance / max(ptDistance, 1e-6)
 */
export function unitsPerPt(scale: PageScaleConfig): number {
  return scale.realDistance / Math.max(scale.ptDistance, 1e-6);
}

/**
 * Linear & Polyline Length in PDF points (`pt`):
 * L_pt = sum_{i=0}^{n-2} hypot(x_{i+1} - x_i, y_{i+1} - y_i)
 */
export function computePolylineLengthPt(
  pts: readonly { x: number; y: number }[],
  closed = false
): number {
  if (pts.length < 2) return 0;
  let sum = 0;
  for (let i = 0; i < pts.length - 1; i++) {
    sum += Math.hypot(pts[i + 1].x - pts[i].x, pts[i + 1].y - pts[i].y);
  }
  if (closed && pts.length >= 3) {
    const first = pts[0];
    const last = pts[pts.length - 1];
    sum += Math.hypot(first.x - last.x, first.y - last.y);
  }
  return sum;
}

/**
 * Rectangle & Polygon Area via Shoelace Formula in square PDF points (`pt^2`):
 * A_{pt^2} = 0.5 * |sum_{i=0}^{n-1} (x_i * y_{(i+1)%n} - x_{(i+1)%n} * y_i)|
 * If exactly 2 points are supplied (a two-corner axis-aligned rectangle), expands to 4 corners first.
 */
export function computePolygonAreaPt2(pts: readonly { x: number; y: number }[]): number {
  if (pts.length < 2) return 0;
  if (pts.length === 2) {
    const w = Math.abs(pts[1].x - pts[0].x);
    const h = Math.abs(pts[1].y - pts[0].y);
    return w * h;
  }
  const n = pts.length;
  let crossSum = 0;
  for (let i = 0; i < n; i++) {
    const next = (i + 1) % n;
    crossSum += pts[i].x * pts[next].y - pts[next].x * pts[i].y;
  }
  return Math.abs(crossSum) * 0.5;
}

/**
 * Volume Estimate:
 * V_cal = A_cal * max(depthCal, 0)
 */
export function computeVolume(areaCal: number, depthCal: number): number {
  return areaCal * Math.max(depthCal, 0);
}

/**
 * Protractor 3-Point Angle in degrees [0, 180]:
 * u = p1 - vertex, v = p2 - vertex
 * theta = acos(clamp((u . v) / (||u|| * ||v||), -1, 1)) * (180 / pi)
 */
export function computeAngleDeg(
  p1: { x: number; y: number },
  vertex: { x: number; y: number },
  p2: { x: number; y: number }
): number {
  const ux = p1.x - vertex.x;
  const uy = p1.y - vertex.y;
  const vx = p2.x - vertex.x;
  const vy = p2.y - vertex.y;

  const magU = Math.hypot(ux, uy);
  const magV = Math.hypot(vx, vy);
  if (magU <= 1e-9 || magV <= 1e-9) return 0;

  const cosTheta = Math.min(1, Math.max(-1, (ux * vx + uy * vy) / (magU * magV)));
  return (Math.acos(cosTheta) * 180) / Math.PI;
}

/**
 * Digital Ruler Edge Projection:
 * Given ruler origin R0(xPt, yPt), angle alpha (in degrees), and lengthPt:
 * d_hat = (cos alpha, sin alpha), n_hat = (-sin alpha, cos alpha).
 * If perpendicular distance |(P - R0) . n_hat| <= snapThresholdPt and
 * longitudinal projection s = (P - R0) . d_hat in [-20, lengthPt + 20],
 * returns projected collinear point along the ruler edge.
 */
export function projectPointToRulerIfNear(
  pt: { x: number; y: number },
  rulerState: DigitalRulerState,
  snapThresholdPt = 14
): { x: number; y: number; snapped: boolean } {
  if (!rulerState.visible) {
    return { x: pt.x, y: pt.y, snapped: false };
  }

  const rad = (rulerState.angleDeg * Math.PI) / 180;
  const dx = Math.cos(rad);
  const dy = Math.sin(rad);
  const nx = -dy;
  const ny = dx;

  const relX = pt.x - rulerState.xPt;
  const relY = pt.y - rulerState.yPt;

  const s = relX * dx + relY * dy;
  const perpDist = relX * nx + relY * ny;

  if (
    Math.abs(perpDist) <= snapThresholdPt &&
    s >= -20 &&
    s <= rulerState.lengthPt + 20
  ) {
    const clampedS = Math.min(rulerState.lengthPt + 20, Math.max(-20, s));
    return {
      x: rulerState.xPt + clampedS * dx,
      y: rulerState.yPt + clampedS * dy,
      snapped: true,
    };
  }

  return { x: pt.x, y: pt.y, snapped: false };
}

/** Format area, perimeter, and optional volume label according to Phase D5 specification. */
export function formatAreaMeasurementLabel(
  areaCal: number,
  perimCal: number,
  depthCal: number,
  unit: CadUnit,
  precision: 0 | 1 | 2 | 3
): string {
  const p = precision;
  const u = unit;
  const volCal = computeVolume(areaCal, depthCal);
  const volPart = depthCal > 0 ? `, Vol: ${volCal.toFixed(p)} ${u}³` : '';
  return `${areaCal.toFixed(p)} ${u}² (Perim: ${perimCal.toFixed(p)} ${u}${volPart})`;
}

/** Group measurements by unit label (`m`, `m²`, `m³`, `ft`, `ft²`, `deg`, etc.) and sum. */
export function computeTakeoffTotals(
  records: readonly MeasurementRecord[]
): Record<string, number> {
  const totals: Record<string, number> = {};
  for (const r of records) {
    if (r.kind === 'tape') continue;
    totals[r.unitLabel] = (totals[r.unitLabel] ?? 0) + r.calibratedValue;
    if (
      (r.kind === 'area.rect' || r.kind === 'area.polygon') &&
      typeof r.volumeCalibrated === 'number' &&
      r.volumeCalibrated > 0
    ) {
      const baseUnit = r.unitLabel.replace('²', '');
      const volUnit = `${baseUnit}³`;
      totals[volUnit] = (totals[volUnit] ?? 0) + r.volumeCalibrated;
    }
  }
  return totals;
}

export function isMeasureTool(toolId: string): boolean {
  return toolId === 'measure' || toolId.startsWith('measure.');
}

export function isMultiPointMeasureTool(toolId: string): boolean {
  return (
    toolId === 'measure.polylength' ||
    toolId === 'measure.perimeter' ||
    toolId === 'measure.polyArea' ||
    toolId === 'measure.protractor'
  );
}

export class MeasureEngine {
  scales = $state<Record<number, PageScaleConfig>>({
    0: createDefaultPageScale(0),
  });
  measurements = $state<MeasurementRecord[]>([]);
  activeDraft = $state<{
    kind: MeasurementKind | 'calibrate';
    pageIndex: number;
    points: { x: number; y: number }[];
    hoverPt: { x: number; y: number } | null;
  } | null>(null);
  calibrationModal = $state<{
    open: boolean;
    pageIndex: number;
    measuredPt: number;
    p1: { x: number; y: number };
    p2: { x: number; y: number };
  } | null>(null);
  ruler = $state<DigitalRulerState>({
    visible: false,
    xPt: 100,
    yPt: 220,
    angleDeg: 0,
    lengthPt: 360,
  });
  endpointStyle = $state<'tick' | 'open-arrow' | 'closed-arrow' | 'circle'>('tick');
  tapeMode = $state<boolean>(false);

  readonly totalsByUnit = $derived<Record<string, number>>(
    computeTakeoffTotals(this.measurements)
  );

  getScaleForPage(pageIndex: number): PageScaleConfig {
    return this.scales[pageIndex] ?? createDefaultPageScale(pageIndex);
  }

  setPageScale(
    config: PageScaleConfig,
    applyToAllPages = false,
    totalPageCount = 1
  ): void {
    const next: Record<number, PageScaleConfig> = { ...this.scales };
    if (applyToAllPages) {
      const count = Math.max(1, totalPageCount);
      for (let i = 0; i < count; i++) {
        next[i] = { ...config, pageIndex: i };
      }
    } else {
      next[config.pageIndex] = { ...config };
    }
    this.scales = next;
    this.recalculateAllMeasurements();
  }

  updateCurrentPageScalePatch(
    pageIndex: number,
    patch: Partial<PageScaleConfig>
  ): void {
    const current = this.getScaleForPage(pageIndex);
    this.setPageScale({ ...current, ...patch, pageIndex }, false);
  }

  openCalibrationModal(
    pageIndex: number,
    measuredPt = 72,
    p1 = { x: 0, y: 0 },
    p2 = { x: 72, y: 0 }
  ): void {
    this.calibrationModal = {
      open: true,
      pageIndex,
      measuredPt: Math.max(1, measuredPt),
      p1,
      p2,
    };
  }

  closeCalibrationModal(): void {
    this.calibrationModal = null;
  }

  toggleRuler(): void {
    this.ruler = {
      ...this.ruler,
      visible: !this.ruler.visible,
    };
  }

  recalculateAllMeasurements(): void {
    this.measurements = this.measurements.map((m) =>
      this.#buildMeasurementRecord(
        m.id,
        m.pageIndex,
        m.kind,
        m.points,
        m.strokeColor,
        m.strokeWidthPt,
        m.endpointStyle
      )
    );
  }

  #buildMeasurementRecord(
    id: string,
    pageIndex: number,
    kind: MeasurementKind,
    points: { x: number; y: number }[],
    strokeColor: string,
    strokeWidthPt: number,
    endpointStyle: 'tick' | 'open-arrow' | 'closed-arrow' | 'circle'
  ): MeasurementRecord {
    const scale = this.getScaleForPage(pageIndex);
    const ratio = unitsPerPt(scale);
    const p = scale.precision;
    const u = scale.unit;

    if (kind === 'protractor') {
      const p1 = points[0] ?? { x: 0, y: 0 };
      const vertex = points[1] ?? p1;
      const p2 = points[2] ?? vertex;
      const deg = computeAngleDeg(p1, vertex, p2);
      return {
        id,
        pageIndex,
        kind,
        points,
        rawPtValue: deg,
        calibratedValue: deg,
        unitLabel: 'deg',
        formattedLabel: `${deg.toFixed(Math.min(p, 2))}°`,
        strokeColor,
        strokeWidthPt,
        endpointStyle,
      };
    }

    if (kind === 'area.rect' || kind === 'area.polygon') {
      const rectPts =
        kind === 'area.rect' && points.length >= 2
          ? [
              { x: points[0].x, y: points[0].y },
              { x: points[points.length - 1].x, y: points[0].y },
              { x: points[points.length - 1].x, y: points[points.length - 1].y },
              { x: points[0].x, y: points[points.length - 1].y },
            ]
          : points;
      const areaPt2 = computePolygonAreaPt2(rectPts);
      const perimPt = computePolylineLengthPt(rectPts, true);
      const areaCal = areaPt2 * ratio * ratio;
      const perimCal = perimPt * ratio;
      const volCal = computeVolume(areaCal, scale.depthValue);
      return {
        id,
        pageIndex,
        kind,
        points: rectPts,
        rawPtValue: areaPt2,
        calibratedValue: areaCal,
        perimeterCalibrated: perimCal,
        volumeCalibrated: volCal,
        unitLabel: `${u}²`,
        formattedLabel: formatAreaMeasurementLabel(
          areaCal,
          perimCal,
          scale.depthValue,
          u,
          p
        ),
        strokeColor,
        strokeWidthPt,
        endpointStyle,
      };
    }

    const closed = false;
    const lenPt = computePolylineLengthPt(points, closed);
    const lenCal = lenPt * ratio;
    const prefix = kind === 'perimeter' ? 'Perim: ' : '';
    return {
      id,
      pageIndex,
      kind,
      points,
      rawPtValue: lenPt,
      calibratedValue: lenCal,
      perimeterCalibrated: kind === 'perimeter' ? lenCal : undefined,
      unitLabel: u,
      formattedLabel: `${prefix}${lenCal.toFixed(p)} ${u}`,
      strokeColor,
      strokeWidthPt,
      endpointStyle,
    };
  }

  /** Map tool ID to MeasurementKind | 'calibrate'. */
  resolveDraftKind(toolId: string): MeasurementKind | 'calibrate' {
    if (this.tapeMode && toolId === 'measure.length') return 'tape';
    switch (toolId) {
      case 'measure.calibrate':
        return 'calibrate';
      case 'measure.polylength':
        return 'polylength';
      case 'measure.perimeter':
        return 'perimeter';
      case 'measure.rectArea':
        return 'area.rect';
      case 'measure.polyArea':
        return 'area.polygon';
      case 'measure.protractor':
        return 'protractor';
      default:
        return this.tapeMode ? 'tape' : 'length';
    }
  }

  startOrAddPoint(
    toolId: string,
    pageIndex: number,
    pt: { x: number; y: number }
  ): MeasurementRecord | null {
    const kind = this.resolveDraftKind(toolId);
    if (!this.activeDraft || this.activeDraft.kind !== kind) {
      this.activeDraft = {
        kind,
        pageIndex,
        points: [{ x: pt.x, y: pt.y }],
        hoverPt: { x: pt.x, y: pt.y },
      };
      return null;
    }

    this.activeDraft = {
      ...this.activeDraft,
      points: [...this.activeDraft.points, { x: pt.x, y: pt.y }],
      hoverPt: { x: pt.x, y: pt.y },
    };

    // Protractor commits automatically on the 3rd click (p1 -> vertex -> p2)
    if (kind === 'protractor' && this.activeDraft.points.length >= 3) {
      return this.commitActiveDraft();
    }

    return null;
  }

  updateHoverPoint(pt: { x: number; y: number }): void {
    if (!this.activeDraft) return;
    this.activeDraft = {
      ...this.activeDraft,
      hoverPt: { x: pt.x, y: pt.y },
    };
  }

  commitTwoPointDrag(
    pt: { x: number; y: number }
  ): MeasurementRecord | null {
    if (!this.activeDraft || this.activeDraft.points.length === 0) return null;
    const start = this.activeDraft.points[0];
    const dist = Math.hypot(pt.x - start.x, pt.y - start.y);
    if (dist < 2) {
      this.activeDraft = null;
      return null;
    }
    this.activeDraft = {
      ...this.activeDraft,
      points: [start, { x: pt.x, y: pt.y }],
      hoverPt: null,
    };
    return this.commitActiveDraft();
  }

  commitActiveDraft(): MeasurementRecord | null {
    if (!this.activeDraft) return null;
    const draft = this.activeDraft;
    this.activeDraft = null;

    if (draft.kind === 'calibrate') {
      if (draft.points.length >= 2) {
        const p1 = draft.points[0];
        const p2 = draft.points[draft.points.length - 1];
        const distPt = Math.max(1, Math.hypot(p2.x - p1.x, p2.y - p1.y));
        this.openCalibrationModal(draft.pageIndex, distPt, p1, p2);
      }
      return null;
    }

    // Transient tape measure disappears on release without cluttering document
    if (draft.kind === 'tape') {
      return null;
    }

    const minPts =
      draft.kind === 'area.polygon' || draft.kind === 'protractor' ? 3 : 2;
    if (draft.points.length < minPts) {
      return null;
    }

    const id = `meas-${Date.now()}-${Math.random().toString(36).slice(2, 6)}`;
    const color = theme.token('--measure-line');
    const record = this.#buildMeasurementRecord(
      id,
      draft.pageIndex,
      draft.kind,
      draft.points,
      color,
      1.5,
      this.endpointStyle
    );

    const before = [...this.measurements];
    const after = [...this.measurements, record];
    this.measurements = after;

    commands.register('measure.commitHistory', (doc) => ({
      next: doc,
      label: `Measure ${record.kind}`,
      onUndo: () => {
        this.measurements = before;
      },
      onRedo: () => {
        this.measurements = after;
      },
    }));
    commands.dispatch({ kind: 'execute', command: 'measure.commitHistory', args: {} });

    return record;
  }

  cancelDraft(): void {
    this.activeDraft = null;
  }

  /** Compute live preview record while dragging or hovering. */
  get previewRecord(): MeasurementRecord | null {
    if (!this.activeDraft) return null;
    const pts = this.activeDraft.hoverPt
      ? [...this.activeDraft.points, this.activeDraft.hoverPt]
      : this.activeDraft.points;
    if (pts.length < 2) return null;
    if (this.activeDraft.kind === 'calibrate') {
      const lenPt = computePolylineLengthPt(pts, false);
      return {
        id: 'draft-calibrate',
        pageIndex: this.activeDraft.pageIndex,
        kind: 'length',
        points: pts,
        rawPtValue: lenPt,
        calibratedValue: lenPt,
        unitLabel: 'pt',
        formattedLabel: `Calibrate: ${lenPt.toFixed(1)} pt`,
        strokeColor: theme.token('--accent'),
        strokeWidthPt: 1.5,
        endpointStyle: 'tick',
      };
    }
    return this.#buildMeasurementRecord(
      'draft-preview',
      this.activeDraft.pageIndex,
      this.activeDraft.kind,
      pts,
      theme.token('--measure-line'),
      1.5,
      this.endpointStyle
    );
  }
}

export const measureEngine = new MeasureEngine();

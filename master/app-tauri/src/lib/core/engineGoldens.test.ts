// app-tauri/src/lib/core/engineGoldens.test.ts
// ADR-018 R2 step 1: characterization goldens of the CURRENT hand-written TS engines, captured
// before any engine is swapped for a craft crate (docs/ENGINE_REUSE_MATRIX.md). Each block writes
// one JSON file under ./__fixtures__/ and fails when the output drifts. A drift is either a bug or
// an intended swap; for an intended one, re-run with `vitest run -u` and explain every changed
// value in the PR (ADR-018 R2 step 4). Known-wrong behaviour is recorded as-is and labelled.
//
// Numbers are written with JSON.stringify (shortest round-trip), so equal text = equal values,
// except that -0 is written as 0 and NaN as null. Timestamps (`t`) and random ids are stripped.
import { describe, it, expect, beforeEach } from 'vitest';
import snapGeometry from './__fixtures__/snap-geometry.standard-form.p0.golden.json';
import {
  SnapEngine,
  SpatialGridIndex,
  segmentIntersection,
  perpendicularFoot,
  extractStrokeSegmentsForPage,
  type SnapSegment,
  type SnapHit,
} from './snapEngine.svelte';
import {
  MeasureEngine,
  ARCHITECTURAL_SCALE_PRESETS,
  createScaleFromPreset,
  unitsPerPt,
  computePolylineLengthPt,
  computePolygonAreaPt2,
  computeAngleDeg,
  computeVolume,
  projectPointToRulerIfNear,
  formatAreaMeasurementLabel,
  computeTakeoffTotals,
  type MeasurementRecord,
} from './measureEngine.svelte';
import {
  ink,
  rdp,
  generateSvgPathFromPoints,
  type ActiveStroke,
  type Point,
} from './inkEngine.svelte';
import {
  shapes,
  computeStrokeBBox,
  generateRevisionCloudPath,
  generatePolylineSvgPath,
  generateSquiggleSvgPath,
  generateLineEndingPath,
  getStrokeDashArray,
} from './shapeAndHatchEngine.svelte';
import { selection, pointInPolygon, rectsIntersect } from './selectionEngine.svelte';
import { applyGrayscaleThresholdToRgba } from './insertEngine.svelte';
import { tools, type LineEnding, type LineStyleId, type ToolId, type ToolPreset } from './toolEngine.svelte';

/* ----------------------------------------------------------------------------------------- */
/* Golden plumbing                                                                            */
/* ----------------------------------------------------------------------------------------- */

/** JSON with primitive-only arrays kept on one line (compact, still diff-friendly). */
function stable(value: unknown, indent = ''): string {
  const next = indent + '  ';
  if (Array.isArray(value)) {
    if (value.length === 0) return '[]';
    if (value.every((v) => v === null || typeof v !== 'object')) {
      return `[${value.map((v) => JSON.stringify(v === undefined ? null : v)).join(', ')}]`;
    }
    return `[\n${value.map((v) => next + stable(v, next)).join(',\n')}\n${indent}]`;
  }
  if (value !== null && typeof value === 'object') {
    const entries = Object.entries(value as Record<string, unknown>).filter(([, v]) => v !== undefined);
    if (entries.length === 0) return '{}';
    return `{\n${entries.map(([k, v]) => `${next}${JSON.stringify(k)}: ${stable(v, next)}`).join(',\n')}\n${indent}}`;
  }
  return JSON.stringify(value === undefined ? null : value);
}

async function golden(name: string, value: unknown): Promise<void> {
  await expect(`${stable(value)}\n`).toMatchFileSnapshot(`./__fixtures__/${name}.golden.json`);
}

/** Deterministic PRNG (LCG) so sampled inputs are identical on every run. */
function lcg(seed: number): () => number {
  let s = seed >>> 0;
  return () => {
    s = (Math.imul(s, 1664525) + 1013904223) >>> 0;
    return s / 4294967296;
  };
}

const r3 = (v: number): number => Math.round(v * 1000) / 1000;
const pts = (list: readonly (readonly [number, number])[], p = 0.5): Point[] => list.map(([x, y]) => ({ x, y, p }));
const plain = (list: readonly { x: number; y: number; p?: number }[]) => list.map((q) => (q.p === undefined ? [q.x, q.y] : [q.x, q.y, q.p]));

function preset(tool: string): ToolPreset {
  const found = tools.presets[tool] ?? tools.presets['pen.a'];
  if (!found) throw new Error(`no preset for ${tool}`);
  return found;
}

function stroke(id: string, tool: ToolId, points: Point[], extra: Partial<ActiveStroke> = {}): ActiveStroke {
  return { id, tool, points, preset: preset(tool), color: 'var(--glass-text)', widthPt: 2, opacity: 1, pageIndex: 0, ...extra };
}

/** The geometric state of a stroke (no preset, colour, or timestamps). */
function geo(s: ActiveStroke) {
  return {
    id: s.id.replace(/-dup-\d+$/, '-dup-N'),
    tool: s.tool,
    points: plain(s.points),
    bbox: s.bbox,
    rotationDeg: s.rotationDeg,
    flipH: s.flipH,
    flipV: s.flipV,
    groupId: s.groupId?.replace(/^group-\d+-\d+$/, 'group-NEW'),
  };
}

const L_POLY = [
  [100, 100],
  [240, 100],
  [240, 140],
  [150, 140],
  [150, 200],
  [100, 200],
] as const;

/* ----------------------------------------------------------------------------------------- */
/* Snap (snapEngine.svelte.ts)                                                                */
/* ----------------------------------------------------------------------------------------- */

describe('golden: snapEngine (current priority solver)', () => {
  it('resolves snaps on corpus/standard-form.pdf page 1 geometry', async () => {
    const segs: SnapSegment[] = snapGeometry.segments.map(([x1, y1, x2, y2]) => ({ x1, y1, x2, y2 }));
    const engine = new SnapEngine();
    engine.setPageSegments(0, segs);
    const hit = (h: SnapHit | null) => (h ? { kind: h.kind, x: h.x, y: h.y, distPt: h.distPt } : null);

    const queries: { x: number; y: number; zoom: number; anchor: { x: number; y: number } | null; grid: boolean }[] = [];
    for (let i = 4; i < Math.min(segs.length, 44); i++) {
      const s = segs[i];
      queries.push({ x: s.x1 + 1.5, y: s.y1 - 1, zoom: 1, anchor: null, grid: false });
      queries.push({ x: (s.x1 + s.x2) / 2 + 0.5, y: (s.y1 + s.y2) / 2 + 0.5, zoom: 2, anchor: null, grid: false });
    }
    const rnd = lcg(20261105);
    for (let i = 0; i < 60; i++) {
      queries.push({
        x: r3(rnd() * snapGeometry.widthPt),
        y: r3(rnd() * snapGeometry.heightPt),
        zoom: [0.5, 1, 2, 4][i % 4],
        anchor: i % 3 === 0 ? { x: 306, y: 396 } : null,
        grid: i % 5 === 0,
      });
    }
    const results = queries.map((q) => {
      engine.gridSnapEnabled = q.grid;
      return { query: q, hit: hit(engine.resolveSnap(0, q.x, q.y, q.anchor, q.zoom)) };
    });
    await golden('snap.standard-form.p0', {
      input: 'snap-geometry.standard-form.p0.golden.json (Rust golden of studio_extract_snap_geometry)',
      segmentCount: segs.length,
      hits: results.filter((r) => r.hit !== null).length,
      results,
    });
  });

  it('records synthetic priority, tolerance and stroke-segment cases', async () => {
    const engine = new SnapEngine();
    engine.setPageSegments(0, [
      { x1: 0, y1: 50, x2: 100, y2: 50 },
      { x1: 50, y1: 0, x2: 50, y2: 100 },
      { x1: 58, y1: 45, x2: 90, y2: 10 },
    ]);
    const cases = [
      { name: 'intersection 0.7pt away vs endpoint 9.3pt away (known weakness: endpoint wins by kind priority)', x: 50.5, y: 50.5, zoom: 1 },
      { name: 'exact intersection, endpoint outside 3pt radius', x: 50, y: 50, zoom: 4 },
      { name: 'midpoint of the oblique segment', x: 74.6, y: 27.4, zoom: 1 },
      { name: 'line 4pt away, outside 3pt radius at zoom 4, no anchor', x: 25, y: 46, zoom: 4 },
      { name: 'endpoint at radius edge (12pt / zoom)', x: 0, y: 62, zoom: 1 },
    ].map((c) => {
      const h = engine.resolveSnap(0, c.x, c.y, null, c.zoom);
      return { ...c, hit: h ? { kind: h.kind, x: h.x, y: h.y, distPt: h.distPt, tokenVar: h.tokenVar } : null };
    });
    const perp = engine.resolveSnap(0, 80, 51, { x: 80, y: 90 }, 1);
    engine.gridSnapEnabled = true;
    engine.enabled = false;
    const gridOnly = engine.resolveSnap(0, 400.3, 391.1, null, 1);
    engine.gridSnapEnabled = false;
    const disabled = engine.resolveSnap(0, 50, 50, null, 1);

    const strokes: ActiveStroke[] = [
      stroke('r', 'shape.rect', pts([[10, 10], [60, 40]]), { bbox: [10, 10, 60, 40] }),
      stroke('poly', 'shape.polygon', pts([[0, 0], [30, 0], [30, 30]])),
      stroke('hatch', 'hatch.region', pts(L_POLY)),
      stroke('ink2', 'pen.a', pts([[5, 5], [9, 9]])),
      stroke('inkN', 'pen.a', pts([[5, 5], [9, 9], [12, 3]])),
      stroke('other-page', 'shape.line', pts([[0, 0], [1, 1]]), { pageIndex: 1 }),
    ];
    const near = (a: number, b: number) => Math.abs(a - b) < 1e-9;
    await golden('snap.synthetic', {
      cases,
      perpendicular: perp,
      gridOnly,
      disabled,
      strokeSegmentsPage0: extractStrokeSegmentsForPage(strokes, 0),
      segmentIntersection: {
        crossing: segmentIntersection({ x1: 0, y1: 0, x2: 10, y2: 10 }, { x1: 0, y1: 10, x2: 10, y2: 0 }),
        touchingEnd: segmentIntersection({ x1: 0, y1: 0, x2: 10, y2: 0 }, { x1: 10, y1: -5, x2: 10, y2: 5 }),
        parallel: segmentIntersection({ x1: 0, y1: 0, x2: 10, y2: 0 }, { x1: 0, y1: 1, x2: 10, y2: 1 }),
        smallTouchingAtOriginMissedByAbsoluteCutoff: segmentIntersection({ x1: 0, y1: 0, x2: 1e-4, y2: 0 }, { x1: 0, y1: 0, x2: 1e-4, y2: 1e-5 }),
        farFromOrigin: segmentIntersection(
          { x1: 1e7, y1: 1e7, x2: 1e7 + 10, y2: 1e7 + 10 },
          { x1: 1e7, y1: 1e7 + 10, x2: 1e7 + 10, y2: 1e7 },
        ),
      },
      perpendicularFoot: {
        inside: perpendicularFoot({ x: 5, y: 7 }, { x1: 0, y1: 0, x2: 10, y2: 0 }),
        beyondEnd: perpendicularFoot({ x: 15, y: 7 }, { x1: 0, y1: 0, x2: 10, y2: 0 }),
        degenerate: perpendicularFoot({ x: 1, y: 1 }, { x1: 3, y1: 3, x2: 3, y2: 3 }),
      },
      gridIndexQuery: (() => {
        const segs: SnapSegment[] = [
          { x1: 0, y1: 0, x2: 31, y2: 0 },
          { x1: 33, y1: 33, x2: 100, y2: 33 },
          { x1: -40, y1: -40, x2: -10, y2: -10 },
        ];
        const index = new SpatialGridIndex(segs, 32);
        return {
          near0: index.queryNear(10, 10, 5).map((s) => segs.indexOf(s)),
          near1: index.queryNear(64, 40, 5).map((s) => segs.indexOf(s)),
          negative: index.queryNear(-20, -20, 2).map((s) => segs.indexOf(s)),
          sanity: near(index.cellSizePt, 32),
        };
      })(),
    });
  });
});

/* ----------------------------------------------------------------------------------------- */
/* Measure (measureEngine.svelte.ts)                                                          */
/* ----------------------------------------------------------------------------------------- */

describe('golden: measureEngine', () => {
  it('records pure measurement math', async () => {
    const far = L_POLY.map(([x, y]) => [x + 1e7, y + 1e7] as const);
    await golden('measure.math', {
      unitsPerPt: ARCHITECTURAL_SCALE_PRESETS.map((p) => ({ id: p.id, ratio: unitsPerPt(createScaleFromPreset(0, p.id)) })),
      unknownPreset: createScaleFromPreset(3, 'nope'),
      polylineLength: {
        open: computePolylineLengthPt(pts(L_POLY)),
        closed: computePolylineLengthPt(pts(L_POLY), true),
        single: computePolylineLengthPt(pts([[1, 1]])),
      },
      polygonArea: {
        lPolygon: computePolygonAreaPt2(pts(L_POLY)),
        lPolygonFarFromOrigin: computePolygonAreaPt2(pts(far)),
        twoPointRect: computePolygonAreaPt2(pts([[10, 10], [40, 30]])),
        clockwise: computePolygonAreaPt2(pts([[0, 0], [0, 10], [10, 10], [10, 0]])),
        selfIntersectingBowtie: computePolygonAreaPt2(pts([[0, 0], [10, 10], [10, 0], [0, 10]])),
      },
      angleDeg: {
        right: computeAngleDeg({ x: 10, y: 0 }, { x: 0, y: 0 }, { x: 0, y: 10 }),
        acute: computeAngleDeg({ x: 10, y: 0 }, { x: 0, y: 0 }, { x: 10, y: 3 }),
        straight: computeAngleDeg({ x: -10, y: 0 }, { x: 0, y: 0 }, { x: 10, y: 0 }),
        reflexReportedAsInterior: computeAngleDeg({ x: 10, y: 0 }, { x: 0, y: 0 }, { x: 10, y: -1 }),
        degenerate: computeAngleDeg({ x: 0, y: 0 }, { x: 0, y: 0 }, { x: 1, y: 1 }),
      },
      volume: [computeVolume(12.5, 3), computeVolume(12.5, -1)],
      ruler: [
        projectPointToRulerIfNear({ x: 150, y: 230 }, { visible: true, xPt: 100, yPt: 220, angleDeg: 0, lengthPt: 360 }),
        projectPointToRulerIfNear({ x: 150, y: 250 }, { visible: true, xPt: 100, yPt: 220, angleDeg: 0, lengthPt: 360 }),
        projectPointToRulerIfNear({ x: 180, y: 300 }, { visible: true, xPt: 100, yPt: 220, angleDeg: 45, lengthPt: 360 }),
        projectPointToRulerIfNear({ x: 90, y: 221 }, { visible: true, xPt: 100, yPt: 220, angleDeg: 0, lengthPt: 360 }),
        projectPointToRulerIfNear({ x: 150, y: 221 }, { visible: false, xPt: 100, yPt: 220, angleDeg: 0, lengthPt: 360 }),
      ],
      areaLabels: [
        formatAreaMeasurementLabel(12.3456, 14.2, 0, 'm', 2),
        formatAreaMeasurementLabel(12.3456, 14.2, 2.5, 'ft', 1),
        formatAreaMeasurementLabel(0.0005, 0.1, 0, 'mm', 3),
      ],
    });
  });

  it('records a calibrated measuring session (length, polylength, perimeter, areas, angle)', async () => {
    const m = new MeasureEngine();
    // Two-point calibration: draw 144 pt, call it 2.5 m.
    m.startOrAddPoint('measure.calibrate', 0, { x: 100, y: 100 });
    m.startOrAddPoint('measure.calibrate', 0, { x: 244, y: 100 });
    m.commitActiveDraft();
    const modal = m.calibrationModal;
    expect(modal).not.toBeNull();
    m.setPageScale({ pageIndex: 0, ptDistance: modal?.measuredPt ?? 0, realDistance: 2.5, unit: 'm', precision: 2, depthValue: 3 });
    m.closeCalibrationModal();

    const draw = (tool: string, list: readonly (readonly [number, number])[]): MeasurementRecord | null => {
      let rec: MeasurementRecord | null = null;
      for (const [x, y] of list) rec = m.startOrAddPoint(tool, 0, { x, y }) ?? rec;
      return rec ?? m.commitActiveDraft();
    };
    const records = [
      draw('measure.length', [[100, 100], [244, 100]]),
      draw('measure.length', [[0, 0], [30, 40]]),
      draw('measure.polylength', [[0, 0], [72, 0], [72, 72]]),
      draw('measure.perimeter', [[0, 0], [72, 0], [72, 72], [0, 72]]),
      draw('measure.rectArea', [[10, 10], [154, 82]]),
      draw('measure.polyArea', L_POLY),
      draw('measure.protractor', [[10, 0], [0, 0], [0, 10]]),
      draw('measure.polyArea', [[0, 0], [10, 0]]),
    ];
    const strip = (r: MeasurementRecord | null) => {
      if (!r) return null;
      const { id: _id, strokeColor: _c, ...rest } = r;
      return rest;
    };
    const before = m.measurements.map(strip);
    // Re-scaling recalculates every stored measurement.
    m.updateCurrentPageScalePatch(0, { unit: 'ft', realDistance: 8.2, precision: 1 });
    await golden('measure.session', {
      calibration: { measuredPt: modal?.measuredPt, scale: m.getScaleForPage(0) },
      committed: records.map(strip),
      stored: before,
      afterRescale: m.measurements.map(strip),
      totalsAfterRescale: computeTakeoffTotals(m.measurements),
    });
  });
});

/* ----------------------------------------------------------------------------------------- */
/* Ink (inkEngine.svelte.ts)                                                                  */
/* ----------------------------------------------------------------------------------------- */

function sampleStrokes(): Record<string, Point[]> {
  const rnd = lcg(7);
  const jitter = () => (rnd() - 0.5) * 1.2;
  const out: Record<string, Point[]> = {};
  out.sine = Array.from({ length: 60 }, (_, i) => ({ x: 10 + i * 3, y: 100 + Math.sin(i / 5) * 20, p: 0.3 + (i % 10) / 20 }));
  out.zigzag = Array.from({ length: 21 }, (_, i) => ({ x: i * 10, y: i % 2 === 0 ? 0 : 25, p: 0.5 }));
  out.circle = Array.from({ length: 49 }, (_, i) => {
    const a = (i / 48) * Math.PI * 2;
    return { x: 200 + Math.cos(a) * 50 + jitter(), y: 200 + Math.sin(a) * 30 + jitter(), p: 0.6 };
  });
  const rect: Point[] = [];
  const corners = [[50, 50], [150, 50], [150, 120], [50, 120], [52, 52]];
  for (let c = 0; c < corners.length - 1; c++) {
    for (let k = 0; k < 10; k++) {
      const t = k / 10;
      rect.push({ x: corners[c][0] + (corners[c + 1][0] - corners[c][0]) * t + jitter(), y: corners[c][1] + (corners[c + 1][1] - corners[c][1]) * t + jitter(), p: 0.5 });
    }
  }
  rect.push({ x: 51, y: 51, p: 0.5 });
  out.roughRect = rect;
  out.triangle = [
    ...Array.from({ length: 8 }, (_, i) => ({ x: 0 + i * 10, y: 0, p: 0.5 })),
    ...Array.from({ length: 8 }, (_, i) => ({ x: 80 - i * 5, y: i * 9, p: 0.5 })),
    ...Array.from({ length: 8 }, (_, i) => ({ x: 40 - i * 5, y: 72 - i * 9, p: 0.5 })),
    { x: 1, y: 1, p: 0.5 },
  ];
  out.noisyLine = Array.from({ length: 30 }, (_, i) => ({ x: i * 4, y: i * 2 + jitter(), p: 0.5 }));
  out.openArc = Array.from({ length: 30 }, (_, i) => {
    const a = (i / 29) * Math.PI;
    return { x: 100 + Math.cos(a) * 60, y: 100 - Math.sin(a) * 60, p: 0.5 };
  });
  out.tiny = [{ x: 0, y: 0, p: 0.5 }, { x: 2, y: 1, p: 0.5 }, { x: 3, y: 3, p: 0.5 }];
  out.hexagon = Array.from({ length: 7 }, (_, i) => {
    const a = (i / 6) * Math.PI * 2;
    return { x: 300 + Math.cos(a) * 40, y: 300 + Math.sin(a) * 40, p: 0.5 };
  });
  return out;
}

describe('golden: inkEngine', () => {
  beforeEach(() => {
    ink.completedStrokes = [];
  });

  it('records RDP simplification and the variable-width outline path', async () => {
    const samples = sampleStrokes();
    const rdpOut: Record<string, Record<string, unknown>> = {};
    for (const [name, list] of Object.entries(samples)) {
      rdpOut[name] = { eps0_5: plain(rdp(list, 0.5)), eps2: plain(rdp(list, 2)), eps5: plain(rdp(list, 5)) };
    }
    const wide = { ...preset('highlighter'), stroke: { ...preset('highlighter').stroke, widthPt: 14 } };
    await golden('ink.rdp-and-outline', {
      rdp: rdpOut,
      outline: {
        empty: generateSvgPathFromPoints([]),
        dot: generateSvgPathFromPoints([{ x: 5, y: 5, p: 0.4 }]),
        zigzagDefaultWidth: generateSvgPathFromPoints(samples.zigzag),
        sineWidth14: generateSvgPathFromPoints(samples.sine, wide),
        duplicatePoints: generateSvgPathFromPoints(pts([[0, 0], [0, 0], [10, 0]])),
      },
    });
  });

  it('records smart-shape recognition', async () => {
    const out: Record<string, unknown> = {};
    for (const [name, list] of Object.entries(sampleStrokes())) {
      const s = stroke(name, 'pen.a', list.map((q) => ({ ...q })));
      const recognized = ink.recognizeShape(s);
      out[name] = { recognized, pointCount: s.points.length, points: plain(s.points) };
    }
    await golden('ink.recognize', out);
  });

  it('records the partial and whole eraser (known: drops samples, does not cut geometry)', async () => {
    const base = () => [
      stroke('line', 'pen.a', Array.from({ length: 21 }, (_, i) => ({ x: i * 10, y: 50, p: 0.5 }))),
      stroke('sparse', 'pen.a', pts([[0, 100], [200, 100]])),
      stroke('sine', 'pen.a', sampleStrokes().sine),
    ];
    const pointers = [
      { name: 'middle of line', pointer: { x: 100, y: 52 }, radius: 12 },
      { name: 'near line start (leaves <2-point chunk)', pointer: { x: 5, y: 50 }, radius: 12 },
      { name: 'crosses sparse segment between samples', pointer: { x: 100, y: 100 }, radius: 12 },
      { name: 'sine crest, big radius', pointer: { x: 52, y: 118 }, radius: 20 },
      { name: 'miss', pointer: { x: 500, y: 500 }, radius: 12 },
    ];
    const partial = pointers.map(({ name, pointer, radius }) => {
      ink.completedStrokes = base();
      const modified = ink.erasePartial(pointer, radius);
      return { name, pointer, radius, modified, strokes: ink.completedStrokes.map((s) => ({ id: s.id, points: plain(s.points) })) };
    });
    const whole = pointers.map(({ name, pointer, radius }) => {
      ink.completedStrokes = base();
      const modified = ink.eraseWhole(pointer, radius);
      return { name, modified, remaining: ink.completedStrokes.map((s) => s.id) };
    });
    await golden('ink.eraser', { partial, whole });
  });
});

/* ----------------------------------------------------------------------------------------- */
/* Shapes, revision cloud, line endings (shapeAndHatchEngine.svelte.ts)                       */
/* ----------------------------------------------------------------------------------------- */

describe('golden: shapeAndHatchEngine', () => {
  beforeEach(() => {
    ink.completedStrokes = [];
    shapes.cancelActiveShape();
  });

  it('records revision-cloud, polyline, squiggle, ending and dash geometry', async () => {
    const endings: LineEnding[] = ['none', 'open', 'open-arrow', 'closed', 'closed-arrow', 'circle', 'square', 'diamond', 'slash', 'butt'];
    const styles: (LineStyleId | undefined)[] = ['solid', 'dashed', 'dotted', 'dash-dot', undefined];
    const tip: Point = { x: 100, y: 50, p: 0.5 };
    const prev: Point = { x: 40, y: 10, p: 0.5 };
    await golden('shapes.paths', {
      revisionCloud: {
        twoPointRect: generateRevisionCloudPath(pts([[10, 10], [110, 70]])),
        rectClockwise: generateRevisionCloudPath(pts([[10, 10], [110, 10], [110, 70], [10, 70]])),
        rectCounterClockwise: generateRevisionCloudPath(pts([[10, 10], [10, 70], [110, 70], [110, 10]])),
        lPolygon: generateRevisionCloudPath(pts(L_POLY)),
        lPolygonChord8: generateRevisionCloudPath(pts(L_POLY), 8),
        lPolygonChord30: generateRevisionCloudPath(pts(L_POLY), 30),
        chordClampedTo8: generateRevisionCloudPath(pts([[0, 0], [40, 0], [40, 40]]), 2),
        alreadyClosed: generateRevisionCloudPath(pts([[0, 0], [40, 0], [40, 40], [0, 0]])),
        single: generateRevisionCloudPath(pts([[0, 0]])),
      },
      polyline: {
        open: generatePolylineSvgPath(pts(L_POLY)),
        closed: generatePolylineSvgPath(pts(L_POLY), true),
        closedTwoPoints: generatePolylineSvgPath(pts([[0, 0], [1, 1]]), true),
      },
      squiggle: generateSquiggleSvgPath(computeStrokeBBox({ points: pts([[10, 20], [57, 32]]) })),
      endings: Object.fromEntries(
        endings.map((e) => [e, { w1: generateLineEndingPath(tip, prev, e, 1), w4: generateLineEndingPath(tip, prev, e, 4) }]),
      ),
      endingDegenerate: generateLineEndingPath(tip, tip, 'open-arrow', 2),
      dash: Object.fromEntries(styles.map((s) => [s ?? 'undefined', [getStrokeDashArray(s, 0.5) ?? null, getStrokeDashArray(s, 3) ?? null]])),
      bbox: {
        points: computeStrokeBBox({ points: pts(L_POLY) }),
        degeneratePoint: computeStrokeBBox({ points: pts([[5, 5]]) }),
        fromBbox: computeStrokeBBox({ points: [], bbox: [50, 40, 10, 20] }),
        empty: computeStrokeBBox({ points: [] }),
      },
    });
  });

  it('records drag-created and click-created shape geometry', async () => {
    const drag = (tool: ToolId, x0: number, y0: number, x1: number, y1: number, shift: boolean) => {
      shapes.startDragShape(x0, y0, tool, 0);
      shapes.updateDragShape(x1, y1, shift);
      const s = shapes.commitDragShape();
      return s ? { tool, shift, points: plain(s.points), bbox: s.bbox, endEnding: s.endEnding } : { tool, shift, committed: false };
    };
    const dragged = [
      drag('shape.rect', 10, 10, 110, 60, false),
      drag('shape.rect', 10, 10, 110, 60, true),
      drag('shape.ellipse', 200, 200, 150, 260, true),
      drag('shape.line', 0, 0, 100, 30, true),
      drag('shape.arrow', 0, 0, 100, 60, true),
      drag('shape.cloud', 40, 40, 140, 100, false),
      drag('hatch.region', 300, 300, 200, 250, false),
      drag('shape.rect', 10, 10, 11, 11, false),
    ];
    const click = (tool: ToolId, list: readonly (readonly [number, number])[]) => {
      for (const [x, y] of list) shapes.addPolylineVertex(x, y, tool, 0);
      const s = shapes.commitPolyline();
      return s ? { tool, points: plain(s.points), bbox: s.bbox } : { tool, committed: false };
    };
    const clicked = [
      click('shape.polygon', L_POLY),
      click('hatch.region', L_POLY),
      click('shape.cloud', [[0, 0], [80, 0], [40, 60]]),
      click('shape.polyline', [[5, 5]]),
    ];
    await golden('shapes.created', { dragged, clicked });
  });
});

/* ----------------------------------------------------------------------------------------- */
/* Selection transforms (selectionEngine.svelte.ts)                                           */
/* ----------------------------------------------------------------------------------------- */

describe('golden: selectionEngine', () => {
  const base = (): ActiveStroke[] => [
    stroke('rect', 'shape.rect', pts([[100, 100], [200, 100], [200, 150], [100, 150]]), { bbox: [100, 100, 200, 150] }),
    stroke('tri', 'shape.polygon', pts([[300, 100], [360, 130], [320, 220]])),
    stroke('ink', 'pen.a', pts([[50, 300], [80, 310], [120, 290], [150, 330]], 0.7), { groupId: 'g1' }),
    stroke('ink2', 'pen.a', pts([[60, 360], [90, 380]]), { groupId: 'g1' }),
  ];

  beforeEach(() => {
    ink.completedStrokes = base();
    selection.clearSelection();
  });

  it('records geometric transforms of the selection', async () => {
    const run = (name: string, ids: string[], op: () => void) => {
      ink.completedStrokes = base();
      selection.selectedIds = ids;
      op();
      return { name, selected: ids, strokes: ink.completedStrokes.map(geo) };
    };
    const all = ['rect', 'tri', 'ink', 'ink2'];
    const results = [
      run('move 12.5,-7', ['rect', 'tri'], () => selection.moveSelected(12.5, -7)),
      run('resize 1.5x0.5 about bounds min', ['rect', 'tri'], () => selection.resizeSelected(1.5, 0.5)),
      run('resize 1.5x0.5 about 200,200', ['rect'], () => selection.resizeSelected(1.5, 0.5, 200, 200)),
      run('resize lockAspect 0.5x2', ['tri'], () => selection.resizeSelected(0.5, 2, undefined, undefined, true)),
      run('resize lockAspect flip -1x1 (known: breaks on flips)', ['tri'], () => selection.resizeSelected(-1, 1, undefined, undefined, true)),
      run('rotate 33.7 (known: rounds to 34)', ['rect'], () => selection.rotateSelected(33.7)),
      run('rotate 33.7 snap15', ['rect', 'tri'], () => selection.rotateSelected(33.7, true)),
      run('rotate 370 twice', ['tri'], () => {
        selection.rotateSelected(370);
        selection.rotateSelected(370);
      }),
      run('flip horizontal', ['rect', 'tri'], () => selection.flipSelected('horizontal')),
      run('flip vertical', ['tri'], () => selection.flipSelected('vertical')),
      ...(['left', 'center', 'right', 'top', 'middle', 'bottom'] as const).map((e) => run(`align ${e}`, all, () => selection.alignSelected(e))),
      ...(['front', 'back', 'forward', 'backward'] as const).map((d) => run(`order ${d}`, ['tri'], () => selection.reorderSelected(d))),
      run('duplicate', ['rect', 'ink', 'ink2'], () => {
        selection.duplicateSelected();
      }),
      run('delete', ['tri'], () => selection.deleteSelected()),
    ];
    await golden('selection.transforms', results);
  });

  it('records hit-testing, marquee and lasso selection', async () => {
    const grid: string[] = [];
    for (let y = 95; y <= 205; y += 10) {
      let row = '';
      for (let x = 95; x <= 245; x += 10) row += pointInPolygon({ x, y }, L_POLY.map(([px, py]) => ({ x: px, y: py }))) ? '1' : '0';
      grid.push(row);
    }
    const hits = [
      [150, 125],
      [95, 95],
      [250, 125],
      [330, 150],
      [70, 340],
      [400, 400],
    ].map(([x, y]) => selection.hitTestAtPoint(x, y)?.id ?? null);

    selection.selectAtPoint(70, 305);
    const groupClick = [...selection.selectedIds];
    selection.selectAtPoint(150, 125, true);
    const shiftAdd = [...selection.selectedIds];
    selection.selectAtPoint(150, 125, true);
    const shiftRemove = [...selection.selectedIds];

    selection.startMarquee(90, 90);
    selection.updateMarquee(305, 160);
    const marquee = selection.commitMarquee();
    selection.startMarquee(150, 125);
    selection.updateMarquee(151, 126);
    const marqueeClick = selection.commitMarquee();

    selection.clearSelection();
    selection.startLasso(280, 90);
    for (const [x, y] of [[380, 90], [380, 240], [280, 240]]) selection.updateLasso(x, y);
    const lasso = selection.commitLasso();

    await golden('selection.hit', {
      lPolygonPointInPolygonGrid: { x0: 95, y0: 95, step: 10, rows: grid },
      rectsIntersect: [
        rectsIntersect({ minX: 0, minY: 0, maxX: 10, maxY: 10 }, { minX: 10, minY: 10, maxX: 20, maxY: 20 }),
        rectsIntersect({ minX: 0, minY: 0, maxX: 10, maxY: 10 }, { minX: 11, minY: 0, maxX: 20, maxY: 10 }),
      ],
      hitTestAtPoint: hits,
      groupClick,
      shiftAdd,
      shiftRemove,
      marquee,
      marqueeClick,
      lasso,
      selectionBounds: selection.selectionBounds,
    });
  });
});

/* ----------------------------------------------------------------------------------------- */
/* Signature thresholding (insertEngine.svelte.ts)                                            */
/* ----------------------------------------------------------------------------------------- */

describe('golden: insertEngine signature thresholding', () => {
  it('records applyGrayscaleThresholdToRgba on a synthetic 8x8 scan', async () => {
    const w = 8;
    const h = 8;
    const rgba = new Uint8ClampedArray(w * h * 4);
    const rnd = lcg(42);
    for (let i = 0; i < w * h; i++) {
      const x = i % w;
      const y = Math.floor(i / w);
      const paper = 200 + Math.round(rnd() * 50);
      const isInk = Math.abs(x - y) <= 1;
      rgba[i * 4] = isInk ? 30 + x * 10 : paper;
      rgba[i * 4 + 1] = isInk ? 40 + y * 8 : paper - 5;
      rgba[i * 4 + 2] = isInk ? 120 : paper - 20;
      rgba[i * 4 + 3] = i % 13 === 0 ? 20 : 255;
    }
    const out: Record<string, number[]> = {};
    for (const t of [0, 120, 168, 255]) out[`threshold${t}`] = Array.from(applyGrayscaleThresholdToRgba(rgba, t));
    await golden('insert.signature-threshold', {
      note: 'known weakness: hard luma cut-off with a fixed ink colour (no colour-to-alpha un-mixing, no Otsu)',
      input: Array.from(rgba),
      ...out,
    });
  });
});

// app-tauri/src/lib/core/snapAndMeasure.test.ts
import { describe, expect, it } from 'vitest';
import {
  pointDistance,
  segmentMidpoint,
  segmentIntersection,
  perpendicularFoot,
  SpatialGridIndex,
  SnapEngine,
} from './snapEngine.svelte';
import {
  unitsPerPt,
  createScaleFromPreset,
  computePolylineLengthPt,
  computePolygonAreaPt2,
  computeVolume,
  computeAngleDeg,
  projectPointToRulerIfNear,
  formatAreaMeasurementLabel,
  computeTakeoffTotals,
  MeasureEngine,
  type PageScaleConfig,
  type DigitalRulerState,
} from './measureEngine.svelte';

describe('Phase D5 — CAD OSNAP Vector Content Snapping Engine', () => {
  it('computes exact endpoint and midpoint coordinates and distances', () => {
    const seg = { x1: 20, y1: 40, x2: 120, y2: 100 };
    const mid = segmentMidpoint(seg);
    expect(mid.x).toBeCloseTo(70, 6);
    expect(mid.y).toBeCloseTo(70, 6);
    expect(pointDistance({ x: 20, y: 40 }, mid)).toBeCloseTo(Math.hypot(50, 30), 6);
  });

  it('computes exact line-segment intersection and rejects non-intersecting or parallel segments', () => {
    const segA = { x1: 0, y1: 50, x2: 100, y2: 50 };
    const segB = { x1: 50, y1: 0, x2: 50, y2: 100 };
    const hit = segmentIntersection(segA, segB);
    expect(hit).not.toBeNull();
    expect(hit?.x).toBeCloseTo(50, 6);
    expect(hit?.y).toBeCloseTo(50, 6);

    // Parallel horizontal segments
    const parallel = { x1: 0, y1: 60, x2: 100, y2: 60 };
    expect(segmentIntersection(segA, parallel)).toBeNull();

    // Disjoint segments whose infinite lines intersect outside [0, 1]
    const disjoint = { x1: 150, y1: 0, x2: 150, y2: 40 };
    expect(segmentIntersection(segA, disjoint)).toBeNull();
  });

  it('computes exact perpendicular foot onto a segment when projection lies within [0, 1]', () => {
    const seg = { x1: 0, y1: 0, x2: 100, y2: 0 };
    const foot = perpendicularFoot({ x: 37, y: 45 }, seg);
    expect(foot).not.toBeNull();
    expect(foot?.x).toBeCloseTo(37, 6);
    expect(foot?.y).toBeCloseTo(0, 6);

    // Anchor outside segment span (t > 1)
    expect(perpendicularFoot({ x: 140, y: 45 }, seg)).toBeNull();
  });

  it('enforces deterministic OSNAP priority: endpoint > intersection > midpoint > perpendicular > grid', () => {
    const engine = new SnapEngine();
    engine.gridSnapEnabled = true;
    engine.gridStepPt = 10;

    // Two segments intersecting at (50, 50), with an endpoint also at (50, 50)
    engine.setPageSegments(0, [
      { x1: 50, y1: 50, x2: 150, y2: 50 },
      { x1: 50, y1: 0, x2: 50, y2: 100 },
    ]);

    // Cursor near (50, 50): endpoint, intersection, midpoint of seg2, and grid all overlap
    const snap = engine.resolveSnap(0, 51, 51, { x: 20, y: 50 }, 1.0, []);
    expect(snap).not.toBeNull();
    expect(snap?.kind).toBe('endpoint');
    expect(snap?.tokenVar).toBe('--snap-indicator-endpoint');
    expect(snap?.x).toBeCloseTo(50, 6);
    expect(snap?.y).toBeCloseTo(50, 6);

    // Now test intersection beating midpoint when no endpoint is at (50, 50)
    engine.setPageSegments(1, [
      { x1: 0, y1: 50, x2: 100, y2: 50 },
      { x1: 50, y1: 0, x2: 50, y2: 100 },
    ]);
    const snapIx = engine.resolveSnap(1, 51, 51, null, 1.0, []);
    expect(snapIx?.kind).toBe('intersection');
    expect(snapIx?.tokenVar).toBe('--snap-indicator-intersection');
  });

  it('indexes segments in SpatialGridIndex (cellSizePt = 32) and retrieves local candidates', () => {
    const grid = new SpatialGridIndex(
      [
        { x1: 10, y1: 10, x2: 25, y2: 25 },
        { x1: 400, y1: 400, x2: 450, y2: 450 },
      ],
      32
    );
    const nearOrigin = grid.queryNear(15, 15, 12);
    expect(nearOrigin.length).toBe(1);
    expect(nearOrigin[0].x1).toBe(10);
  });
});

describe('Phase D5 — CAD Scale Calibration, Measurement Math & Digital Ruler', () => {
  it('converts PDF points to calibrated metric (m, mm) and imperial (ft, in) units including architectural presets', () => {
    const scale100 = createScaleFromPreset(0, '1:100', 2, 0);
    // At 1:100, 72 pt (1 inch = 25.4 mm on paper) = 2540 mm = 2.54 m
    expect(scale100.unit).toBe('m');
    expect(unitsPerPt(scale100) * 72).toBeCloseTo(2.54, 4);

    const imperialQuarter = createScaleFromPreset(0, '1/4"=1\'-0"', 2, 0);
    // 1/4" = 18 pt on paper represents 1.0 ft
    expect(imperialQuarter.unit).toBe('ft');
    expect(unitsPerPt(imperialQuarter) * 36).toBeCloseTo(2.0, 6);
  });

  it('accumulates multi-segment polylength and closed perimeter accurately', () => {
    const pts = [
      { x: 0, y: 0 },
      { x: 30, y: 40 }, // 50 pt
      { x: 30, y: 140 }, // 100 pt
    ];
    expect(computePolylineLengthPt(pts, false)).toBeCloseTo(150, 6);
    const closedRightTriangle = [
      { x: 0, y: 0 },
      { x: 30, y: 0 },
      { x: 30, y: 40 },
    ];
    expect(computePolylineLengthPt(closedRightTriangle, true)).toBeCloseTo(120, 6);
  });

  it('computes Shoelace polygon area, rectangle area, and volume multiplication with formatted labels', () => {
    // 100 pt x 50 pt rectangle = 5000 pt^2
    const rectPts = [
      { x: 10, y: 20 },
      { x: 110, y: 20 },
      { x: 110, y: 70 },
      { x: 10, y: 70 },
    ];
    const areaPt2 = computePolygonAreaPt2(rectPts);
    expect(areaPt2).toBeCloseTo(5000, 6);

    // Calibrated scale: 10 pt = 1 m (ratio = 0.1 m/pt => area ratio = 0.01 m^2/pt^2)
    const scale: PageScaleConfig = {
      pageIndex: 0,
      ptDistance: 10,
      realDistance: 1,
      unit: 'm',
      precision: 2,
      depthValue: 3,
    };
    const ratio = unitsPerPt(scale);
    const areaCal = areaPt2 * ratio * ratio; // 50 m^2
    const perimCal = computePolylineLengthPt(rectPts, true) * ratio; // 300 * 0.1 = 30 m
    const volCal = computeVolume(areaCal, scale.depthValue); // 50 * 3 = 150 m^3

    expect(areaCal).toBeCloseTo(50, 6);
    expect(perimCal).toBeCloseTo(30, 6);
    expect(volCal).toBeCloseTo(150, 6);

    const label = formatAreaMeasurementLabel(areaCal, perimCal, scale.depthValue, 'm', 2);
    expect(label).toBe('50.00 m² (Perim: 30.00 m, Vol: 150.00 m³)');
  });

  it('computes 3-point protractor angles for 0°, 45°, 90°, and 180°', () => {
    const v = { x: 0, y: 0 };
    expect(computeAngleDeg({ x: 10, y: 0 }, v, { x: 20, y: 0 })).toBeCloseTo(0, 5);
    expect(computeAngleDeg({ x: 10, y: 0 }, v, { x: 10, y: 10 })).toBeCloseTo(45, 5);
    expect(computeAngleDeg({ x: 10, y: 0 }, v, { x: 0, y: 10 })).toBeCloseTo(90, 5);
    expect(computeAngleDeg({ x: 10, y: 0 }, v, { x: -10, y: 0 })).toBeCloseTo(180, 5);
  });

  it('projects near points collinearly onto the Digital Ruler edge and ignores far points', () => {
    const ruler: DigitalRulerState = {
      visible: true,
      xPt: 100,
      yPt: 200,
      angleDeg: 0,
      lengthPt: 300,
    };
    // Point 8 pt away perpendicularly from the horizontal ruler edge at x = 220
    const near = projectPointToRulerIfNear({ x: 220, y: 208 }, ruler, 14);
    expect(near.snapped).toBe(true);
    expect(near.x).toBeCloseTo(220, 6);
    expect(near.y).toBeCloseTo(200, 6);

    // Point 40 pt away perpendicularly should not snap
    const far = projectPointToRulerIfNear({ x: 220, y: 240 }, ruler, 14);
    expect(far.snapped).toBe(false);
    expect(far.y).toBe(240);
  });

  it('aggregates Quantity Takeoff totals by unit (m, m², m³, deg)', () => {
    const me = new MeasureEngine();
    me.setPageScale(
      {
        pageIndex: 0,
        ptDistance: 10,
        realDistance: 1,
        unit: 'm',
        precision: 2,
        depthValue: 2,
      },
      false
    );

    // Add a 100 pt (10 m) length measurement
    me.startOrAddPoint('measure.length', 0, { x: 0, y: 0 });
    me.commitTwoPointDrag({ x: 100, y: 0 });

    // Add a 100x50 pt (50 m^2, 100 m^3 volume) rectangle area measurement
    me.startOrAddPoint('measure.rectArea', 0, { x: 0, y: 0 });
    me.commitTwoPointDrag({ x: 100, y: 50 });

    const totals = computeTakeoffTotals(me.measurements);
    expect(totals['m']).toBeCloseTo(10, 5);
    expect(totals['m²']).toBeCloseTo(50, 5);
    expect(totals['m³']).toBeCloseTo(100, 5);
  });
});

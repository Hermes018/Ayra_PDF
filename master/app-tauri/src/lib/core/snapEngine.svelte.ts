// app-tauri/src/lib/core/snapEngine.svelte.ts
// Phase D5 — Deterministic CAD OSNAP Content Snapping Engine (Uniform Grid Index + Priority Solver)

import type { ActiveStroke } from './inkEngine.svelte';

export type SnapKind = 'endpoint' | 'midpoint' | 'intersection' | 'perpendicular' | 'grid';

export interface SnapHit {
  x: number;
  y: number;
  kind: SnapKind;
  distPt: number;
  tokenVar: string;
}

export interface SnapSegment {
  x1: number;
  y1: number;
  x2: number;
  y2: number;
}

export const SNAP_TOKEN_MAP: Readonly<Record<SnapKind, string>> = Object.freeze({
  endpoint: '--snap-indicator-endpoint',
  midpoint: '--snap-indicator-midpoint',
  intersection: '--snap-indicator-intersection',
  perpendicular: '--snap-indicator-perpendicular',
  grid: '--snap-indicator-grid',
});

const SNAP_PRIORITY: Readonly<Record<SnapKind, number>> = Object.freeze({
  endpoint: 1,
  intersection: 2,
  midpoint: 3,
  perpendicular: 4,
  grid: 5,
});

/** Euclidean distance in PDF points between two 2D points. */
export function pointDistance(
  a: { x: number; y: number },
  b: { x: number; y: number }
): number {
  return Math.hypot(a.x - b.x, a.y - b.y);
}

/** Extract the two endpoints of a segment. */
export function segmentEndpoints(seg: SnapSegment): [{ x: number; y: number }, { x: number; y: number }] {
  return [
    { x: seg.x1, y: seg.y1 },
    { x: seg.x2, y: seg.y2 },
  ];
}

/** Exact midpoint M = ((x1 + x2)/2, (y1 + y2)/2) of a segment. */
export function segmentMidpoint(seg: SnapSegment): { x: number; y: number } {
  return {
    x: (seg.x1 + seg.x2) / 2,
    y: (seg.y1 + seg.y2) / 2,
  };
}

/**
 * Exact Line-Segment Intersection:
 * D = (x1 - x2)(y3 - y4) - (y1 - y2)(x3 - x4)
 * If |D| > 1e-9, computes t, u in [0, 1] and returns (x1 + t(x2 - x1), y1 + t(y2 - y1)).
 */
export function segmentIntersection(
  a: SnapSegment,
  b: SnapSegment
): { x: number; y: number } | null {
  const { x1, y1, x2, y2 } = a;
  const { x1: x3, y1: y3, x2: x4, y2: y4 } = b;

  const denom = (x1 - x2) * (y3 - y4) - (y1 - y2) * (x3 - x4);
  if (!Number.isFinite(denom) || Math.abs(denom) <= 1e-9) {
    return null;
  }

  const t = ((x1 - x3) * (y3 - y4) - (y1 - y3) * (x3 - x4)) / denom;
  const u = -((x1 - x2) * (y1 - y3) - (y1 - y2) * (x1 - x3)) / denom;

  const eps = 1e-7;
  if (t >= -eps && t <= 1 + eps && u >= -eps && u <= 1 + eps) {
    const tc = Math.min(1, Math.max(0, t));
    return {
      x: x1 + tc * (x2 - x1),
      y: y1 + tc * (y2 - y1),
    };
  }
  return null;
}

/**
 * Exact Perpendicular Foot from reference anchor A(x0, y0) onto segment S1 -> S2:
 * v = S2 - S1
 * t = ((A - S1) . v) / ||v||^2
 * If t in [0, 1], foot is F = S1 + t * v.
 */
export function perpendicularFoot(
  fromPt: { x: number; y: number },
  seg: SnapSegment
): { x: number; y: number } | null {
  const vx = seg.x2 - seg.x1;
  const vy = seg.y2 - seg.y1;
  const lenSq = vx * vx + vy * vy;
  if (lenSq <= 1e-9) {
    return null;
  }

  const t = ((fromPt.x - seg.x1) * vx + (fromPt.y - seg.y1) * vy) / lenSq;
  if (t >= 0 && t <= 1) {
    return {
      x: seg.x1 + t * vx,
      y: seg.y1 + t * vy,
    };
  }
  return null;
}

/**
 * Uniform spatial grid index (`cellSizePt = 32`) for sub-millisecond candidate segment lookup.
 */
export class SpatialGridIndex {
  readonly cellSizePt: number;
  #cells = new Map<string, number[]>();
  #segments: SnapSegment[] = [];

  constructor(segments: SnapSegment[] = [], cellSizePt = 32) {
    this.cellSizePt = Math.max(4, cellSizePt);
    this.rebuild(segments);
  }

  #key(cx: number, cy: number): string {
    return `${cx},${cy}`;
  }

  rebuild(segments: SnapSegment[]): void {
    this.#cells.clear();
    this.#segments = segments;
    const cs = this.cellSizePt;

    for (let idx = 0; idx < segments.length; idx++) {
      const s = segments[idx];
      const minCx = Math.floor(Math.min(s.x1, s.x2) / cs);
      const maxCx = Math.floor(Math.max(s.x1, s.x2) / cs);
      const minCy = Math.floor(Math.min(s.y1, s.y2) / cs);
      const maxCy = Math.floor(Math.max(s.y1, s.y2) / cs);

      for (let cx = minCx; cx <= maxCx; cx++) {
        for (let cy = minCy; cy <= maxCy; cy++) {
          const k = this.#key(cx, cy);
          let bucket = this.#cells.get(k);
          if (!bucket) {
            bucket = [];
            this.#cells.set(k, bucket);
          }
          bucket.push(idx);
        }
      }
    }
  }

  queryNear(xPt: number, yPt: number, radiusPt: number): SnapSegment[] {
    const cs = this.cellSizePt;
    const minCx = Math.floor((xPt - radiusPt) / cs);
    const maxCx = Math.floor((xPt + radiusPt) / cs);
    const minCy = Math.floor((yPt - radiusPt) / cs);
    const maxCy = Math.floor((yPt + radiusPt) / cs);

    const seen = new Set<number>();
    const result: SnapSegment[] = [];

    for (let cx = minCx; cx <= maxCx; cx++) {
      for (let cy = minCy; cy <= maxCy; cy++) {
        const bucket = this.#cells.get(this.#key(cx, cy));
        if (!bucket) continue;
        for (const idx of bucket) {
          if (!seen.has(idx)) {
            seen.add(idx);
            result.push(this.#segments[idx]);
          }
        }
      }
    }
    return result;
  }
}

/** Extract snap segments from committed studio vector strokes on a given page. */
export function extractStrokeSegmentsForPage(
  completedStrokes: ActiveStroke[],
  pageIndex: number
): SnapSegment[] {
  const out: SnapSegment[] = [];
  for (const s of completedStrokes) {
    if ((s.pageIndex ?? 0) !== pageIndex) continue;
    if (
      s.tool === 'shape.rect' ||
      s.tool === 'review.highlight' ||
      s.tool === 'review.redact'
    ) {
      if (s.bbox) {
        const [x0, y0, x1, y1] = s.bbox;
        out.push(
          { x1: x0, y1: y0, x2: x1, y2: y0 },
          { x1: x1, y1: y0, x2: x1, y2: y1 },
          { x1: x1, y1: y1, x2: x0, y2: y1 },
          { x1: x0, y1: y1, x2: x0, y2: y0 }
        );
      } else if (s.points.length >= 2) {
        const p0 = s.points[0];
        const p1 = s.points[s.points.length - 1];
        const x0 = Math.min(p0.x, p1.x);
        const y0 = Math.min(p0.y, p1.y);
        const x1 = Math.max(p0.x, p1.x);
        const y1 = Math.max(p0.y, p1.y);
        out.push(
          { x1: x0, y1: y0, x2: x1, y2: y0 },
          { x1: x1, y1: y0, x2: x1, y2: y1 },
          { x1: x1, y1: y1, x2: x0, y2: y1 },
          { x1: x0, y1: y1, x2: x0, y2: y0 }
        );
      }
      continue;
    }

    if (
      s.tool === 'shape.line' ||
      s.tool === 'shape.arrow' ||
      s.tool === 'shape.polyline' ||
      s.tool === 'shape.polygon' ||
      s.tool === 'shape.cloud' ||
      s.tool === 'hatch.region' ||
      s.points.length === 2
    ) {
      for (let i = 0; i < s.points.length - 1; i++) {
        out.push({
          x1: s.points[i].x,
          y1: s.points[i].y,
          x2: s.points[i + 1].x,
          y2: s.points[i + 1].y,
        });
      }
      if (
        (s.tool === 'shape.polygon' ||
          s.tool === 'shape.cloud' ||
          s.tool === 'hatch.region') &&
        s.points.length >= 3
      ) {
        const first = s.points[0];
        const last = s.points[s.points.length - 1];
        out.push({ x1: last.x, y1: last.y, x2: first.x, y2: first.y });
      }
    }
  }
  return out;
}

export class SnapEngine {
  enabled = $state<boolean>(true);
  gridSnapEnabled = $state<boolean>(false);
  gridStepPt = $state<number>(18); // 0.25 in (18 pt)
  activeSnap = $state<SnapHit | null>(null);
  pageSegments = $state<Record<number, SnapSegment[]>>({});

  #gridCache = new Map<number, { count: number; index: SpatialGridIndex }>();

  setPageSegments(pageIndex: number, segments: SnapSegment[]): void {
    this.pageSegments = {
      ...this.pageSegments,
      [pageIndex]: segments,
    };
    this.#gridCache.set(pageIndex, {
      count: segments.length,
      index: new SpatialGridIndex(segments, 32),
    });
  }

  clearSnap(): void {
    this.activeSnap = null;
  }

  /**
   * Resolve OSNAP hit in PDF user-space points (`pt`) or world coordinates.
   * Priority order when multiple hits fall within `radiusPt`:
   *   `endpoint` > `intersection` > `midpoint` > `perpendicular` > `grid`
   */
  resolveSnap(
    pageIndex: number,
    xPt: number,
    yPt: number,
    anchorPt: { x: number; y: number } | null,
    zoomScale: number,
    completedStrokes: ActiveStroke[] = []
  ): SnapHit | null {
    if (!this.enabled && !this.gridSnapEnabled) {
      this.activeSnap = null;
      return null;
    }

    const radiusPt = 12 / Math.max(zoomScale, 0.1);
    const cursor = { x: xPt, y: yPt };

    const baseSegs = this.pageSegments[pageIndex] ?? [];
    let cached = this.#gridCache.get(pageIndex);
    if (!cached || cached.count !== baseSegs.length) {
      cached = {
        count: baseSegs.length,
        index: new SpatialGridIndex(baseSegs, 32),
      };
      this.#gridCache.set(pageIndex, cached);
    }

    const nearBase = cached.index.queryNear(xPt, yPt, radiusPt * 2);
    const strokeSegs = extractStrokeSegmentsForPage(completedStrokes, pageIndex);
    const candidates = [...nearBase, ...strokeSegs];

    const hits: SnapHit[] = [];

    if (this.enabled && candidates.length > 0) {
      // 1. Endpoints & Midpoints & Perpendicular Feet
      for (const seg of candidates) {
        const [p1, p2] = segmentEndpoints(seg);
        const d1 = pointDistance(cursor, p1);
        if (d1 <= radiusPt) {
          hits.push({
            x: p1.x,
            y: p1.y,
            kind: 'endpoint',
            distPt: d1,
            tokenVar: SNAP_TOKEN_MAP.endpoint,
          });
        }
        const d2 = pointDistance(cursor, p2);
        if (d2 <= radiusPt) {
          hits.push({
            x: p2.x,
            y: p2.y,
            kind: 'endpoint',
            distPt: d2,
            tokenVar: SNAP_TOKEN_MAP.endpoint,
          });
        }

        const mid = segmentMidpoint(seg);
        const dMid = pointDistance(cursor, mid);
        if (dMid <= radiusPt) {
          hits.push({
            x: mid.x,
            y: mid.y,
            kind: 'midpoint',
            distPt: dMid,
            tokenVar: SNAP_TOKEN_MAP.midpoint,
          });
        }

        if (anchorPt) {
          const foot = perpendicularFoot(anchorPt, seg);
          if (foot) {
            const dPerp = pointDistance(cursor, foot);
            if (dPerp <= radiusPt) {
              hits.push({
                x: foot.x,
                y: foot.y,
                kind: 'perpendicular',
                distPt: dPerp,
                tokenVar: SNAP_TOKEN_MAP.perpendicular,
              });
            }
          }
        }
      }

      // 2. Pairwise Segment Intersections (bounded to max 96 local segments for 120Hz budget)
      const cap = Math.min(candidates.length, 96);
      for (let i = 0; i < cap; i++) {
        for (let j = i + 1; j < cap; j++) {
          const ix = segmentIntersection(candidates[i], candidates[j]);
          if (!ix) continue;
          const dIx = pointDistance(cursor, ix);
          if (dIx <= radiusPt) {
            hits.push({
              x: ix.x,
              y: ix.y,
              kind: 'intersection',
              distPt: dIx,
              tokenVar: SNAP_TOKEN_MAP.intersection,
            });
          }
        }
      }
    }

    // 3. Optional Grid Snap fallback
    if (this.gridSnapEnabled && this.gridStepPt > 0) {
      const gx = Math.round(xPt / this.gridStepPt) * this.gridStepPt;
      const gy = Math.round(yPt / this.gridStepPt) * this.gridStepPt;
      const dGrid = pointDistance(cursor, { x: gx, y: gy });
      if (dGrid <= radiusPt) {
        hits.push({
          x: gx,
          y: gy,
          kind: 'grid',
          distPt: dGrid,
          tokenVar: SNAP_TOKEN_MAP.grid,
        });
      }
    }

    if (hits.length === 0) {
      this.activeSnap = null;
      return null;
    }

    hits.sort((a, b) => {
      const prioDiff = SNAP_PRIORITY[a.kind] - SNAP_PRIORITY[b.kind];
      if (prioDiff !== 0) return prioDiff;
      return a.distPt - b.distPt;
    });

    const winner = hits[0];
    this.activeSnap = winner;
    return winner;
  }
}

export const snapEngine = new SnapEngine();

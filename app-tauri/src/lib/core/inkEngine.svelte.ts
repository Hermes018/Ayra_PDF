// app-tauri/src/lib/core/inkEngine.svelte.ts
// Centralized Svelte 5 controller for vector inking, laser pointer decay, smart shapes & boolean eraser.
import { tools, type ToolId, type ToolPreset } from './toolEngine.svelte';
import { theme } from './themeContract.svelte';

export interface Point {
  x: number;
  y: number;
  p: number; // Pressure [0, 1]
  t?: number; // Timestamp in milliseconds
}

export type EraserPoint = { x: number; y: number; p?: number; t?: number };

export interface ActiveStroke {
  id: string;
  tool: ToolId;
  points: Point[];
  preset: ToolPreset;
  color: string;
  widthPt: number;
  opacity: number;
  isHighlighter?: boolean;
}

/** Distance from point p to line segment ab */
function distToSegment(
  p: EraserPoint,
  a: EraserPoint,
  b: EraserPoint
): number {
  const dx = b.x - a.x;
  const dy = b.y - a.y;
  const l2 = dx * dx + dy * dy;
  if (l2 === 0) return Math.hypot(p.x - a.x, p.y - a.y);
  const t = Math.max(0, Math.min(1, ((p.x - a.x) * dx + (p.y - a.y) * dy) / l2));
  return Math.hypot(p.x - (a.x + t * dx), p.y - (a.y + t * dy));
}

/** Ramer-Douglas-Peucker polyline simplification */
function rdp(points: Point[], epsilon: number): Point[] {
  if (points.length < 3) return points;
  let dmax = 0;
  let index = 0;
  const first = points[0];
  const last = points[points.length - 1];
  for (let i = 1; i < points.length - 1; i++) {
    const d = distToSegment(points[i], first, last);
    if (d > dmax) {
      dmax = d;
      index = i;
    }
  }
  if (dmax > epsilon) {
    const rec1 = rdp(points.slice(0, index + 1), epsilon);
    const rec2 = rdp(points.slice(index), epsilon);
    return rec1.slice(0, -1).concat(rec2);
  } else {
    return [first, last];
  }
}

/**
 * Generate variable-width Catmull-Rom spline outline path (the figcraft pattern).
 * Returns a closed SVG path data string that fills the variable-width ribbon.
 */
export function generateSvgPathFromPoints(points: Point[], preset?: ToolPreset): string {
  if (!points || points.length === 0) return '';
  const baseWidth = preset?.stroke?.widthPt ?? 2;

  if (points.length === 1) {
    const p0 = points[0];
    const r = Math.max(0.75, (baseWidth * Math.max(0.1, p0.p)) / 2);
    return `M ${p0.x - r} ${p0.y} a ${r} ${r} 0 1 0 ${r * 2} 0 a ${r} ${r} 0 1 0 ${-r * 2} 0 Z`;
  }

  const n = points.length;
  const left: Point[] = new Array(n);
  const right: Point[] = new Array(n);

  for (let i = 0; i < n; i++) {
    const curr = points[i];
    let tx = 0;
    let ty = 0;

    if (i === 0) {
      tx = points[1].x - curr.x;
      ty = points[1].y - curr.y;
    } else if (i === n - 1) {
      tx = curr.x - points[i - 1].x;
      ty = curr.y - points[i - 1].y;
    } else {
      tx = points[i + 1].x - points[i - 1].x;
      ty = points[i + 1].y - points[i - 1].y;
    }

    const len = Math.hypot(tx, ty);
    const nx = len > 1e-6 ? -ty / len : 0;
    const ny = len > 1e-6 ? tx / len : 1;

    // Radius at sample point i
    const r = Math.max(0.5, (baseWidth * Math.max(0.08, curr.p)) / 2);
    left[i] = { x: curr.x + nx * r, y: curr.y + ny * r, p: curr.p };
    right[i] = { x: curr.x - nx * r, y: curr.y - ny * r, p: curr.p };
  }

  // Build Catmull-Rom spline along left edge, cap at end, Catmull-Rom along right edge backwards, cap at start
  let d = `M ${left[0].x.toFixed(2)} ${left[0].y.toFixed(2)}`;

  // Left curve forward
  for (let i = 0; i < n - 1; i++) {
    const p0 = left[Math.max(0, i - 1)];
    const p1 = left[i];
    const p2 = left[i + 1];
    const p3 = left[Math.min(n - 1, i + 2)];

    const cp1x = p1.x + (p2.x - p0.x) / 6;
    const cp1y = p1.y + (p2.y - p0.y) / 6;
    const cp2x = p2.x - (p3.x - p1.x) / 6;
    const cp2y = p2.y - (p3.y - p1.y) / 6;

    d += ` C ${cp1x.toFixed(2)} ${cp1y.toFixed(2)}, ${cp2x.toFixed(2)} ${cp2y.toFixed(2)}, ${p2.x.toFixed(2)} ${p2.y.toFixed(2)}`;
  }

  // End cap to right[n - 1]
  const endR = Math.max(0.5, (baseWidth * Math.max(0.08, points[n - 1].p)) / 2);
  d += ` A ${endR.toFixed(2)} ${endR.toFixed(2)} 0 0 1 ${right[n - 1].x.toFixed(2)} ${right[n - 1].y.toFixed(2)}`;

  // Right curve backward
  for (let i = n - 1; i > 0; i--) {
    const p0 = right[Math.min(n - 1, i + 1)];
    const p1 = right[i];
    const p2 = right[i - 1];
    const p3 = right[Math.max(0, i - 2)];

    const cp1x = p1.x + (p2.x - p0.x) / 6;
    const cp1y = p1.y + (p2.y - p0.y) / 6;
    const cp2x = p2.x - (p3.x - p1.x) / 6;
    const cp2y = p2.y - (p3.y - p1.y) / 6;

    d += ` C ${cp1x.toFixed(2)} ${cp1y.toFixed(2)}, ${cp2x.toFixed(2)} ${cp2y.toFixed(2)}, ${p2.x.toFixed(2)} ${p2.y.toFixed(2)}`;
  }

  // Start cap back to left[0]
  const startR = Math.max(0.5, (baseWidth * Math.max(0.08, points[0].p)) / 2);
  d += ` A ${startR.toFixed(2)} ${startR.toFixed(2)} 0 0 1 ${left[0].x.toFixed(2)} ${left[0].y.toFixed(2)} Z`;

  return d;
}

export class InkEngine {
  currentStroke = $state<ActiveStroke | null>(null);
  completedStrokes = $state<ActiveStroke[]>([]);
  laserPoints = $state<Point[]>([]);

  #laserRafId: number | null = null;
  #dwellTimeout: ReturnType<typeof setTimeout> | null = null;
  #strokeSeq = 0;

  get isDrawing(): boolean {
    return this.currentStroke !== null;
  }

  getSvgPath(stroke: ActiveStroke): string {
    return generateSvgPathFromPoints(stroke.points, stroke.preset);
  }

  /** Start a new stroke */
  startStroke(x: number, y: number, pressure: number, toolId?: ToolId): void {
    this.clearDwellTimeout();
    const activeTool = toolId ?? tools.currentToolId;
    const preset = tools.activePreset ?? {
      id: `preset.${activeTool}`,
      tool: activeTool,
      label: activeTool,
      stroke: {
        colorToken: null,
        colorHex: theme.token('--glass-text'),
        widthPt: activeTool === 'highlighter' ? 14 : 1.5,
        opacity: activeTool === 'highlighter' ? 0.35 : 1.0,
        dash: [],
        lineCap: 'round',
        startEnd: 'none',
        endEnd: 'none',
      },
      fill: { kind: 'none', colorHex: '', opacity: 0, hatch: 'solid', hatchScale: 1, hatchAngleDeg: 0 },
      font: { family: 'Inter', sizePt: 12, weight: 400, italic: false, leading: 1.2, trackingEm: 0, colorHex: '' },
      pressure: { x1: 0.25, y1: 0.1, x2: 0.25, y2: 1 },
      tiltShading: 0.1,
      smartShapes: true,
    };

    const isHighlighter = activeTool === 'highlighter';
    const color = preset.stroke.colorHex || (isHighlighter ? theme.token('--snap-indicator-intersection') : theme.token('--glass-text'));

    this.currentStroke = {
      id: `stroke-${Date.now()}-${++this.#strokeSeq}`,
      tool: activeTool,
      points: [{ x, y, p: pressure, t: Date.now() }],
      preset,
      color,
      widthPt: preset.stroke.widthPt,
      opacity: preset.stroke.opacity,
      isHighlighter,
    };
  }

  /** Add point to current stroke */
  addPoint(x: number, y: number, pressure: number): void {
    if (!this.currentStroke) return;
    const last = this.currentStroke.points[this.currentStroke.points.length - 1];
    const dist = Math.hypot(x - last.x, y - last.y);

    // Filter sub-pixel jitter
    if (dist < 0.8 && this.currentStroke.points.length > 1) return;

    this.currentStroke.points.push({ x, y, p: pressure, t: Date.now() });

    // Smart shape dwell detection (if speed drops to ~0 for 350ms)
    if (this.currentStroke.preset.smartShapes && dist < 2.5) {
      if (!this.#dwellTimeout) {
        this.#dwellTimeout = setTimeout(() => {
          this.recognizeShape();
        }, 350);
      }
    } else {
      this.clearDwellTimeout();
    }
  }

  /** End current stroke and commit to completedStrokes */
  endStroke(): ActiveStroke | null {
    this.clearDwellTimeout();
    const stroke = this.currentStroke;
    if (!stroke) return null;

    if (stroke.points.length > 0) {
      this.completedStrokes.push(stroke);
    }
    this.currentStroke = null;
    return stroke;
  }

  /** Cancel active stroke without committing */
  cancelStroke(): void {
    this.clearDwellTimeout();
    this.currentStroke = null;
  }

  clearDwellTimeout(): void {
    if (this.#dwellTimeout) {
      clearTimeout(this.#dwellTimeout);
      this.#dwellTimeout = null;
    }
  }

  /**
   * Smart Shape Recognition (350ms dwell):
   * Classifies current stroke points as Line, Rectangle, Triangle, or Circle.
   */
  recognizeShape(): boolean {
    if (!this.currentStroke || this.currentStroke.points.length < 5) return false;
    const pts = this.currentStroke.points;

    let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
    for (const p of pts) {
      if (p.x < minX) minX = p.x;
      if (p.y < minY) minY = p.y;
      if (p.x > maxX) maxX = p.x;
      if (p.y > maxY) maxY = p.y;
    }

    const width = maxX - minX;
    const height = maxY - minY;
    const diag = Math.hypot(width, height);
    if (diag < 10) return false;

    const first = pts[0];
    const last = pts[pts.length - 1];
    const dStartEnd = Math.hypot(last.x - first.x, last.y - first.y);

    // Calculate total path length
    let pathLen = 0;
    for (let i = 1; i < pts.length; i++) {
      pathLen += Math.hypot(pts[i].x - pts[i - 1].x, pts[i].y - pts[i - 1].y);
    }

    // 1. Straight Line check: max orthogonal deviation < 7% of length
    let maxDev = 0;
    for (const p of pts) {
      const dev = distToSegment(p, first, last);
      if (dev > maxDev) maxDev = dev;
    }

    if (dStartEnd > 15 && maxDev / dStartEnd < 0.08) {
      this.currentStroke.points = [
        { x: first.x, y: first.y, p: first.p, t: first.t },
        { x: last.x, y: last.y, p: last.p, t: last.t },
      ];
      return true;
    }

    // 2. Closed shapes
    const isClosed = dStartEnd < 0.22 * pathLen;
    if (isClosed) {
      // Circle check
      const cx = (minX + maxX) / 2;
      const cy = (minY + maxY) / 2;
      const targetR = (width + height) / 4;
      let radialVar = 0;
      for (const p of pts) {
        const r = Math.hypot(p.x - cx, p.y - cy);
        radialVar += Math.pow(r - targetR, 2);
      }
      const radialStd = Math.sqrt(radialVar / pts.length);
      const aspect = Math.abs(width - height) / Math.max(width, height);

      if (radialStd / targetR < 0.16 && aspect < 0.25) {
        // Snap to circle
        const circlePts: Point[] = [];
        const numSteps = 32;
        for (let i = 0; i <= numSteps; i++) {
          const theta = (i / numSteps) * Math.PI * 2;
          circlePts.push({
            x: cx + Math.cos(theta) * targetR,
            y: cy + Math.sin(theta) * targetR,
            p: 0.5,
          });
        }
        this.currentStroke.points = circlePts;
        return true;
      }

      // Polygon classification via RDP
      const simplified = rdp(pts, Math.max(4, 0.04 * diag));
      let verts = [...simplified];
      if (verts.length > 2 && Math.hypot(verts[0].x - verts[verts.length - 1].x, verts[0].y - verts[verts.length - 1].y) < 0.2 * pathLen) {
        verts.pop();
      }

      // 3. Triangle
      if (verts.length === 3) {
        this.currentStroke.points = [
          { x: verts[0].x, y: verts[0].y, p: 0.5 },
          { x: verts[1].x, y: verts[1].y, p: 0.5 },
          { x: verts[2].x, y: verts[2].y, p: 0.5 },
          { x: verts[0].x, y: verts[0].y, p: 0.5 },
        ];
        return true;
      }

      // 4. Rectangle
      if (verts.length === 4) {
        this.currentStroke.points = [
          { x: minX, y: minY, p: 0.5 },
          { x: maxX, y: minY, p: 0.5 },
          { x: maxX, y: maxY, p: 0.5 },
          { x: minX, y: maxY, p: 0.5 },
          { x: minX, y: minY, p: 0.5 },
        ];
        return true;
      }
    }

    return false;
  }

  /* ---------------- Laser Pointer ---------------- */
  addLaserPoint(x: number, y: number): void {
    this.laserPoints.push({ x, y, p: 1.0, t: Date.now() });
    if (!this.#laserRafId && typeof requestAnimationFrame !== 'undefined') {
      this.#laserRafId = requestAnimationFrame(this.updateLaser.bind(this));
    }
  }

  updateLaser(): void {
    const now = Date.now();
    this.laserPoints = this.laserPoints.filter((p) => now - (p.t ?? now) <= 1800);

    if (this.laserPoints.length > 0 && typeof requestAnimationFrame !== 'undefined') {
      this.#laserRafId = requestAnimationFrame(this.updateLaser.bind(this));
    } else {
      this.#laserRafId = null;
    }
  }

  getLaserSvgPath(): string {
    if (this.laserPoints.length < 2) return '';
    const now = Date.now();
    // Decay-weighted points
    const decayed = this.laserPoints.map((p) => {
      const age = now - (p.t ?? now);
      const alpha = Math.max(0, 1 - age / 1800);
      return { ...p, p: alpha * 1.5 };
    });
    return generateSvgPathFromPoints(decayed, {
      id: 'laser-glow',
      tool: 'laser',
      label: 'Laser',
      stroke: { colorToken: null, colorHex: '', widthPt: 10, opacity: 0.8, dash: [], lineCap: 'round', startEnd: 'none', endEnd: 'none' },
      fill: { kind: 'none', colorHex: '', opacity: 0, hatch: 'solid', hatchScale: 1, hatchAngleDeg: 0 },
      font: { family: 'Inter', sizePt: 12, weight: 400, italic: false, leading: 1.2, trackingEm: 0, colorHex: '' },
      pressure: { x1: 0.25, y1: 0.1, x2: 0.25, y2: 1 },
      tiltShading: 0,
      smartShapes: false,
    });
  }

  getLaserCoreSvgPath(): string {
    if (this.laserPoints.length < 2) return '';
    const now = Date.now();
    const decayed = this.laserPoints.map((p) => {
      const age = now - (p.t ?? now);
      const alpha = Math.max(0, 1 - age / 1800);
      return { ...p, p: alpha };
    });
    return generateSvgPathFromPoints(decayed, {
      id: 'laser-core',
      tool: 'laser',
      label: 'Laser Core',
      stroke: { colorToken: null, colorHex: '', widthPt: 3, opacity: 1.0, dash: [], lineCap: 'round', startEnd: 'none', endEnd: 'none' },
      fill: { kind: 'none', colorHex: '', opacity: 0, hatch: 'solid', hatchScale: 1, hatchAngleDeg: 0 },
      font: { family: 'Inter', sizePt: 12, weight: 400, italic: false, leading: 1.2, trackingEm: 0, colorHex: '' },
      pressure: { x1: 0.25, y1: 0.1, x2: 0.25, y2: 1 },
      tiltShading: 0,
      smartShapes: false,
    });
  }

  /* ---------------- Boolean Eraser ---------------- */
  /** Whole eraser: removes any stroke whose trajectory intersects the eraser radius */
  eraseWhole(pointer: EraserPoint, radius = 12): boolean {
    const origLen = this.completedStrokes.length;
    this.completedStrokes = this.completedStrokes.filter((stroke) => {
      // Check segment distances
      for (let i = 0; i < stroke.points.length - 1; i++) {
        const d = distToSegment(pointer, stroke.points[i], stroke.points[i + 1]);
        if (d <= radius) return false; // Hit -> remove whole stroke
      }
      return true;
    });
    return this.completedStrokes.length !== origLen;
  }

  /** Partial eraser: splits stroke into multiple segments when eraser bisects it */
  erasePartial(pointer: EraserPoint, radius = 12): boolean {
    let modified = false;
    const nextStrokes: ActiveStroke[] = [];

    for (const stroke of this.completedStrokes) {
      let hit = false;
      const chunks: Point[][] = [];
      let currentChunk: Point[] = [];

      for (const pt of stroke.points) {
        const d = Math.hypot(pt.x - pointer.x, pt.y - pointer.y);
        if (d > radius) {
          currentChunk.push(pt);
        } else {
          hit = true;
          if (currentChunk.length >= 2) {
            chunks.push(currentChunk);
          }
          currentChunk = [];
        }
      }
      if (currentChunk.length >= 2) {
        chunks.push(currentChunk);
      }

      if (hit) {
        modified = true;
        // Create new active strokes from chunks
        for (let idx = 0; idx < chunks.length; idx++) {
          nextStrokes.push({
            ...stroke,
            id: `${stroke.id}-part-${idx}`,
            points: chunks[idx],
          });
        }
      } else {
        nextStrokes.push(stroke);
      }
    }

    if (modified) {
      this.completedStrokes = nextStrokes;
    }
    return modified;
  }
}

export const ink = new InkEngine();

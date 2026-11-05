// app-tauri/src/lib/core/shapeAndHatchEngine.svelte.ts
// Geometric shapes, multi-segment polylines, scalloped revision clouds, procedural hatching,
// text boxes, review markups (highlight, underline, squiggle, strikethrough), and redactions.

import {
  tools,
  type ToolId,
  type ToolPreset,
  type LineStyleId,
  type LineEnding,
  type HatchPatternExtendedId,
} from './toolEngine.svelte';
import { ink, type ActiveStroke, type Point } from './inkEngine.svelte';
import { commands } from './commandEngine.svelte';
import { theme } from './themeContract.svelte';

export interface BoundingBox {
  minX: number;
  minY: number;
  maxX: number;
  maxY: number;
  width: number;
  height: number;
  cx: number;
  cy: number;
}

export function computeStrokeBBox(stroke: Pick<ActiveStroke, 'points' | 'bbox'>): BoundingBox {
  if (stroke.points && stroke.points.length > 0) {
    let minX = Infinity;
    let minY = Infinity;
    let maxX = -Infinity;
    let maxY = -Infinity;
    for (const p of stroke.points) {
      if (p.x < minX) minX = p.x;
      if (p.y < minY) minY = p.y;
      if (p.x > maxX) maxX = p.x;
      if (p.y > maxY) maxY = p.y;
    }
    const width = Math.max(1, maxX - minX);
    const height = Math.max(1, maxY - minY);
    return {
      minX,
      minY,
      maxX: minX + width,
      maxY: minY + height,
      width,
      height,
      cx: minX + width / 2,
      cy: minY + height / 2,
    };
  }
  if (stroke.bbox) {
    const [x0, y0, x1, y1] = stroke.bbox;
    const minX = Math.min(x0, x1);
    const minY = Math.min(y0, y1);
    const maxX = Math.max(x0, x1);
    const maxY = Math.max(y0, y1);
    const width = Math.max(1, maxX - minX);
    const height = Math.max(1, maxY - minY);
    return {
      minX,
      minY,
      maxX: minX + width,
      maxY: minY + height,
      width,
      height,
      cx: minX + width / 2,
      cy: minY + height / 2,
    };
  }
  return { minX: 0, minY: 0, maxX: 1, maxY: 1, width: 1, height: 1, cx: 0.5, cy: 0.5 };
}

/** Convert a `LineStyleId` and stroke width into an SVG `stroke-dasharray` string. */
export function getStrokeDashArray(style: LineStyleId | undefined, widthPt = 2): string | undefined {
  const w = Math.max(1, widthPt);
  switch (style) {
    case 'dashed':
      return `${w * 4} ${w * 2.5}`;
    case 'dotted':
      return `${w * 0.8} ${w * 2.2}`;
    case 'dash-dot':
      return `${w * 4.5} ${w * 2} ${w * 1} ${w * 2}`;
    default:
      return undefined;
  }
}

/** Generate a scalloped cubic Bézier revision cloud path around a polygon or rectangle. */
export function generateRevisionCloudPath(points: Point[], arcChord = 18): string {
  if (!points || points.length < 2) return '';
  let vertices = points;
  if (points.length === 2) {
    const x0 = Math.min(points[0].x, points[1].x);
    const y0 = Math.min(points[0].y, points[1].y);
    const x1 = Math.max(points[0].x, points[1].x);
    const y1 = Math.max(points[0].y, points[1].y);
    vertices = [
      { x: x0, y: y0, p: 0.5 },
      { x: x1, y: y0, p: 0.5 },
      { x: x1, y: y1, p: 0.5 },
      { x: x0, y: y1, p: 0.5 },
    ];
  }

  const closed = [...vertices];
  const first = closed[0];
  const last = closed[closed.length - 1];
  if (Math.hypot(last.x - first.x, last.y - first.y) > 1e-3) {
    closed.push({ x: first.x, y: first.y, p: 0.5 });
  }

  const chord = Math.max(8, arcChord);
  let d = `M ${closed[0].x.toFixed(2)} ${closed[0].y.toFixed(2)}`;

  for (let i = 0; i < closed.length - 1; i++) {
    const a = closed[i];
    const b = closed[i + 1];
    const dx = b.x - a.x;
    const dy = b.y - a.y;
    const segLen = Math.hypot(dx, dy);
    if (segLen < 1e-3) continue;

    const steps = Math.max(1, Math.round(segLen / chord));
    const stepX = dx / steps;
    const stepY = dy / steps;
    // Outward normal (assuming clockwise winding)
    const nx = dy / segLen;
    const ny = -dx / segLen;
    const bulge = Math.min(chord * 0.55, segLen / steps * 0.6);

    for (let s = 0; s < steps; s++) {
      const sx = a.x + stepX * s;
      const sy = a.y + stepY * s;
      const ex = a.x + stepX * (s + 1);
      const ey = a.y + stepY * (s + 1);
      const cp1x = sx + stepX * 0.25 + nx * bulge;
      const cp1y = sy + stepY * 0.25 + ny * bulge;
      const cp2x = sx + stepX * 0.75 + nx * bulge;
      const cp2y = sy + stepY * 0.75 + ny * bulge;
      d += ` C ${cp1x.toFixed(2)} ${cp1y.toFixed(2)}, ${cp2x.toFixed(2)} ${cp2y.toFixed(2)}, ${ex.toFixed(2)} ${ey.toFixed(2)}`;
    }
  }

  return `${d} Z`;
}

/** Generate an open or closed polyline SVG path from vertices. */
export function generatePolylineSvgPath(
  points: readonly { x: number; y: number; p?: number; t?: number }[],
  closed = false
): string {
  if (!points || points.length === 0) return '';
  let d = `M ${points[0].x.toFixed(2)} ${points[0].y.toFixed(2)}`;
  for (let i = 1; i < points.length; i++) {
    d += ` L ${points[i].x.toFixed(2)} ${points[i].y.toFixed(2)}`;
  }
  if (closed && points.length >= 3) {
    d += ' Z';
  }
  return d;
}

/** Generate a sine-wave squiggle SVG path across a horizontal bounding box. */
export function generateSquiggleSvgPath(bbox: BoundingBox): string {
  const yBase = bbox.maxY - 2;
  const waveLen = 8;
  const amp = 2.5;
  let d = `M ${bbox.minX.toFixed(2)} ${yBase.toFixed(2)}`;
  let x = bbox.minX;
  let dir = 1;
  while (x < bbox.maxX) {
    const nextX = Math.min(bbox.maxX, x + waveLen / 2);
    const midX = (x + nextX) / 2;
    const peakY = yBase + dir * amp;
    d += ` Q ${midX.toFixed(2)} ${peakY.toFixed(2)}, ${nextX.toFixed(2)} ${yBase.toFixed(2)}`;
    x = nextX;
    dir *= -1;
  }
  return d;
}

/** Generate SVG path data for line endings (arrows, circles, diamonds, slashes, butts). */
export function generateLineEndingPath(
  tip: Point,
  prev: Point,
  ending: LineEnding | undefined,
  widthPt = 2
): { d: string; filled: boolean } | null {
  if (!ending || ending === 'none') return null;
  const dx = tip.x - prev.x;
  const dy = tip.y - prev.y;
  const len = Math.hypot(dx, dy);
  if (len < 1e-3) return null;

  const ux = dx / len;
  const uy = dy / len;
  const nx = -uy;
  const ny = ux;
  const size = Math.max(6, widthPt * 3.2);

  switch (ending) {
    case 'open':
    case 'open-arrow': {
      const bx = tip.x - ux * size;
      const by = tip.y - uy * size;
      const l1x = bx + nx * (size * 0.55);
      const l1y = by + ny * (size * 0.55);
      const l2x = bx - nx * (size * 0.55);
      const l2y = by - ny * (size * 0.55);
      return {
        d: `M ${l1x.toFixed(2)} ${l1y.toFixed(2)} L ${tip.x.toFixed(2)} ${tip.y.toFixed(2)} L ${l2x.toFixed(2)} ${l2y.toFixed(2)}`,
        filled: false,
      };
    }
    case 'closed':
    case 'closed-arrow': {
      const bx = tip.x - ux * size;
      const by = tip.y - uy * size;
      const l1x = bx + nx * (size * 0.55);
      const l1y = by + ny * (size * 0.55);
      const l2x = bx - nx * (size * 0.55);
      const l2y = by - ny * (size * 0.55);
      return {
        d: `M ${l1x.toFixed(2)} ${l1y.toFixed(2)} L ${tip.x.toFixed(2)} ${tip.y.toFixed(2)} L ${l2x.toFixed(2)} ${l2y.toFixed(2)} Z`,
        filled: true,
      };
    }
    case 'circle': {
      const r = size * 0.4;
      return {
        d: `M ${(tip.x - r).toFixed(2)} ${tip.y.toFixed(2)} a ${r.toFixed(2)} ${r.toFixed(2)} 0 1 0 ${(r * 2).toFixed(2)} 0 a ${r.toFixed(2)} ${r.toFixed(2)} 0 1 0 ${(-r * 2).toFixed(2)} 0 Z`,
        filled: true,
      };
    }
    case 'diamond': {
      const r = size * 0.5;
      const p1x = tip.x + ux * r;
      const p1y = tip.y + uy * r;
      const p2x = tip.x + nx * r;
      const p2y = tip.y + ny * r;
      const p3x = tip.x - ux * r;
      const p3y = tip.y - uy * r;
      const p4x = tip.x - nx * r;
      const p4y = tip.y - ny * r;
      return {
        d: `M ${p1x.toFixed(2)} ${p1y.toFixed(2)} L ${p2x.toFixed(2)} ${p2y.toFixed(2)} L ${p3x.toFixed(2)} ${p3y.toFixed(2)} L ${p4x.toFixed(2)} ${p4y.toFixed(2)} Z`,
        filled: true,
      };
    }
    case 'slash': {
      const r = size * 0.55;
      const sx = tip.x + (ux + nx) * r * 0.7;
      const sy = tip.y + (uy + ny) * r * 0.7;
      const ex = tip.x - (ux + nx) * r * 0.7;
      const ey = tip.y - (uy + ny) * r * 0.7;
      return {
        d: `M ${sx.toFixed(2)} ${sy.toFixed(2)} L ${ex.toFixed(2)} ${ey.toFixed(2)}`,
        filled: false,
      };
    }
    case 'butt':
    case 'square': {
      const r = size * 0.5;
      const sx = tip.x + nx * r;
      const sy = tip.y + ny * r;
      const ex = tip.x - nx * r;
      const ey = tip.y - ny * r;
      return {
        d: `M ${sx.toFixed(2)} ${sy.toFixed(2)} L ${ex.toFixed(2)} ${ey.toFixed(2)}`,
        filled: false,
      };
    }
    default:
      return null;
  }
}

export function isShapeOrMarkupTool(toolId: ToolId): boolean {
  return (
    toolId === 'shape' ||
    toolId.startsWith('shape.') ||
    toolId.startsWith('hatch.') ||
    toolId === 'text.review' ||
    toolId.startsWith('review.') ||
    toolId === 'insert.text' ||
    toolId === 'insert.textBox' ||
    toolId === 'insert.note'
  );
}

export function isMultiClickTool(toolId: ToolId): boolean {
  return (
    toolId === 'shape.polyline' ||
    toolId === 'shape.polygon' ||
    toolId === 'shape.cloud' ||
    toolId === 'hatch.region'
  );
}

export class ShapeAndHatchEngine {
  /** Active shape/markup being dragged or clicked into existence */
  activeShape = $state<ActiveStroke | null>(null);
  /** Cursor preview point for multi-click polyline / polygon / cloud tools */
  previewPoint = $state<Point | null>(null);
  /** Inline text editor state for `insert.text` / `insert.textBox` or Structural Edit mode */
  editingTextBox = $state<{
    id: string;
    x: number;
    y: number;
    width: number;
    height: number;
    text: string;
    fontSize: number;
    color: string;
    pageIndex: number;
    isStructural: boolean;
  } | null>(null);

  #seq = 0;

  get isCreating(): boolean {
    return this.activeShape !== null;
  }

  /** Start a two-point drag shape (`shape.line`, `shape.arrow`, `shape.rect`, `shape.ellipse`, `review.*`) */
  startDragShape(x: number, y: number, toolId?: ToolId, pageIndex = 0): void {
    const activeTool = toolId ?? tools.currentToolId;
    const preset = tools.activePreset;
    const color =
      activeTool === 'review.redact'
        ? theme.token('--danger')
        : preset?.stroke.colorHex || theme.token('--glass-text');
    const hatchPattern: HatchPatternExtendedId =
      activeTool.startsWith('hatch') && (!preset?.fill.hatch || preset.fill.hatch === 'none')
        ? 'diagonal-steel'
        : (preset?.fill.hatch ?? 'none');

    this.activeShape = {
      id: `shape-${Date.now()}-${++this.#seq}`,
      tool: activeTool,
      points: [
        { x, y, p: 0.5, t: Date.now() },
        { x, y, p: 0.5, t: Date.now() },
      ],
      preset: preset as ToolPreset,
      color,
      widthPt: preset?.stroke.widthPt ?? 2,
      opacity: activeTool === 'review.highlight' ? 0.35 : (preset?.stroke.opacity ?? 1.0),
      pageIndex,
      bbox: [x, y, x + 1, y + 1],
      fillColor: preset?.fill.colorHex || color,
      fillOpacity: preset?.fill.opacity ?? 0.25,
      hatchPattern,
      hatchScale: preset?.fill.hatchScale ?? 1.0,
      lineStyle: preset?.stroke.lineStyle ?? 'solid',
      startEnding: preset?.stroke.startEnd ?? 'none',
      endEnding:
        activeTool === 'shape.arrow' && (!preset?.stroke.endEnd || preset.stroke.endEnd === 'none')
          ? 'open-arrow'
          : (preset?.stroke.endEnd ?? 'none'),
      rotationDeg: 0,
    };
  }

  /** Update drag endpoint */
  updateDragShape(x: number, y: number, shiftSnap = false): void {
    if (!this.activeShape || this.activeShape.points.length < 2) return;
    const start = this.activeShape.points[0];
    let targetX = x;
    let targetY = y;

    if (shiftSnap) {
      const dx = x - start.x;
      const dy = y - start.y;
      if (this.activeShape.tool === 'shape.line' || this.activeShape.tool === 'shape.arrow') {
        const angle = Math.atan2(dy, dx);
        const snappedAngle = Math.round(angle / (Math.PI / 4)) * (Math.PI / 4);
        const dist = Math.hypot(dx, dy);
        targetX = start.x + Math.cos(snappedAngle) * dist;
        targetY = start.y + Math.sin(snappedAngle) * dist;
      } else {
        const side = Math.max(Math.abs(dx), Math.abs(dy));
        targetX = start.x + Math.sign(dx || 1) * side;
        targetY = start.y + Math.sign(dy || 1) * side;
      }
    }

    this.activeShape.points[1] = { x: targetX, y: targetY, p: 0.5, t: Date.now() };
    this.activeShape.bbox = [
      Math.min(start.x, targetX),
      Math.min(start.y, targetY),
      Math.max(start.x, targetX),
      Math.max(start.y, targetY),
    ];
  }

  /** Commit a two-point drag shape to `ink.completedStrokes` with undo support */
  commitDragShape(): ActiveStroke | null {
    const s = this.activeShape;
    if (!s) return null;
    this.activeShape = null;
    this.previewPoint = null;

    const bbox = computeStrokeBBox(s);
    if (bbox.width < 2 && bbox.height < 2) return null;

    // Expand points for rectangle/ellipse/cloud if created via 2-point drag
    if (
      s.tool === 'shape.rect' ||
      s.tool === 'shape.cloud' ||
      s.tool === 'hatch.region' ||
      s.tool === 'hatch.pick'
    ) {
      s.points = [
        { x: bbox.minX, y: bbox.minY, p: 0.5 },
        { x: bbox.maxX, y: bbox.minY, p: 0.5 },
        { x: bbox.maxX, y: bbox.maxY, p: 0.5 },
        { x: bbox.minX, y: bbox.maxY, p: 0.5 },
      ];
    }

    this.pushStrokeUndoable(s, `Add ${s.tool}`);
    return s;
  }

  /** Add vertex to a multi-click polyline, polygon, revision cloud, or hatch region */
  addPolylineVertex(x: number, y: number, toolId?: ToolId, pageIndex = 0): void {
    const activeTool = toolId ?? tools.currentToolId;
    if (!this.activeShape) {
      const preset = tools.activePreset;
      const color = preset?.stroke.colorHex || theme.token('--glass-text');
      const hatchPattern: HatchPatternExtendedId =
        activeTool.startsWith('hatch') && (!preset?.fill.hatch || preset.fill.hatch === 'none')
          ? 'diagonal-steel'
          : (preset?.fill.hatch ?? 'none');

      this.activeShape = {
        id: `poly-${Date.now()}-${++this.#seq}`,
        tool: activeTool,
        points: [{ x, y, p: 0.5, t: Date.now() }],
        preset: preset as ToolPreset,
        color,
        widthPt: preset?.stroke.widthPt ?? 2,
        opacity: preset?.stroke.opacity ?? 1.0,
        pageIndex,
        fillColor: preset?.fill.colorHex || color,
        fillOpacity: preset?.fill.opacity ?? 0.25,
        hatchPattern,
        hatchScale: preset?.fill.hatchScale ?? 1.0,
        lineStyle: preset?.stroke.lineStyle ?? 'solid',
        startEnding: preset?.stroke.startEnd ?? 'none',
        endEnding: preset?.stroke.endEnd ?? 'none',
        rotationDeg: 0,
      };
      this.previewPoint = { x, y, p: 0.5 };
      return;
    }

    this.activeShape.points = [...this.activeShape.points, { x, y, p: 0.5, t: Date.now() }];
    this.previewPoint = { x, y, p: 0.5 };
  }

  updatePolylinePreview(x: number, y: number): void {
    if (!this.activeShape) return;
    this.previewPoint = { x, y, p: 0.5 };
  }

  /** Commit active multi-click polyline / polygon / cloud (on double-click or Enter) */
  commitPolyline(): ActiveStroke | null {
    const s = this.activeShape;
    if (!s) return null;
    this.activeShape = null;
    this.previewPoint = null;

    if (s.points.length < 2) return null;
    const bbox = computeStrokeBBox(s);
    s.bbox = [bbox.minX, bbox.minY, bbox.maxX, bbox.maxY];

    this.pushStrokeUndoable(s, `Add ${s.tool}`);
    return s;
  }

  cancelActiveShape(): void {
    this.activeShape = null;
    this.previewPoint = null;
    this.editingTextBox = null;
  }

  /** Open an inline rich text box editor at `(x, y)` in world coordinates */
  openTextBoxEditor(x: number, y: number, pageIndex = 0, isStructural = false): void {
    const preset = tools.activePreset;
    const color = preset?.stroke.colorHex || theme.token('--glass-text');
    const fontSize = preset?.font.sizePt ?? 14;
    this.editingTextBox = {
      id: `text-${Date.now()}-${++this.#seq}`,
      x,
      y,
      width: 220,
      height: 54,
      text: isStructural ? 'Edited PDF Paragraph' : 'Annotation Text',
      fontSize,
      color,
      pageIndex,
      isStructural,
    };
  }

  /** Commit the active inline text box as a canvas annotation (or return info for Structural Edit IPC) */
  commitTextBox(): {
    stroke: ActiveStroke | null;
    structuralEdit: {
      pageIndex: number;
      bboxPt: [number, number, number, number];
      newText: string;
      fontSize: number;
      color: string;
    } | null;
  } {
    const box = this.editingTextBox;
    if (!box) return { stroke: null, structuralEdit: null };
    this.editingTextBox = null;

    const trimmed = box.text.trim();
    if (!trimmed) return { stroke: null, structuralEdit: null };

    if (box.isStructural) {
      return {
        stroke: null,
        structuralEdit: {
          pageIndex: box.pageIndex,
          bboxPt: [box.x, box.y, box.x + box.width, box.y + box.height],
          newText: trimmed,
          fontSize: box.fontSize,
          color: box.color,
        },
      };
    }

    const preset = tools.activePreset;
    const stroke: ActiveStroke = {
      id: box.id,
      tool: 'insert.textBox',
      points: [
        { x: box.x, y: box.y, p: 0.5 },
        { x: box.x + box.width, y: box.y + box.height, p: 0.5 },
      ],
      preset: preset as ToolPreset,
      color: box.color,
      widthPt: 1.5,
      opacity: preset?.stroke.opacity ?? 1.0,
      pageIndex: box.pageIndex,
      bbox: [box.x, box.y, box.x + box.width, box.y + box.height],
      text: trimmed,
      fontSize: box.fontSize,
      rotationDeg: 0,
    };

    this.pushStrokeUndoable(stroke, 'Add Text Box');
    return { stroke, structuralEdit: null };
  }

  /** Push a stroke to `ink.completedStrokes` and record an undoable command on `commandEngine` */
  pushStrokeUndoable(stroke: ActiveStroke, label: string): void {
    const before = [...ink.completedStrokes];
    const after = [...before, stroke];
    ink.completedStrokes = after;
    this.recordStrokeHistory(before, after, label);
  }

  recordStrokeHistory(before: ActiveStroke[], after: ActiveStroke[], label: string): void {
    const handlerId = 'studio.strokes.mutate';
    commands.register(handlerId, (doc, args) => {
      const payload = args as {
        prevStrokes: ActiveStroke[];
        nextStrokes: ActiveStroke[];
        label: string;
      };
      ink.completedStrokes = payload.nextStrokes;
      return {
        next: Object.freeze({ rev: doc.rev + 1, pages: doc.pages }),
        label: payload.label,
        onUndo: () => {
          ink.completedStrokes = payload.prevStrokes;
        },
        onRedo: () => {
          ink.completedStrokes = payload.nextStrokes;
        },
      };
    });
    commands.dispatch({
      kind: 'execute',
      command: handlerId,
      args: { prevStrokes: before, nextStrokes: after, label },
    });
  }
}

export const shapes = new ShapeAndHatchEngine();

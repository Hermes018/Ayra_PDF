// app-tauri/src/lib/core/selectionEngine.svelte.ts
// Multi-Select, Marquee, Lasso (point-in-polygon), Group/Ungroup, Align, Flip, Z-Order,
// Duplicate (Ctrl+D), Format Painter, and Snapshot Stamp Engine.

import { ink, type ActiveStroke, type Point } from './inkEngine.svelte';
import { computeStrokeBBox, shapes, type BoundingBox } from './shapeAndHatchEngine.svelte';
import { tools, type ToolPreset } from './toolEngine.svelte';
import { theme } from './themeContract.svelte';

export type AlignEdge = 'left' | 'center' | 'right' | 'top' | 'middle' | 'bottom';
export type FlipAxis = 'horizontal' | 'vertical';
export type ZOrderDirection = 'front' | 'back' | 'forward' | 'backward';
export type ResizeHandleId = 'nw' | 'n' | 'ne' | 'e' | 'se' | 's' | 'sw' | 'w';

/** Ray-casting point-in-polygon hit test. */
export function pointInPolygon(pt: { x: number; y: number }, polygon: readonly { x: number; y: number }[]): boolean {
  if (polygon.length < 3) return false;
  let inside = false;
  for (let i = 0, j = polygon.length - 1; i < polygon.length; j = i++) {
    const xi = polygon[i].x;
    const yi = polygon[i].y;
    const xj = polygon[j].x;
    const yj = polygon[j].y;

    const intersect =
      yi > pt.y !== yj > pt.y &&
      pt.x < ((xj - xi) * (pt.y - yi)) / (yj - yi + 1e-12) + xi;
    if (intersect) inside = !inside;
  }
  return inside;
}

export function rectsIntersect(
  a: { minX: number; minY: number; maxX: number; maxY: number },
  b: { minX: number; minY: number; maxX: number; maxY: number }
): boolean {
  return a.minX <= b.maxX && a.maxX >= b.minX && a.minY <= b.maxY && a.maxY >= b.minY;
}

function cloneStroke(s: ActiveStroke): ActiveStroke {
  return {
    ...s,
    points: s.points.map((p) => ({ ...p })),
    bbox: s.bbox ? [...s.bbox] : undefined,
  };
}

export class SelectionEngine {
  selectedIds = $state<string[]>([]);
  marqueeRect = $state<{ x0: number; y0: number; x1: number; y1: number } | null>(null);
  lassoPoints = $state<Point[]>([]);
  copiedStyle = $state<{
    color: string;
    widthPt: number;
    opacity: number;
    fillColor?: string;
    fillOpacity?: number;
    hatchPattern?: ActiveStroke['hatchPattern'];
    hatchScale?: number;
    lineStyle?: ActiveStroke['lineStyle'];
    startEnding?: ActiveStroke['startEnding'];
    endEnding?: ActiveStroke['endEnding'];
  } | null>(null);
  formatPainterActive = $state(false);

  #groupSeq = 0;
  #dupSeq = 0;

  readonly hasSelection = $derived(this.selectedIds.length > 0);
  readonly selectedStrokes = $derived(
    ink.completedStrokes.filter((s) => this.selectedIds.includes(s.id))
  );

  readonly selectionBounds = $derived.by<BoundingBox | null>(() => {
    const list = this.selectedStrokes;
    if (list.length === 0) return null;
    let minX = Infinity;
    let minY = Infinity;
    let maxX = -Infinity;
    let maxY = -Infinity;
    for (const s of list) {
      const b = computeStrokeBBox(s);
      if (b.minX < minX) minX = b.minX;
      if (b.minY < minY) minY = b.minY;
      if (b.maxX > maxX) maxX = b.maxX;
      if (b.maxY > maxY) maxY = b.maxY;
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
  });

  clearSelection(): void {
    this.selectedIds = [];
    this.marqueeRect = null;
    this.lassoPoints = [];
  }

  /** Hit-test a single world point against completed strokes (topmost first). */
  hitTestAtPoint(x: number, y: number, tolerance = 10): ActiveStroke | null {
    for (let i = ink.completedStrokes.length - 1; i >= 0; i--) {
      const s = ink.completedStrokes[i];
      const b = computeStrokeBBox(s);
      if (
        x >= b.minX - tolerance &&
        x <= b.maxX + tolerance &&
        y >= b.minY - tolerance &&
        y <= b.maxY + tolerance
      ) {
        return s;
      }
    }
    return null;
  }

  /** Select a stroke at `(x, y)` (expanding to all members of its `groupId`). */
  selectAtPoint(x: number, y: number, additiveShift = false): ActiveStroke | null {
    const hit = this.hitTestAtPoint(x, y);
    if (!hit) {
      if (!additiveShift) this.selectedIds = [];
      return null;
    }

    const groupMembers = hit.groupId
      ? ink.completedStrokes.filter((s) => s.groupId === hit.groupId).map((s) => s.id)
      : [hit.id];

    if (this.formatPainterActive && this.copiedStyle) {
      this.applyFormatPainterToIds(groupMembers);
      this.formatPainterActive = false;
      this.selectedIds = groupMembers;
      return hit;
    }

    if (additiveShift) {
      const alreadySelected = this.selectedIds.includes(hit.id);
      if (alreadySelected) {
        this.selectedIds = this.selectedIds.filter((id) => !groupMembers.includes(id));
      } else {
        const merged = new Set([...this.selectedIds, ...groupMembers]);
        this.selectedIds = Array.from(merged);
      }
    } else {
      this.selectedIds = groupMembers;
    }
    return hit;
  }

  /* ---------------- Marquee Selection ---------------- */
  startMarquee(x: number, y: number): void {
    this.marqueeRect = { x0: x, y0: y, x1: x, y1: y };
  }

  updateMarquee(x: number, y: number): void {
    if (!this.marqueeRect) return;
    this.marqueeRect = { ...this.marqueeRect, x1: x, y1: y };
  }

  commitMarquee(additiveShift = false): string[] {
    const m = this.marqueeRect;
    this.marqueeRect = null;
    if (!m) return this.selectedIds;

    const box = {
      minX: Math.min(m.x0, m.x1),
      minY: Math.min(m.y0, m.y1),
      maxX: Math.max(m.x0, m.x1),
      maxY: Math.max(m.y0, m.y1),
    };

    if (box.maxX - box.minX < 3 && box.maxY - box.minY < 3) {
      this.selectAtPoint(m.x0, m.y0, additiveShift);
      return this.selectedIds;
    }

    const matchedIds = new Set<string>(additiveShift ? this.selectedIds : []);
    for (const s of ink.completedStrokes) {
      const sb = computeStrokeBBox(s);
      if (rectsIntersect(box, sb)) {
        if (s.groupId) {
          for (const gm of ink.completedStrokes) {
            if (gm.groupId === s.groupId) matchedIds.add(gm.id);
          }
        } else {
          matchedIds.add(s.id);
        }
      }
    }
    this.selectedIds = Array.from(matchedIds);
    return this.selectedIds;
  }

  /* ---------------- Lasso Selection ---------------- */
  startLasso(x: number, y: number): void {
    this.lassoPoints = [{ x, y, p: 0.5 }];
  }

  updateLasso(x: number, y: number): void {
    if (this.lassoPoints.length === 0) return;
    this.lassoPoints = [...this.lassoPoints, { x, y, p: 0.5 }];
  }

  commitLasso(additiveShift = false): string[] {
    const poly = this.lassoPoints;
    this.lassoPoints = [];
    if (poly.length < 3) return this.selectedIds;

    const matchedIds = new Set<string>(additiveShift ? this.selectedIds : []);
    for (const s of ink.completedStrokes) {
      const b = computeStrokeBBox(s);
      const centerInside = pointInPolygon({ x: b.cx, y: b.cy }, poly);
      const anyVertexInside = s.points.some((pt) => pointInPolygon(pt, poly));
      if (centerInside || anyVertexInside) {
        if (s.groupId) {
          for (const gm of ink.completedStrokes) {
            if (gm.groupId === s.groupId) matchedIds.add(gm.id);
          }
        } else {
          matchedIds.add(s.id);
        }
      }
    }
    this.selectedIds = Array.from(matchedIds);
    return this.selectedIds;
  }

  /* ---------------- Group / Ungroup ---------------- */
  groupSelected(): string | null {
    if (this.selectedIds.length < 2) return null;
    const gid = `group-${Date.now()}-${++this.#groupSeq}`;
    const before = ink.completedStrokes.map(cloneStroke);
    const after = ink.completedStrokes.map((s) =>
      this.selectedIds.includes(s.id) ? { ...cloneStroke(s), groupId: gid } : cloneStroke(s)
    );
    shapes.recordStrokeHistory(before, after, 'Group Annotations');
    return gid;
  }

  ungroupSelected(): void {
    if (this.selectedIds.length === 0) return;
    const before = ink.completedStrokes.map(cloneStroke);
    const after = ink.completedStrokes.map((s) => {
      if (this.selectedIds.includes(s.id)) {
        const copy = cloneStroke(s);
        delete copy.groupId;
        return copy;
      }
      return cloneStroke(s);
    });
    shapes.recordStrokeHistory(before, after, 'Ungroup Annotations');
  }

  /* ---------------- Align Selected ---------------- */
  alignSelected(edge: AlignEdge): void {
    const bounds = this.selectionBounds;
    if (!bounds || this.selectedIds.length < 2) return;

    const before = ink.completedStrokes.map(cloneStroke);
    const after = ink.completedStrokes.map((s) => {
      if (!this.selectedIds.includes(s.id)) return cloneStroke(s);
      const sb = computeStrokeBBox(s);
      let dx = 0;
      let dy = 0;
      switch (edge) {
        case 'left':
          dx = bounds.minX - sb.minX;
          break;
        case 'center':
          dx = bounds.cx - sb.cx;
          break;
        case 'right':
          dx = bounds.maxX - sb.maxX;
          break;
        case 'top':
          dy = bounds.minY - sb.minY;
          break;
        case 'middle':
          dy = bounds.cy - sb.cy;
          break;
        case 'bottom':
          dy = bounds.maxY - sb.maxY;
          break;
      }
      return this.#translateStroke(s, dx, dy);
    });

    shapes.recordStrokeHistory(before, after, `Align ${edge}`);
  }

  /* ---------------- Flip Selected ---------------- */
  flipSelected(axis: FlipAxis): void {
    const bounds = this.selectionBounds;
    if (!bounds || this.selectedIds.length === 0) return;
    const cx = bounds.cx;
    const cy = bounds.cy;

    const before = ink.completedStrokes.map(cloneStroke);
    const after = ink.completedStrokes.map((s) => {
      if (!this.selectedIds.includes(s.id)) return cloneStroke(s);
      const copy = cloneStroke(s);
      copy.points = copy.points.map((pt) => ({
        ...pt,
        x: axis === 'horizontal' ? 2 * cx - pt.x : pt.x,
        y: axis === 'vertical' ? 2 * cy - pt.y : pt.y,
      }));
      if (copy.bbox) {
        const [x0, y0, x1, y1] = copy.bbox;
        const nx0 = axis === 'horizontal' ? 2 * cx - x1 : x0;
        const nx1 = axis === 'horizontal' ? 2 * cx - x0 : x1;
        const ny0 = axis === 'vertical' ? 2 * cy - y1 : y0;
        const ny1 = axis === 'vertical' ? 2 * cy - y0 : y1;
        copy.bbox = [Math.min(nx0, nx1), Math.min(ny0, ny1), Math.max(nx0, nx1), Math.max(ny0, ny1)];
      }
      if (axis === 'horizontal') copy.flipH = !copy.flipH;
      if (axis === 'vertical') copy.flipV = !copy.flipV;
      return copy;
    });

    shapes.recordStrokeHistory(before, after, `Flip ${axis}`);
  }

  /* ---------------- Z-Order Reordering ---------------- */
  reorderSelected(dir: ZOrderDirection): void {
    if (this.selectedIds.length === 0) return;
    const before = ink.completedStrokes.map(cloneStroke);
    const current = ink.completedStrokes.map(cloneStroke);
    const isSel = (s: ActiveStroke) => this.selectedIds.includes(s.id);

    let after: ActiveStroke[] = current;
    if (dir === 'front') {
      const unsel = current.filter((s) => !isSel(s));
      const sel = current.filter(isSel);
      after = [...unsel, ...sel];
    } else if (dir === 'back') {
      const unsel = current.filter((s) => !isSel(s));
      const sel = current.filter(isSel);
      after = [...sel, ...unsel];
    } else if (dir === 'forward') {
      after = [...current];
      for (let i = after.length - 2; i >= 0; i--) {
        if (isSel(after[i]) && !isSel(after[i + 1])) {
          const tmp = after[i];
          after[i] = after[i + 1];
          after[i + 1] = tmp;
        }
      }
    } else if (dir === 'backward') {
      after = [...current];
      for (let i = 1; i < after.length; i++) {
        if (isSel(after[i]) && !isSel(after[i - 1])) {
          const tmp = after[i];
          after[i] = after[i - 1];
          after[i - 1] = tmp;
        }
      }
    }

    shapes.recordStrokeHistory(before, after, `Order ${dir}`);
  }

  /* ---------------- Move / Resize / Rotate / Delete / Duplicate ---------------- */
  moveSelected(dx: number, dy: number): void {
    if (this.selectedIds.length === 0 || (dx === 0 && dy === 0)) return;
    const before = ink.completedStrokes.map(cloneStroke);
    const after = ink.completedStrokes.map((s) =>
      this.selectedIds.includes(s.id) ? this.#translateStroke(s, dx, dy) : cloneStroke(s)
    );
    shapes.recordStrokeHistory(before, after, 'Move Selection');
  }

  resizeSelected(scaleX: number, scaleY: number, anchorX?: number, anchorY?: number, lockAspect = false): void {
    const bounds = this.selectionBounds;
    if (!bounds || this.selectedIds.length === 0) return;
    const ox = anchorX ?? bounds.minX;
    const oy = anchorY ?? bounds.minY;
    const sx = lockAspect ? Math.max(scaleX, scaleY) : scaleX;
    const sy = lockAspect ? Math.max(scaleX, scaleY) : scaleY;

    const before = ink.completedStrokes.map(cloneStroke);
    const after = ink.completedStrokes.map((s) => {
      if (!this.selectedIds.includes(s.id)) return cloneStroke(s);
      const copy = cloneStroke(s);
      copy.points = copy.points.map((pt) => ({
        ...pt,
        x: ox + (pt.x - ox) * sx,
        y: oy + (pt.y - oy) * sy,
      }));
      const nb = computeStrokeBBox(copy);
      copy.bbox = [nb.minX, nb.minY, nb.maxX, nb.maxY];
      return copy;
    });
    shapes.recordStrokeHistory(before, after, 'Resize Selection');
  }

  rotateSelected(deltaDeg: number, snap15 = false): void {
    const bounds = this.selectionBounds;
    if (!bounds || this.selectedIds.length === 0) return;
    const angleDeg = snap15 ? Math.round(deltaDeg / 15) * 15 : Math.round(deltaDeg);
    const rad = (angleDeg * Math.PI) / 180;
    const cos = Math.cos(rad);
    const sin = Math.sin(rad);
    const cx = bounds.cx;
    const cy = bounds.cy;

    const before = ink.completedStrokes.map(cloneStroke);
    const after = ink.completedStrokes.map((s) => {
      if (!this.selectedIds.includes(s.id)) return cloneStroke(s);
      const copy = cloneStroke(s);
      copy.points = copy.points.map((pt) => {
        const dx = pt.x - cx;
        const dy = pt.y - cy;
        return {
          ...pt,
          x: cx + dx * cos - dy * sin,
          y: cy + dx * sin + dy * cos,
        };
      });
      copy.rotationDeg = ((copy.rotationDeg ?? 0) + angleDeg) % 360;
      const nb = computeStrokeBBox(copy);
      copy.bbox = [nb.minX, nb.minY, nb.maxX, nb.maxY];
      return copy;
    });
    shapes.recordStrokeHistory(before, after, `Rotate ${angleDeg}°`);
  }

  duplicateSelected(offsetX = 16, offsetY = 16): string[] {
    if (this.selectedIds.length === 0) return [];
    const before = ink.completedStrokes.map(cloneStroke);
    const clones: ActiveStroke[] = [];
    const newIds: string[] = [];
    const newGroupMap = new Map<string, string>();

    for (const s of ink.completedStrokes) {
      if (!this.selectedIds.includes(s.id)) continue;
      const shifted = this.#translateStroke(s, offsetX, offsetY);
      const newId = `${s.id}-dup-${++this.#dupSeq}`;
      shifted.id = newId;
      if (s.groupId) {
        if (!newGroupMap.has(s.groupId)) {
          newGroupMap.set(s.groupId, `group-${Date.now()}-${++this.#groupSeq}`);
        }
        shifted.groupId = newGroupMap.get(s.groupId);
      }
      clones.push(shifted);
      newIds.push(newId);
    }

    const after = [...before, ...clones];
    shapes.recordStrokeHistory(before, after, 'Duplicate Selection');
    this.selectedIds = newIds;
    return newIds;
  }

  deleteSelected(): void {
    if (this.selectedIds.length === 0) return;
    const before = ink.completedStrokes.map(cloneStroke);
    const after = before.filter((s) => !this.selectedIds.includes(s.id));
    this.selectedIds = [];
    shapes.recordStrokeHistory(before, after, 'Delete Selection');
  }

  /* ---------------- Format Painter ---------------- */
  copyStyleFromSelected(): boolean {
    const first = this.selectedStrokes[0];
    if (!first) return false;
    this.copiedStyle = {
      color: first.color,
      widthPt: first.widthPt,
      opacity: first.opacity,
      fillColor: first.fillColor,
      fillOpacity: first.fillOpacity,
      hatchPattern: first.hatchPattern,
      hatchScale: first.hatchScale,
      lineStyle: first.lineStyle,
      startEnding: first.startEnding,
      endEnding: first.endEnding,
    };
    this.formatPainterActive = true;
    return true;
  }

  applyFormatPainterToIds(targetIds: string[]): void {
    const style = this.copiedStyle;
    if (!style || targetIds.length === 0) return;
    const before = ink.completedStrokes.map(cloneStroke);
    const after = ink.completedStrokes.map((s) => {
      if (!targetIds.includes(s.id)) return cloneStroke(s);
      return {
        ...cloneStroke(s),
        color: style.color,
        widthPt: style.widthPt,
        opacity: style.opacity,
        fillColor: style.fillColor,
        fillOpacity: style.fillOpacity,
        hatchPattern: style.hatchPattern,
        hatchScale: style.hatchScale,
        lineStyle: style.lineStyle,
        startEnding: style.startEnding,
        endEnding: style.endEnding,
      };
    });
    shapes.recordStrokeHistory(before, after, 'Apply Format Painter');
  }

  /* ---------------- Snapshot Tool ---------------- */
  createSnapshotStamp(
    box: { minX: number; minY: number; maxX: number; maxY: number },
    dataUrl: string,
    pageIndex = 0
  ): ActiveStroke {
    const preset = tools.activePreset;
    const stroke: ActiveStroke = {
      id: `snapshot-${Date.now()}-${++this.#dupSeq}`,
      tool: 'select.snapshot',
      points: [
        { x: box.minX, y: box.minY, p: 0.5 },
        { x: box.maxX, y: box.minY, p: 0.5 },
        { x: box.maxX, y: box.maxY, p: 0.5 },
        { x: box.minX, y: box.maxY, p: 0.5 },
      ],
      preset: preset as ToolPreset,
      color: theme.token('--accent'),
      widthPt: 1.5,
      opacity: 1.0,
      pageIndex,
      bbox: [box.minX, box.minY, box.maxX, box.maxY],
      snapshotDataUrl: dataUrl,
      rotationDeg: 0,
    };
    shapes.pushStrokeUndoable(stroke, 'Create Snapshot Stamp');
    this.selectedIds = [stroke.id];
    return stroke;
  }

  #translateStroke(s: ActiveStroke, dx: number, dy: number): ActiveStroke {
    const copy = cloneStroke(s);
    copy.points = copy.points.map((pt) => ({
      ...pt,
      x: pt.x + dx,
      y: pt.y + dy,
    }));
    if (copy.bbox) {
      copy.bbox = [
        copy.bbox[0] + dx,
        copy.bbox[1] + dy,
        copy.bbox[2] + dx,
        copy.bbox[3] + dy,
      ];
    }
    return copy;
  }
}

export const selection = new SelectionEngine();

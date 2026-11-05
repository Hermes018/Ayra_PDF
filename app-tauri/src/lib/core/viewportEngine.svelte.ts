// app-tauri/src/lib/core/viewportEngine.svelte.ts
export type RealUnit = 'mm' | 'cm' | 'm' | 'in' | 'ft';
export const MM_PER_UNIT: Readonly<Record<RealUnit, number>> = Object.freeze({ mm: 1, cm: 10, m: 1000, in: 25.4, ft: 304.8 });
export const MM_PER_PT = 25.4 / 72;
export type Rotation = 0 | 90 | 180 | 270;
export interface Pt { x: number; y: number }

/** Row-vector affine [a b c d e f] identical to DOMMatrix/Canvas setTransform semantics. */
export class Affine {
  constructor(readonly a = 1, readonly b = 0, readonly c = 0, readonly d = 1, readonly e = 0, readonly f = 0) {}
  multiply(m: Affine): Affine { // this · m  (apply m first, then this)
    return new Affine(
      this.a * m.a + this.c * m.b,
      this.b * m.a + this.d * m.b,
      this.a * m.c + this.c * m.d,
      this.b * m.c + this.d * m.d,
      this.a * m.e + this.c * m.f + this.e,
      this.b * m.e + this.d * m.f + this.f
    );
  }
  invert(): Affine | null {
    const det = this.a * this.d - this.b * this.c;
    if (Math.abs(det) < 1e-12) return null;
    return new Affine(
      this.d / det,
      -this.b / det,
      -this.c / det,
      this.a / det,
      (this.c * this.f - this.d * this.e) / det,
      (this.b * this.e - this.a * this.f) / det
    );
  }
  apply(p: Pt): Pt {
    return { x: this.a * p.x + this.c * p.y + this.e, y: this.b * p.x + this.d * p.y + this.f };
  }
  static translate(x: number, y: number): Affine { return new Affine(1, 0, 0, 1, x, y); }
  static scale(s: number): Affine { return new Affine(s, 0, 0, s, 0, 0); }
  static rotate(deg: Rotation): Affine {
    const [cos, sin] = deg === 0 ? [1, 0] : deg === 90 ? [0, 1] : deg === 180 ? [-1, 0] : [0, -1];
    return new Affine(cos, sin, -sin, cos, 0, 0);
  }
  toDOMMatrix(): DOMMatrix {
    if (typeof DOMMatrix !== 'undefined') {
      return new DOMMatrix([this.a, this.b, this.c, this.d, this.e, this.f]);
    }
    // Fallback for non-browser environments (e.g. Node test runner)
    return {
      a: this.a, b: this.b, c: this.c, d: this.d, e: this.e, f: this.f,
      toString: () => `matrix(${this.a}, ${this.b}, ${this.c}, ${this.d}, ${this.e}, ${this.f})`
    } as unknown as DOMMatrix;
  }
}

export interface Calibration { unit: RealUnit; /** real-unit length per 1 PDF pt */ unitsPerPt: number; precision: number }
export interface PageGeom { widthPt: number; heightPt: number; /** page origin in world (CSS px @ zoom 1) */ worldX: number; worldY: number; rotation: Rotation }
export interface ViewState { id: string; zoom: number; panX: number; panY: number; linkGroup: string | null; dpr: number }

export const ZOOM_MIN = 0.05, ZOOM_MAX = 64;
const PT_TO_CSS = 96 / 72; // 1 pt at zoom 1 = 1.333 CSS px

export class ViewportCoordinateEngine {
  views = $state<Record<string, ViewState>>({});
  pages = $state<PageGeom[]>([]);
  calibrations = $state<Record<number, Calibration>>({}); // keyed by page index
  defaultCalibration = $state<Calibration>({ unit: 'mm', unitsPerPt: MM_PER_PT, precision: 2 });

  upsertView(v: ViewState): void { this.views = { ...this.views, [v.id]: v }; }
  closeView(id: string): void { const { [id]: _drop, ...rest } = this.views; this.views = rest; }

  /** world(CSS px) → screen(CSS px) for a view. */
  worldToScreen(viewId: string): Affine {
    const v = this.views[viewId];
    if (!v) return new Affine();
    return Affine.translate(v.panX, v.panY).multiply(Affine.scale(v.zoom));
  }
  /** PDF user space (pt, origin bottom-left, y up) → screen CSS px. */
  pdfToScreen(viewId: string, page: number): Affine {
    const g = this.pages[page];
    if (!g) return new Affine();
    const flip = new Affine(PT_TO_CSS, 0, 0, -PT_TO_CSS, 0, g.heightPt * PT_TO_CSS);
    const w = g.widthPt * PT_TO_CSS, h = g.heightPt * PT_TO_CSS;
    const rot = Affine.rotate(g.rotation);
    const recenter = g.rotation === 90 ? Affine.translate(h, 0) : g.rotation === 180 ? Affine.translate(w, h)
      : g.rotation === 270 ? Affine.translate(0, w) : new Affine();
    return this.worldToScreen(viewId).multiply(Affine.translate(g.worldX, g.worldY))
      .multiply(recenter).multiply(rot).multiply(flip);
  }
  screenToPdf(viewId: string, page: number, p: Pt): Pt | null {
    const inv = this.pdfToScreen(viewId, page).invert();
    return inv ? inv.apply(p) : null;
  }
  calibrationFor(page: number): Calibration { return this.calibrations[page] ?? this.defaultCalibration; }
  ptToReal(page: number, lengthPt: number): number { return lengthPt * this.calibrationFor(page).unitsPerPt; }
  ptAreaToReal(page: number, areaPt2: number): number { const k = this.calibrationFor(page).unitsPerPt; return areaPt2 * k * k; }
  /** Two-point calibration: user states that segment (a,b) in pt measures `realLength` `unit`. */
  calibrate(page: number, a: Pt, b: Pt, realLength: number, unit: RealUnit, precision: number): boolean {
    const d = Math.hypot(b.x - a.x, b.y - a.y);
    if (!(d > 1e-6) || !(realLength > 0) || !Number.isFinite(realLength)) return false;
    this.calibrations = { ...this.calibrations, [page]: { unit, unitsPerPt: realLength / d, precision } };
    return true;
  }
  convert(value: number, from: RealUnit, to: RealUnit): number { return (value * MM_PER_UNIT[from]) / MM_PER_UNIT[to]; }

  /** Zoom about a screen anchor; propagates to linked split-views. */
  zoomAt(viewId: string, factor: number, anchor: Pt): void {
    const v = this.views[viewId];
    if (!v) return;
    const zoom = Math.min(ZOOM_MAX, Math.max(ZOOM_MIN, v.zoom * factor));
    const k = zoom / v.zoom;
    this.#apply(v, { zoom, panX: anchor.x - (anchor.x - v.panX) * k, panY: anchor.y - (anchor.y - v.panY) * k });
  }
  panBy(viewId: string, dx: number, dy: number): void {
    const v = this.views[viewId];
    if (v) this.#apply(v, { panX: v.panX + dx, panY: v.panY + dy });
  }

  /** Continuous exponential trackpad pinch or wheel zoom about screen anchor */
  pinchZoomAt(deltaY: number, deltaMode: number, screenX: number, screenY: number, viewId = 'main-canvas'): void {
    let normalizedDeltaY = deltaY;
    if (deltaMode === 1) normalizedDeltaY *= 16;
    else if (deltaMode === 2) normalizedDeltaY *= 100;
    const factor = Math.exp(-normalizedDeltaY * 0.01);
    this.zoomAt(viewId, factor, { x: screenX, y: screenY });
  }

  /** Pans the viewport by delta screen pixels (for 2-finger trackpad scroll) */
  panByDelta(dxScreen: number, dyScreen: number, viewId = 'main-canvas'): void {
    this.panBy(viewId, -dxScreen, -dyScreen);
  }

  /** Fit active view to page width */
  fitToWidth(viewportWidthPx: number, pageWidthPt = 595.28, viewId = 'main-canvas'): void {
    const v = this.views[viewId];
    if (!v) return;
    const targetW = pageWidthPt * PT_TO_CSS;
    const padding = 64;
    const availW = Math.max(100, viewportWidthPx - padding);
    const zoom = Math.min(ZOOM_MAX, Math.max(ZOOM_MIN, availW / targetW));
    const panX = (viewportWidthPx - targetW * zoom) / 2;
    this.#apply(v, { zoom, panX, panY: 40 });
  }

  /** Fit active view to full page */
  fitToPage(viewportWidthPx: number, viewportHeightPx: number, pageWidthPt = 595.28, pageHeightPt = 841.89, viewId = 'main-canvas'): void {
    const v = this.views[viewId];
    if (!v) return;
    const targetW = pageWidthPt * PT_TO_CSS;
    const targetH = pageHeightPt * PT_TO_CSS;
    const paddingX = 64;
    const paddingY = 80;
    const zoomW = Math.max(100, viewportWidthPx - paddingX) / targetW;
    const zoomH = Math.max(100, viewportHeightPx - paddingY) / targetH;
    const zoom = Math.min(ZOOM_MAX, Math.max(ZOOM_MIN, Math.min(zoomW, zoomH)));
    const panX = (viewportWidthPx - targetW * zoom) / 2;
    const panY = Math.max(20, (viewportHeightPx - targetH * zoom) / 2);
    this.#apply(v, { zoom, panX, panY });
  }

  /** Layout pages in a vertical flow with a standard pt gap. */
  layoutPages(pageSizes: ReadonlyArray<{ widthPt: number; heightPt: number; rotation?: Rotation }>, gapPt = 16): void {
    let curY = 32;
    this.pages = pageSizes.map((p) => {
      const rot = p.rotation ?? 0;
      const g: PageGeom = {
        widthPt: p.widthPt,
        heightPt: p.heightPt,
        worldX: 32,
        worldY: curY,
        rotation: rot,
      };
      curY += p.heightPt * PT_TO_CSS + gapPt * PT_TO_CSS;
      return g;
    });
  }

  #apply(src: ViewState, patch: Pick<ViewState, 'panX' | 'panY'> & Partial<Pick<ViewState, 'zoom'>>): void {
    const next = { ...this.views };
    for (const v of Object.values(next)) {
      if (v.id === src.id || (src.linkGroup !== null && v.linkGroup === src.linkGroup)) next[v.id] = { ...v, ...patch };
    }
    this.views = next;
  }
}

export const viewport = new ViewportCoordinateEngine();

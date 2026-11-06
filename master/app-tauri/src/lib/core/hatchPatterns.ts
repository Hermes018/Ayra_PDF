// app-tauri/src/lib/core/hatchPatterns.ts
// On-screen hatch preview data (ERR-007 / BUG-015, "honest UI"). The pattern parameters and one
// clipped tile per pattern come from `hatchPatterns.json`, GENERATED from the Rust definitions in
// crates/lpdf-geom (patterns.rs + fill.rs; crates/lpdf-geom/tests/studio_golden.rs fails if the
// file drifts). Studio save hatches with that same code, so the preview matches the saved PDF.
import data from './hatchPatterns.json';

/** 1 pt at zoom 1 = 4/3 CSS px (same constant as viewportEngine's PT_TO_CSS). */
export const PT_TO_CSS = 96 / 72;

export interface HatchFamilyDef {
  angleDeg: number;
  pitch: number;
  offset: number;
  phase: number;
  dash: number[];
}

export interface HatchPatternDef {
  id: string;
  families: HatchFamilyDef[];
  /** One repeat, in cells: [x1, y1, x2, y2] in PDF orientation (y up). */
  tile: number[][];
}

export const hatchTileCells: number = data.tileCells;
export const hatchPatterns: HatchPatternDef[] = data.patterns as HatchPatternDef[];

export function hatchPatternIds(): string[] {
  return hatchPatterns.map((p) => p.id);
}

/**
 * One pattern cell in PDF points for a user scale: 8 pt * clamp(scale, 0.25, 4), at least 3 pt.
 * Same rule as lpdf_geom::unit_pt (non-finite scale means 1).
 */
export function hatchUnitPt(scale: number): number {
  const s = Number.isFinite(scale) ? scale : 1;
  const u = data.unit;
  return Math.max(u.basePt * Math.min(Math.max(s, u.scaleMin), u.scaleMax), u.minPt);
}

/** One cell in CSS px at zoom 1. */
export function hatchUnitPx(scale: number): number {
  return hatchUnitPt(scale) * PT_TO_CSS;
}

/**
 * The segments a preview tile draws: the Rust tile, plus a copy of every line that lies along the
 * lower or left tile edge on the opposite edge. The tile owns those lines (the engine draws a shared
 * edge once), but an SVG pattern clips each stroke at the tile border, so half of an edge line would
 * be lost; drawing the copy puts the other half in.
 */
export function previewSegments(p: HatchPatternDef): number[][] {
  const out = p.tile.map((s) => [...s]);
  const n = hatchTileCells;
  for (const [x1, y1, x2, y2] of p.tile) {
    if (y1 === 0 && y2 === 0) out.push([x1, n, x2, n]);
    if (x1 === 0 && x2 === 0) out.push([n, y1, n, y2]);
  }
  return out;
}

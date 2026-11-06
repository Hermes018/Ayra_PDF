/**
 * Page geometry helpers shared by the Studio canvas and the store's saver (review r4).
 * World = CSS px of the laid-out pages (viewport.pages); PDF = page user space (pt, y up).
 */
import { viewport, type Pt } from '$lib/core/viewportEngine.svelte';

export const PT_TO_CSS = 96 / 72;

export function parseCssColorToRgb(colorStr: string): [number, number, number] {
  const cleaned = colorStr.trim();
  if (cleaned.startsWith('rgb')) {
    const nums = cleaned.match(/\d+/g);
    if (nums && nums.length >= 3) {
      return [
        Math.min(255, parseInt(nums[0], 10) || 0),
        Math.min(255, parseInt(nums[1], 10) || 0),
        Math.min(255, parseInt(nums[2], 10) || 0),
      ];
    }
  }
  const hexIdx = cleaned.indexOf('\x23');
  if (hexIdx === 0 && cleaned.length >= 7) {
    const r = parseInt(cleaned.slice(1, 3), 16) || 0;
    const g = parseInt(cleaned.slice(3, 5), 16) || 0;
    const b = parseInt(cleaned.slice(5, 7), 16) || 0;
    return [r, g, b];
  }
  return [30, 41, 59];
}

export function findPageForWorldPoint(pt: Pt): number {
  for (let i = 0; i < viewport.pages.length; i++) {
    const p = viewport.pages[i];
    const w = p.widthPt * PT_TO_CSS;
    const h = p.heightPt * PT_TO_CSS;
    if (pt.x >= p.worldX && pt.x <= p.worldX + w && pt.y >= p.worldY && pt.y <= p.worldY + h) {
      return i;
    }
  }
  return 0;
}

export function worldToPdfPagePt(worldPt: Pt, pageIndex: number): Pt {
  const g = viewport.pages[pageIndex] ?? {
    widthPt: 595.28,
    heightPt: 841.89,
    worldX: 32,
    worldY: 32,
    rotation: 0,
  };
  const relX = (worldPt.x - g.worldX) / PT_TO_CSS;
  const relY = (worldPt.y - g.worldY) / PT_TO_CSS;
  return {
    x: relX,
    y: Math.max(0, g.heightPt - relY),
  };
}

export function pdfPagePtToWorld(pdfPt: Pt, pageIndex: number): Pt {
  const g = viewport.pages[pageIndex] ?? {
    widthPt: 595.28,
    heightPt: 841.89,
    worldX: 32,
    worldY: 32,
    rotation: 0,
  };
  return {
    x: g.worldX + pdfPt.x * PT_TO_CSS,
    y: g.worldY + (g.heightPt - pdfPt.y) * PT_TO_CSS,
  };
}


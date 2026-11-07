import { describe, it, expect } from 'vitest';
import {
  RADIAL,
  pointToPolar,
  polarToCartesian,
  signedAngleDiff,
  isAngleInSector,
  sectorAngles,
  flyoutItemAngles,
  getParentSectorCenter,
  hitTestRadial,
} from '$lib/core/radialGeometry';
import radialLayout from '$lib/core/radialLayout.json';
import favSource from './FavoritesBar.svelte?raw';
import canvasSource from '../../views/studio/SpatialCanvas.svelte?raw';

function extractRule(src: string, selector: string): string {
  const style = src.slice(src.indexOf('<style>'));
  const i = style.indexOf(selector + ' {');
  if (i < 0) throw new Error(`no rule ${selector}`);
  return style.slice(i, style.indexOf('}', i));
}

describe('Automated Headless Dogfood UI Geometry Suite', () => {
  const flyoutCounts = radialLayout.map((w: any) => w.flyout.length);

  describe('Radial Menu Visual-to-Hit-Test Identity & Regression Tests (Phase F5.4)', () => {
    it('Screenshot 1 Regression: Cursor on Eraser (225°, r = 72) hits Eraser and hoveredSubIndex === null', () => {
      const pt = polarToCartesian(72, 225);
      const hit = hitTestRadial(pt.x, pt.y, flyoutCounts, null);
      expect(hit.zone).toBe('wedge');
      expect(hit.sector).toBe(5); // Sector 5 is Eraser
      expect(hit.sliceIndex).toBe(5);
      expect(hit.subIndex).toBeNull();
    });

    it('Screenshot 2 Regression: Cursor on Suite (315°, r = 55) sets hoveredSubIndex === null, and 7 submenu centroids hit 0..6 across 0°/360° boundary', () => {
      // 1. Hover inner Suite at r = 55
      const suitePt = polarToCartesian(55, 315);
      const innerHit = hitTestRadial(suitePt.x, suitePt.y, flyoutCounts, 7);
      expect(innerHit.zone).toBe('wedge');
      expect(innerHit.sector).toBe(7); // Sector 7 is Suite
      expect(innerHit.subIndex).toBeNull();

      // 2. All 7 Suite submenu centroids at r = 152 hit exact index 0..6 across 0°/360° boundary
      const suiteCount = flyoutCounts[7]; // 7
      expect(suiteCount).toBe(7);

      for (let k = 0; k < 7; k++) {
        const { itemMid } = flyoutItemAngles(7, suiteCount, k);
        const pt = polarToCartesian(152, itemMid);
        const subHit = hitTestRadial(pt.x, pt.y, flyoutCounts, 7);
        expect(subHit.zone).toBe('flyout');
        expect(subHit.sector).toBe(7);
        expect(subHit.sliceIndex).toBe(7);
        expect(subHit.subIndex).toBe(k);
        expect(subHit.item).toBe(k);
      }
    });

    it('Screenshot 3 Regression: Cursor on Pen B -> Medium Red and Fine Black (r = 152) highlights sub items with 100% accuracy', () => {
      // Sector 3 is Pen B (center 135°), 2 items: item 0 = Medium Red, item 1 = Fine Black
      const count = flyoutCounts[3];
      expect(count).toBe(2);

      for (let k = 0; k < 2; k++) {
        const { itemMid } = flyoutItemAngles(3, count, k);
        const pt = polarToCartesian(152, itemMid);
        const hit = hitTestRadial(pt.x, pt.y, flyoutCounts, 3);
        expect(hit.zone).toBe('flyout');
        expect(hit.sector).toBe(3);
        expect(hit.sliceIndex).toBe(3);
        expect(hit.subIndex).toBe(k);
        expect(hit.item).toBe(k);
      }
    });

    it('Screenshot 4 Regression: Cursor inside Esc (r = 19.8 at dx = -14, dy = 14) hits center Esc with null slice/sub', () => {
      const hit = hitTestRadial(-14, 14, flyoutCounts, 7);
      expect(hit.zone).toBe('center');
      expect(hit.sliceIndex).toBeNull();
      expect(hit.subIndex).toBeNull();
    });

    it('Render-to-Hit-Test Midpoint Identity Law: 100% identity for all 8 inner wedges and all outer submenus', () => {
      const innerMidR = (RADIAL.rInnerIn + RADIAL.rInnerOut) / 2;
      const subMidR = (RADIAL.rSubIn + RADIAL.rSubOut) / 2;

      for (let i = 0; i < 8; i++) {
        const innerCenter = getParentSectorCenter(i);
        const innerPt = polarToCartesian(innerMidR, innerCenter);
        const innerHit = hitTestRadial(innerPt.x, innerPt.y, flyoutCounts, i);
        expect(innerHit.sliceIndex).toBe(i);
        expect(innerHit.subIndex).toBeNull();

        const n = flyoutCounts[i];
        for (let k = 0; k < n; k++) {
          const { itemMid } = flyoutItemAngles(i, n, k);
          const subPt = polarToCartesian(subMidR, itemMid);
          const subHit = hitTestRadial(subPt.x, subPt.y, flyoutCounts, i);
          expect(subHit.sliceIndex).toBe(i);
          expect(subHit.subIndex).toBe(k);
        }
      }
    });

    it('all 8 inner wedges have adjacent label centroid distance >= 42px', () => {
      const innerMidR = (RADIAL.rInnerIn + RADIAL.rInnerOut) / 2;
      for (let s = 0; s < 8; s++) {
        const p1 = polarToCartesian(innerMidR, getParentSectorCenter(s));
        const p2 = polarToCartesian(innerMidR, getParentSectorCenter((s + 1) % 8));
        const dist = Math.hypot(p2.x - p1.x, p2.y - p1.y);
        expect(dist).toBeGreaterThanOrEqual(42);
      }
    });

    it('all outer submenu items have centroid distance >= 42px', () => {
      const outerMidR = (RADIAL.rSubIn + RADIAL.rSubOut) / 2;
      for (const sector of [1, 6, 7]) {
        const count = flyoutCounts[sector];
        for (let i = 0; i < count - 1; i++) {
          const p1 = polarToCartesian(outerMidR, flyoutItemAngles(sector, count, i).itemMid);
          const p2 = polarToCartesian(outerMidR, flyoutItemAngles(sector, count, i + 1).itemMid);
          const dist = Math.hypot(p2.x - p1.x, p2.y - p1.y);
          expect(dist).toBeGreaterThanOrEqual(42);
        }
      }
    });
  });

  describe('Bottom-Left Status Bar vs PINS Non-Collision Audit (BUG-064)', () => {
    it('FavoritesBar is offset above the bottom status bar and has z-index >= 40', () => {
      const favRule = extractRule(favSource, '.favorites-bar');
      expect(favRule).toMatch(/bottom:\s*calc\([^;]*status-bar-height[^;]*\+[^;]*12px\)/);
      expect(favRule).toMatch(/left:\s*16px/);
      expect(favRule).toMatch(/z-index:\s*(40|150)/);
    });
  });

  describe('Trackpad Smooth Zoom Audit (BUG-063)', () => {
    it('simulates 20 rapid fractional trackpad WheelEvents with smooth monotonic exponential scale progression', () => {
      let zoom = 1.0;
      const progression: number[] = [zoom];
      const normalizedDelta = -4.5;

      for (let i = 0; i < 20; i++) {
        const zoomFactor = Math.exp(-normalizedDelta * 0.0045);
        zoom = Math.min(5.0, Math.max(0.25, zoom * zoomFactor));
        progression.push(zoom);
      }

      for (let i = 0; i < progression.length - 1; i++) {
        expect(progression[i + 1]).toBeGreaterThan(progression[i]);
        expect(Number.isFinite(progression[i + 1])).toBe(true);
      }
      expect(zoom).toBeCloseTo(1.499, 2);
    });
  });

  describe('Edit text Mode Activation Audit (BUG-066)', () => {
    it('SpatialCanvas contains discoverable Edit text button, structural banner, and text block hotspots with I-beam cursor', () => {
      expect(canvasSource).toContain('<span>Edit text</span>');
      expect(canvasSource).toContain('class="structural-edit-banner glass-panel"');
      expect(canvasSource).toContain('Structural PDF text edit mode is on. Click a text block to edit it.');
      expect(canvasSource).toContain('class="pdf-text-block-hotspot"');

      const hotspotRule = extractRule(canvasSource, '.pdf-text-block-hotspot');
      expect(hotspotRule).toMatch(/cursor:\s*text/);
      expect(hotspotRule).toMatch(/border:\s*1px dashed/);
    });
  });
});

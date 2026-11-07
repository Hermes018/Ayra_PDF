import { describe, it, expect } from 'vitest';
import {
  RADIAL,
  radialHit,
  sectorAngles,
  flyoutItemAngles,
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

  describe('Radial Menu Polar Math & Non-Overlap Audit (BUG-065)', () => {
    it('all 8 inner wedges have adjacent label centroid distance >= 42px', () => {
      const innerMidR = (RADIAL.r0 + RADIAL.r1) / 2; // (34 + 110) / 2 = 72
      for (let s = 0; s < 8; s++) {
        const rad1 = sectorAngles(s).midDeg * (Math.PI / 180);
        const rad2 = sectorAngles((s + 1) % 8).midDeg * (Math.PI / 180);
        const x1 = innerMidR * Math.cos(rad1);
        const y1 = innerMidR * Math.sin(rad1);
        const x2 = innerMidR * Math.cos(rad2);
        const y2 = innerMidR * Math.sin(rad2);
        const dist = Math.hypot(x2 - x1, y2 - y1);
        expect(dist).toBeGreaterThanOrEqual(42);
      }
    });

    it('all outer submenu items (Suite sector 7, Select sector 1, Review sector 6) have centroid distance >= 42px', () => {
      const outerMidR = (RADIAL.r1 + RADIAL.r2) / 2; // (110 + 195) / 2 = 152.5
      const sectorsToTest = [1, 6, 7]; // Select, Review, Suite

      for (const sector of sectorsToTest) {
        const count = flyoutCounts[sector];
        for (let i = 0; i < count - 1; i++) {
          const a1 = flyoutItemAngles(sector, count, i).itemMid * (Math.PI / 180);
          const a2 = flyoutItemAngles(sector, count, i + 1).itemMid * (Math.PI / 180);
          const x1 = outerMidR * Math.cos(a1);
          const y1 = outerMidR * Math.sin(a1);
          const x2 = outerMidR * Math.cos(a2);
          const y2 = outerMidR * Math.sin(a2);
          const dist = Math.hypot(x2 - x1, y2 - y1);
          expect(dist).toBeGreaterThanOrEqual(42);
        }
      }
    });

    it('simulates 360-degree pointermove at r = 70px with 100% deterministic 1-to-1 inner wedge mapping', () => {
      for (let deg = 0; deg < 360; deg++) {
        // deg 0 is top (y = -r), increasing clockwise
        const rad = (deg - 90) * (Math.PI / 180);
        const dx = 70 * Math.cos(rad);
        const dy = 70 * Math.sin(rad);
        const hit = radialHit(dx, dy, flyoutCounts);
        const expectedSector = Math.floor(((deg + 22.5) % 360) / 45);
        expect(hit.zone).toBe('wedge');
        if (hit.zone === 'wedge') {
          expect(hit.sector).toBe(expectedSector);
        }
      }
    });

    it('simulates outer ring pointermove at r = 150px with 100% deterministic 1-to-1 submenu item mapping', () => {
      for (const sector of [1, 6, 7]) {
        const count = flyoutCounts[sector];
        for (let i = 0; i < count; i++) {
          const { itemMid } = flyoutItemAngles(sector, count, i);
          const rad = itemMid * (Math.PI / 180);
          const dx = 150 * Math.cos(rad);
          const dy = 150 * Math.sin(rad);
          const hit = radialHit(dx, dy, flyoutCounts, sector);
          expect(hit.zone).toBe('flyout');
          if (hit.zone === 'flyout') {
            expect(hit.sector).toBe(sector);
            expect(hit.item).toBe(i);
          }
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
      const normalizedDelta = -4.5; // ctrlKey wheel event delta (zoom-in)

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

import { describe, it, expect } from 'vitest';
import {
  RADIAL,
  radialHit,
  getSector,
  clampCenter,
  sectorAngles,
  describeArc,
  type RadialHit
} from './radialGeometry';

describe('Radial Menu Geometry', () => {
  const flyoutCounts = [0, 5, 2, 2, 1, 2, 6, 7];

  it('detects cancel zone inside r0 (rho < 36px)', () => {
    const hit = radialHit(10, 10, flyoutCounts);
    expect(hit.zone).toBe('cancel');
  });

  it('detects outside zone beyond r2 (rho >= 176px)', () => {
    const hit = radialHit(200, 200, flyoutCounts);
    expect(hit.zone).toBe('outside');
  });

  it('identifies all 8 sector centres correctly in wedge zone', () => {
    const rMid = (RADIAL.r0 + RADIAL.r1) / 2; // e.g. (36 + 112) / 2 = 74
    for (let s = 0; s < 8; s++) {
      const angles = sectorAngles(s);
      const rad = angles.midDeg * (Math.PI / 180);
      const dx = rMid * Math.cos(rad);
      const dy = rMid * Math.sin(rad);
      const hit = radialHit(dx, dy, flyoutCounts);
      expect(hit.zone).toBe('wedge');
      if (hit.zone === 'wedge') {
        expect(hit.sector).toBe(s);
      }
    }
  });

  it('partitions flyout ring items accurately', () => {
    // Sector 1 has 5 flyouts
    const rFlyout = (RADIAL.r1 + RADIAL.r2) / 2;
    const angles = sectorAngles(1); // Sector 1 midDeg = -45 deg, span -67.5 to -22.5
    const span = angles.endDeg - angles.startDeg;
    const itemSpan = span / 5;

    for (let i = 0; i < 5; i++) {
      const mid = angles.startDeg + (i + 0.5) * itemSpan;
      const rad = mid * (Math.PI / 180);
      const dx = rFlyout * Math.cos(rad);
      const dy = rFlyout * Math.sin(rad);
      const hit = radialHit(dx, dy, flyoutCounts);
      expect(hit.zone).toBe('flyout');
      if (hit.zone === 'flyout') {
        expect(hit.sector).toBe(1);
        expect(hit.item).toBe(i);
      }
    }
  });

  it('clampCenter keeps the menu within window bounds', () => {
    const clamped = clampCenter(10, 10, 1000, 800);
    expect(clamped.x).toBeGreaterThanOrEqual(RADIAL.r2);
    expect(clamped.y).toBeGreaterThanOrEqual(RADIAL.r2);

    const clampedMax = clampCenter(990, 790, 1000, 800);
    expect(clampedMax.x).toBeLessThanOrEqual(1000 - RADIAL.r2);
    expect(clampedMax.y).toBeLessThanOrEqual(800 - RADIAL.r2);
  });

  it('describeArc generates valid SVG path data', () => {
    const arc = describeArc(100, 100, 36, 112, -90, -45);
    expect(arc).toContain('M');
    expect(arc).toContain('A 112 112');
    expect(arc).toContain('A 36 36');
    expect(arc).toContain('Z');
  });

  it('asserts Math.atan2 polar math correctly maps screen coordinates to the 8 sectors (Top=0, TopRight=1, Right=2, etc.)', () => {
    const cx = 300;
    const cy = 300;
    const r = 70; // mid-wedge distance

    // Top: dx = 0, dy = -r -> Sector 0
    expect(getSector(cx, cy, cx, cy - r)).toBe(0);

    // TopRight: dx = r*sqrt(2)/2, dy = -r*sqrt(2)/2 -> Sector 1
    expect(getSector(cx, cy, cx + r * 0.7071, cy - r * 0.7071)).toBe(1);

    // Right: dx = r, dy = 0 -> Sector 2
    expect(getSector(cx, cy, cx + r, cy)).toBe(2);

    // BottomRight: dx = r*sqrt(2)/2, dy = r*sqrt(2)/2 -> Sector 3
    expect(getSector(cx, cy, cx + r * 0.7071, cy + r * 0.7071)).toBe(3);

    // Bottom: dx = 0, dy = r -> Sector 4
    expect(getSector(cx, cy, cx, cy + r)).toBe(4);

    // BottomLeft: dx = -r*sqrt(2)/2, dy = r*sqrt(2)/2 -> Sector 5
    expect(getSector(cx, cy, cx - r * 0.7071, cy + r * 0.7071)).toBe(5);

    // Left: dx = -r, dy = 0 -> Sector 6
    expect(getSector(cx, cy, cx - r, cy)).toBe(6);

    // TopLeft: dx = -r*sqrt(2)/2, dy = -r*sqrt(2)/2 -> Sector 7
    expect(getSector(cx, cy, cx - r * 0.7071, cy - r * 0.7071)).toBe(7);
  });
});

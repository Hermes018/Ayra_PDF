import { describe, it, expect } from 'vitest';
import {
  RADIAL,
  radialHit,
  getSector,
  clampCenter,
  sectorAngles,
  flyoutItemAngles,
  describeArc,
  type RadialHit
} from './radialGeometry';

describe('Radial Menu Geometry', () => {
  const flyoutCounts = [0, 5, 2, 2, 1, 2, 6, 7];

  it('detects cancel zone inside r0 (rho < 34px)', () => {
    const hit = radialHit(10, 10, flyoutCounts);
    expect(hit.zone).toBe('cancel');
  });

  it('detects outside zone beyond r2 (rho > 195px)', () => {
    const hit = radialHit(210, 210, flyoutCounts);
    expect(hit.zone).toBe('outside');
  });

  it('identifies all 8 sector centres correctly in wedge zone', () => {
    const rMid = (RADIAL.r0 + RADIAL.r1) / 2; // (34 + 110) / 2 = 72
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

  it('partitions flyout ring items accurately with fan layout', () => {
    // Sector 1 has 5 flyouts
    const rFlyout = (RADIAL.r1 + RADIAL.r2) / 2;
    for (let i = 0; i < 5; i++) {
      const { itemMid } = flyoutItemAngles(1, 5, i);
      const rad = itemMid * (Math.PI / 180);
      const dx = rFlyout * Math.cos(rad);
      const dy = rFlyout * Math.sin(rad);
      const hit = radialHit(dx, dy, flyoutCounts, 1);
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
    const arc = describeArc(0, 0, 34, 110, -90, -45);
    expect(arc).toContain('M');
    expect(arc).toContain('A 110 110');
    expect(arc).toContain('A 34 34');
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

  it('eliminates angular drift using lockedWedge in flyout ring', () => {
    const rFlyout = 150; // in [RADIAL.r1, RADIAL.r2]
    const { itemMid } = flyoutItemAngles(1, 5, 2);
    // Slight drift ±5 degrees
    const driftAngleRad = (itemMid + 5) * (Math.PI / 180);
    const dx = rFlyout * Math.cos(driftAngleRad);
    const dy = rFlyout * Math.sin(driftAngleRad);

    const hitLocked = radialHit(dx, dy, flyoutCounts, 1);
    expect(hitLocked.zone).toBe('flyout');
    if (hitLocked.zone === 'flyout') {
      expect(hitLocked.sector).toBe(1);
    }
  });

  it('respects backdrop dismissal zone beyond outer radius', () => {
    expect(radialHit(210, 0, flyoutCounts).zone).toBe('outside');
    expect(radialHit(0, -250, flyoutCounts).zone).toBe('outside');
  });

  it('respects cancel hub zone inside r0 radius', () => {
    expect(radialHit(0, 0, flyoutCounts).zone).toBe('cancel');
    expect(radialHit(15, -15, flyoutCounts).zone).toBe('cancel');
    expect(radialHit(33, 0, flyoutCounts).zone).toBe('cancel');
  });
});

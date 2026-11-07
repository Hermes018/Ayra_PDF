import { describe, it, expect } from 'vitest';
import {
  RADIAL,
  radialHit,
  hitTestRadial,
  getSector,
  clampCenter,
  sectorAngles,
  flyoutItemAngles,
  describeArc,
  pointToPolar,
  polarToCartesian,
  signedAngleDiff,
  isAngleInSector,
  type RadialHit
} from './radialGeometry';

describe('Radial Menu Geometry', () => {
  const flyoutCounts = [0, 5, 2, 2, 1, 2, 6, 7];

  it('detects cancel / center zone inside rEsc (r < 36px)', () => {
    const hit = hitTestRadial(10, 10, flyoutCounts);
    expect(hit.zone).toBe('center');
  });

  it('detects outside zone beyond r2 (r > 205px)', () => {
    const hit = hitTestRadial(210, 210, flyoutCounts);
    expect(hit.zone).toBe('outside');
  });

  it('identifies all 8 sector centres correctly in wedge zone', () => {
    const rMid = (RADIAL.rInnerIn + RADIAL.rInnerOut) / 2; // 73px
    for (let s = 0; s < 8; s++) {
      const angles = sectorAngles(s);
      const pt = polarToCartesian(rMid, angles.midDeg);
      const hit = hitTestRadial(pt.x, pt.y, flyoutCounts);
      expect(hit.zone).toBe('wedge');
      if (hit.zone === 'wedge') {
        expect(hit.sector).toBe(s);
        expect(hit.sliceIndex).toBe(s);
      }
    }
  });

  it('partitions flyout ring items accurately with fan layout', () => {
    const rFlyout = (RADIAL.rSubIn + RADIAL.rSubOut) / 2;
    for (let i = 0; i < 5; i++) {
      const { itemMid } = flyoutItemAngles(1, 5, i);
      const pt = polarToCartesian(rFlyout, itemMid);
      const hit = hitTestRadial(pt.x, pt.y, flyoutCounts, 1);
      expect(hit.zone).toBe('flyout');
      if (hit.zone === 'flyout') {
        expect(hit.sector).toBe(1);
        expect(hit.item).toBe(i);
        expect(hit.subIndex).toBe(i);
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
    const arc = describeArc(0, 0, 36, 110, 315, 45);
    expect(arc).toContain('M');
    expect(arc).toContain('A 110 110');
    expect(arc).toContain('A 36 36');
    expect(arc).toContain('Z');
  });

  it('pointToPolar maps 0° at top, 90° at right, 180° at bottom, 270° at left', () => {
    expect(pointToPolar(0, -10).angleDeg).toBe(0);
    expect(pointToPolar(10, 0).angleDeg).toBe(90);
    expect(pointToPolar(0, 10).angleDeg).toBe(180);
    expect(pointToPolar(-10, 0).angleDeg).toBe(270);
  });

  it('signedAngleDiff computes minimal wrap-safe difference in [-180, +180)', () => {
    expect(signedAngleDiff(10, 350)).toBe(20);
    expect(signedAngleDiff(350, 10)).toBe(-20);
    expect(signedAngleDiff(0, 180)).toBe(-180);
  });

  it('eliminates angular drift using activeSliceIndex in flyout ring', () => {
    const rFlyout = 150;
    const { itemMid } = flyoutItemAngles(1, 5, 2);
    const pt = polarToCartesian(rFlyout, itemMid + 5);

    const hitLocked = hitTestRadial(pt.x, pt.y, flyoutCounts, 1);
    expect(hitLocked.zone).toBe('flyout');
    if (hitLocked.zone === 'flyout') {
      expect(hitLocked.sector).toBe(1);
    }
  });

  it('respects backdrop dismissal zone beyond outer radius', () => {
    expect(hitTestRadial(210, 0, flyoutCounts).zone).toBe('outside');
    expect(hitTestRadial(0, -250, flyoutCounts).zone).toBe('outside');
  });

  it('respects cancel hub zone inside rEsc radius', () => {
    expect(hitTestRadial(0, 0, flyoutCounts).zone).toBe('center');
    expect(hitTestRadial(15, -15, flyoutCounts).zone).toBe('center');
    expect(hitTestRadial(33, 0, flyoutCounts).zone).toBe('center');
  });
});

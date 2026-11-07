// app-tauri/src/lib/core/radialGeometry.ts
// Industry-standard signed-angle radial geometry & hit-testing engine

export const RADIAL = Object.freeze({
  r0: 36,        // Center cancel / Esc button radius
  rEsc: 36,      // Synonym for r0
  r1: 110,       // Inner ring outer radius
  rInnerIn: 36,  // Inner ring inner radius
  rInnerOut: 110,// Inner ring outer radius
  r2: 195,       // Outer submenu outer radius
  rSubIn: 112,   // Outer submenu inner radius
  rSubOut: 195,  // Outer submenu outer radius
  sectors: 8,    // 8 primary sectors
  itemSpanDeg: 28, // Each outer submenu wedge is 28° wide
});

export type RadialHit =
  | { zone: 'cancel'; sector: null; sliceIndex: null; item: null; subIndex: null }
  | { zone: 'center'; sector: null; sliceIndex: null; item: null; subIndex: null }
  | { zone: 'wedge'; sector: number; sliceIndex: number; item: null; subIndex: null }
  | { zone: 'flyout'; sector: number; sliceIndex: number; item: number; subIndex: number }
  | { zone: 'outside'; sector: null; sliceIndex: null; item: null; subIndex: null };

/**
 * Clockwise polar coordinate conversion where:
 * 0° = Top (0, -1)
 * 90° = Right (+1, 0)
 * 180° = Bottom (0, +1)
 * 270° = Left (-1, 0)
 */
export function pointToPolar(dx: number, dy: number): { r: number; angleDeg: number } {
  const r = Math.hypot(dx, dy);
  const angleDeg = ((Math.atan2(dx, -dy) * 180) / Math.PI + 360) % 360;
  return { r, angleDeg };
}

/**
 * Cartesian coordinates for a polar coordinate (r, deg)
 */
export function polarToCartesian(r: number, deg: number): { x: number; y: number } {
  const rad = (deg * Math.PI) / 180;
  return { x: r * Math.sin(rad), y: -r * Math.cos(rad) };
}

/**
 * Wrap-safe signed angular distance in [-180, +180)
 */
export function signedAngleDiff(targetDeg: number, centerDeg: number): number {
  return ((targetDeg - centerDeg + 540) % 360) - 180;
}

/**
 * Returns whether target angle falls within a sector centered at sectorCenterDeg with span sectorSpanDeg
 */
export function isAngleInSector(angleDeg: number, sectorCenterDeg: number, sectorSpanDeg: number): boolean {
  return Math.abs(signedAngleDiff(angleDeg, sectorCenterDeg)) <= sectorSpanDeg / 2 + 1e-6;
}

/**
 * Center angle of parent sector i (0 = Top / Pan, 1 = TopRight, 2 = Right / Pen A, ...)
 */
export function getParentSectorCenter(sliceIndex: number): number {
  return (sliceIndex * (360 / RADIAL.sectors)) % 360;
}

/**
 * Sector angles for primary wedge i in clockwise degrees
 */
export function sectorAngles(sector: number): { startDeg: number; endDeg: number; midDeg: number } {
  const midDeg = getParentSectorCenter(sector);
  const halfSpan = 360 / RADIAL.sectors / 2; // 22.5
  return {
    startDeg: (midDeg - halfSpan + 360) % 360,
    endDeg: (midDeg + halfSpan + 360) % 360,
    midDeg,
  };
}

/**
 * Submenu item k angles for parent sector i with count items
 */
export function flyoutItemAngles(
  sector: number,
  count: number,
  idx: number
): {
  itemStart: number;
  itemEnd: number;
  itemMid: number;
  itemSpan: number;
} {
  const parentCenterDeg = getParentSectorCenter(sector);
  const itemSpan = RADIAL.itemSpanDeg;
  const totalSpan = itemSpan * count;
  const itemMid = (parentCenterDeg - totalSpan / 2 + (idx + 0.5) * itemSpan + 360) % 360;
  const itemStart = (itemMid - itemSpan / 2 + 360) % 360;
  const itemEnd = (itemMid + itemSpan / 2 + 360) % 360;
  return { itemStart, itemEnd, itemMid, itemSpan };
}

/**
 * Maps screen coordinates relative to a center point to one of the 8 radial sectors.
 */
export function getSector(
  centerX: number,
  centerY: number,
  clientX: number,
  clientY: number
): number {
  const dx = clientX - centerX;
  const dy = clientY - centerY;
  const { angleDeg } = pointToPolar(dx, dy);
  const sliceAngle = 360 / RADIAL.sectors;
  return Math.floor(((angleDeg + sliceAngle / 2) % 360) / sliceAngle);
}

/**
 * Deterministic mathematical polar hit-testing with zero origin drift & wrap safety.
 */
export function hitTestRadial(
  dx: number,
  dy: number,
  flyoutCounts: readonly number[],
  activeSliceIndex: number | null = null
): RadialHit {
  const { r, angleDeg } = pointToPolar(dx, dy);

  if (r < RADIAL.rEsc) {
    return { zone: 'center', sector: null, sliceIndex: null, item: null, subIndex: null };
  }
  if (r > 205) {
    return { zone: 'outside', sector: null, sliceIndex: null, item: null, subIndex: null };
  }

  // Inner ring: 36 <= r <= 110
  if (r <= RADIAL.rInnerOut) {
    const sliceAngle = 360 / RADIAL.sectors; // 45°
    const sliceIndex = Math.floor(((angleDeg + sliceAngle / 2) % 360) / sliceAngle);
    return { zone: 'wedge', sector: sliceIndex, sliceIndex, item: null, subIndex: null };
  }

  // Outer submenu ring: 110 < r <= 205
  // If we have an active parent with flyout items, check its fan first
  if (activeSliceIndex !== null) {
    const count = flyoutCounts[activeSliceIndex] ?? 0;
    if (count > 0) {
      for (let k = 0; k < count; k++) {
        const { itemMid, itemSpan } = flyoutItemAngles(activeSliceIndex, count, k);
        if (isAngleInSector(angleDeg, itemMid, itemSpan)) {
          return { zone: 'flyout', sector: activeSliceIndex, sliceIndex: activeSliceIndex, item: k, subIndex: k };
        }
      }
    }
  }

  // Fallback to freeSliceIndex
  const sliceAngle = 360 / RADIAL.sectors;
  const freeSliceIndex = Math.floor(((angleDeg + sliceAngle / 2) % 360) / sliceAngle);
  const freeCount = flyoutCounts[freeSliceIndex] ?? 0;
  if (freeCount > 0) {
    for (let k = 0; k < freeCount; k++) {
      const { itemMid, itemSpan } = flyoutItemAngles(freeSliceIndex, freeCount, k);
      if (isAngleInSector(angleDeg, itemMid, itemSpan)) {
        return { zone: 'flyout', sector: freeSliceIndex, sliceIndex: freeSliceIndex, item: k, subIndex: k };
      }
    }
  }

  return { zone: 'wedge', sector: freeSliceIndex, sliceIndex: freeSliceIndex, item: null, subIndex: null };
}

/**
 * Backward compatibility alias for radialHit
 */
export function radialHit(
  dx: number,
  dy: number,
  flyoutCounts: readonly number[],
  lockedWedge: number | null = null
): RadialHit {
  return hitTestRadial(dx, dy, flyoutCounts, lockedWedge);
}

export function clampCenter(x: number, y: number, w: number, h: number): { x: number; y: number } {
  const r = RADIAL.r2;
  return { x: Math.min(w - r, Math.max(r, x)), y: Math.min(h - r, Math.max(r, y)) };
}

/**
 * Clockwise SVG arc generator for polar sectors
 */
export function describeArc(
  cx: number,
  cy: number,
  rInner: number,
  rOuter: number,
  startDeg: number,
  endDeg: number
): string {
  const p1 = polarToCartesian(rOuter, startDeg);
  const p2 = polarToCartesian(rOuter, endDeg);
  const p3 = polarToCartesian(rInner, endDeg);
  const p4 = polarToCartesian(rInner, startDeg);

  let span = signedAngleDiff(endDeg, startDeg);
  if (span <= 0) span += 360;
  const largeArc = span > 180 ? 1 : 0;

  return [
    `M ${(cx + p1.x).toFixed(2)} ${(cy + p1.y).toFixed(2)}`,
    `A ${rOuter} ${rOuter} 0 ${largeArc} 1 ${(cx + p2.x).toFixed(2)} ${(cy + p2.y).toFixed(2)}`,
    `L ${(cx + p3.x).toFixed(2)} ${(cy + p3.y).toFixed(2)}`,
    `A ${rInner} ${rInner} 0 ${largeArc} 0 ${(cx + p4.x).toFixed(2)} ${(cy + p4.y).toFixed(2)}`,
    'Z',
  ].join(' ');
}

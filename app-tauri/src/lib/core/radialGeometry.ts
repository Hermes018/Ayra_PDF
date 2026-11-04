// app-tauri/src/lib/core/radialGeometry.ts
export const RADIAL = Object.freeze({ r0: 36, r1: 112, r2: 176, sectors: 8 });

export type RadialHit =
  | { zone: 'cancel' }
  | { zone: 'wedge'; sector: number }
  | { zone: 'flyout'; sector: number; item: number }
  | { zone: 'outside' };

/**
 * Maps screen coordinates relative to a center point to one of the 8 radial sectors.
 * Top is 0, TopRight is 1, Right is 2, BottomRight is 3,
 * Bottom is 4, BottomLeft is 5, Left is 6, TopLeft is 7.
 */
export function getSector(
  centerX: number,
  centerY: number,
  clientX: number,
  clientY: number
): number {
  const dx = clientX - centerX;
  const dy = clientY - centerY;
  const distance = Math.sqrt(dx * dx + dy * dy);
  let angle = Math.atan2(dy, dx) + Math.PI / 2; // Shift so Top is 0
  if (angle < 0) angle += 2 * Math.PI;
  return Math.floor((angle + Math.PI / 8) / (Math.PI / 4)) % RADIAL.sectors;
}

export function radialHit(dx: number, dy: number, flyoutCounts: readonly number[]): RadialHit {
  const distance = Math.sqrt(dx * dx + dy * dy);
  if (distance < RADIAL.r0) return { zone: 'cancel' };
  if (distance >= RADIAL.r2) return { zone: 'outside' };
  let angle = Math.atan2(dy, dx) + Math.PI / 2; // Shift so Top is 0
  if (angle < 0) angle += 2 * Math.PI;
  const sector = Math.floor((angle + Math.PI / 8) / (Math.PI / 4)) % RADIAL.sectors;
  if (distance < RADIAL.r1) return { zone: 'wedge', sector };
  const n = flyoutCounts[sector] ?? 0;
  if (n === 0) return { zone: 'wedge', sector };
  const norm = (angle + Math.PI / 8) % (2 * Math.PI);
  const within = norm - sector * (Math.PI / 4);
  return {
    zone: 'flyout',
    sector,
    item: Math.min(n - 1, Math.max(0, Math.floor((within / (Math.PI / 4)) * n)))
  };
}

export function clampCenter(x: number, y: number, w: number, h: number): { x: number; y: number } {
  const r = RADIAL.r2;
  return { x: Math.min(w - r, Math.max(r, x)), y: Math.min(h - r, Math.max(r, y)) };
}

export function sectorAngles(sector: number): { startDeg: number; endDeg: number; midDeg: number } {
  // Sector 0 is centered at -90 deg (Top)
  const midDeg = -90 + sector * 45;
  return {
    startDeg: midDeg - 22.5,
    endDeg: midDeg + 22.5,
    midDeg,
  };
}

export function describeArc(
  cx: number,
  cy: number,
  rInner: number,
  rOuter: number,
  startDeg: number,
  endDeg: number
): string {
  const toRad = Math.PI / 180;
  const a1 = startDeg * toRad;
  const a2 = endDeg * toRad;

  const p1x = cx + rOuter * Math.cos(a1);
  const p1y = cy + rOuter * Math.sin(a1);
  const p2x = cx + rOuter * Math.cos(a2);
  const p2y = cy + rOuter * Math.sin(a2);

  const p3x = cx + rInner * Math.cos(a2);
  const p3y = cy + rInner * Math.sin(a2);
  const p4x = cx + rInner * Math.cos(a1);
  const p4y = cy + rInner * Math.sin(a1);

  const largeArc = endDeg - startDeg > 180 ? 1 : 0;

  return [
    `M ${p1x.toFixed(2)} ${p1y.toFixed(2)}`,
    `A ${rOuter} ${rOuter} 0 ${largeArc} 1 ${p2x.toFixed(2)} ${p2y.toFixed(2)}`,
    `L ${p3x.toFixed(2)} ${p3y.toFixed(2)}`,
    `A ${rInner} ${rInner} 0 ${largeArc} 0 ${p4x.toFixed(2)} ${p4y.toFixed(2)}`,
    'Z',
  ].join(' ');
}

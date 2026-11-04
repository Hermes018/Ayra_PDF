// app-tauri/src/lib/core/radialGeometry.ts
export const RADIAL = Object.freeze({ r0: 36, r1: 112, r2: 176, sectors: 8 });

export type RadialHit =
  | { zone: 'cancel' }
  | { zone: 'wedge'; sector: number }
  | { zone: 'flyout'; sector: number; item: number }
  | { zone: 'outside' };

export function radialHit(dx: number, dy: number, flyoutCounts: readonly number[]): RadialHit {
  const rho = Math.hypot(dx, dy);
  if (rho < RADIAL.r0) return { zone: 'cancel' };
  if (rho >= RADIAL.r2) return { zone: 'outside' };
  const deg = (Math.atan2(dy, dx) * 180) / Math.PI; // screen y-down: -90 = up
  const norm = (((deg + 90 + 22.5) % 360) + 360) % 360;
  const sector = Math.floor(norm / 45) % RADIAL.sectors;
  if (rho < RADIAL.r1) return { zone: 'wedge', sector };
  const n = flyoutCounts[sector] ?? 0;
  if (n === 0) return { zone: 'wedge', sector };
  const within = norm - sector * 45; // 0..45 across the wedge arc
  return { zone: 'flyout', sector, item: Math.min(n - 1, Math.floor((within / 45) * n)) };
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

// Pure placement for tooltips: centred under the anchor, clamped to the viewport, flipped above when there is no room below.
export interface Box { left: number; top: number; right: number; bottom: number; width: number; height: number }
export interface Placement { left: number; top: number; side: 'below' | 'above' }

export function placeTooltip(
  anchor: Box,
  tip: { width: number; height: number },
  viewport: { width: number; height: number },
  gap: number,
): Placement {
  const maxLeft = Math.max(gap, viewport.width - tip.width - gap);
  const left = Math.min(Math.max(anchor.left + anchor.width / 2 - tip.width / 2, gap), maxLeft);
  const belowTop = anchor.bottom + gap;
  const fitsBelow = belowTop + tip.height <= viewport.height - gap;
  if (fitsBelow) return { left, top: belowTop, side: 'below' };
  const aboveTop = anchor.top - gap - tip.height;
  return { left, top: Math.max(gap, aboveTop), side: 'above' };
}

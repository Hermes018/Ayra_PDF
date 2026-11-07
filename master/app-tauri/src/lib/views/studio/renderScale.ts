// app-tauri/src/lib/views/studio/renderScale.ts
// Industry-standard progressive HiDPI zoom re-rasterization scale calculation
// Protected by 8GB RAM / WebKitGTK texture budget caps.

export const MIN_BASE_DPR = 1.5; // Crisp baseline even on 1x displays
export const MAX_RENDER_SCALE = 4.0; // Up to ~300-400 DPI equivalent
export const MAX_CANVAS_DIM = 4096; // 8GB RAM WebKitGTK hardware texture safety cap

export function computeTargetRenderScale(
  zoom: number,
  devicePixelRatio: number = 1,
  basePageWidthPt: number = 612,
  basePageHeightPt: number = 792
): number {
  const dpr = Math.max(devicePixelRatio || 1, MIN_BASE_DPR);
  const desiredScale = Math.min(MAX_RENDER_SCALE, Math.max(1.0, zoom * dpr));
  const maxDimPt = Math.max(basePageWidthPt, basePageHeightPt, 1);
  const dimCappedScale = MAX_CANVAS_DIM / maxDimPt;
  // Quantize to 0.25 steps to avoid thrashing on tiny 0.01 zoom changes
  const raw = Math.min(desiredScale, dimCappedScale);
  const quantized = Math.round(raw * 4) / 4;
  if (quantized * maxDimPt > MAX_CANVAS_DIM) {
    return Math.max(0.25, Math.floor(raw * 4) / 4);
  }
  return quantized;
}

/**
 * The Studio's ONE saver (review r4): owned by the document store, not by canvas panes.
 *
 * A save snapshots (path, seq, markups) first and re-checks after every await. If the document
 * changed (tab switch, reload, restore still running) it refuses with a clear message and writes
 * nothing: it can never write the wrong document's strokes.
 */
import { invoke } from '@tauri-apps/api/core';
import { appState } from '$lib/state.svelte';
import { ink } from '$lib/core/inkEngine.svelte';
import { inserts } from '$lib/core/insertEngine.svelte';
import { measureEngine } from '$lib/core/measureEngine.svelte';
import { computeStrokeBBox } from '$lib/core/shapeAndHatchEngine.svelte';
import { freeOutputPath } from './outputNames';
import { findPageForWorldPoint, parseCssColorToRgb, worldToPdfPagePt } from './pageGeometry';
import {
  currentToken,
  documentSnapshot,
  isCurrent,
  isDocumentRestored,
  liveDocumentState,
  liveGeometryOk,
  markDocumentSavedAs,
  setStoreSaver,
} from './documentSessions';

export type SaveResult =
  | { ok: true; outputPath: string; savedCount: number; flattened: boolean; message: string }
  | { ok: false; message: string };

/** Accurate refusal for each reason the live document is not ready (review r5). */
function notReadyMessage(): string {
  switch (liveDocumentState()) {
    case 'locked':
      return 'Nothing to save: the document is password-protected and was not opened. Close its tab and open it again to enter the password.';
    case 'failed':
      return 'Nothing to save: the document could not be opened. Nothing was written.';
    case 'hostile':
      return 'Nothing to save: the file looks damaged or hostile and was not opened. Nothing was written.';
    default:
      return 'Save refused: the document is still loading its markups. Nothing was written; save again in a moment.';
  }
}
const NO_GEOMETRY =
  'Save refused: the page layout of this document is missing or incomplete, so markups cannot be placed correctly. Nothing was written; reopen the document.';
const IN_FLIGHT = 'This document is already being saved. Wait for that save to finish.';
const savingNow = new Set<string>();
const CHANGED =
  'Save refused: the document changed while the save was being prepared (tab switch or reload). Nothing was written; save again.';

/** Save the live document's markups to a NEW file; the saved copy opens as its own tab. */
export async function saveLiveDocument(flatten: boolean, expectedPath?: string): Promise<SaveResult> {
  const token = currentToken();
  if (!token) return { ok: false, message: 'Open a PDF file first to save or flatten annotations.' };
  if (expectedPath !== undefined && token.path !== expectedPath) return { ok: false, message: CHANGED };
  if (!isDocumentRestored(token.path)) return { ok: false, message: notReadyMessage() };
  if (!liveGeometryOk()) return { ok: false, message: NO_GEOMETRY };
  const inputPath = token.path;
  if (savingNow.has(inputPath)) return { ok: false, message: IN_FLIGHT };
  savingNow.add(inputPath);
  try {
    // BUG-021: a new, collision-safe file every time (never an earlier save or an existing file).
    const outputPath = await freeOutputPath(inputPath, flatten ? 'flattened' : 'annotated');
    if (!isCurrent(token) || !isDocumentRestored(inputPath)) return { ok: false, message: CHANGED };
    if (!liveGeometryOk()) return { ok: false, message: NO_GEOMETRY };
    // Everything below up to invoke is synchronous: the snapshot is exactly what is written.
    const savedSnapshot = documentSnapshot(inputPath);
    const dtos = ink.completedStrokes.map((s) => {
      const b = computeStrokeBBox(s);
      const pageIndex = s.pageIndex ?? findPageForWorldPoint({ x: b.cx, y: b.cy });
      const pdfPts = s.points.map((pt) => {
        const conv = worldToPdfPagePt({ x: pt.x, y: pt.y }, pageIndex);
        return { x: conv.x, y: conv.y, p: pt.p, t: pt.t ?? null };
      });
      const minPt = worldToPdfPagePt({ x: b.minX, y: b.maxY }, pageIndex);
      const maxPt = worldToPdfPagePt({ x: b.maxX, y: b.minY }, pageIndex);
      const rgb = parseCssColorToRgb(s.color);
      return {
        id: s.id,
        kind: s.tool,
        pageIndex,
        points: pdfPts,
        bboxPt: [
          Math.min(minPt.x, maxPt.x),
          Math.min(minPt.y, maxPt.y),
          Math.max(minPt.x, maxPt.x),
          Math.max(minPt.y, maxPt.y),
        ],
        colorRgb: rgb,
        fillRgb: s.fillColor ? parseCssColorToRgb(s.fillColor) : null,
        widthPt: s.widthPt,
        opacity: s.opacity,
        lineStyle: s.lineStyle ?? 'solid',
        startEnding: s.startEnding ?? 'none',
        endEnding: s.endEnding ?? 'none',
        hatchPattern: s.hatchPattern ?? 'none',
        hatchScale: s.hatchScale ?? 1.0,
        text: s.text ?? null,
        fontSize: s.fontSize ?? null,
        groupId: s.groupId ?? null,
        rotationDeg: s.rotationDeg ?? 0,
        snapshotDataUrl: s.snapshotDataUrl ?? null,
        author: s.author ?? inserts.activeAuthor,
        measurementKind: null,
        calibratedValue: null,
        unitLabel: null,
        perimeterValue: null,
        volumeValue: null,
        pageScale: measureEngine.getScaleForPage(pageIndex)
          ? {
              pageIndex,
              ptDistance: measureEngine.getScaleForPage(pageIndex).ptDistance,
              realDistance: measureEngine.getScaleForPage(pageIndex).realDistance,
              unit: measureEngine.getScaleForPage(pageIndex).unit,
              precision: measureEngine.getScaleForPage(pageIndex).precision,
              defaultDepth: measureEngine.getScaleForPage(pageIndex).depthValue,
            }
          : null,
      };
    });

    const measureDtos = measureEngine.measurements.map((m) => {
      const pdfPts = m.points.map((pt) => {
        const conv = worldToPdfPagePt({ x: pt.x, y: pt.y }, m.pageIndex);
        return { x: conv.x, y: conv.y, p: 0.5, t: null };
      });
      // BUG-022: save the perimeter as the closed outline that was measured.
      if (m.kind === 'perimeter' && pdfPts.length >= 3) pdfPts.push({ ...pdfPts[0] });
      const xs = pdfPts.map((p) => p.x);
      const ys = pdfPts.map((p) => p.y);
      const minX = xs.length ? Math.min(...xs) : 0;
      const minY = ys.length ? Math.min(...ys) : 0;
      const maxX = xs.length ? Math.max(...xs) : 10;
      const maxY = ys.length ? Math.max(...ys) : 10;
      const sc = measureEngine.getScaleForPage(m.pageIndex);
      return {
        id: m.id,
        kind: `measure.${m.kind}`,
        pageIndex: m.pageIndex,
        points: pdfPts,
        bboxPt: [minX, minY, maxX, maxY],
        colorRgb: parseCssColorToRgb(m.strokeColor),
        fillRgb: null,
        widthPt: m.strokeWidthPt,
        opacity: 1.0,
        lineStyle: 'solid',
        startEnding: m.endpointStyle === 'open-arrow' ? 'open-arrow' : 'slash',
        endEnding: m.endpointStyle === 'open-arrow' ? 'open-arrow' : 'slash',
        hatchPattern:
          m.kind === 'area.rect' || m.kind === 'area.polygon' ? 'diagonal-steel' : 'none',
        hatchScale: 1.0,
        text: m.formattedLabel,
        fontSize: 10,
        groupId: null,
        rotationDeg: 0,
        snapshotDataUrl: null,
        author: inserts.activeAuthor,
        measurementKind: m.kind,
        calibratedValue: m.calibratedValue,
        unitLabel: m.unitLabel,
        perimeterValue: m.perimeterCalibrated ?? null,
        volumeValue: m.volumeCalibrated ?? null,
        pageScale: {
          pageIndex: sc.pageIndex,
          ptDistance: sc.ptDistance,
          realDistance: sc.realDistance,
          unit: sc.unit,
          precision: sc.precision,
          defaultDepth: sc.depthValue,
        },
      };
    });


    const res = await invoke<{
      outputPath: string;
      savedCount: number;
      flattened: boolean;
      pageCount: number;
    }>('studio_save_annotations', {
      inputPath,
      outputPath,
      annotations: [...dtos, ...measureDtos],
      flatten,
    });
    // Mark clean exactly what was written (strokes drawn meanwhile stay unsaved).
    if (savedSnapshot !== null) markDocumentSavedAs(inputPath, savedSnapshot);
    // The saved copy opens as its own document tab; the original stays open, unchanged.
    appState.addFiles([res.outputPath]);
    return {
      ok: true,
      outputPath: res.outputPath,
      savedCount: res.savedCount,
      flattened: flatten,
      message: flatten
        ? `Flattened ${res.savedCount} annotations to ${res.outputPath}`
        : `Saved ${res.savedCount} editable annotations to ${res.outputPath}`,
    };
  } catch (err) {
    return { ok: false, message: `Save failed: ${String(err)}` };
  } finally {
    savingNow.delete(inputPath);
  }
}

/** Last save outcome message, for the tab bar's close dialog. */
export let lastSaveMessage = '';

setStoreSaver(async (path) => {
  const r = await saveLiveDocument(false, path);
  lastSaveMessage = r.message;
  return r.ok;
});

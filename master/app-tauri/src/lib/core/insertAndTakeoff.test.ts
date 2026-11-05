import { describe, it, expect, beforeEach } from 'vitest';
import {
  inserts,
  svgPathToPdfOps,
  compileLatexToVector,
  applyGrayscaleThresholdToRgba,
  parsePageRangeSpec,
  buildOfflineWavBytes,
  recognizeHandwritingStrokesToText,
} from './insertEngine.svelte';
import { ink } from './inkEngine.svelte';
import { measureEngine } from './measureEngine.svelte';
import { selection } from './selectionEngine.svelte';

describe('Phase D6 — Insert Suite, LaTeX, Signatures, Audio Notes, Markup Library & Quantity Takeoff', () => {
  beforeEach(() => {
    ink.completedStrokes = [];
    selection.selectedIds = [];
    measureEngine.measurements = [];
    inserts.pageOverlay = 'none';
    inserts.ledgerOpen = false;
  });

  it('translates SVG path d commands (M, L, C, Q, Z) to PDF path operators (m, l, c, h)', () => {
    const ops = svgPathToPdfOps('M 10 20 L 110 20 Q 120 40 110 60 Z');
    expect(ops).toContain('10.00 20.00 m');
    expect(ops).toContain('110.00 20.00 l');
    expect(ops).toContain(' c');
    expect(ops).toContain('h');
  });

  it('compiles offline LaTeX equations into formatted symbols, vector SVG path & PDF ops', () => {
    const eq = compileLatexToVector('\\frac{-b \\pm \\sqrt{b^2 - 4ac}}{2a}');
    expect(eq.formattedDisplay).toContain('±');
    expect(eq.formattedDisplay).toContain('√(b² - 4ac)');
    expect(eq.svgPathD).toContain('M 4 4');
    expect(eq.pdfOpsStream).toContain('m');
    expect(eq.widthPt).toBeGreaterThanOrEqual(108);
    expect(eq.heightPt).toBe(54);

    const stroke = inserts.stampLatexEquation('\\sum_{i=1}^{n} x_i', 120, 160, 0);
    expect(stroke.tool).toBe('insert.latex');
    expect(stroke.text).toContain('∑');
    expect(ink.completedStrokes.length).toBe(1);
  });

  it('applies grayscale threshold background removal for webcam/image signatures', () => {
    // 2 pixels: pixel 0 is bright white paper (250,250,250,255), pixel 1 is dark ink (20,25,30,255)
    const input = new Uint8ClampedArray([250, 250, 250, 255, 20, 25, 30, 255]);
    const out = applyGrayscaleThresholdToRgba(input, 168);
    // White background becomes transparent (alpha = 0)
    expect(out[3]).toBe(0);
    // Dark ink remains opaque (alpha >= 180)
    expect(out[7]).toBeGreaterThanOrEqual(180);
  });

  it('stamps Draw, Type, and Webcam signatures and Callout / Note / Stamp markups', () => {
    const sig = inserts.stampSignature({
      mode: 'type',
      typedName: 'J. Doe, PE',
      xPt: 100,
      yPt: 200,
      pageIndex: 0,
    });
    expect(sig.tool).toBe('insert.signature');
    expect(sig.text).toContain('J. Doe, PE');

    const callout = inserts.stampQuickInsert('callout.multi', 150, 250, 0, 'Check Beam B4');
    expect(callout.tool).toBe('callout.multi');
    expect(callout.startEnding).toBe('closed-arrow');
    expect(callout.points.length).toBe(5);
    expect(ink.completedStrokes.length).toBe(2);
  });

  it('builds valid RIFF WAVE PCM bytes and attaches offline audio notes', async () => {
    const wav = buildOfflineWavBytes(0.5);
    expect(wav.byteLength).toBeGreaterThan(44);
    expect(String.fromCharCode(wav[0], wav[1], wav[2], wav[3])).toBe('RIFF');

    await inserts.startAudioRecording();
    const note = await inserts.stopAndAttachAudioNote(null, 220, 300, 0);
    expect(note.audioBytes.length).toBeGreaterThan(44);
    expect(ink.completedStrokes.some((s) => s.tool === 'insert.audio')).toBe(true);
  });

  it('parses page range specifications (All, Even, Odd, 1-5) and pastes markups across pages', () => {
    expect(parsePageRangeSpec('All', 4)).toEqual([0, 1, 2, 3]);
    expect(parsePageRangeSpec('Even', 5)).toEqual([1, 3]);
    expect(parsePageRangeSpec('Odd', 5)).toEqual([0, 2, 4]);
    expect(parsePageRangeSpec('2-4, 6', 6)).toEqual([1, 2, 3, 5]);

    const baseStroke = inserts.stampQuickInsert('insert.stamp', 80, 90, 0, 'APPROVED');
    selection.selectedIds = [baseStroke.id];

    const clones = inserts.pasteStrokesToPages('2-4', 4);
    expect(clones.length).toBe(3);
    expect(clones.map((c) => c.pageIndex)).toEqual([1, 2, 3]);
  });

  it('exports Quantity Takeoff CSV records and cycles non-destructive Page Background overlays', async () => {
    expect(inserts.cyclePageOverlay()).toBe('engineering-grid');
    expect(inserts.cyclePageOverlay()).toBe('isometric-grid');
    expect(inserts.cyclePageOverlay()).toBe('ruled-paper');
    expect(inserts.cyclePageOverlay()).toBe('none');

    measureEngine.startOrAddPoint('measure.length', 0, { x: 100, y: 100 });
    measureEngine.commitTwoPointDrag({ x: 244, y: 100 });

    const csvRows = await inserts.exportTakeoffCsv('/tmp/test-takeoff.csv');
    expect(csvRows.length).toBe(1);
    expect(csvRows[0].raw_pt).toBeCloseTo(144, 1);
  });

  it('verifies Phase D6.1 final 5 offline features (7.10, 8.4, 8.7, 8.10, 8.12)', async () => {
    // 1. 8.4 Shared annotation visibility / author filtering
    inserts.activeAuthor = 'Architect A';
    const s1 = inserts.stampQuickInsert('insert.note', 100, 100, 0, 'Note by A');
    inserts.activeAuthor = 'Engineer B';
    const s2 = inserts.stampQuickInsert('insert.note', 140, 140, 0, 'Note by B');
    expect(inserts.getAvailableAuthors()).toContain('Architect A');
    expect(inserts.getAvailableAuthors()).toContain('Engineer B');
    inserts.authorFilter = 'Architect A';
    expect(inserts.isStrokeVisibleForAuthorFilter(s1)).toBe(true);
    expect(inserts.isStrokeVisibleForAuthorFilter(s2)).toBe(false);
    inserts.authorFilter = 'ALL';

    // 2. 8.10 Handwriting recognition (ink-to-text)
    ink.startStroke(50, 50, 0.7, 'pen.a');
    ink.addPoint(50, 95, 0.7);
    const drawn = ink.endStroke();
    expect(drawn).not.toBeNull();
    selection.selectedIds = [drawn!.id];
    const recognized = recognizeHandwritingStrokesToText([drawn!]);
    expect(recognized).toContain('Ink Note:');
    const converted = inserts.convertSelectedInkToText([drawn!]);
    expect(converted?.tool).toBe('insert.textBox');
    expect(converted?.text).toContain('Ink Note:');

    // 3. 7.10 Workspace document tags, folders & favorites
    const docs = await inserts.upsertWorkspaceDocument({
      path: '/tmp/structural-plan.pdf',
      title: 'structural-plan.pdf',
      folder: 'Structural Set',
      tags: ['foundation', 'rev-3'],
      favorite: true,
      last_opened_ms: 1700000100000,
    });
    expect(docs.some((d) => d.title === 'structural-plan.pdf' && d.favorite)).toBe(true);

    // 4. 8.7 Watch-folder auto-ingest
    const scanned = await inserts.scanWatchFolder('/tmp/watch-ingest');
    expect(Array.isArray(scanned)).toBe(true);

    // 5. 8.12 Auto table & form field detection
    const regions = await inserts.detectTablesAndFields(null, 0);
    expect(regions.length).toBeGreaterThanOrEqual(2);
    expect(regions.some((r) => r.kind === 'table')).toBe(true);
    expect(regions.some((r) => r.kind === 'form-field')).toBe(true);
  });
});

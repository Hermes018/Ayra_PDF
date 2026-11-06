<script lang="ts">
  import { onMount, onDestroy, tick, untrack } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { appState } from '$lib/state.svelte';
  import { IpcClient, pickFilesDialog } from '$lib/ipc';
  import { viewport, type Pt } from '$lib/core/viewportEngine.svelte';
  import { tools, type ToolPreset, type ToolId } from '$lib/core/toolEngine.svelte';
  import { commands } from '$lib/core/commandEngine.svelte';
  import { theme } from '$lib/core/themeContract.svelte';
  import { ink, type ActiveStroke } from '$lib/core/inkEngine.svelte';
  import {
    shapes,
    computeStrokeBBox,
    getStrokeDashArray,
    generateRevisionCloudPath,
    generatePolylineSvgPath,
    generateSquiggleSvgPath,
    generateLineEndingPath,
    isShapeOrMarkupTool,
    isMultiClickTool,
  } from '$lib/core/shapeAndHatchEngine.svelte';
  import { selection } from '$lib/core/selectionEngine.svelte';
  import { snapEngine } from '$lib/core/snapEngine.svelte';
  import {
    measureEngine,
    isMeasureTool,
    isMultiPointMeasureTool,
    projectPointToRulerIfNear,
    type PageScaleConfig,
    type CadUnit,
  } from '$lib/core/measureEngine.svelte';
  import defaultPresets from '$lib/core/defaultPresets.json';
  import DockableToolbar from '$lib/components/studio/DockableToolbar.svelte';
  import PropertyPill from '$lib/components/studio/PropertyPill.svelte';
  import Dialog from '$lib/components/ui/Dialog.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import Input from '$lib/components/ui/Input.svelte';
  import RadialMenu from '$lib/components/studio/RadialMenu.svelte';
  import ShortcutSettings from '$lib/components/studio/ShortcutSettings.svelte';
  import FavoritesBar from '$lib/components/studio/FavoritesBar.svelte';
  import StudioIcon from '$lib/components/studio/StudioIcon.svelte';
  import HatchDefs from '$lib/components/studio/HatchDefs.svelte';
  import MeasureOverlay from '$lib/components/studio/MeasureOverlay.svelte';
  import CalibrationModal from '$lib/components/studio/CalibrationModal.svelte';
  import AnnotationLedger from '$lib/components/studio/AnnotationLedger.svelte';
  import { inserts, type PageOverlayTemplate } from '$lib/core/insertEngine.svelte';
  import { freeOutputPath } from './outputNames';
  import { findPageForWorldPoint, parseCssColorToRgb, pdfPagePtToWorld, worldToPdfPagePt } from './pageGeometry';
  import { saveLiveDocument } from './studioSave';
  import {
    activate,
    FOLLOWER_WAIT,
    areScalesLoaded,
    completeRestore,
    markScalesLoaded,
    currentToken,
    whenDocumentLive,
    deactivate,
    documentSession,
    failLoad,
    isCurrent,
    isDocumentRestored,
    shouldProbeDocument,
    type LoadToken,
  } from './documentSessions';
  import { enforceLabsGate, labsVisible, shortcutsForSave, studioToolLabel } from '$lib/components/studio/labs.svelte';

  // `follower`: a second pane (Labs split view). It never activates or loads documents; it waits
  // for the store's live document and only paints it.
  let { viewId = 'main-canvas', follower = false } = $props<{ viewId?: string; follower?: boolean }>();

  let canvasContainer: HTMLDivElement | null = $state(null);
  let isPanning = $state(false);
  let panStart: Pt = { x: 0, y: 0 };
  let editMode = $state<'markup' | 'structural'>('markup');
  let statusNotice = $state<string | null>(null);

  // Selection dragging / resizing / rotating state
  let selectionDragMode = $state<'none' | 'move' | 'resize' | 'rotate' | 'marquee' | 'lasso' | 'snapshot'>('none');
  let selectionDragStart: Pt = { x: 0, y: 0 };

  // Radial Menu & Shortcuts modal state
  let radialVisible = $state(false);
  let radialPos = $state<Pt>({ x: 300, y: 300 });
  let shortcutsVisible = $state(false);

  // PDF rendering state
  let pageCanvases: Record<number, HTMLCanvasElement | null> = $state({});
  let currentPdfDoc: any = null;
  let loadedFilePath = $state<string | null>(null);

  // Extracted PDF text blocks for discoverable structural text editing
  export interface PdfTextBlockDto {
    id: string;
    pageIndex: number;
    bboxPt: [number, number, number, number];
    text: string;
    fontSize: number;
  }

  let pageTextBlocks = $state<PdfTextBlockDto[]>([]);
  let hoveredBlockId = $state<string | null>(null);

  async function loadPageTextBlocks() {
    const path = loadedFilePath ?? appState.currentFile;
    if (!path || editMode !== 'structural') {
      pageTextBlocks = [];
      return;
    }
    try {
      const blocks = await invoke<PdfTextBlockDto[]>('studio_extract_text_blocks', {
        inputPath: path,
        pageIndex: Math.max(0, activePageNumber - 1),
      });
      pageTextBlocks = blocks ?? [];
    } catch (e) {
      console.warn('Failed to extract PDF text blocks for structural edit:', e);
      pageTextBlocks = [];
    }
  }

  $effect(() => {
    if (editMode === 'structural') {
      void loadPageTextBlocks();
    } else {
      pageTextBlocks = [];
    }
  });

  /** BUG-013: the text block under a world point (smallest wins), using the same box as its hotspot. */
  function hitTestTextBlock(worldPt: Pt): PdfTextBlockDto | null {
    let best: PdfTextBlockDto | null = null;
    let bestArea = Infinity;
    for (const block of pageTextBlocks) {
      if (!viewport.pages[block.pageIndex]) continue;
      const topLeft = pdfPagePtToWorld({ x: block.bboxPt[0], y: block.bboxPt[1] + block.bboxPt[3] }, block.pageIndex);
      const w = Math.max(block.bboxPt[2] * PT_TO_CSS, 20);
      const h = Math.max(block.bboxPt[3] * PT_TO_CSS, 12);
      if (worldPt.x >= topLeft.x && worldPt.x <= topLeft.x + w && worldPt.y >= topLeft.y && worldPt.y <= topLeft.y + h) {
        if (w * h < bestArea) {
          best = block;
          bestArea = w * h;
        }
      }
    }
    return best;
  }

  function handleEditTextBlock(block: PdfTextBlockDto) {
    const pageIdx = block.pageIndex;
    const g = viewport.pages[pageIdx];
    if (!g) return;
    const pdfX = block.bboxPt[0];
    const pdfYTop = block.bboxPt[1] + block.bboxPt[3];
    const worldPt = pdfPagePtToWorld({ x: pdfX, y: pdfYTop }, pageIdx);
    const w = block.bboxPt[2] * PT_TO_CSS;
    const h = block.bboxPt[3] * PT_TO_CSS;

    shapes.openTextBoxEditor(
      worldPt.x,
      worldPt.y,
      pageIdx,
      true,
      block.text,
      block.fontSize,
      Math.max(w, 120),
      Math.max(h, 36)
    );
  }

  function isInsertMediaTool(tid: string): boolean {
    if (!labsVisible(tid)) return false;
    return (
      tid === 'insert.latex' ||
      tid === 'insert.signature' ||
      tid === 'insert.audio' ||
      tid === 'insert.stamp' ||
      tid === 'insert.image'
    );
  }

  function handleConfigureInsertTool(tid: string) {
    if (!labsVisible(tid)) return;
    const pageIdx = Math.max(0, activePageNumber - 1);
    const pg = viewport.pages[pageIdx] ?? { worldX: 32, worldY: 32, widthPt: 595.28, heightPt: 841.89 };
    const centerX = pg.worldX + (pg.widthPt * PT_TO_CSS) / 2;
    const centerY = pg.worldY + (pg.heightPt * PT_TO_CSS) / 2;
    if (tid === 'insert.latex' || tid === 'insert.signature' || tid === 'insert.audio') {
      inserts.openInsertModalForTool(tid, centerX, centerY, pageIdx);
    } else {
      inserts.stampQuickInsert(tid as ToolId, centerX, centerY, pageIdx);
    }
  }

  const PT_TO_CSS = 96 / 72;

  let activeTouchPoints = new Map<number, Pt>();
  let touchInitialDistance = 0;
  let touchInitialMidpoint: Pt = { x: 0, y: 0 };
  let touchInitialZoom = 1.0;
  let wheelBehavior = $state<'auto' | 'zoom' | 'pan'>('auto');

  /** True only when a real PDF is loaded and laid out; drives the empty state and
   *  disables document-only controls (no phantom "Page 1 / 2" on an empty canvas). */
  let hasDocument = $derived(viewport.pages.length > 0 && !!(loadedFilePath ?? appState.currentFile));
  const NEEDS_DOC = 'Open a PDF first';

  async function openPdfFromEmptyState() {
    try {
      const picked = await pickFilesDialog(false, ['pdf']);
      if (picked.length > 0) appState.addFiles(picked);
    } catch (e) {
      statusNotice = `Could not open the file picker: ${e instanceof Error ? e.message : String(e)}`;
    }
  }

  let activePageNumber = $derived.by(() => {
    const v = viewport.views[viewId] ?? currentView;
    const w = canvasContainer ? canvasContainer.clientWidth : 800;
    const h = canvasContainer ? canvasContainer.clientHeight : 600;
    const centerX = -v.panX / v.zoom + w / (2 * v.zoom);
    const centerY = -v.panY / v.zoom + h / (2 * v.zoom);
    return Math.max(1, findPageForWorldPoint({ x: centerX, y: centerY }) + 1);
  });

  function toggleRadialAt(x?: number, y?: number) {
    if (radialVisible) {
      radialVisible = false;
    } else {
      const posX = x ?? (typeof window !== 'undefined' ? window.innerWidth / 2 : 300);
      const posY = y ?? (typeof window !== 'undefined' ? window.innerHeight / 2 : 300);
      radialPos = { x: posX, y: posY };
      radialVisible = true;
    }
  }

  function openRadialAt(x: number, y: number) {
    radialPos = { x, y };
    radialVisible = true;
  }

  function handleZoomSelect(e: Event) {
    const val = (e.target as HTMLSelectElement).value;
    const w = canvasContainer ? canvasContainer.clientWidth : 800;
    const h = canvasContainer ? canvasContainer.clientHeight : 600;
    if (val === 'fit-width' || val === 'fit-page') {
      // BUG-020: fit the active page's real size and world offset, not an assumed A4 page at 0,0.
      const pageIdx = Math.min(Math.max(0, activePageNumber - 1), Math.max(0, viewport.pages.length - 1));
      viewport.fitPageInView(viewId, w, h, pageIdx, val === 'fit-width' ? 'width' : 'page');
    } else {
      const pct = parseFloat(val);
      if (Number.isFinite(pct) && pct > 0) {
        viewport.upsertView({ ...currentView, zoom: pct / 100 });
      }
    }
  }

  function handleRotatePage90() {
    const activeIdx = activePageNumber - 1;
    const page = viewport.pages[activeIdx];
    if (page) {
      const nextRot = ((page.rotation + 90) % 360) as 0 | 90 | 180 | 270;
      viewport.pages = viewport.pages.map((p, idx) => idx === activeIdx ? { ...p, rotation: nextRot } : p);
      statusNotice = `Page ${activePageNumber} rotated to ${nextRot}°`;
    }
  }

  let currentView = $derived(
    viewport.views[viewId] ?? {
      id: viewId,
      zoom: 1.0,
      panX: 0,
      panY: 0,
      linkGroup: null,
      dpr: typeof window !== 'undefined' ? window.devicePixelRatio || 1 : 1,
    }
  );

  let worldTransform = $derived(viewport.worldToScreen(viewId).toDOMMatrix().toString());
  let hasRedactions = $derived(
    ink.completedStrokes.some((s) => s.tool === 'review.redact')
  );

  /** Reads the document's saved markups (world coords of the current layout). Never touches the engines. */
  async function loadPersistedAnnotations(path: string): Promise<ActiveStroke[]> {
    try {
      const loaded = await invoke<
        Array<{
          id: string;
          kind: string;
          pageIndex: number;
          points: Array<{ x: number; y: number; p: number; t?: number }>;
          bboxPt: [number, number, number, number];
          colorRgb: [number, number, number];
          widthPt: number;
          opacity: number;
          lineStyle?: string;
          startEnding?: string;
          endEnding?: string;
          hatchPattern?: string;
          hatchScale?: number;
          text?: string;
          fontSize?: number;
          groupId?: string;
          rotationDeg?: number;
          snapshotDataUrl?: string;
        }>
      >('studio_load_annotations', { inputPath: path });
      if (Array.isArray(loaded) && loaded.length > 0) {
        const restored: ActiveStroke[] = loaded.map((dto) => {
          const pIdx = dto.pageIndex ?? 0;
          const worldPts = dto.points.map((pt) => {
            const w = pdfPagePtToWorld({ x: pt.x, y: pt.y }, pIdx);
            return { x: w.x, y: w.y, p: pt.p ?? 0.5, t: pt.t };
          });
          const p0 = pdfPagePtToWorld({ x: dto.bboxPt[0], y: dto.bboxPt[1] }, pIdx);
          const p1 = pdfPagePtToWorld({ x: dto.bboxPt[2], y: dto.bboxPt[3] }, pIdx);
          const preset = (tools.presets[dto.kind] ?? tools.activePreset) as ToolPreset;
          return {
            id: dto.id,
            tool: (dto.kind as ActiveStroke['tool']) || 'pen.a',
            points: worldPts,
            preset,
            color: theme.token('--glass-text'),
            widthPt: dto.widthPt ?? 1.5,
            opacity: dto.opacity ?? 1.0,
            pageIndex: pIdx,
            bbox: [
              Math.min(p0.x, p1.x),
              Math.min(p0.y, p1.y),
              Math.max(p0.x, p1.x),
              Math.max(p0.y, p1.y),
            ],
            lineStyle: (dto.lineStyle as ActiveStroke['lineStyle']) ?? 'solid',
            startEnding: (dto.startEnding as ActiveStroke['startEnding']) ?? 'none',
            endEnding: (dto.endEnding as ActiveStroke['endEnding']) ?? 'none',
            hatchPattern: (dto.hatchPattern as ActiveStroke['hatchPattern']) ?? 'none',
            hatchScale: dto.hatchScale ?? 1.0,
            text: dto.text,
            fontSize: dto.fontSize,
            groupId: dto.groupId,
            rotationDeg: dto.rotationDeg ?? 0,
            snapshotDataUrl: dto.snapshotDataUrl,
          };
        });
        return restored;
      }
    } catch {
      // Safe fallback when no sidecar exists or in unit tests
    }
    return [];
  }

  /** Returns true when a new file was written (the store's saver does the work). */
  async function saveAnnotationsToPdf(flatten: boolean): Promise<boolean> {
    const r = await saveLiveDocument(flatten);
    statusNotice = r.message;
    return r.ok;
  }

  async function commitStructuralTextOrAnnotation() {
    const result = shapes.commitTextBox();
    if (!result.structuralEdit) return;

    const inputPath = loadedFilePath ?? appState.currentFile;
    if (!inputPath) {
      statusNotice = 'Open a PDF file first to commit structural text edits.';
      return;
    }
    const edit = result.structuralEdit;
    const p0 = worldToPdfPagePt({ x: edit.bboxPt[0], y: edit.bboxPt[3] }, edit.pageIndex);
    const p1 = worldToPdfPagePt({ x: edit.bboxPt[2], y: edit.bboxPt[1] }, edit.pageIndex);
    const bboxPt: [number, number, number, number] = [
      Math.min(p0.x, p1.x),
      Math.min(p0.y, p1.y),
      Math.max(p0.x, p1.x),
      Math.max(p0.y, p1.y),
    ];

    try {
      const outputPath = await freeOutputPath(inputPath, 'edited');
      const res = await invoke<{ outputPath: string }>('studio_edit_pdf_text_block', {
        inputPath,
        outputPath,
        pageIndex: edit.pageIndex,
        bboxPt,
        newText: edit.newText,
        fontSize: edit.fontSize,
        colorRgb: parseCssColorToRgb(edit.color),
      });
      statusNotice = `Structural text saved to ${res.outputPath}`;
      appState.addFiles([res.outputPath]);
    } catch (err) {
      statusNotice = `Structural text edit failed: ${String(err)}`;
    }
  }

  function captureSnapshotDataUrl(box: { minX: number; minY: number; maxX: number; maxY: number }): string {
    const pageIndex = findPageForWorldPoint({
      x: (box.minX + box.maxX) / 2,
      y: (box.minY + box.maxY) / 2,
    });
    const sourceCanvas = pageCanvases[pageIndex];
    const pageGeom = viewport.pages[pageIndex];
    if (sourceCanvas && pageGeom && typeof document !== 'undefined') {
      try {
        const cropW = Math.max(16, Math.round(box.maxX - box.minX));
        const cropH = Math.max(16, Math.round(box.maxY - box.minY));
        const off = document.createElement('canvas');
        off.width = cropW;
        off.height = cropH;
        const ctx = off.getContext('2d');
        if (ctx) {
          const dpr = window.devicePixelRatio || 1;
          const sx = Math.max(0, (box.minX - pageGeom.worldX) * dpr);
          const sy = Math.max(0, (box.minY - pageGeom.worldY) * dpr);
          ctx.drawImage(
            sourceCanvas,
            sx,
            sy,
            cropW * dpr,
            cropH * dpr,
            0,
            0,
            cropW,
            cropH
          );
          return off.toDataURL('image/png');
        }
      } catch {
        // Fallback below if canvas taint or headless test environment
      }
    }
    return 'data:image/svg+xml;utf8,<svg xmlns="http://www.w3.org/2000/svg" width="120" height="80"><rect width="120" height="80" fill="none" stroke="currentColor" stroke-dasharray="4 2"/></svg>';
  }

  // BUG-011: encrypted PDFs. pdf.js rejects with PasswordException (code 1 = password needed,
  // 2 = incorrect). We ask, retry with the password, and never store it (DO-15: cleared after use).
  let passwordPrompt = $state<{ fileName: string; incorrect: boolean; resolve: (pw: string | null) => void } | null>(null);
  let passwordValue = $state('');

  function isPasswordError(err: unknown): err is { code?: number } {
    return typeof err === 'object' && err !== null && (err as { name?: string }).name === 'PasswordException';
  }

  function askPdfPassword(path: string, incorrect: boolean): Promise<string | null> {
    return new Promise((resolve) => {
      passwordValue = '';
      passwordPrompt = { fileName: path.split(/[\\/]/).pop() || path, incorrect, resolve };
    });
  }

  function finishPasswordPrompt(submit: boolean) {
    const prompt = passwordPrompt;
    if (!prompt) return;
    const pw = submit ? passwordValue : null;
    passwordValue = '';
    passwordPrompt = null;
    prompt.resolve(pw);
  }

  /** Set by openPdfDocument: the last opened document needed a password. */
  let lastOpenWasEncrypted = false;

  /** Opens the document, prompting for a password as often as needed. `null` = user cancelled. */
  async function openPdfDocument(pdfjs: any, bytes: number[], path: string): Promise<any | null> {
    lastOpenWasEncrypted = false;
    let password: string | undefined;
    for (;;) {
      try {
        // pdf.js transfers (detaches) the buffer, so every attempt gets a fresh copy.
        const src: { data: Uint8Array; password?: string } = { data: new Uint8Array(bytes) };
        if (password !== undefined) src.password = password;
        return await pdfjs.getDocument(src).promise;
      } catch (err) {
        if (!isPasswordError(err)) throw err;
        lastOpenWasEncrypted = true;
        const next = await askPdfPassword(path, err.code === 2);
        if (next === null) {
          statusNotice = `"${path.split(/[\\/]/).pop() || path}" is password-protected and was not opened. Close its tab and open it again to enter the password.`;
          return null;
        }
        password = next;
      }
    }
  }

  function baseName(path: string): string {
    return path.split(/[\\/]/).pop() || path;
  }

  /**
   * BUG-018: the viewer (pdf.js) can show something for a damaged file while the engine can only
   * open it in fallback (degraded) mode or not at all. Ask the engine and say so.
   */
  async function reportDocumentHealth(path: string, token: LoadToken) {
    let res: Awaited<ReturnType<typeof IpcClient.probeDocument>>;
    try {
      res = await IpcClient.probeDocument(path);
    } catch {
      return; // backend unreachable (browser preview / unit tests without a mock)
    }
    if (!isCurrent(token)) return;
    if (!res.ok && res.errorCode === 'F4_RESOURCE_GUARD') {
      // Engines contract: a resource guard tripped. The file is not opened (no markups, no pages).
      failLoad(token, 'hostile');
      viewport.pages = [];
      if (currentPdfDoc) {
        await currentPdfDoc.destroy();
        currentPdfDoc = null;
      }
      statusNotice = `"${baseName(path)}" looks damaged or hostile and was not opened (${res.error}).`;
      return;
    }
    if (!res.ok) {
      statusNotice = `"${baseName(path)}" is damaged: ${res.error}. What you see may be incomplete, and saving or editing may fail.`;
    } else if (res.degraded) {
      statusNotice = `"${baseName(path)}": Opened in repair mode (degraded). The file is damaged; some content may be missing.`;
    }
  }

  /**
   * Load (or, in 'render' mode, only repaint) `path` for the store token `token`. Every await is
   * followed by an isCurrent check: a stale continuation never touches engines or the layout.
   * 'load': layout → read saved markups → store.completeRestore (drawing allowed) → paint → snap/scales.
   * 'render': the engines already hold the restored document (e.g. after Studio → Organize → Studio).
   */
  async function loadPdfIntoCanvas(path: string, token: LoadToken, mode: 'load' | 'render') {
    if (!path || typeof window === 'undefined') return;
    const live = () => isCurrent(token);
    try {
      const pdfjs = (window as any)['pdfjs-dist/build/pdf'] || (window as any).pdfjsLib;
      if (!pdfjs) {
        documentBusy = false;
        failLoad(token, 'failed');
        return;
      }
      pdfjs.GlobalWorkerOptions.workerSrc = '/pdf.worker.min.js';

      const bytes: number[] = await invoke('read_file_bytes', { path });
      if (!live()) return;

      if (currentPdfDoc) {
        await currentPdfDoc.destroy();
        currentPdfDoc = null;
      }
      if (!live()) return;
      const pdf = await openPdfDocument(pdfjs, bytes, path);
      if (!live()) {
        await pdf?.destroy?.();
        return;
      }
      if (!pdf) {
        viewport.pages = []; // never leave the previous document on screen under this tab
        loadedFilePath = path;
        documentBusy = false;
        failLoad(token, 'locked'); // password prompt cancelled
        return;
      }
      const encrypted = lastOpenWasEncrypted;
      currentPdfDoc = pdf;
      loadedFilePath = path;

      if (mode === 'load') {
        const pageSizes: Array<{ widthPt: number; heightPt: number }> = [];
        for (let i = 1; i <= pdf.numPages; i++) {
          const page = await pdf.getPage(i);
          const vp = page.getViewport({ scale: 1.0 });
          pageSizes.push({ widthPt: vp.width, heightPt: vp.height });
        }
        if (!live()) return;
        viewport.layoutPages(pageSizes);
        seedPageBorderSnap(); // replaced by the extracted geometry once it loads
        await tick();
        if (!live()) return;

        // Restore markups BEFORE drawing is allowed and before the slow page renders. The store
        // merges the session (or the saved markups) with anything added while loading.
        const fromDisk = documentSession(path) ? null : await loadPersistedAnnotations(path);
        if (!completeRestore(token, fromDisk, pdf.numPages)) return;
        documentBusy = false;

        if (pdf.numPages === 0) {
          statusNotice = `"${baseName(path)}" has no pages to show.`;
        }
        // Probe once per document per run, silently, and never for password-protected files.
        if (!encrypted && shouldProbeDocument(path)) void reportDocumentHealth(path, token);
      } else {
        // Engines already hold this restored document; only make sure the page layout matches
        // (deterministic from page sizes, never touches markups).
        if (viewport.pages.length !== pdf.numPages) {
          const sizes: Array<{ widthPt: number; heightPt: number }> = [];
          for (let i = 1; i <= pdf.numPages; i++) {
            const vp = (await pdf.getPage(i)).getViewport({ scale: 1.0 });
            sizes.push({ widthPt: vp.width, heightPt: vp.height });
          }
          if (!live()) return;
          viewport.layoutPages(sizes);
        }
        await tick();
        if (!live()) return;
      }

      const baseScale = (96 / 72) * (window.devicePixelRatio || 1);
      for (let i = 1; i <= pdf.numPages; i++) {
        const page = await pdf.getPage(i);
        if (!live()) return;
        const canvasEl = pageCanvases[i - 1];
        if (!canvasEl) continue;
        const vp = page.getViewport({ scale: baseScale });
        canvasEl.width = vp.width;
        canvasEl.height = vp.height;
        const ctx = canvasEl.getContext('2d');
        if (ctx) {
          await page.render({ canvasContext: ctx, viewport: vp }).promise;
        }
        if (!live()) return;
      }

      if (mode === 'load') {
        await loadSnapGeometryAndScales(path, pdf.numPages, live, !areScalesLoaded(path));
        if (live()) markScalesLoaded(token);
        if (!live()) return;
      }
      if (editMode === 'structural') {
        await loadPageTextBlocks();
      }
    } catch (err) {
      // A newer load cancels in-flight renders: expected, logged at debug level only.
      if ((err as { name?: string } | null)?.name === 'RenderingCancelledException') {
        console.debug('SpatialCanvas: render cancelled (RenderingCancelledException) for', path);
        return;
      }
      console.error('Failed to render PDF in SpatialCanvas:', err);
      if (!live() || unmounted) return; // the user already switched to another document
      const msg = err instanceof Error ? err.message : String(err);
      if (isDocumentRestored(path)) {
        // Painting failed after the markups were restored: the layout belongs to this document,
        // keep it (Save stays correct) and say what failed.
        statusNotice = `Some pages of "${baseName(path)}" could not be drawn (${msg}). Your markups are safe.`;
        return;
      }
      documentBusy = false;
      failLoad(token, 'failed');
      statusNotice = `Could not open "${baseName(path)}": the file is damaged or not a PDF (${msg}).`;
      viewport.pages = []; // never leave the previous document on screen under this tab
      loadedFilePath = path;
    }
  }

  /** Scales (only when there is no session: a session's scales win) and snap geometry, with a
   *  liveness check after every await so a slow reply never lands on another document. */
  async function loadSnapGeometryAndScales(
    path: string,
    pageCount: number,
    live: () => boolean = () => true,
    loadScales = true
  ) {
    if (loadScales) try {
      const scales = await invoke<
        Array<{
          pageIndex: number;
          ptDistance: number;
          realDistance: number;
          unit: string;
          precision: number;
          defaultDepth?: number;
        }>
      >('studio_load_page_scales', { inputPath: path });
      if (!live()) return;
      if (Array.isArray(scales) && scales.length > 0) {
        const nextScales: Record<number, PageScaleConfig> = { ...measureEngine.scales };
        for (const sc of scales) {
          nextScales[sc.pageIndex] = {
            pageIndex: sc.pageIndex,
            ptDistance: sc.ptDistance || 72,
            realDistance: sc.realDistance || 25.4,
            unit: (sc.unit as CadUnit) || 'mm',
            precision: (Math.min(3, Math.max(0, sc.precision ?? 2)) as 0 | 1 | 2 | 3),
            depthValue: sc.defaultDepth ?? 0,
          };
        }
        measureEngine.scales = nextScales;
        measureEngine.recalculateAllMeasurements();
      }
    } catch {
      // Safe fallback when no scales sidecar exists
    }

    const limit = Math.min(pageCount, 12);
    for (let p = 0; p < limit; p++) {
      try {
        const geom = await invoke<{
          pageIndex: number;
          widthPt: number;
          heightPt: number;
          segments: Array<{ x1: number; y1: number; x2: number; y2: number }>;
        }>('studio_extract_snap_geometry', { inputPath: path, pageIndex: p });
        if (!live()) return;
        if (geom && Array.isArray(geom.segments)) {
          const worldSegs = geom.segments.map((s) => {
            const p1 = pdfPagePtToWorld({ x: s.x1, y: s.y1 }, p);
            const p2 = pdfPagePtToWorld({ x: s.x2, y: s.y2 }, p);
            return { x1: p1.x, y1: p1.y, x2: p2.x, y2: p2.y };
          });
          snapEngine.setPageSegments(p, worldSegs);
        }
      } catch {
        // Safe fallback in test environment
      }
    }
  }

  async function savePageScalesToSidecar(scales: PageScaleConfig[]) {
    const inputPath = loadedFilePath ?? appState.currentFile;
    if (!inputPath) return;
    try {
      await invoke('studio_save_page_scales', {
        inputPath,
        pageScales: scales.map((s) => ({
          pageIndex: s.pageIndex,
          ptDistance: s.ptDistance,
          realDistance: s.realDistance,
          unit: s.unit,
          precision: s.precision,
          defaultDepth: s.depthValue,
        })),
      });
    } catch {
      // Safe fallback in test environment
    }
  }

  // BUG-016: the selected document tab. Switching stashes the outgoing document (view, markups,
  // measurements) and clears the engines + undo history before the next document is loaded.
  // Only the selected file and the loaded file drive this effect; the switch itself reads and
  // writes engine state, which must not re-trigger it (that would re-stash an emptied session).
  let requestedFile: string | null = null;
  /** The store token of the document this canvas shows (review r4: the store owns all state). */
  let activeToken: LoadToken | null = null;
  let unmounted = false;
  /** True while the shown document's markups are being restored: tools and inserts are blocked. */
  let documentBusy = $state(false);

  /** Page-border snap segments for a fresh layout (the extracted geometry replaces them later). */
  /** Follower pane: wait for the primary pane to restore `file`, then paint it; retry on timeout. */
  async function followLiveDocument(file: string) {
    while (!unmounted && requestedFile === file) {
      const ok = await whenDocumentLive(file, FOLLOWER_WAIT.ms);
      const t = currentToken();
      if (ok && !unmounted && t && t.path === file && requestedFile === file) {
        loadPdfIntoCanvas(file, t, 'render');
        return;
      }
      if (t && t.path !== file) return; // another document became live: a new request will follow
    }
  }

  function seedPageBorderSnap() {
    viewport.pages.forEach((pg, idx) => {
      const w = pg.widthPt * PT_TO_CSS;
      const h = pg.heightPt * PT_TO_CSS;
      const x0 = pg.worldX;
      const y0 = pg.worldY;
      snapEngine.setPageSegments(idx, [
        { x1: x0, y1: y0, x2: x0 + w, y2: y0 },
        { x1: x0 + w, y1: y0, x2: x0 + w, y2: y0 + h },
        { x1: x0 + w, y1: y0 + h, x2: x0, y2: y0 + h },
        { x1: x0, y1: y0 + h, x2: x0, y2: y0 },
      ]);
    });
  }
  $effect(() => {
    const file = appState.currentFile;
    untrack(() => {
      if (follower) {
        if (file && file !== requestedFile) {
          requestedFile = file;
          void followLiveDocument(file);
        }
        return;
      }
      if (file && file !== requestedFile) {
        if (requestedFile !== null && editMode === 'structural') editMode = 'markup';
        requestedFile = file;
        eraseGestureBefore = null; // never commit an erase gesture into another document
        const token = activate(file, viewId);
        activeToken = token;
        documentBusy = token.mode === 'load';
        loadPdfIntoCanvas(file, token, token.mode);
      } else if (!file && requestedFile) {
        requestedFile = null;
        activeToken = null;
        deactivate();
        loadedFilePath = null;
        viewport.pages = [];
      }
    });
  });

  async function loadStudioPrefs() {
    try {
      const prefs = await invoke<{
        dockEdge: string;
        floatX: number;
        floatY: number;
        shortcuts: Record<string, string>;
        favorites: string[];
        historyLimit: number;
      }>('prefs_load');
      if (prefs) {
        if (prefs.shortcuts && Object.keys(prefs.shortcuts).length > 0) {
          commands.shortcuts = { ...commands.shortcuts, ...prefs.shortcuts };
        }
        if (Array.isArray(prefs.favorites) && prefs.favorites.length > 0) {
          tools.favorites = [...prefs.favorites];
        }
        if (typeof prefs.historyLimit === 'number' && prefs.historyLimit > 0) {
          commands.historyLimit = prefs.historyLimit;
        }
        // Restored shortcuts may target a Labs tool: park them while Labs is off.
        enforceLabsGate();
      }
    } catch {
      // Safe fallback in browser/unit-test environments without Tauri IPC
    }
  }

  async function persistStudioPrefs() {
    try {
      await invoke('prefs_save', {
        prefs: {
          dockEdge: 'left',
          floatX: 16,
          floatY: 80,
          shortcuts: shortcutsForSave(),
          favorites: tools.favorites,
          historyLimit: commands.historyLimit,
        },
      });
    } catch {
      // Safe fallback in browser/unit-test environments
    }
  }

  onMount(() => {
    viewport.upsertView({
      id: viewId,
      zoom: 1.0,
      panX: 40,
      panY: 40,
      linkGroup: null,
      dpr: window.devicePixelRatio || 1,
    });

    if (Object.keys(tools.presets).length === 0) {
      tools.load(defaultPresets as any, [
        'preset.pen.a',
        'preset.highlighter',
        'preset.eraser.partial',
      ]);
    }

    // A follower pane (Labs split view) never registers handlers or the keyboard: one owner only.
    if (follower) {
      return () => {
        viewport.closeView(viewId);
      };
    }

    commands.onRadialToggle = () => {
      toggleRadialAt();
    };

    // Register Phase D4 & D5 global command handlers (zero local keydown listeners)
    commands.register('selection.duplicate', (doc) => {
      selection.duplicateSelected();
      return { next: doc, label: 'Duplicate Selection' };
    });
    commands.register('selection.group', (doc) => {
      selection.groupSelected();
      return { next: doc, label: 'Group Selection' };
    });
    commands.register('selection.ungroup', (doc) => {
      selection.ungroupSelected();
      return { next: doc, label: 'Ungroup Selection' };
    });
    commands.register('selection.delete', (doc) => {
      selection.deleteSelected();
      return { next: doc, label: 'Delete Selection' };
    });
    commands.register('studio.confirm', (doc) => {
      if (measureEngine.activeDraft) {
        measureEngine.commitActiveDraft();
      } else if (shapes.isCreating && isMultiClickTool(tools.currentToolId)) {
        shapes.commitPolyline();
      } else if (shapes.editingTextBox) {
        commitStructuralTextOrAnnotation();
      }
      return { next: doc, label: 'Confirm Active Operation' };
    });
    commands.register('studio.cancel', (doc) => {
      measureEngine.cancelDraft();
      shapes.cancelActiveShape();
      selection.clearSelection();
      return { next: doc, label: 'Cancel Operation' };
    });

    const detachKeyboard =
      typeof window !== 'undefined' ? commands.attachKeyboard(window) : () => {};

    loadStudioPrefs();

    return () => {
      detachKeyboard();
      commands.onRadialToggle = null;
      viewport.closeView(viewId);
    };
  });

  onDestroy(() => {
    // The engines stay live in the document store: a canvas never resets or stashes them.
    unmounted = true;
    if (currentPdfDoc) {
      currentPdfDoc.destroy();
    }
  });

  function handleContextMenu(e: MouseEvent) {
    e.preventDefault();
    toggleRadialAt(e.clientX, e.clientY);
  }

  function handleWheel(e: WheelEvent) {
    e.preventDefault();
    e.stopPropagation();
    if (!canvasContainer) return;

    const rect = canvasContainer.getBoundingClientRect();
    const anchor: Pt = {
      x: e.clientX - rect.left,
      y: e.clientY - rect.top,
    };

    if (e.ctrlKey || e.metaKey) {
      viewport.pinchZoomAt(e.deltaY, e.deltaMode, anchor.x, anchor.y, viewId);
      return;
    }

    if (wheelBehavior === 'zoom') {
      viewport.pinchZoomAt(e.deltaY * 0.65, e.deltaMode, anchor.x, anchor.y, viewId);
      return;
    }

    if (wheelBehavior === 'pan') {
      viewport.panByDelta(e.shiftKey ? e.deltaY : e.deltaX, e.shiftKey ? e.deltaX : e.deltaY, viewId);
      return;
    }

    // Default 'auto' mode
    if (Math.abs(e.deltaX) > 0) {
      viewport.panByDelta(e.deltaX, e.deltaY, viewId);
    } else if (e.shiftKey || e.altKey) {
      viewport.panByDelta(e.shiftKey ? e.deltaY : 0, e.shiftKey ? 0 : e.deltaY, viewId);
    } else {
      viewport.pinchZoomAt(e.deltaY * 0.65, e.deltaMode, anchor.x, anchor.y, viewId);
    }
  }

  function getWorldPoint(clientX: number, clientY: number): Pt {
    if (!canvasContainer) return { x: clientX, y: clientY };
    const rect = canvasContainer.getBoundingClientRect();
    const screenX = clientX - rect.left;
    const screenY = clientY - rect.top;
    const inv = viewport.worldToScreen(viewId).invert();
    return inv ? inv.apply({ x: screenX, y: screenY }) : { x: screenX, y: screenY };
  }

  function resolveSnappedWorldPoint(rawWorld: Pt, pageIndex: number): Pt {
    const rulerProj = projectPointToRulerIfNear(rawWorld, measureEngine.ruler, 14);
    if (rulerProj.snapped) {
      return { x: rulerProj.x, y: rulerProj.y };
    }
    const anchor =
      measureEngine.activeDraft && measureEngine.activeDraft.points.length > 0
        ? measureEngine.activeDraft.points[measureEngine.activeDraft.points.length - 1]
        : shapes.activeShape && shapes.activeShape.points.length > 0
          ? shapes.activeShape.points[shapes.activeShape.points.length - 1]
          : null;
    const hit = snapEngine.resolveSnap(
      pageIndex,
      rawWorld.x,
      rawWorld.y,
      anchor,
      currentView.zoom,
      ink.completedStrokes
    );
    return hit ? { x: hit.x, y: hit.y } : rawWorld;
  }

  function handleDoubleClick(e: MouseEvent) {
    if (measureEngine.activeDraft && isMultiPointMeasureTool(tools.currentToolId)) {
      e.preventDefault();
      measureEngine.commitActiveDraft();
      return;
    }
    if (shapes.isCreating && isMultiClickTool(tools.currentToolId)) {
      e.preventDefault();
      shapes.commitPolyline();
    }
  }

  // BUG-019: one eraser gesture (pointer down → up) = one undo step.
  let eraseGestureBefore: ActiveStroke[] | null = null;

  function finishEraseGesture() {
    const before = eraseGestureBefore;
    eraseGestureBefore = null;
    if (!before) return;
    const after = ink.completedStrokes;
    const changed = before.length !== after.length || before.some((st, i) => st !== after[i]);
    if (changed) {
      shapes.recordStrokeHistory(before, [...after], 'Erase');
    }
  }

  function handlePointerDown(e: PointerEvent) {
    if (!canvasContainer) return;

    const target = e.target as HTMLElement | null;
    if (
      target?.closest(
        '.studio-top-bar, .spatial-hud, .dockable-toolbar, .property-pill, .favorites-bar, .modal-backdrop, .radial-menu, .selection-action-bar, .inline-text-editor, .digital-ruler-widget, button, select, input, textarea'
      )
    ) {
      return;
    }

    if (e.pointerType === 'touch') {
      activeTouchPoints.set(e.pointerId, { x: e.clientX, y: e.clientY });
      if (activeTouchPoints.size === 2) {
        if (ink.isDrawing) ink.cancelStroke();
        isPanning = false;
        const pts = Array.from(activeTouchPoints.values());
        touchInitialDistance = Math.hypot(pts[1].x - pts[0].x, pts[1].y - pts[0].y);
        touchInitialMidpoint = { x: (pts[0].x + pts[1].x) / 2, y: (pts[0].y + pts[1].y) / 2 };
        touchInitialZoom = currentView.zoom;
        return;
      }
    }

    // BUG-013: #wet-layer sits above the text-block hotspots, so in Edit Text mode a primary click
    // on a block edits it here, whatever the active tool (Pan used to take the click).
    if (editMode === 'structural' && e.button === 0 && !commands.spaceSpringActive) {
      const block = hitTestTextBlock(getWorldPoint(e.clientX, e.clientY));
      if (block) {
        handleEditTextBlock(block);
        return;
      }
    }

    const isPanAction = e.button === 1 || commands.spaceSpringActive || tools.activeTool === 'pan';

    // Review r3: no markups while a document's saved markups are still being restored.
    if (activeToken && !isDocumentRestored(activeToken.path) && !isPanAction) return;

    if (isPanAction) {
      isPanning = true;
      panStart = { x: e.clientX, y: e.clientY };
      try {
        (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
      } catch {}
      return;
    }

    const rawWorldPt = getWorldPoint(e.clientX, e.clientY);
    const pageIndex = findPageForWorldPoint(rawWorldPt);
    const worldPt = resolveSnappedWorldPoint(rawWorldPt, pageIndex);
    const pressure = e.pointerType === 'pen' ? e.pressure || 0.5 : 0.5;
    const activeTool = tools.currentToolId;

    // Digital Ruler Toggle Tool
    if (activeTool === 'measure.ruler') {
      measureEngine.ruler = {
        ...measureEngine.ruler,
        visible: true,
        xPt: worldPt.x,
        yPt: worldPt.y,
      };
      return;
    }

    // Phase D5 CAD Calibration & Measurement Suite
    if (isMeasureTool(activeTool)) {
      if (isMultiPointMeasureTool(activeTool)) {
        measureEngine.startOrAddPoint(activeTool, pageIndex, worldPt);
        return;
      }
      try {
        (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
      } catch {}
      measureEngine.startOrAddPoint(activeTool, pageIndex, worldPt);
      return;
    }

    // Phase D6 Insert Suite (LaTeX, Signature, Audio Note Modals + Callout / Stamp / Image Quick Stamp)
    if (
      (activeTool === 'insert.latex' && labsVisible(activeTool)) ||
      activeTool === 'insert.signature' ||
      activeTool === 'insert.audio'
    ) {
      inserts.openInsertModalForTool(activeTool, worldPt.x, worldPt.y, pageIndex);
      return;
    }

    if (
      activeTool === 'callout.single' ||
      activeTool === 'callout.multi' ||
      activeTool === 'insert.stamp' ||
      activeTool === 'insert.image'
    ) {
      inserts.stampQuickInsert(activeTool, worldPt.x, worldPt.y, pageIndex);
      return;
    }

    // Structural Edit Mode or Inline Text Box tool
    if (
      editMode === 'structural' ||
      activeTool === 'review.textEdit' ||
      activeTool === 'insert.text' ||
      activeTool === 'insert.textBox' ||
      activeTool === 'insert.note'
    ) {
      shapes.openTextBoxEditor(
        worldPt.x,
        worldPt.y,
        pageIndex,
        editMode === 'structural' || activeTool === 'review.textEdit'
      );
      return;
    }

    // Selection / Marquee / Lasso / Snapshot tools
    if (
      activeTool === 'select' ||
      activeTool.startsWith('select.') ||
      activeTool === 'snapshot'
    ) {
      try {
        (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
      } catch {}

      selectionDragStart = worldPt;
      if (activeTool === 'select.lasso') {
        selectionDragMode = 'lasso';
        selection.startLasso(worldPt.x, worldPt.y);
        return;
      }
      if (activeTool === 'select.snapshot' || activeTool === 'snapshot') {
        selectionDragMode = 'snapshot';
        selection.startMarquee(worldPt.x, worldPt.y);
        return;
      }

      const hit = selection.hitTestAtPoint(worldPt.x, worldPt.y);
      if (hit) {
        selection.selectAtPoint(worldPt.x, worldPt.y, e.shiftKey);
        selectionDragMode = 'move';
      } else {
        selectionDragMode = 'marquee';
        selection.startMarquee(worldPt.x, worldPt.y);
      }
      return;
    }

    // Multi-click Polyline / Polygon / Revision Cloud / Hatch Region
    if (isMultiClickTool(activeTool)) {
      shapes.addPolylineVertex(worldPt.x, worldPt.y, activeTool, pageIndex);
      return;
    }

    try {
      (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    } catch {}

    // Two-point drag geometric shapes & review markups
    if (isShapeOrMarkupTool(activeTool)) {
      shapes.startDragShape(worldPt.x, worldPt.y, activeTool, pageIndex);
      return;
    }

    if (activeTool === 'laser') {
      ink.addLaserPoint(worldPt.x, worldPt.y);
      return;
    }

    if (activeTool === 'eraser.whole' || activeTool === 'eraser.partial') {
      finishEraseGesture();
      eraseGestureBefore = [...ink.completedStrokes];
      try {
        (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
      } catch {}
      if (activeTool === 'eraser.whole') ink.eraseWhole(worldPt, 18);
      else ink.erasePartial(worldPt, 12);
      return;
    }

    // Standard freehand pressure inking
    ink.startStroke(worldPt.x, worldPt.y, pressure, activeTool);

    const rect = canvasContainer.getBoundingClientRect();
    commands.dispatch({
      kind: 'pointer',
      phase: 'down',
      x: e.clientX - rect.left,
      y: e.clientY - rect.top,
      pressure,
      tiltX: e.tiltX || 0,
      tiltY: e.tiltY || 0,
      twist: e.twist || 0,
      pointerType: e.pointerType === 'pen' || e.pointerType === 'touch' ? e.pointerType : 'mouse',
      buttons: e.buttons,
      viewId,
      tMs: performance.now(),
    });
  }

  function handlePointerMove(e: PointerEvent) {
    if (editMode === 'structural' && e.pointerType !== 'touch') {
      hoveredBlockId = hitTestTextBlock(getWorldPoint(e.clientX, e.clientY))?.id ?? null;
    }
    if (e.pointerType === 'touch' && activeTouchPoints.has(e.pointerId)) {
      activeTouchPoints.set(e.pointerId, { x: e.clientX, y: e.clientY });
      if (activeTouchPoints.size === 2) {
        const pts = Array.from(activeTouchPoints.values());
        const currentDist = Math.hypot(pts[1].x - pts[0].x, pts[1].y - pts[0].y);
        const currentMid = { x: (pts[0].x + pts[1].x) / 2, y: (pts[0].y + pts[1].y) / 2 };

        if (touchInitialDistance > 0 && canvasContainer) {
          const ratio = currentDist / touchInitialDistance;
          const targetZoom = Math.max(0.1, Math.min(16.0, touchInitialZoom * ratio));
          const deltaPanX = currentMid.x - touchInitialMidpoint.x;
          const deltaPanY = currentMid.y - touchInitialMidpoint.y;
          const rect = canvasContainer.getBoundingClientRect();
          const screenMidX = currentMid.x - rect.left;
          const screenMidY = currentMid.y - rect.top;
          const k = targetZoom / currentView.zoom;
          const newPanX = screenMidX - (screenMidX - currentView.panX) * k + deltaPanX;
          const newPanY = screenMidY - (screenMidY - currentView.panY) * k + deltaPanY;

          viewport.upsertView({
            ...currentView,
            zoom: targetZoom,
            panX: newPanX,
            panY: newPanY,
          });
          touchInitialMidpoint = currentMid;
          touchInitialDistance = currentDist;
          touchInitialZoom = targetZoom;
        }
        return;
      }
    }

    if (isPanning) {
      const dx = e.clientX - panStart.x;
      const dy = e.clientY - panStart.y;
      panStart = { x: e.clientX, y: e.clientY };
      viewport.panBy(viewId, dx, dy);
      return;
    }

    if (!canvasContainer) return;

    const rawWorld = getWorldPoint(e.clientX, e.clientY);
    const pageIndex = findPageForWorldPoint(rawWorld);
    const currentWorld = resolveSnappedWorldPoint(rawWorld, pageIndex);

    if (measureEngine.activeDraft) {
      measureEngine.updateHoverPoint(currentWorld);
      return;
    }

    if (selectionDragMode === 'marquee' || selectionDragMode === 'snapshot') {
      selection.updateMarquee(currentWorld.x, currentWorld.y);
      return;
    }
    if (selectionDragMode === 'lasso') {
      selection.updateLasso(currentWorld.x, currentWorld.y);
      return;
    }
    if (selectionDragMode === 'move' && e.buttons > 0) {
      const dx = currentWorld.x - selectionDragStart.x;
      const dy = currentWorld.y - selectionDragStart.y;
      if (Math.hypot(dx, dy) > 1.5) {
        selection.moveSelected(dx, dy);
        selectionDragStart = currentWorld;
      }
      return;
    }

    if (shapes.isCreating) {
      if (isMultiClickTool(tools.currentToolId)) {
        shapes.updatePolylinePreview(currentWorld.x, currentWorld.y);
      } else if (e.buttons > 0) {
        shapes.updateDragShape(currentWorld.x, currentWorld.y, e.shiftKey);
      }
      return;
    }

    const events = typeof e.getCoalescedEvents === 'function' ? e.getCoalescedEvents() : [e];
    for (const ev of events) {
      const coalescedRaw = getWorldPoint(ev.clientX, ev.clientY);
      const rulerPt = projectPointToRulerIfNear(coalescedRaw, measureEngine.ruler, 14);
      const worldPt = rulerPt.snapped ? { x: rulerPt.x, y: rulerPt.y } : coalescedRaw;
      const pressure = ev.pointerType === 'pen' ? ev.pressure || 0.5 : 0.5;

      if (tools.activeToolId === 'laser') {
        ink.addLaserPoint(worldPt.x, worldPt.y);
      } else if (tools.activeToolId === 'eraser.whole') {
        if (ev.buttons > 0) ink.eraseWhole(worldPt, 18);
      } else if (tools.activeToolId === 'eraser.partial') {
        if (ev.buttons > 0) ink.erasePartial(worldPt, 12);
      } else if (ink.isDrawing) {
        ink.addPoint(worldPt.x, worldPt.y, pressure);
      }
    }

    const rect = canvasContainer.getBoundingClientRect();
    commands.dispatch({
      kind: 'pointer',
      phase: e.buttons > 0 ? 'move' : 'hover',
      x: e.clientX - rect.left,
      y: e.clientY - rect.top,
      pressure: e.pointerType === 'pen' ? e.pressure || 0.5 : 0.5,
      tiltX: e.tiltX || 0,
      tiltY: e.tiltY || 0,
      twist: e.twist || 0,
      pointerType: e.pointerType === 'pen' || e.pointerType === 'touch' ? e.pointerType : 'mouse',
      buttons: e.buttons,
      viewId,
      tMs: performance.now(),
    });
  }

  function handlePointerUp(e: PointerEvent) {
    if (e.pointerType === 'touch') {
      activeTouchPoints.delete(e.pointerId);
      if (activeTouchPoints.size < 2) {
        touchInitialDistance = 0;
      }
    }

    if (isPanning) {
      isPanning = false;
      try {
        if ((e.currentTarget as HTMLElement).hasPointerCapture(e.pointerId)) {
          (e.currentTarget as HTMLElement).releasePointerCapture(e.pointerId);
        }
      } catch {}
      return;
    }

    if (
      measureEngine.activeDraft &&
      !isMultiPointMeasureTool(tools.currentToolId)
    ) {
      const rawWorld = getWorldPoint(e.clientX, e.clientY);
      const pageIndex = findPageForWorldPoint(rawWorld);
      const snappedWorld = resolveSnappedWorldPoint(rawWorld, pageIndex);
      measureEngine.commitTwoPointDrag(snappedWorld);
      try {
        if ((e.currentTarget as HTMLElement).hasPointerCapture(e.pointerId)) {
          (e.currentTarget as HTMLElement).releasePointerCapture(e.pointerId);
        }
      } catch {}
      return;
    }

    if (selectionDragMode !== 'none') {
      const mode = selectionDragMode;
      selectionDragMode = 'none';
      if (mode === 'marquee') {
        selection.commitMarquee(e.shiftKey);
      } else if (mode === 'lasso') {
        selection.commitLasso(e.shiftKey);
      } else if (mode === 'snapshot' && selection.marqueeRect) {
        const m = selection.marqueeRect;
        selection.marqueeRect = null;
        const box = {
          minX: Math.min(m.x0, m.x1),
          minY: Math.min(m.y0, m.y1),
          maxX: Math.max(m.x0, m.x1),
          maxY: Math.max(m.y0, m.y1),
        };
        if (box.maxX - box.minX >= 8 && box.maxY - box.minY >= 8) {
          const dataUrl = captureSnapshotDataUrl(box);
          selection.createSnapshotStamp(
            box,
            dataUrl,
            findPageForWorldPoint({ x: (box.minX + box.maxX) / 2, y: (box.minY + box.maxY) / 2 })
          );
          tools.selectTool('select.marquee');
        }
      }
      try {
        if ((e.currentTarget as HTMLElement).hasPointerCapture(e.pointerId)) {
          (e.currentTarget as HTMLElement).releasePointerCapture(e.pointerId);
        }
      } catch {}
      return;
    }

    if (shapes.isCreating && !isMultiClickTool(tools.currentToolId)) {
      shapes.commitDragShape();
      try {
        if ((e.currentTarget as HTMLElement).hasPointerCapture(e.pointerId)) {
          (e.currentTarget as HTMLElement).releasePointerCapture(e.pointerId);
        }
      } catch {}
      return;
    }

    finishEraseGesture();

    if (ink.isDrawing) {
      const before = [...ink.completedStrokes];
      const committed = ink.endStroke();
      if (committed) {
        const after = [...ink.completedStrokes];
        shapes.recordStrokeHistory(before, after, `Ink ${committed.tool}`);
      }
    }

    try {
      if ((e.currentTarget as HTMLElement).hasPointerCapture(e.pointerId)) {
        (e.currentTarget as HTMLElement).releasePointerCapture(e.pointerId);
      }
    } catch {}

    if (!canvasContainer) return;
    const rect = canvasContainer.getBoundingClientRect();
    commands.dispatch({
      kind: 'pointer',
      phase: 'up',
      x: e.clientX - rect.left,
      y: e.clientY - rect.top,
      pressure: 0,
      tiltX: e.tiltX || 0,
      tiltY: e.tiltY || 0,
      twist: 0,
      pointerType: e.pointerType === 'pen' || e.pointerType === 'touch' ? e.pointerType : 'mouse',
      buttons: e.buttons,
      viewId,
      tMs: performance.now(),
    });
  }

  function handlePointerCancel(e: PointerEvent) {
    if (e.pointerType === 'touch') {
      activeTouchPoints.delete(e.pointerId);
      if (activeTouchPoints.size < 2) {
        touchInitialDistance = 0;
      }
    }
    if (isPanning) {
      isPanning = false;
    }
    finishEraseGesture();
    selectionDragMode = 'none';
    if (ink.isDrawing) {
      ink.cancelStroke();
    }
    try {
      if ((e.currentTarget as HTMLElement).hasPointerCapture(e.pointerId)) {
        (e.currentTarget as HTMLElement).releasePointerCapture(e.pointerId);
      }
    } catch {}
  }

  function getFillForStroke(s: ActiveStroke): string {
    if (s.tool === 'review.redact') return 'url(\x23hatch-redact-danger)';
    if (s.tool === 'review.highlight') return s.color;
    if (!s.hatchPattern || s.hatchPattern === 'none') return 'none';
    if (s.hatchPattern === 'solid') return s.fillColor || s.color;
    return `url(\x23hatch-${s.hatchPattern})`;
  }
</script>

<div
  class="spatial-viewport"
  bind:this={canvasContainer}
  onwheel={handleWheel}
  role="region"
  aria-label="Drawboard Spatial Studio Canvas"
  tabindex="-1"
>
  <!-- Studio Top Command Bar (4 Zones). The host spans the canvas so the bar can never be
       wider than the canvas; container queries collapse labels to icons on narrow canvases. -->
  <div class="studio-top-bar-host">
  <header
    class="studio-top-bar glass-panel"
    role="toolbar"
    tabindex={0}
    aria-label="Drawboard Studio Top Command Bar"
    onpointerdown={(e) => e.stopPropagation()}
  >
    <!-- Zone 1: Workspace & Mode -->
    <div class="top-zone zone-workspace">
      <button
        type="button"
        class="top-icon-btn"
        onclick={() => appState.toggleSidebar()}
        title="Toggle Left Sidebar (Ctrl+B)"
        aria-label="Toggle Left Sidebar"
      >
        <span style="font-size: 14px;">☰</span>
      </button>

      <div class="mode-segmented-switch">
        <button
          type="button"
          class="top-btn"
          class:active={editMode === 'markup'}
          onclick={() => {
            editMode = 'markup';
          }}
          title="Markup Mode (Fast 60-120fps Annotations & CAD Tools)"
        >
          <StudioIcon name="pen.a" size={13} />
          <span>Markup</span>
        </button>
        <button
          type="button"
          class="top-btn"
          class:active={editMode === 'structural'}
          disabled={!hasDocument}
          onclick={() => {
            editMode = 'structural';
          }}
          title={hasDocument ? 'Structural PDF Edit Mode (Direct Text & Font Manipulation)' : NEEDS_DOC}
        >
          <StudioIcon name="text.review" size={13} />
          <span>Edit Text</span>
        </button>
      </div>

      {#if hasDocument}
        <div class="page-indicator-badge" title="Page {activePageNumber} of {viewport.pages.length}">
          <span class="collapsible-label">Page</span> {activePageNumber} / {viewport.pages.length}
        </div>
      {/if}
    </div>

    <div class="top-zone-divider"></div>

    <!-- Zone 2: Viewport & Zoom -->
    <div class="top-zone zone-viewport">
      <button
        type="button"
        class="top-icon-btn"
        disabled={!hasDocument}
        onclick={() => viewport.zoomAt(viewId, 0.8, { x: 300, y: 300 })}
        title={hasDocument ? 'Zoom Out' : NEEDS_DOC}
        aria-label="Zoom Out"
      >
        <StudioIcon name="zoom.out" size={14} />
      </button>

      <select
        class="top-select"
        value={Math.round(currentView.zoom * 100).toString()}
        onchange={handleZoomSelect}
        disabled={!hasDocument}
        aria-label="Zoom Level Preset"
        title="Zoom Preset (50% - 200%, Fit Width, Fit Page)"
      >
        <option value="50">50%</option>
        <option value="75">75%</option>
        <option value="100">100%</option>
        <option value="125">125%</option>
        <option value="150">150%</option>
        <option value="200">200%</option>
        <option value="fit-width">Fit Width</option>
        <option value="fit-page">Fit Page</option>
      </select>

      <button
        type="button"
        class="top-icon-btn"
        disabled={!hasDocument}
        onclick={() => viewport.zoomAt(viewId, 1.25, { x: 300, y: 300 })}
        title={hasDocument ? 'Zoom In' : NEEDS_DOC}
        aria-label="Zoom In"
      >
        <StudioIcon name="zoom.in" size={14} />
      </button>

      <button
        type="button"
        class="top-icon-btn"
        disabled={!hasDocument}
        onclick={handleRotatePage90}
        title={hasDocument ? 'Rotate Page 90° Clockwise' : NEEDS_DOC}
        aria-label="Rotate Page 90° Clockwise"
      >
        <StudioIcon name="reset" size={14} />
      </button>

      <select
        class="top-select"
        value={inserts.pageOverlay}
        onchange={(e) => {
          inserts.pageOverlay = (e.target as HTMLSelectElement).value as PageOverlayTemplate;
        }}
        disabled={!hasDocument}
        aria-label="Page Background Overlay Template"
        title="Non-destructive Page Background Overlay (Engineering Grid, Isometric, Ruled)"
      >
        <option value="none">No overlay</option>
        <option value="engineering-grid">Eng grid</option>
        <option value="isometric-grid">Iso grid</option>
        <option value="ruled-paper">Ruled</option>
      </select>
    </div>

    <div class="top-zone-divider"></div>

    <!-- Zone 3: History & Assist -->
    <div class="top-zone zone-history-assist">
      <button
        type="button"
        class="top-icon-btn"
        disabled={!commands.canUndo}
        onclick={() => commands.undo()}
        title="Undo ({commands.undoLabel ?? 'None'})"
        aria-label="Undo"
      >
        <StudioIcon name="undo" size={14} />
      </button>
      <button
        type="button"
        class="top-icon-btn"
        disabled={!commands.canRedo}
        onclick={() => commands.redo()}
        title="Redo"
        aria-label="Redo"
      >
        <StudioIcon name="redo" size={14} />
      </button>

      <button
        type="button"
        class="top-btn"
        class:active={snapEngine.enabled}
        disabled={!hasDocument}
        aria-pressed={snapEngine.enabled}
        aria-label="Snap to drawing geometry"
        onclick={() => {
          snapEngine.enabled = !snapEngine.enabled;
        }}
        title={hasDocument ? `Snap to drawing geometry (OSNAP): ${snapEngine.enabled ? 'on' : 'off'}` : NEEDS_DOC}
      >
        <StudioIcon name="snap" size={13} />
        <span class="collapsible-label">Snap</span>
      </button>

      <button
        type="button"
        class="top-btn"
        class:active={measureEngine.ruler.visible}
        disabled={!hasDocument}
        aria-pressed={measureEngine.ruler.visible}
        aria-label="Digital drafting ruler"
        onclick={() => {
          measureEngine.ruler = {
            ...measureEngine.ruler,
            visible: !measureEngine.ruler.visible,
          };
        }}
        title={hasDocument ? 'Digital drafting ruler' : NEEDS_DOC}
      >
        <StudioIcon name="ruler" size={13} />
        <span class="collapsible-label">Ruler</span>
      </button>
    </div>

    <div class="top-zone-divider"></div>

    <!-- Zone 4: Actions & Persistence -->
    <div class="top-zone zone-actions">
      {#if shapes.editingTextBox}
        <button
          type="button"
          class="top-action-btn primary-btn"
          onclick={commitStructuralTextOrAnnotation}
          title="Commit Active Text Block Edit"
        >
          Commit Text
        </button>
      {/if}

      <button
        type="button"
        class="top-action-btn"
        disabled={!hasDocument}
        data-testid="save-pdf-btn"
        onclick={() => saveAnnotationsToPdf(false)}
        title={hasDocument ? 'Save markups as editable annotations (writes a new file atomically)' : NEEDS_DOC}
      >
        Save <span class="collapsible-label">PDF</span>
      </button>
      <button
        type="button"
        class="top-action-btn"
        disabled={!hasDocument}
        data-testid="flatten-pdf-btn"
        onclick={() => saveAnnotationsToPdf(true)}
        title={hasDocument ? 'Flatten markups permanently into the page content' : NEEDS_DOC}
      >
        Flatten <span class="collapsible-label">PDF</span>
      </button>
      {#if hasRedactions}
        <button
          type="button"
          class="top-action-btn danger-btn"
          data-testid="commit-redactions-btn"
          onclick={() => saveAnnotationsToPdf(true)}
          title="Permanently Sanitize & Strip Underlying Content in Redacted Regions"
        >
          Commit Redactions
        </button>
      {/if}

      <button
        type="button"
        class="top-action-btn ledger-btn"
        class:primary-btn={inserts.ledgerOpen}
        onclick={() => inserts.toggleLedger()}
        aria-label="Ledger: markups, library and quantity takeoff"
        title="Ledger: markup list, reusable library and quantity takeoff"
      >
        <StudioIcon name="note" size={13} />
        <span class="collapsible-label">Ledger</span>
      </button>
    </div>
  </header>
  </div>

  {#if !hasDocument}
    <div class="studio-empty-state glass-panel" data-testid="studio-empty-state">
      <StudioIcon name="pen.a" size={28} />
      <h2>Open a PDF to begin</h2>
      <p>Mark up, measure and edit drawings and documents. Your files never leave this computer.</p>
      <button type="button" class="top-action-btn primary-btn" onclick={openPdfFromEmptyState}>
        Open PDF…
      </button>
    </div>
  {/if}

  {#if passwordPrompt}
    <!-- Keeps canvas pointer gestures from starting under the dialog. -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div onpointerdown={(e) => e.stopPropagation()}>
      <form
        onsubmit={(e) => {
          e.preventDefault();
          finishPasswordPrompt(true);
        }}
      >
        <!-- Presentation only (ui/Dialog sits above every toolbar layer): same prompt, same retry and cancel paths. -->
        <Dialog
          open={true}
          title="Password required"
          ariaLabel="Password required"
          onclose={() => finishPasswordPrompt(false)}
          data-testid="pdf-password-dialog"
        >
          <p class="pdf-password-text">"{passwordPrompt.fileName}" is encrypted. Enter its password to open it.</p>
          <Input
            type="password"
            label="Document password"
            autocomplete="off"
            bind:value={passwordValue}
            data-testid="pdf-password-input"
          />
          {#if passwordPrompt.incorrect}
            <span class="pdf-password-error" role="alert" data-testid="pdf-password-error">Incorrect password. Try again.</span>
          {/if}
          {#snippet actions()}
            <Button variant="secondary" data-testid="pdf-password-cancel" onclick={() => finishPasswordPrompt(false)}>Cancel</Button>
            <Button variant="primary" type="submit" data-testid="pdf-password-submit" onclick={() => {}}>Open</Button>
          {/snippet}
        </Dialog>
      </form>
    </div>
  {/if}

  {#if statusNotice}
    <div class="studio-status-banner glass-panel" role="status" data-testid="studio-status-banner">
      <span>{statusNotice}</span>
      <button type="button" class="banner-close" onclick={() => { statusNotice = null; }} aria-label="Dismiss Notice">
        <StudioIcon name="close" size={12} />
      </button>
    </div>
  {/if}

  {#if editMode === 'structural'}
    <div class="structural-edit-banner glass-panel" data-testid="structural-edit-banner">
      <div class="banner-content">
        <StudioIcon name="text.review" size={16} />
        <span>Structural PDF Text Edit Mode Active — Click any text block to edit native PDF streams</span>
      </div>
      <button
        type="button"
        class="top-action-btn primary-btn done-btn"
        onclick={() => (editMode = 'markup')}
        data-testid="exit-structural-btn"
      >
        Done Editing
      </button>
    </div>
  {/if}

  <!-- Contextual Selection Action Bar (Group, Ungroup, Align, Flip, Z-Order, Duplicate, Format Painter) -->
  {#if selection.hasSelection && selection.selectionBounds}
    <div
      class="selection-action-bar glass-panel"
      role="toolbar"
      tabindex={0}
      aria-label="Selection Manipulation Toolbar"
      onpointerdown={(e) => e.stopPropagation()}
    >
      <span class="sel-count">{selection.selectedIds.length} selected</span>
      <div class="hud-divider"></div>
      <button type="button" class="sel-btn" onclick={() => selection.groupSelected()} title="Group Selected (Ctrl+G)">Group</button>
      <button type="button" class="sel-btn" onclick={() => selection.ungroupSelected()} title="Ungroup Selected (Ctrl+Shift+G)">Ungroup</button>
      <div class="hud-divider"></div>
      <button type="button" class="sel-btn" onclick={() => selection.alignSelected('left')} title="Align Left">Left</button>
      <button type="button" class="sel-btn" onclick={() => selection.alignSelected('center')} title="Align Center">Center</button>
      <button type="button" class="sel-btn" onclick={() => selection.alignSelected('right')} title="Align Right">Right</button>
      <button type="button" class="sel-btn" onclick={() => selection.alignSelected('top')} title="Align Top">Top</button>
      <button type="button" class="sel-btn" onclick={() => selection.alignSelected('middle')} title="Align Middle">Middle</button>
      <button type="button" class="sel-btn" onclick={() => selection.alignSelected('bottom')} title="Align Bottom">Bottom</button>
      <div class="hud-divider"></div>
      <button type="button" class="sel-btn" onclick={() => selection.flipSelected('horizontal')} title="Flip Horizontal">Flip H</button>
      <button type="button" class="sel-btn" onclick={() => selection.flipSelected('vertical')} title="Flip Vertical">Flip V</button>
      <div class="hud-divider"></div>
      <button type="button" class="sel-btn" onclick={() => selection.reorderSelected('front')} title="Bring to Front">Front</button>
      <button type="button" class="sel-btn" onclick={() => selection.reorderSelected('back')} title="Send to Back">Back</button>
      <button type="button" class="sel-btn" onclick={() => selection.reorderSelected('forward')} title="Bring Forward">+1</button>
      <button type="button" class="sel-btn" onclick={() => selection.reorderSelected('backward')} title="Send Backward">-1</button>
      <div class="hud-divider"></div>
      <button type="button" class="sel-btn" onclick={() => selection.duplicateSelected()} title="Duplicate (Ctrl+D)">Duplicate</button>
      <button
        type="button"
        class="sel-btn"
        onclick={() => {
          inserts.pastePagesModalOpen = true;
        }}
        title="Multi-Page Advanced Paste (All, Even, Odd, Range)"
      >
        Paste to Pages...
      </button>
      <button
        type="button"
        class="sel-btn"
        class:active={selection.formatPainterActive}
        onclick={() => selection.copyStyleFromSelected()}
        title="Format Painter: Copy Style & Click Target Object"
      >
        Copy Style
      </button>
      <button type="button" class="sel-btn danger-btn" onclick={() => selection.deleteSelected()} title="Delete Selected (Del)">Delete</button>
    </div>
  {/if}

  <!-- LAYER A: #pdf-layer (Hardware-accelerated PDF.js raster surface) -->
  <div id="pdf-layer" class="pdf-layer world" style:transform={worldTransform}>
    {#each viewport.pages as page, index}
      <div
        class="spatial-page"
        style:left="{page.worldX}px"
        style:top="{page.worldY}px"
        style:width="{page.widthPt * PT_TO_CSS}px"
        style:height="{page.heightPt * PT_TO_CSS}px"
      >
        <div class="page-badge">
          Page {index + 1} ({Math.round(page.widthPt)} × {Math.round(page.heightPt)} pt)
        </div>

        <div class="page-render-surface">
          <canvas class="page-pdf-canvas" bind:this={pageCanvases[index]}></canvas>
          {#if inserts.pageOverlay !== 'none'}
            <svg
              class="page-template-overlay"
              viewBox="0 0 {page.widthPt * PT_TO_CSS} {page.heightPt * PT_TO_CSS}"
              preserveAspectRatio="none"
              aria-label="Page Background Overlay ({inserts.pageOverlay})"
            >
              <defs>
                {#if inserts.pageOverlay === 'engineering-grid'}
                  <pattern
                    id="tpl-eng-{index}"
                    width="24"
                    height="24"
                    patternUnits="userSpaceOnUse"
                  >
                    <path
                      d="M 24 0 L 0 0 0 24"
                      fill="none"
                      stroke="var(--canvas-grid-minor)"
                      stroke-width="0.7"
                    />
                  </pattern>
                  <pattern
                    id="tpl-eng-major-{index}"
                    width="120"
                    height="120"
                    patternUnits="userSpaceOnUse"
                  >
                    <rect width="120" height="120" fill="url(\x23tpl-eng-{index})" />
                    <path
                      d="M 120 0 L 0 0 0 120"
                      fill="none"
                      stroke="var(--canvas-grid-major)"
                      stroke-width="1.2"
                    />
                  </pattern>
                {:else if inserts.pageOverlay === 'isometric-grid'}
                  <pattern
                    id="tpl-iso-{index}"
                    width="48"
                    height="27.71"
                    patternUnits="userSpaceOnUse"
                  >
                    <path
                      d="M 0 0 L 48 27.71 M 48 0 L 0 27.71 M 24 0 L 24 27.71"
                      fill="none"
                      stroke="var(--canvas-grid-major)"
                      stroke-width="0.75"
                    />
                  </pattern>
                {:else if inserts.pageOverlay === 'ruled-paper'}
                  <pattern
                    id="tpl-ruled-{index}"
                    width="100"
                    height="28"
                    patternUnits="userSpaceOnUse"
                  >
                    <line
                      x1="0"
                      y1="27"
                      x2="100"
                      y2="27"
                      stroke="var(--canvas-grid-major)"
                      stroke-width="0.9"
                    />
                  </pattern>
                {/if}
              </defs>
              <rect
                width="100%"
                height="100%"
                fill={inserts.pageOverlay === 'engineering-grid'
                  ? `url(\x23tpl-eng-major-${index})`
                  : inserts.pageOverlay === 'isometric-grid'
                    ? `url(\x23tpl-iso-${index})`
                    : `url(\x23tpl-ruled-${index})`}
              />
            </svg>
          {/if}
        </div>

        {#if editMode === 'structural' && index === (activePageNumber - 1)}
          <div class="text-blocks-overlay" role="presentation">
            {#each pageTextBlocks as block (block.id)}
              {@const relX = block.bboxPt[0] * PT_TO_CSS}
              {@const relY = (page.heightPt - (block.bboxPt[1] + block.bboxPt[3])) * PT_TO_CSS}
              {@const relW = Math.max(block.bboxPt[2] * PT_TO_CSS, 20)}
              {@const relH = Math.max(block.bboxPt[3] * PT_TO_CSS, 12)}
              <!-- svelte-ignore a11y_click_events_have_key_events -->
              <div
                class="pdf-text-block-hotspot"
                class:hovered={hoveredBlockId === block.id}
                style="left: {relX}px; top: {relY}px; width: {relW}px; height: {relH}px;"
                role="button"
                tabindex="0"
                onclick={() => handleEditTextBlock(block)}
                ondblclick={() => handleEditTextBlock(block)}
                onpointerenter={() => (hoveredBlockId = block.id)}
                onpointerleave={() => (hoveredBlockId = null)}
                title="Click to edit: {block.text.slice(0, 40)}"
                aria-label="Edit text: {block.text.slice(0, 40)}"
              >
                <span class="hotspot-badge">✎ Click to edit PDF text ({block.fontSize.toFixed(0)}pt)</span>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    {/each}
  </div>

  <!-- LAYER B: #vector-layer (Rendered completed ink strokes, shapes, clouds, hatch fills, review markups, text boxes & snapshots) -->
  <svg id="vector-layer" class="vector-layer" role="presentation">
    <HatchDefs scale={tools.activePreset?.fill.hatchScale ?? 1.0} />
    <g style:transform={worldTransform}>
      {#each ink.completedStrokes.filter((st) => inserts.isStrokeVisibleForAuthorFilter(st)) as stroke (stroke.id)}
        {@const bbox = computeStrokeBBox(stroke)}
        {@const dash = getStrokeDashArray(stroke.lineStyle, stroke.widthPt)}
        {@const fillUrl = getFillForStroke(stroke)}
        {#if stroke.tool === 'shape.rect' || stroke.tool === 'review.highlight' || stroke.tool === 'review.redact' || stroke.tool === 'hatch.pick'}
          <rect
            x={bbox.minX}
            y={bbox.minY}
            width={bbox.width}
            height={bbox.height}
            fill={fillUrl}
            fill-opacity={stroke.tool === 'review.highlight' ? 0.35 : (stroke.fillOpacity ?? 0.35)}
            stroke={stroke.tool === 'review.highlight' ? 'none' : stroke.color}
            stroke-width={stroke.widthPt}
            stroke-dasharray={dash}
            opacity={stroke.opacity}
            style={stroke.tool === 'review.highlight' ? 'mix-blend-mode: multiply;' : ''}
          />
        {:else if stroke.tool === 'shape.ellipse'}
          <ellipse
            cx={bbox.cx}
            cy={bbox.cy}
            rx={bbox.width / 2}
            ry={bbox.height / 2}
            fill={fillUrl}
            fill-opacity={stroke.fillOpacity ?? 0.35}
            stroke={stroke.color}
            stroke-width={stroke.widthPt}
            stroke-dasharray={dash}
            opacity={stroke.opacity}
          />
        {:else if stroke.tool === 'shape.cloud'}
          <path
            d={generateRevisionCloudPath(stroke.points)}
            fill={fillUrl}
            fill-opacity={stroke.fillOpacity ?? 0.25}
            stroke={stroke.color}
            stroke-width={stroke.widthPt}
            stroke-dasharray={dash}
            opacity={stroke.opacity}
          />
        {:else if stroke.tool === 'shape.polygon' || stroke.tool === 'hatch.region'}
          <path
            d={generatePolylineSvgPath(stroke.points, true)}
            fill={fillUrl}
            fill-opacity={stroke.fillOpacity ?? 0.35}
            stroke={stroke.color}
            stroke-width={stroke.widthPt}
            stroke-dasharray={dash}
            opacity={stroke.opacity}
          />
        {:else if stroke.tool === 'shape.line' || stroke.tool === 'shape.arrow' || stroke.tool === 'shape.polyline'}
          {@const startMarker = stroke.points.length >= 2 ? generateLineEndingPath(stroke.points[0], stroke.points[1], stroke.startEnding, stroke.widthPt) : null}
          {@const endMarker = stroke.points.length >= 2 ? generateLineEndingPath(stroke.points[stroke.points.length - 1], stroke.points[stroke.points.length - 2], stroke.endEnding, stroke.widthPt) : null}
          <path
            d={generatePolylineSvgPath(stroke.points, false)}
            fill="none"
            stroke={stroke.color}
            stroke-width={stroke.widthPt}
            stroke-dasharray={dash}
            stroke-linecap="round"
            stroke-linejoin="round"
            opacity={stroke.opacity}
          />
          {#if startMarker}
            <path
              d={startMarker.d}
              fill={startMarker.filled ? stroke.color : 'none'}
              stroke={stroke.color}
              stroke-width={stroke.widthPt}
            />
          {/if}
          {#if endMarker}
            <path
              d={endMarker.d}
              fill={endMarker.filled ? stroke.color : 'none'}
              stroke={stroke.color}
              stroke-width={stroke.widthPt}
            />
          {/if}
        {:else if stroke.tool === 'review.underline'}
          <line
            x1={bbox.minX}
            y1={bbox.maxY}
            x2={bbox.maxX}
            y2={bbox.maxY}
            stroke={stroke.color}
            stroke-width={Math.max(1.5, stroke.widthPt)}
            opacity={stroke.opacity}
          />
        {:else if stroke.tool === 'review.strikeout' || stroke.tool === 'review.strikethrough'}
          <line
            x1={bbox.minX}
            y1={bbox.cy}
            x2={bbox.maxX}
            y2={bbox.cy}
            stroke={stroke.color}
            stroke-width={Math.max(1.5, stroke.widthPt)}
            opacity={stroke.opacity}
          />
        {:else if stroke.tool === 'review.squiggly' || stroke.tool === 'review.squiggle'}
          <path
            d={generateSquiggleSvgPath(bbox)}
            fill="none"
            stroke={stroke.color}
            stroke-width={Math.max(1.5, stroke.widthPt)}
            opacity={stroke.opacity}
          />
        {:else if (stroke.tool === 'insert.text' || stroke.tool === 'insert.textBox' || stroke.tool === 'insert.note' || stroke.tool === 'insert.latex' || stroke.tool === 'insert.signature' || stroke.tool === 'insert.audio' || stroke.tool === 'insert.stamp' || stroke.tool === 'insert.image' || stroke.tool === 'callout.single' || stroke.tool === 'callout.multi') && (stroke.text || stroke.snapshotDataUrl)}
          <g>
            {#if stroke.tool === 'callout.single' || stroke.tool === 'callout.multi'}
              <path
                d={generatePolylineSvgPath(stroke.points, false)}
                fill="none"
                stroke={stroke.color}
                stroke-width={stroke.widthPt}
                stroke-linecap="round"
                stroke-linejoin="round"
              />
            {/if}
            {#if stroke.snapshotDataUrl}
              <image
                href={stroke.snapshotDataUrl}
                x={bbox.minX}
                y={bbox.minY}
                width={bbox.width}
                height={bbox.height}
                preserveAspectRatio="xMidYMid meet"
              />
            {:else}
              <rect
                x={bbox.minX}
                y={bbox.minY}
                width={bbox.width}
                height={bbox.height}
                fill="var(--glass-bg)"
                stroke={stroke.color}
                stroke-width={stroke.tool === 'insert.stamp' ? 2.2 : 1.2}
                stroke-dasharray={stroke.tool === 'insert.stamp' || stroke.tool === 'insert.latex' ? undefined : '3 2'}
                rx="4"
              />
            {/if}
            {#if stroke.text}
              <text
                x={bbox.minX + 8}
                y={bbox.minY + (stroke.fontSize ?? 14) + 6}
                fill={stroke.color}
                font-size="{stroke.fontSize ?? 14}px"
                font-weight={stroke.tool === 'insert.stamp' ? '700' : '500'}
                font-style={stroke.tool === 'insert.signature' || stroke.tool === 'insert.latex' ? 'italic' : 'normal'}
                font-family="Inter, sans-serif"
              >
                {stroke.text}
              </text>
            {/if}
          </g>
        {:else if (stroke.tool === 'select.snapshot' || stroke.tool === 'snapshot') && stroke.snapshotDataUrl}
          <g
            transform="translate({bbox.cx}, {bbox.cy}) scale({stroke.flipH ? -1 : 1}, {stroke.flipV ? -1 : 1}) translate({-bbox.cx}, {-bbox.cy})"
          >
            <image
              href={stroke.snapshotDataUrl}
              x={bbox.minX}
              y={bbox.minY}
              width={bbox.width}
              height={bbox.height}
              preserveAspectRatio="none"
            />
            <rect
              x={bbox.minX}
              y={bbox.minY}
              width={bbox.width}
              height={bbox.height}
              fill="none"
              stroke="var(--accent)"
              stroke-width="1.5"
              stroke-dasharray="4 2"
            />
          </g>
        {:else}
          <path
            d={ink.getSvgPath(stroke)}
            fill={stroke.color}
            opacity={stroke.opacity}
            style={stroke.isHighlighter || stroke.tool === 'highlighter'
              ? 'mix-blend-mode: multiply;'
              : ''}
          />
        {/if}
      {/each}

      <!-- Selection Bounding Box & 8 Resize Handles + Rotation Handle -->
      {#if selection.hasSelection && selection.selectionBounds}
        {@const sb = selection.selectionBounds}
        <g class="selection-gizmo">
          <rect
            x={sb.minX - 4}
            y={sb.minY - 4}
            width={sb.width + 8}
            height={sb.height + 8}
            fill="var(--canvas-selection)"
            stroke="var(--accent)"
            stroke-width="1.5"
            stroke-dasharray="5 3"
          />
          <!-- Top Rotation Stem & Handle -->
          <line
            x1={sb.cx}
            y1={sb.minY - 4}
            x2={sb.cx}
            y2={sb.minY - 22}
            stroke="var(--accent)"
            stroke-width="1.5"
          />
          <circle
            cx={sb.cx}
            cy={sb.minY - 22}
            r="5"
            fill="var(--laser-core)"
            stroke="var(--accent)"
            stroke-width="2"
          />
          <!-- 8 Resize Handles -->
          {#each [
            [sb.minX - 4, sb.minY - 4],
            [sb.cx, sb.minY - 4],
            [sb.maxX + 4, sb.minY - 4],
            [sb.maxX + 4, sb.cy],
            [sb.maxX + 4, sb.maxY + 4],
            [sb.cx, sb.maxY + 4],
            [sb.minX - 4, sb.maxY + 4],
            [sb.minX - 4, sb.cy],
          ] as [hx, hy]}
            <rect
              x={hx - 4}
              y={hy - 4}
              width="8"
              height="8"
              fill="var(--laser-core)"
              stroke="var(--accent)"
              stroke-width="1.5"
              rx="1.5"
            />
          {/each}
        </g>
      {/if}
    </g>
  </svg>

  <!-- LAYER C: #wet-layer (Active stroke, active shape preview, marquee/lasso preview & laser decay) -->
  <svg
    id="wet-layer"
    class="wet-layer"
    role="presentation"
    class:cursor-pan={tools.activeToolId === 'pan'}
    class:cursor-pen={tools.activeToolId.startsWith('pen') || tools.activeToolId === 'highlighter'}
    class:cursor-eraser={tools.activeToolId.startsWith('eraser')}
    class:cursor-laser={tools.activeToolId === 'laser'}
    class:cursor-busy={documentBusy}
    onpointerdown={handlePointerDown}
    onpointermove={handlePointerMove}
    onpointerup={handlePointerUp}
    onpointercancel={handlePointerCancel}
    ondblclick={handleDoubleClick}
    oncontextmenu={handleContextMenu}
  >
    <g style:transform={worldTransform}>
      {#if ink.currentStroke}
        <path
          d={ink.getSvgPath(ink.currentStroke)}
          fill={ink.currentStroke.color}
          opacity={ink.currentStroke.opacity}
          style={ink.currentStroke.isHighlighter || ink.currentStroke.tool === 'highlighter'
            ? 'mix-blend-mode: multiply;'
            : ''}
        />
      {/if}

      {#if shapes.activeShape}
        {@const activePts = shapes.previewPoint
          ? [...shapes.activeShape.points, shapes.previewPoint]
          : shapes.activeShape.points}
        {@const abox = computeStrokeBBox({ points: activePts, bbox: shapes.activeShape.bbox })}
        {#if shapes.activeShape.tool === 'shape.cloud'}
          <path
            d={generateRevisionCloudPath(activePts)}
            fill={getFillForStroke(shapes.activeShape)}
            fill-opacity="0.25"
            stroke={shapes.activeShape.color}
            stroke-width={shapes.activeShape.widthPt}
            stroke-dasharray={getStrokeDashArray(shapes.activeShape.lineStyle, shapes.activeShape.widthPt)}
          />
        {:else if shapes.activeShape.tool === 'shape.ellipse'}
          <ellipse
            cx={abox.cx}
            cy={abox.cy}
            rx={abox.width / 2}
            ry={abox.height / 2}
            fill={getFillForStroke(shapes.activeShape)}
            fill-opacity="0.25"
            stroke={shapes.activeShape.color}
            stroke-width={shapes.activeShape.widthPt}
          />
        {:else if shapes.activeShape.tool === 'shape.rect' || shapes.activeShape.tool === 'review.highlight' || shapes.activeShape.tool === 'review.redact'}
          <rect
            x={abox.minX}
            y={abox.minY}
            width={abox.width}
            height={abox.height}
            fill={getFillForStroke(shapes.activeShape)}
            fill-opacity="0.3"
            stroke={shapes.activeShape.color}
            stroke-width={shapes.activeShape.widthPt}
          />
        {:else}
          <path
            d={generatePolylineSvgPath(
              activePts,
              shapes.activeShape.tool === 'shape.polygon' ||
                shapes.activeShape.tool === 'hatch.region'
            )}
            fill={getFillForStroke(shapes.activeShape)}
            fill-opacity="0.25"
            stroke={shapes.activeShape.color}
            stroke-width={shapes.activeShape.widthPt}
            stroke-dasharray={getStrokeDashArray(shapes.activeShape.lineStyle, shapes.activeShape.widthPt)}
          />
        {/if}
      {/if}

      {#if selection.marqueeRect}
        {@const mx = Math.min(selection.marqueeRect.x0, selection.marqueeRect.x1)}
        {@const my = Math.min(selection.marqueeRect.y0, selection.marqueeRect.y1)}
        {@const mw = Math.abs(selection.marqueeRect.x1 - selection.marqueeRect.x0)}
        {@const mh = Math.abs(selection.marqueeRect.y1 - selection.marqueeRect.y0)}
        <rect
          x={mx}
          y={my}
          width={mw}
          height={mh}
          fill="var(--canvas-selection)"
          stroke="var(--accent)"
          stroke-width="1.5"
          stroke-dasharray="4 2"
        />
      {/if}

      {#if selection.lassoPoints.length > 1}
        <path
          d={generatePolylineSvgPath(selection.lassoPoints, true)}
          fill="var(--canvas-selection)"
          stroke="var(--accent)"
          stroke-width="1.5"
          stroke-dasharray="4 2"
        />
      {/if}

      {#if ink.laserPoints.length > 1}
        <path d={ink.getLaserSvgPath()} fill="var(--laser-glow)" opacity="0.85" />
        <path d={ink.getLaserCoreSvgPath()} fill="var(--laser-core)" opacity="1.0" />
      {/if}

      <!-- Phase D5 CAD Measurements, OSNAP Indicators & Interactive Digital Ruler -->
      <MeasureOverlay activePageIndex={0} />
    </g>
  </svg>

  <!-- Phase D5 Page Scale Calibration Modal -->
  <CalibrationModal onSaveScales={savePageScalesToSidecar} />

  <!-- Phase D6 Annotation History, Quantity Takeoff CSV, Markup Library & Insert Suite Modals -->
  <AnnotationLedger
    activePdfPath={loadedFilePath ?? appState.currentFile}
    totalPages={viewport.pages.length}
    onNotice={(msg) => {
      statusNotice = msg;
    }}
  />

  <!-- Inline Rich Text Box / Structural PDF Text Block Editor Overlay -->
  {#if shapes.editingTextBox}
    <div
      class="inline-text-editor glass-panel"
      data-testid="inline-pdf-text-editor"
      role="dialog"
      tabindex="-1"
      aria-label="Inline Text Editor"
      onpointerdown={(e) => e.stopPropagation()}
    >
      <div class="editor-header">
        <span>
          {shapes.editingTextBox.isStructural
            ? 'Structural PDF Text Edit (Native Stream)'
            : 'Canvas Rich Text Annotation'}
        </span>
      </div>
      <textarea
        class="editor-textarea"
        rows="3"
        bind:value={shapes.editingTextBox.text}
        aria-label="Annotation or PDF Text Content"
      ></textarea>
      <div class="editor-actions">
        <label class="editor-size-label">
          Size:
          <input
            type="number"
            min="8"
            max="72"
            bind:value={shapes.editingTextBox.fontSize}
            class="editor-size-input"
          />
          pt
        </label>
        <div class="editor-btns">
          <button
            type="button"
            class="top-action-btn"
            onclick={() => shapes.cancelActiveShape()}
          >
            Cancel
          </button>
          <button
            type="button"
            class="top-action-btn primary-btn"
            onclick={commitStructuralTextOrAnnotation}
          >
            Commit Text
          </button>
        </div>
      </div>
    </div>
  {/if}

  <!-- Dockable Markup Toolbar (Top, Left, Right, Floating) -->
  <DockableToolbar
    onToggleRadial={toggleRadialAt}
    onOpenShortcuts={() => {
      shortcutsVisible = true;
    }}
    onToggleStructuralEdit={() => {
      editMode = editMode === 'structural' ? 'markup' : 'structural';
    }}
    isStructuralEdit={editMode === 'structural'}
  />

  <!-- Pinned Favorites Bar -->
  <FavoritesBar onPersistPrefs={persistStudioPrefs} />

  <!-- Active Tool Reactive Property Pill -->
  <PropertyPill />

  <!-- Floating Media / Insert Configuration Prompt Pill -->
  {#if isInsertMediaTool(tools.currentToolId)}
    <div class="insert-config-pill glass-panel" data-testid="insert-config-pill">
      <span>Click canvas to place {studioToolLabel(tools.currentToolId, tools.activeToolLabel)}, or configure options:</span>
      <button
        type="button"
        class="top-action-btn primary-btn"
        onclick={() => handleConfigureInsertTool(tools.currentToolId)}
      >
        Configure...
      </button>
    </div>
  {/if}

  <!-- 8-Sector Radial Menu (Right-click / Pen Barrel / UI Trigger / '/') -->
  <RadialMenu bind:visible={radialVisible} x={radialPos.x} y={radialPos.y} />

  <!-- Custom Keyboard Shortcuts Settings Modal -->
  <ShortcutSettings bind:visible={shortcutsVisible} onClose={persistStudioPrefs} />
</div>

<style>
  .spatial-viewport {
    position: relative;
    width: 100%;
    height: 100%;
    min-height: calc(100vh - 28px);
    overflow: hidden;
    background-color: var(--canvas-bg);
    background-image:
      radial-gradient(circle, var(--canvas-grid-minor) 1px, transparent 1px),
      radial-gradient(circle, var(--canvas-grid-major) 1.5px, transparent 1.5px);
    background-size: 16px 16px, 96px 96px;
    background-position: 0 0, 0 0;
    user-select: none;
    touch-action: none;
    cursor: default;
    outline: none;
  }

  .pdf-layer {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    transform-origin: 0 0;
    will-change: transform;
    pointer-events: none;
    z-index: 1;
  }

  .vector-layer {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    pointer-events: none;
    overflow: visible;
    z-index: 5;
  }

  .pdf-password-text {
    margin: 0;
  }

  .pdf-password-error {
    color: var(--danger);
    font-size: var(--font-size-footnote);
  }

  .wet-layer {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    pointer-events: auto;
    touch-action: none;
    overflow: visible;
    z-index: 10;
  }

  .vector-layer g,
  .wet-layer g {
    transform-origin: 0 0;
    will-change: transform;
  }

  .wet-layer.cursor-busy {
    cursor: progress;
  }

  .cursor-pan {
    cursor: grab;
  }
  .cursor-pan:active {
    cursor: grabbing;
  }
  .cursor-pen {
    cursor: crosshair;
  }
  .cursor-eraser {
    cursor: cell;
  }
  .cursor-laser {
    cursor: crosshair;
  }

  .spatial-page {
    position: absolute;
    background-color: var(--laser-core);
    box-shadow: 0 10px 30px var(--canvas-page-shadow);
    border-radius: 4px;
    overflow: hidden;
    pointer-events: auto;
  }

  .page-badge {
    position: absolute;
    top: -24px;
    left: 0;
    font-size: 11px;
    font-weight: 600;
    color: var(--glass-text);
    opacity: 0.7;
    white-space: nowrap;
  }

  .page-render-surface {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--laser-core);
  }

  .page-pdf-canvas {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    pointer-events: none;
  }

  .page-template-overlay {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    pointer-events: none;
    opacity: 0.72;
  }

  /* 4-Zone Frosted Glass Top Command Bar */
  .studio-top-bar-host {
    position: absolute;
    top: 12px;
    left: 12px;
    right: 12px;
    z-index: 160;
    display: flex;
    justify-content: center;
    pointer-events: none;
    container-type: inline-size;
    container-name: studio-top;
  }

  .studio-top-bar {
    position: relative;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 12px;
    background: var(--glass-bg);
    backdrop-filter: blur(var(--glass-blur));
    -webkit-backdrop-filter: blur(var(--glass-blur));
    border: 1px solid var(--glass-border);
    box-shadow: var(--glass-shadow);
    border-radius: 9999px;
    color: var(--glass-text);
    pointer-events: auto;
    /* Never wider than the canvas (the old 100vw cap ignored the sidebar and clipped both ends). */
    max-width: 100%;
    overflow-x: auto;
    scrollbar-width: none;
  }

  /* Narrow canvas: labels give way to icons; names stay in tooltips and aria-labels. */
  @container studio-top (max-width: 1100px) {
    .studio-top-bar .collapsible-label {
      display: none;
    }
    .studio-top-bar {
      gap: 6px;
      padding: 4px 8px;
    }
  }

  .studio-top-bar :is(.top-btn, .top-action-btn, .top-select):disabled {
    opacity: 0.4;
    cursor: default;
  }

  .ledger-btn {
    display: inline-flex;
    align-items: center;
    gap: 5px;
  }

  .studio-empty-state {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    z-index: 40;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
    max-width: 380px;
    padding: 28px 32px;
    text-align: center;
    background: var(--glass-bg);
    backdrop-filter: blur(var(--glass-blur));
    -webkit-backdrop-filter: blur(var(--glass-blur));
    border: 1px solid var(--glass-border);
    box-shadow: var(--glass-shadow);
    border-radius: 18px;
    color: var(--glass-text);
  }

  .studio-empty-state h2 {
    margin: 4px 0 0;
    font-size: 17px;
    font-weight: 600;
  }

  .studio-empty-state p {
    margin: 0 0 6px;
    font-size: 13px;
    line-height: 1.45;
    opacity: 0.75;
  }

  .top-zone {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-shrink: 0;
  }

  .top-zone-divider {
    width: 1px;
    height: 18px;
    background: var(--glass-border);
    flex-shrink: 0;
  }

  .mode-segmented-switch {
    display: flex;
    background: var(--canvas-selection);
    border-radius: 9999px;
    padding: 2px;
    gap: 2px;
  }

  .page-indicator-badge {
    font-size: 11px;
    font-weight: 600;
    color: var(--glass-text);
    padding: 2px 8px;
    background: var(--canvas-selection);
    border-radius: 9999px;
    white-space: nowrap;
    opacity: 0.85;
  }

  .top-btn {
    display: flex;
    align-items: center;
    gap: 4px;
    border: none;
    background: transparent;
    color: var(--glass-text);
    padding: 4px 8px;
    border-radius: 9999px;
    font-size: 11px;
    font-weight: 500;
    cursor: pointer;
    white-space: nowrap;
    transition: background var(--motion-duration) ease, color var(--motion-duration) ease;
  }

  .top-btn:hover {
    background: var(--canvas-selection);
  }

  .top-btn.active {
    background: var(--accent);
    color: var(--laser-core);
  }

  .top-icon-btn {
    width: 26px;
    height: 26px;
    display: flex;
    align-items: center;
    justify-content: center;
    border: none;
    background: transparent;
    color: var(--glass-text);
    border-radius: 4px;
    cursor: pointer;
    font-size: 13px;
    font-weight: 600;
    transition: background 0.15s ease;
  }

  .top-icon-btn:hover:not(:disabled) {
    background: var(--canvas-selection);
  }

  .top-icon-btn:disabled {
    opacity: 0.35;
    cursor: not-allowed;
  }

  .top-select {
    background: var(--canvas-selection);
    color: var(--glass-text);
    border: 1px solid var(--glass-border);
    border-radius: 9999px;
    padding: 3px 8px;
    font-size: 11px;
    font-weight: 600;
    cursor: pointer;
    outline: none;
  }

  .top-action-btn {
    border: 1px solid var(--glass-border);
    background: var(--canvas-selection);
    color: var(--glass-text);
    padding: 4px 10px;
    border-radius: 9999px;
    font-size: 11px;
    font-weight: 600;
    cursor: pointer;
    white-space: nowrap;
    transition: background 0.15s ease, color 0.15s ease;
  }

  .top-action-btn:hover {
    background: var(--accent);
    color: var(--laser-core);
  }

  .primary-btn {
    background: var(--accent);
    color: var(--laser-core);
  }

  .danger-btn {
    background: var(--danger);
    color: var(--laser-core);
  }

  .studio-status-banner {
    position: absolute;
    top: 64px;
    left: 50%;
    transform: translateX(-50%);
    z-index: 165;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 14px;
    background: var(--glass-bg);
    border: 1px solid var(--glass-border);
    border-radius: 9999px;
    font-size: 11px;
    color: var(--glass-text);
    pointer-events: auto;
  }

  .banner-close {
    border: none;
    background: transparent;
    color: var(--glass-text);
    cursor: pointer;
    padding: 2px;
  }

  .selection-action-bar {
    position: absolute;
    top: 64px;
    left: 50%;
    transform: translateX(-50%);
    z-index: 160;
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 5px 12px;
    background: var(--glass-bg);
    backdrop-filter: blur(var(--glass-blur));
    border: 1px solid var(--glass-border);
    box-shadow: var(--glass-shadow);
    border-radius: 9999px;
    color: var(--glass-text);
    pointer-events: auto;
    font-size: 11px;
  }

  .sel-count {
    font-weight: 600;
    padding: 0 4px;
  }

  .sel-btn {
    border: none;
    background: transparent;
    color: var(--glass-text);
    padding: 3px 7px;
    border-radius: 6px;
    font-size: 11px;
    font-weight: 500;
    cursor: pointer;
  }

  .sel-btn:hover,
  .sel-btn.active {
    background: var(--accent);
    color: var(--laser-core);
  }

  .inline-text-editor {
    position: absolute;
    top: 120px;
    left: 50%;
    transform: translateX(-50%);
    z-index: 170;
    width: 340px;
    padding: 12px;
    background: var(--glass-bg);
    backdrop-filter: blur(var(--glass-blur));
    border: 1px solid var(--glass-border);
    box-shadow: var(--glass-shadow);
    border-radius: 12px;
    color: var(--glass-text);
    pointer-events: auto;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .editor-header {
    font-size: 12px;
    font-weight: 600;
  }

  .editor-textarea {
    width: 100%;
    background: var(--canvas-bg);
    color: var(--glass-text);
    border: 1px solid var(--glass-border);
    border-radius: 6px;
    padding: 6px;
    font-size: 12px;
    resize: vertical;
  }

  .editor-actions {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .editor-size-label {
    font-size: 11px;
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .editor-size-input {
    width: 48px;
    background: var(--canvas-bg);
    color: var(--glass-text);
    border: 1px solid var(--glass-border);
    border-radius: 4px;
    padding: 2px 4px;
    font-size: 11px;
  }

  .editor-btns {
    display: flex;
    gap: 6px;
  }

  .structural-edit-banner {
    position: absolute;
    top: 52px;
    left: 50%;
    transform: translateX(-50%);
    z-index: 160;
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 8px 16px;
    background: var(--glass-bg);
    backdrop-filter: blur(var(--glass-blur));
    -webkit-backdrop-filter: blur(var(--glass-blur));
    border: 1px solid var(--accent);
    border-radius: 9999px;
    box-shadow: 0 4px 16px var(--glass-shadow);
    color: var(--glass-text);
    font-size: 12px;
  }

  .banner-content {
    display: flex;
    align-items: center;
    gap: 8px;
    font-weight: 500;
  }

  .done-btn {
    padding: 4px 12px;
    font-size: 11px;
    font-weight: 600;
  }

  .text-blocks-overlay {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    pointer-events: none;
    z-index: 10;
  }

  .pdf-text-block-hotspot {
    position: absolute;
    pointer-events: auto;
    border: 1px dashed var(--accent);
    background: rgba(37, 99, 235, 0.08);
    border-radius: 2px;
    cursor: text;
    transition: background 0.1s ease, border-color 0.1s ease;
  }

  .pdf-text-block-hotspot:hover,
  .pdf-text-block-hotspot.hovered {
    background: rgba(37, 99, 235, 0.2);
    border: 1.5px solid var(--focus-ring);
    box-shadow: 0 0 6px rgba(37, 99, 235, 0.3);
  }

  .hotspot-badge {
    position: absolute;
    bottom: calc(100% + 2px);
    left: 0;
    background: var(--accent);
    color: var(--laser-core);
    font-size: 10px;
    font-weight: 600;
    padding: 1px 6px;
    border-radius: 4px;
    white-space: nowrap;
    opacity: 0;
    pointer-events: none;
    transition: opacity 0.1s ease;
    box-shadow: 0 2px 6px var(--glass-shadow);
  }

  .pdf-text-block-hotspot:hover .hotspot-badge,
  .pdf-text-block-hotspot.hovered .hotspot-badge {
    opacity: 1;
  }

  .insert-config-pill {
    position: absolute;
    bottom: 76px;
    left: 50%;
    transform: translateX(-50%);
    z-index: 140;
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 6px 14px;
    background: var(--glass-bg);
    backdrop-filter: blur(var(--glass-blur));
    -webkit-backdrop-filter: blur(var(--glass-blur));
    border: 1px solid var(--glass-border);
    border-radius: 9999px;
    box-shadow: var(--glass-shadow);
    color: var(--glass-text);
    font-size: 12px;
  }
</style>

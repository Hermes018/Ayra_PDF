<script lang="ts">
  import { ink } from '$lib/core/inkEngine.svelte';
  import { measureEngine } from '$lib/core/measureEngine.svelte';
  import { selection } from '$lib/core/selectionEngine.svelte';
  import {
    inserts,
    compileLatexToVector,
    applyGrayscaleThresholdToRgba,
    type SignatureMode,
  } from '$lib/core/insertEngine.svelte';
  import StudioIcon from './StudioIcon.svelte';

  let {
    activePdfPath = null,
    totalPages = 1,
    onNotice,
  }: {
    activePdfPath?: string | null;
    totalPages?: number;
    onNotice?: (msg: string) => void;
  } = $props();

  let activeTab = $state<'ledger' | 'library' | 'workspace'>('ledger');
  let filterQuery = $state<string>('');
  let filterKind = $state<'all' | 'annotations' | 'measurements'>('all');
  let sortBy = $state<'page' | 'type'>('page');
  let newLibraryTitle = $state<string>('Custom Markup Stamp');
  let docTagsInput = $state<string>('structural, rev-2');
  let docFolderInput = $state<string>('Project Blueprints');

  // Signature draw points for Draw mode
  let sigDrawPoints = $state<Array<{ x: number; y: number; p: number }>>([]);
  let isDrawingSig = $state(false);
  let webcamPreviewDataUrl = $state<string | null>(null);

  let latexPreview = $derived(compileLatexToVector(inserts.latexDraft));

  interface LedgerEntry {
    id: string;
    category: 'annotation' | 'measurement';
    pageIndex: number;
    typeLabel: string;
    summary: string;
  }

  let ledgerRows = $derived.by(() => {
    const rows: LedgerEntry[] = [];
    if (filterKind === 'all' || filterKind === 'annotations') {
      for (const s of ink.completedStrokes) {
        if (!inserts.isStrokeVisibleForAuthorFilter(s)) continue;
        rows.push({
          id: s.id,
          category: 'annotation',
          pageIndex: s.pageIndex ?? 0,
          typeLabel: s.tool,
          summary: s.text ? s.text : `${s.points.length} pts (${s.widthPt}pt)`,
        });
      }
    }
    if (filterKind === 'all' || filterKind === 'measurements') {
      for (const m of measureEngine.measurements) {
        rows.push({
          id: m.id,
          category: 'measurement',
          pageIndex: m.pageIndex,
          typeLabel: `measure.${m.kind}`,
          summary: m.formattedLabel,
        });
      }
    }

    const q = filterQuery.trim().toLowerCase();
    const filtered = q
      ? rows.filter(
          (r) =>
            r.typeLabel.toLowerCase().includes(q) ||
            r.summary.toLowerCase().includes(q) ||
            String(r.pageIndex + 1).includes(q)
        )
      : rows;

    return filtered.sort((a, b) => {
      if (sortBy === 'page') return a.pageIndex - b.pageIndex;
      return a.typeLabel.localeCompare(b.typeLabel);
    });
  });

  async function handleExportTakeoffCsv() {
    const base = activePdfPath ? activePdfPath.replace(/\.pdf$/i, '') : 'studio-takeoff';
    const outPath = `${base}.takeoff.csv`;
    const records = await inserts.exportTakeoffCsv(outPath);
    onNotice?.(`Exported ${records.length} takeoff measurements to ${outPath}`);
  }

  async function handleSaveSelectedToLibrary() {
    const target =
      selection.selectedIds.length > 0
        ? ink.completedStrokes.find((s) => selection.selectedIds.includes(s.id))
        : ink.completedStrokes[ink.completedStrokes.length - 1];
    if (!target) {
      onNotice?.('Draw or select a markup on the canvas first to save to the library.');
      return;
    }
    const saved = await inserts.saveStrokeToMarkupLibrary(target, newLibraryTitle);
    onNotice?.(`Saved "${saved.title}" to Reusable Markup Library.`);
  }

  function handleSigPointerDown(e: PointerEvent) {
    isDrawingSig = true;
    const rect = (e.currentTarget as SVGSVGElement).getBoundingClientRect();
    sigDrawPoints = [{ x: e.clientX - rect.left, y: e.clientY - rect.top, p: e.pressure || 0.6 }];
  }

  function handleSigPointerMove(e: PointerEvent) {
    if (!isDrawingSig) return;
    const rect = (e.currentTarget as SVGSVGElement).getBoundingClientRect();
    sigDrawPoints = [
      ...sigDrawPoints,
      { x: e.clientX - rect.left, y: e.clientY - rect.top, p: e.pressure || 0.6 },
    ];
  }

  function handleSigPointerUp() {
    isDrawingSig = false;
  }

  function runThresholdScanSimulation() {
    if (typeof document === 'undefined') return;
    const canvas = document.createElement('canvas');
    canvas.width = 180;
    canvas.height = 56;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    // Paint paper background + dark signature ink, then run `applyGrayscaleThresholdToRgba`
    ctx.fillStyle = 'rgb(245, 245, 245)';
    ctx.fillRect(0, 0, 180, 56);
    ctx.fillStyle = 'rgb(18, 24, 38)';
    ctx.font = 'italic 20px serif';
    ctx.fillText(inserts.signatureTypedName || 'Verified Sign', 12, 36);

    const imgData = ctx.getImageData(0, 0, 180, 56);
    const filtered = applyGrayscaleThresholdToRgba(imgData.data, inserts.signatureThreshold);
    imgData.data.set(filtered);
    ctx.putImageData(imgData, 0, 0);
    webcamPreviewDataUrl = canvas.toDataURL('image/png');
  }
</script>

<!-- Collapsible Left-Side Annotation History & Quantity Takeoff Drawer -->
{#if inserts.ledgerOpen}
  <aside
    class="annotation-ledger-drawer glass-panel"
    aria-label="Annotation History and Quantity Takeoff Drawer"
    onpointerdown={(e) => e.stopPropagation()}
  >
    <header class="drawer-header">
      <div class="drawer-tabs">
        <button
          type="button"
          class="tab-btn"
          class:active={activeTab === 'ledger'}
          onclick={() => (activeTab = 'ledger')}
        >
          Ledger ({ledgerRows.length})
        </button>
        <button
          type="button"
          class="tab-btn"
          class:active={activeTab === 'library'}
          onclick={() => {
            activeTab = 'library';
            inserts.loadMarkupLibraryFromBackend();
          }}
        >
          Vault ({inserts.markupLibrary.length})
        </button>
        <button
          type="button"
          class="tab-btn"
          class:active={activeTab === 'workspace'}
          onclick={() => {
            activeTab = 'workspace';
            inserts.loadWorkspaceDocuments();
          }}
        >
          Docs ({inserts.workspaceDocs.length})
        </button>
      </div>
      <button
        type="button"
        class="icon-close-btn"
        onclick={() => (inserts.ledgerOpen = false)}
        aria-label="Close Annotation Ledger"
      >
        <StudioIcon name="close" size={13} />
      </button>
    </header>

    {#if activeTab === 'ledger'}
      <div class="ledger-controls">
        <input
          type="search"
          class="ledger-search"
          placeholder="Filter by tool, label, or page..."
          bind:value={filterQuery}
          aria-label="Filter Annotations and Measurements"
        />
        <div class="ledger-filter-row">
          <select
            class="ledger-select"
            bind:value={filterKind}
            aria-label="Filter Category"
          >
            <option value="all">All Items</option>
            <option value="annotations">Markups ({ink.completedStrokes.length})</option>
            <option value="measurements">Takeoff ({measureEngine.measurements.length})</option>
          </select>
          <select
            class="ledger-select"
            bind:value={inserts.authorFilter}
            aria-label="Filter by Author"
          >
            <option value="ALL">Author: All</option>
            {#each inserts.getAvailableAuthors() as authorName}
              <option value={authorName}>{authorName}</option>
            {/each}
          </select>
          <select
            class="ledger-select"
            bind:value={sortBy}
            aria-label="Sort Ledger By"
          >
            <option value="page">Sort: Page</option>
            <option value="type">Sort: Tool</option>
          </select>
        </div>
        <div class="ledger-filter-row">
          <button
            type="button"
            class="ledger-action-btn"
            onclick={() => {
              const converted = inserts.convertSelectedInkToText();
              if (converted) {
                onNotice?.(`Converted ink to editable text: "${converted.text}"`);
              } else {
                onNotice?.('Draw or select a freehand ink stroke first.');
              }
            }}
            title="Convert selected handwritten ink strokes to editable text box (8.10)"
          >
            Ink → Text
          </button>
          <button
            type="button"
            class="ledger-action-btn"
            onclick={async () => {
              const regions = await inserts.detectTablesAndFields(activePdfPath, 0);
              onNotice?.(`Detected ${regions.length} table/form regions on page.`);
            }}
            title="Auto-detect tables and fillable form fields on page (8.12)"
          >
            Auto-Detect Tables
          </button>
        </div>
      </div>

      <div class="ledger-table-wrap">
        {#if ledgerRows.length === 0}
          <div class="empty-ledger">No annotations or measurements recorded yet.</div>
        {:else}
          <table class="ledger-table">
            <thead>
              <tr>
                <th>Pg</th>
                <th>Type</th>
                <th>Value / Content</th>
              </tr>
            </thead>
            <tbody>
              {#each ledgerRows as row (row.id)}
                <tr
                  class="ledger-row"
                  onclick={() => {
                    if (row.category === 'annotation') {
                      selection.selectedIds = [row.id];
                    }
                  }}
                >
                  <td>P{row.pageIndex + 1}</td>
                  <td><span class="type-pill">{row.typeLabel}</span></td>
                  <td class="summary-cell">{row.summary}</td>
                </tr>
              {/each}
            </tbody>
          </table>
        {/if}
      </div>

      <footer class="drawer-footer">
        <button
          type="button"
          class="ledger-action-btn primary"
          onclick={handleExportTakeoffCsv}
          title="Export all calibrated measurements to CSV spreadsheet"
        >
          Export Takeoff (CSV)
        </button>
        <button
          type="button"
          class="ledger-action-btn"
          onclick={() => (inserts.pastePagesModalOpen = true)}
          title="Stamp selected markup across page ranges"
        >
          Paste to Pages...
        </button>
      </footer>
    {:else if activeTab === 'library'}
      <!-- Reusable Markup Library Vault -->
      <div class="library-controls">
        <input
          type="text"
          class="ledger-search"
          placeholder="Name for selected markup..."
          bind:value={newLibraryTitle}
          aria-label="New Markup Library Preset Title"
        />
        <button
          type="button"
          class="ledger-action-btn primary"
          onclick={handleSaveSelectedToLibrary}
        >
          + Save Current Markup to Vault
        </button>
      </div>

      <div class="library-list">
        {#each inserts.markupLibrary as item (item.id)}
          <div class="library-card">
            <div class="lib-meta">
              <span class="lib-title">{item.title}</span>
              <span class="lib-sub">{item.category} • {item.tool_id}</span>
            </div>
            <button
              type="button"
              class="ledger-action-btn"
              onclick={() => {
                inserts.stampFromMarkupLibrary(item, 200, 220, 0);
                onNotice?.(`Stamped "${item.title}" onto canvas.`);
              }}
            >
              Stamp
            </button>
          </div>
        {/each}
      </div>
    {:else}
      <!-- Workspace Document Tags, Folders, Favorites & Watch-Folder Auto-Ingest (7.10 & 8.7) -->
      <div class="library-controls">
        <input
          type="text"
          class="ledger-search"
          placeholder="Folder name (e.g. Project Blueprints)..."
          bind:value={docFolderInput}
          aria-label="Workspace Document Folder"
        />
        <input
          type="text"
          class="ledger-search"
          placeholder="Comma-separated tags (e.g. structural, rev-2)..."
          bind:value={docTagsInput}
          aria-label="Workspace Document Tags"
        />
        <button
          type="button"
          class="ledger-action-btn primary"
          onclick={async () => {
            const path = activePdfPath || 'untitled-drawing.pdf';
            const title = path.split(/[\\/]/).pop() || path;
            const tags = docTagsInput
              .split(',')
              .map((t) => t.trim())
              .filter(Boolean);
            await inserts.upsertWorkspaceDocument({
              path,
              title,
              folder: docFolderInput.trim() || 'General',
              tags,
              favorite: true,
              last_opened_ms: Date.now(),
            });
            onNotice?.(`Tagged & favorited "${title}" in ${docFolderInput}.`);
          }}
        >
          ★ Favorite & Tag Active PDF
        </button>

        <div class="ledger-filter-row">
          <input
            type="text"
            class="ledger-search"
            placeholder="Local watch folder path..."
            bind:value={inserts.watchFolderPath}
            aria-label="Watch Folder Path"
          />
          <button
            type="button"
            class="ledger-action-btn"
            onclick={async () => {
              const found = await inserts.scanWatchFolder();
              onNotice?.(`Watch folder scanned: ${found.length} PDFs ingested.`);
            }}
          >
            Scan
          </button>
        </div>
      </div>

      <div class="library-list">
        {#each inserts.workspaceDocs as doc (doc.path)}
          <div class="library-card">
            <div class="lib-meta">
              <span class="lib-title">{doc.favorite ? '★ ' : ''}{doc.title}</span>
              <span class="lib-sub">{doc.folder} • {doc.tags.join(', ')}</span>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </aside>
{/if}

<!-- LaTeX Vector Equation Modal (`insert.latex`) -->
{#if inserts.latexModalOpen}
  <div
    class="studio-modal-card glass-panel"
    role="dialog"
    tabindex="-1"
    aria-label="LaTeX Equation Vector Editor"
    onpointerdown={(e) => e.stopPropagation()}
  >
    <div class="modal-head">
      <span>LaTeX Mathematical Vector Stamp (Offline Engine)</span>
      <button type="button" class="icon-close-btn" onclick={() => (inserts.latexModalOpen = false)}>
        <StudioIcon name="close" size={12} />
      </button>
    </div>
    <input
      type="text"
      class="ledger-search"
      bind:value={inserts.latexDraft}
      aria-label="LaTeX Formula Source"
    />
    <div class="latex-preview-box">
      <svg
        width={latexPreview.widthPt}
        height={latexPreview.heightPt}
        viewBox="0 0 {latexPreview.widthPt} {latexPreview.heightPt}"
      >
        <path
          d={latexPreview.svgPathD}
          fill="none"
          stroke="var(--accent)"
          stroke-width="1.5"
        />
        <text
          x="14"
          y={Math.round(latexPreview.heightPt / 2) + 5}
          fill="var(--glass-text)"
          font-size="14"
          font-family="serif"
        >
          {latexPreview.formattedDisplay}
        </text>
      </svg>
    </div>
    <div class="modal-actions">
      <button type="button" class="ledger-action-btn" onclick={() => (inserts.latexModalOpen = false)}>
        Cancel
      </button>
      <button
        type="button"
        class="ledger-action-btn primary"
        onclick={() => inserts.stampLatexEquation()}
      >
        Stamp Vector Equation
      </button>
    </div>
  </div>
{/if}

<!-- Signature Modal (`insert.signature`: Draw / Type / Webcam Threshold) -->
{#if inserts.signatureModalOpen}
  <div
    class="studio-modal-card glass-panel"
    role="dialog"
    tabindex="-1"
    aria-label="Signature Capture Modal"
    onpointerdown={(e) => e.stopPropagation()}
  >
    <div class="modal-head">
      <span>Architectural Signature Stamp</span>
      <button
        type="button"
        class="icon-close-btn"
        onclick={() => (inserts.signatureModalOpen = false)}
      >
        <StudioIcon name="close" size={12} />
      </button>
    </div>
    <div class="drawer-tabs">
      {#each ['draw', 'type', 'webcam'] as mode}
        <button
          type="button"
          class="tab-btn"
          class:active={inserts.signatureMode === mode}
          onclick={() => (inserts.signatureMode = mode as SignatureMode)}
        >
          {mode.toUpperCase()}
        </button>
      {/each}
    </div>

    {#if inserts.signatureMode === 'draw'}
      <svg
        class="sig-pad"
        width="260"
        height="76"
        onpointerdown={handleSigPointerDown}
        onpointermove={handleSigPointerMove}
        onpointerup={handleSigPointerUp}
        role="img"
        aria-label="Freehand Signature Pad"
      >
        {#if sigDrawPoints.length > 1}
          <path
            d={sigDrawPoints
              .map((p, i) => `${i === 0 ? 'M' : 'L'} ${p.x.toFixed(1)} ${p.y.toFixed(1)}`)
              .join(' ')}
            fill="none"
            stroke="var(--accent)"
            stroke-width="2.2"
            stroke-linecap="round"
          />
        {/if}
      </svg>
    {:else if inserts.signatureMode === 'type'}
      <input
        type="text"
        class="ledger-search"
        bind:value={inserts.signatureTypedName}
        aria-label="Typed Signature Name"
      />
    {:else}
      <div class="webcam-controls">
        <label class="thresh-label">
          BG Removal Threshold ({inserts.signatureThreshold}):
          <input
            type="range"
            min="60"
            max="230"
            bind:value={inserts.signatureThreshold}
            onchange={runThresholdScanSimulation}
          />
        </label>
        <button type="button" class="ledger-action-btn" onclick={runThresholdScanSimulation}>
          Capture & Threshold Scan
        </button>
        {#if webcamPreviewDataUrl}
          <img
            src={webcamPreviewDataUrl}
            alt="Thresholded transparent signature preview"
            class="sig-img-preview"
          />
        {/if}
      </div>
    {/if}

    <div class="modal-actions">
      <button
        type="button"
        class="ledger-action-btn"
        onclick={() => (inserts.signatureModalOpen = false)}
      >
        Cancel
      </button>
      <button
        type="button"
        class="ledger-action-btn primary"
        onclick={() =>
          inserts.stampSignature({
            mode: inserts.signatureMode,
            typedName: inserts.signatureTypedName,
            drawnPoints: sigDrawPoints,
            dataUrl: webcamPreviewDataUrl ?? undefined,
          })}
      >
        Stamp Signature
      </button>
    </div>
  </div>
{/if}

<!-- Voice Audio Note Modal (`insert.audio`: MediaRecorder + PDF `/Sound`) -->
{#if inserts.audioModalOpen}
  <div
    class="studio-modal-card glass-panel"
    role="dialog"
    tabindex="-1"
    aria-label="Voice Audio Note Recorder"
    onpointerdown={(e) => e.stopPropagation()}
  >
    <div class="modal-head">
      <span>Voice Audio Note (PDF /Sound Attachment)</span>
      <button type="button" class="icon-close-btn" onclick={() => (inserts.audioModalOpen = false)}>
        <StudioIcon name="close" size={12} />
      </button>
    </div>
    <div class="waveform-bars">
      {#each inserts.liveWaveform as bar}
        <span class="wave-bar" style:height="{Math.max(6, Math.round(bar * 0.36))}px"></span>
      {/each}
    </div>
    <div class="modal-actions">
      {#if !inserts.isRecordingAudio}
        <button
          type="button"
          class="ledger-action-btn"
          onclick={() => inserts.startAudioRecording()}
        >
          ● Start Recording
        </button>
      {/if}
      <button
        type="button"
        class="ledger-action-btn primary"
        onclick={() => inserts.stopAndAttachAudioNote(activePdfPath)}
      >
        ■ Commit Audio Note to Page
      </button>
    </div>
  </div>
{/if}

<!-- Multi-Page Advanced Paste Modal (`Paste to Pages...`) -->
{#if inserts.pastePagesModalOpen}
  <div
    class="studio-modal-card glass-panel"
    role="dialog"
    tabindex="-1"
    aria-label="Multi-Page Advanced Paste"
    onpointerdown={(e) => e.stopPropagation()}
  >
    <div class="modal-head">
      <span>Paste Selected Markup to Pages...</span>
      <button
        type="button"
        class="icon-close-btn"
        onclick={() => (inserts.pastePagesModalOpen = false)}
      >
        <StudioIcon name="close" size={12} />
      </button>
    </div>
    <div class="drawer-tabs">
      {#each ['All', 'Even', 'Odd', '1-5'] as presetRange}
        <button
          type="button"
          class="tab-btn"
          class:active={inserts.pasteRangeSpec === presetRange}
          onclick={() => (inserts.pasteRangeSpec = presetRange)}
        >
          {presetRange}
        </button>
      {/each}
    </div>
    <input
      type="text"
      class="ledger-search"
      bind:value={inserts.pasteRangeSpec}
      placeholder="All, Even, Odd, or 1-5, 8"
      aria-label="Target Page Range Specification"
    />
    <div class="modal-actions">
      <button
        type="button"
        class="ledger-action-btn"
        onclick={() => (inserts.pastePagesModalOpen = false)}
      >
        Cancel
      </button>
      <button
        type="button"
        class="ledger-action-btn primary"
        onclick={() => {
          const created = inserts.pasteStrokesToPages(
            inserts.pasteRangeSpec,
            Math.max(2, totalPages)
          );
          onNotice?.(`Pasted ${created.length} markup copies across (${inserts.pasteRangeSpec}).`);
        }}
      >
        Apply Multi-Page Stamp
      </button>
    </div>
  </div>
{/if}

<style>
  .annotation-ledger-drawer {
    position: absolute;
    top: 68px;
    left: 68px;
    bottom: 84px;
    width: 320px;
    z-index: 165;
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 12px;
    background: var(--glass-bg);
    backdrop-filter: blur(var(--glass-blur));
    border: 1px solid var(--glass-border);
    box-shadow: var(--glass-shadow);
    border-radius: 14px;
    color: var(--glass-text);
    pointer-events: auto;
    font-size: 11.5px;
  }

  .drawer-header,
  .modal-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-weight: 600;
    font-size: 12px;
  }

  .drawer-tabs {
    display: flex;
    gap: 4px;
    background: var(--canvas-selection);
    padding: 2px;
    border-radius: 8px;
  }

  .tab-btn {
    border: none;
    background: transparent;
    color: var(--glass-text);
    padding: 4px 8px;
    border-radius: 6px;
    font-size: 11px;
    font-weight: 600;
    cursor: pointer;
  }

  .tab-btn.active {
    background: var(--accent);
    color: var(--laser-core);
  }

  .icon-close-btn {
    border: none;
    background: transparent;
    color: var(--glass-text);
    cursor: pointer;
    padding: 4px;
    border-radius: 4px;
  }

  .ledger-controls,
  .library-controls {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .ledger-search {
    width: 100%;
    background: var(--canvas-bg);
    color: var(--glass-text);
    border: 1px solid var(--glass-border);
    border-radius: 6px;
    padding: 5px 8px;
    font-size: 11px;
  }

  .ledger-filter-row {
    display: flex;
    gap: 6px;
  }

  .ledger-select {
    flex: 1;
    background: var(--canvas-selection);
    color: var(--glass-text);
    border: 1px solid var(--glass-border);
    border-radius: 6px;
    padding: 4px 6px;
    font-size: 11px;
  }

  .ledger-table-wrap,
  .library-list {
    flex: 1;
    overflow-y: auto;
    border: 1px solid var(--glass-border);
    border-radius: 8px;
    padding: 4px;
  }

  .ledger-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 11px;
  }

  .ledger-table th {
    text-align: left;
    padding: 4px 6px;
    border-bottom: 1px solid var(--glass-border);
    opacity: 0.75;
  }

  .ledger-row {
    cursor: pointer;
  }

  .ledger-row:hover {
    background: var(--canvas-selection);
  }

  .ledger-row td {
    padding: 5px 6px;
    border-bottom: 1px solid var(--glass-border);
  }

  .type-pill {
    padding: 1px 5px;
    border-radius: 4px;
    background: var(--canvas-selection);
    font-size: 10px;
    font-weight: 600;
  }

  .summary-cell {
    max-width: 140px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .empty-ledger {
    padding: 18px;
    text-align: center;
    opacity: 0.65;
  }

  .drawer-footer,
  .modal-actions {
    display: flex;
    gap: 6px;
    justify-content: flex-end;
  }

  .ledger-action-btn {
    border: 1px solid var(--glass-border);
    background: var(--canvas-selection);
    color: var(--glass-text);
    padding: 5px 10px;
    border-radius: 8px;
    font-size: 11px;
    font-weight: 600;
    cursor: pointer;
  }

  .ledger-action-btn.primary {
    background: var(--accent);
    color: var(--laser-core);
  }

  .library-card {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 8px;
    border-bottom: 1px solid var(--glass-border);
  }

  .lib-meta {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .lib-title {
    font-weight: 600;
  }

  .lib-sub {
    font-size: 10px;
    opacity: 0.7;
  }

  .studio-modal-card {
    position: absolute;
    top: 130px;
    left: 50%;
    transform: translateX(-50%);
    z-index: 175;
    width: 340px;
    padding: 14px;
    background: var(--glass-bg);
    backdrop-filter: blur(var(--glass-blur));
    border: 1px solid var(--glass-border);
    box-shadow: var(--glass-shadow);
    border-radius: 14px;
    color: var(--glass-text);
    pointer-events: auto;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .latex-preview-box,
  .sig-pad {
    background: var(--canvas-bg);
    border: 1px solid var(--glass-border);
    border-radius: 8px;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 6px;
  }

  .webcam-controls {
    display: flex;
    flex-direction: column;
    gap: 6px;
    font-size: 11px;
  }

  .thresh-label {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .sig-img-preview {
    max-width: 180px;
    height: 56px;
    border: 1px dashed var(--accent);
    border-radius: 6px;
  }

  .waveform-bars {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 5px;
    height: 44px;
    background: var(--canvas-bg);
    border: 1px solid var(--glass-border);
    border-radius: 8px;
  }

  .wave-bar {
    width: 5px;
    background: var(--measure-line);
    border-radius: 3px;
  }
</style>

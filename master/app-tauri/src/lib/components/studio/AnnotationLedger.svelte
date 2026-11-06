<script lang="ts">
  import { ink } from '$lib/core/inkEngine.svelte';
  import { measureEngine } from '$lib/core/measureEngine.svelte';
  import { selection } from '$lib/core/selectionEngine.svelte';
  import {
    inserts,
    compileLatexToVector,
    applyGrayscaleThresholdToRgba,
    THRESHOLD_SIM_PAPER,
    THRESHOLD_SIM_INK,
    type SignatureMode,
  } from '$lib/core/insertEngine.svelte';
  import StudioIcon from './StudioIcon.svelte';
  import { freeOutputPath } from '$lib/views/studio/outputNames';
  import Button from '$lib/components/ui/Button.svelte';
  import IconButton from '$lib/components/ui/IconButton.svelte';
  import Input from '$lib/components/ui/Input.svelte';
  import Select from '$lib/components/ui/Select.svelte';
  import Dialog from '$lib/components/ui/Dialog.svelte';
  import { INK_GUESS_ID, INK_GUESS_LABS, LABS_BADGE, MATH_TEXT_LABS, labsVisible } from './labs.svelte';

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
  let filterKind = $state<string>('all');
  let sortBy = $state<string>('page');
  let newLibraryTitle = $state<string>('Custom Markup Stamp');
  let docTagsInput = $state<string>('structural, rev-2');
  let docFolderInput = $state<string>('Project Blueprints');

  // Signature draw points for Draw mode
  // BUG-017: every pen-down → pen-up on the pad is kept (the old code kept only the last stroke).
  let sigStrokes = $state<Array<Array<{ x: number; y: number; p: number }>>>([]);
  let sigNotice = $state<string | null>(null);
  let isDrawingSig = $state(false);
  let webcamPreviewDataUrl = $state<string | null>(null);

  const SIGNATURE_MODES: SignatureMode[] = ['draw', 'type', 'webcam'];
  const SIGNATURE_MODE_LABEL: Record<SignatureMode, string> = { draw: 'Draw', type: 'Type', webcam: 'Webcam' };

  let kindOptions = $derived([
    { value: 'all', label: 'All items' },
    { value: 'annotations', label: `Markups (${ink.completedStrokes.length})` },
    { value: 'measurements', label: `Takeoff (${measureEngine.measurements.length})` },
  ]);
  let authorOptions = $derived([
    { value: 'ALL', label: 'Author: all' },
    ...inserts.getAvailableAuthors().map((a: string) => ({ value: a, label: a })),
  ]);
  const SORT_OPTIONS = [
    { value: 'page', label: 'Sort: page' },
    { value: 'type', label: 'Sort: tool' },
  ];
  // Closing the audio dialog while recording must not leave a hidden running recording: ask first.
  let audioConfirming = $state(false);
  function onAudioClose() {
    if (audioConfirming) {
      audioConfirming = false; // Escape on the confirm = keep recording
      inserts.audioModalOpen = true;
    } else if (inserts.isRecordingAudio) {
      audioConfirming = true;
      inserts.audioModalOpen = true;
    } else {
      inserts.audioModalOpen = false;
    }
  }
  // Select / Input work with strings; the engine stores the threshold as a number.
  let thresholdText = $state(String(inserts.signatureThreshold));
  $effect(() => {
    inserts.signatureThreshold = Number(thresholdText) || inserts.signatureThreshold;
  });

  let latexPreview = $derived(compileLatexToVector(inserts.latexDraft));
  // Library items that insert a Labs tool (e.g. the built-in equation) are hidden while Labs is off.
  let visibleLibrary = $derived(inserts.markupLibrary.filter((it) => labsVisible(it.tool_id)));

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
    // A new, collision-safe name each time (the backend refuses an existing file); never the PDF itself.
    const outPath = await freeOutputPath(activePdfPath || 'studio-takeoff', 'takeoff', new Date(), undefined, 'csv');
    try {
      const records = await inserts.exportTakeoffCsv(outPath, activePdfPath ? [activePdfPath] : []);
      onNotice?.(`Exported ${records.length} takeoff measurements to ${outPath}`);
    } catch (e) {
      console.error('takeoff export failed', e);
      onNotice?.("Couldn't export the takeoff. The file was not written; choose another location and try again.");
    }
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
    sigNotice = null;
    const rect = (e.currentTarget as SVGSVGElement).getBoundingClientRect();
    try {
      (e.currentTarget as SVGSVGElement).setPointerCapture(e.pointerId);
    } catch {
      // pointer capture unsupported: drawing still works inside the pad
    }
    sigStrokes = [...sigStrokes, [{ x: e.clientX - rect.left, y: e.clientY - rect.top, p: e.pressure || 0.6 }]];
  }

  function handleSigPointerMove(e: PointerEvent) {
    if (!isDrawingSig) return;
    const rect = (e.currentTarget as SVGSVGElement).getBoundingClientRect();
    const last = sigStrokes.length - 1;
    if (last < 0) return;
    const pt = { x: e.clientX - rect.left, y: e.clientY - rect.top, p: e.pressure || 0.6 };
    sigStrokes = sigStrokes.map((st, i) => (i === last ? [...st, pt] : st));
  }

  function handleSigPointerUp() {
    isDrawingSig = false;
  }

  function sigPathD(st: Array<{ x: number; y: number }>): string {
    return st.map((p, i) => `${i === 0 ? 'M' : 'L'} ${p.x.toFixed(1)} ${p.y.toFixed(1)}`).join(' ');
  }

  function handleStampSignature() {
    if (inserts.signatureMode === 'draw') {
      const placed = inserts.stampDrawnSignature(sigStrokes);
      if (placed.length === 0) {
        sigNotice = 'Draw your signature in the box first.';
        return;
      }
      sigStrokes = [];
      sigNotice = null;
      return;
    }
    inserts.stampSignature({
      mode: inserts.signatureMode,
      typedName: inserts.signatureTypedName,
      dataUrl: webcamPreviewDataUrl ?? undefined,
    });
  }

  function runThresholdScanSimulation() {
    if (typeof document === 'undefined') return;
    const canvas = document.createElement('canvas');
    canvas.width = 180;
    canvas.height = 56;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    // Paint paper background + dark signature ink, then run `applyGrayscaleThresholdToRgba`
    ctx.fillStyle = THRESHOLD_SIM_PAPER;
    ctx.fillRect(0, 0, 180, 56);
    ctx.fillStyle = THRESHOLD_SIM_INK;
    ctx.font = 'italic 20px serif';
    ctx.fillText(inserts.signatureTypedName || 'Verified Sign', 12, 36);

    const imgData = ctx.getImageData(0, 0, 180, 56);
    const filtered = applyGrayscaleThresholdToRgba(imgData.data, inserts.signatureThreshold);
    imgData.data.set(filtered);
    ctx.putImageData(imgData, 0, 0);
    webcamPreviewDataUrl = canvas.toDataURL('image/png');
  }
</script>

<!-- Collapsible left-side annotation history and quantity takeoff drawer -->
{#if inserts.ledgerOpen}
  <aside
    class="annotation-ledger-drawer glass-panel"
    aria-label="Annotation History and Quantity Takeoff Drawer"
    onpointerdown={(e) => e.stopPropagation()}
  >
    <header class="drawer-header">
      <div class="drawer-tabs">
        <Button variant="plain" size="sm" pressed={activeTab === 'ledger'} onclick={() => (activeTab = 'ledger')}>
          Ledger ({ledgerRows.length})
        </Button>
        <Button
          variant="plain"
          size="sm"
          pressed={activeTab === 'library'}
          onclick={() => {
            activeTab = 'library';
            inserts.loadMarkupLibraryFromBackend();
          }}
        >
          Vault ({visibleLibrary.length})
        </Button>
        <Button
          variant="plain"
          size="sm"
          pressed={activeTab === 'workspace'}
          onclick={() => {
            activeTab = 'workspace';
            inserts.loadWorkspaceDocuments();
          }}
        >
          Docs ({inserts.workspaceDocs.length})
        </Button>
      </div>
      <IconButton icon="close" size="sm" ariaLabel="Close annotation ledger" onclick={() => (inserts.ledgerOpen = false)} />
    </header>

    {#if activeTab === 'ledger'}
      <div class="ledger-controls">
        <Input
          type="search"
          label="Filter annotations and measurements"
          hideLabel
          placeholder="Filter by tool, label, or page..."
          bind:value={filterQuery}
        />
        <div class="ledger-filter-row">
          <Select label="Filter category" hideLabel bind:value={filterKind} options={kindOptions} />
          <Select label="Filter by author" hideLabel bind:value={inserts.authorFilter} options={authorOptions} />
          <Select label="Sort ledger by" hideLabel bind:value={sortBy} options={SORT_OPTIONS} />
        </div>
        <div class="ledger-filter-row">
          {#if labsVisible(INK_GUESS_ID)}
            <!-- Labs-only (8.10): see ./labs.svelte.ts -->
            <Button
              variant="secondary"
              size="sm"
              data-testid="ledger-ink-guess-btn"
              ariaLabel={INK_GUESS_LABS.label}
              title={`${INK_GUESS_LABS.label}: ${INK_GUESS_LABS.note}`}
              onclick={() => {
                const converted = inserts.convertSelectedInkToText();
                if (converted) {
                  onNotice?.(`${INK_GUESS_LABS.label}: guessed "${converted.text}" from stroke shapes.`);
                } else {
                  onNotice?.('Draw or select a freehand ink stroke first.');
                }
              }}
            >
              {INK_GUESS_LABS.name}
              <span class="labs-badge" data-testid="labs-badge">{LABS_BADGE}</span>
            </Button>
          {/if}
          <Button
            variant="secondary"
            size="sm"
            title="Find tables and fillable form fields on the page (8.12)"
            onclick={async () => {
              const regions = await inserts.detectTablesAndFields(activePdfPath, 0);
              onNotice?.(`Detected ${regions.length} table/form regions on page.`);
            }}
          >
            Detect tables
          </Button>
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
                <th>Value / content</th>
              </tr>
            </thead>
            <tbody>
              {#each ledgerRows as row (row.id)}
                <tr
                  class="ledger-row"
                  data-testid="ledger-row"
                  data-row-id={row.id}
                  data-category={row.category}
                  data-type={row.typeLabel}
                  data-page={row.pageIndex + 1}
                >
                  <td>P{row.pageIndex + 1}</td>
                  <td>
                    <!-- The row is selected from this button (keyboard and pointer). -->
                    <Button
                      variant="plain"
                      size="sm"
                      title="Select this item on the page"
                      onclick={() => {
                        if (row.category === 'annotation') {
                          selection.selectedIds = [row.id];
                        }
                      }}
                    >
                      <span class="type-pill">{row.typeLabel}</span>
                    </Button>
                  </td>
                  <td class="summary-cell">{row.summary}</td>
                </tr>
              {/each}
            </tbody>
          </table>
        {/if}
      </div>

      <footer class="drawer-footer">
        <Button
          variant="primary"
          size="sm"
          title="Export all calibrated measurements to a CSV spreadsheet"
          onclick={handleExportTakeoffCsv}
        >
          Export takeoff (CSV)
        </Button>
        <Button
          variant="secondary"
          size="sm"
          title="Stamp selected markup across page ranges"
          onclick={() => (inserts.pastePagesModalOpen = true)}
        >
          Paste to pages…
        </Button>
      </footer>
    {:else if activeTab === 'library'}
      <!-- Reusable markup library vault -->
      <div class="library-controls">
        <Input label="Name for selected markup" bind:value={newLibraryTitle} placeholder="Name for selected markup..." />
        <Button variant="primary" size="sm" onclick={handleSaveSelectedToLibrary}>Save current markup to vault</Button>
      </div>

      <div class="library-list">
        {#each visibleLibrary as item (item.id)}
          <div class="library-card">
            <div class="lib-meta">
              <span class="lib-title">{item.title}</span>
              <span class="lib-sub">{item.category} • {item.tool_id}</span>
            </div>
            <Button
              variant="secondary"
              size="sm"
              onclick={() => {
                if (!inserts.insertGuard()) {
                  onNotice?.('The document is still loading its markups. Try again in a moment.');
                  return;
                }
                inserts.stampFromMarkupLibrary(item, 200, 220, 0);
                onNotice?.(`Stamped "${item.title}" onto canvas.`);
              }}
            >
              Stamp
            </Button>
          </div>
        {/each}
      </div>
    {:else}
      <!-- Workspace document tags, folders, favorites and watch-folder auto-ingest (7.10 and 8.7) -->
      <div class="library-controls">
        <Input label="Workspace document folder" bind:value={docFolderInput} placeholder="Folder name (e.g. Project Blueprints)..." />
        <Input label="Workspace document tags" bind:value={docTagsInput} placeholder="Comma-separated tags (e.g. structural, rev-2)..." />
        <Button
          variant="primary"
          size="sm"
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
          Favorite and tag active PDF
        </Button>

        <div class="ledger-filter-row">
          <Input label="Watch folder path" bind:value={inserts.watchFolderPath} placeholder="Local watch folder path..." />
          <Button
            variant="secondary"
            size="sm"
            onclick={async () => {
              const found = await inserts.scanWatchFolder();
              onNotice?.(`Watch folder scanned: ${found.length} PDFs ingested.`);
            }}
          >
            Scan
          </Button>
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

<!-- Labs-only (2.16) math text dialog (`insert.latex`); see ./labs.svelte.ts -->
{#if labsVisible('insert.latex')}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div onpointerdown={(e) => e.stopPropagation()}>
    <Dialog
      bind:open={inserts.latexModalOpen}
      title={MATH_TEXT_LABS.label}
      onclose={() => (inserts.latexModalOpen = false)}
      data-testid="math-text-dialog"
    >
      <div class="modal-stack">
        <span class="labs-badge" data-testid="labs-badge">{LABS_BADGE}</span>
        <Input label={MATH_TEXT_LABS.inputLabel} bind:value={inserts.latexDraft} />
        <p class="labs-note" data-testid="math-text-note">{MATH_TEXT_LABS.note}</p>
        <div class="latex-preview-box">
          <svg
            width={latexPreview.widthPt}
            height={latexPreview.heightPt}
            viewBox="0 0 {latexPreview.widthPt} {latexPreview.heightPt}"
          >
            <path d={latexPreview.svgPathD} fill="none" stroke="var(--accent)" stroke-width="1.5" />
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
      </div>
      {#snippet actions()}
        <Button variant="secondary" onclick={() => (inserts.latexModalOpen = false)}>Cancel</Button>
        <Button variant="primary" onclick={() => inserts.stampLatexEquation()}>{MATH_TEXT_LABS.insertAction}</Button>
      {/snippet}
    </Dialog>
  </div>
{/if}

<!-- Signature modal (`insert.signature`: draw / type / webcam threshold) -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div onpointerdown={(e) => e.stopPropagation()}>
  <Dialog
    bind:open={inserts.signatureModalOpen}
    title="Architectural signature stamp"
    ariaLabel="Signature capture modal"
    onclose={() => (inserts.signatureModalOpen = false)}
  >
    <div class="modal-stack">
      <div class="drawer-tabs">
        {#each SIGNATURE_MODES as mode}
          <Button
            variant="plain"
            size="sm"
            pressed={inserts.signatureMode === mode}
            data-testid="signature-tab-{mode}"
            onclick={() => (inserts.signatureMode = mode)}
          >
            {SIGNATURE_MODE_LABEL[mode]}
          </Button>
        {/each}
      </div>

      {#if inserts.signatureMode === 'draw'}
        <svg
          class="sig-pad"
          width="260"
          height="76"
          data-testid="signature-pad"
          onpointerdown={handleSigPointerDown}
          onpointermove={handleSigPointerMove}
          onpointerup={handleSigPointerUp}
          onpointercancel={handleSigPointerUp}
          role="img"
          aria-label="Freehand signature pad"
        >
          {#each sigStrokes as st, i (i)}
            {#if st.length > 1}
              <path
                d={sigPathD(st)}
                fill="none"
                stroke="var(--accent)"
                stroke-width="2.2"
                stroke-linecap="round"
                stroke-linejoin="round"
              />
            {/if}
          {/each}
        </svg>
        <div class="sig-pad-row">
          <span class="sig-hint" data-testid="signature-draw-hint" role={sigNotice ? 'alert' : undefined}>
            {sigNotice ?? 'Draw your signature in the box.'}
          </span>
          <Button
            variant="secondary"
            size="sm"
            data-testid="signature-clear-btn"
            onclick={() => {
              sigStrokes = [];
              sigNotice = null;
            }}
          >
            Clear
          </Button>
        </div>
      {:else if inserts.signatureMode === 'type'}
        <Input label="Typed signature name" bind:value={inserts.signatureTypedName} />
      {:else}
        <div class="webcam-controls">
          <Input
            type="range"
            min={60}
            max={230}
            label="Background removal threshold ({thresholdText})"
            bind:value={thresholdText}
            onchange={runThresholdScanSimulation}
          />
          <Button variant="secondary" size="sm" onclick={runThresholdScanSimulation}>Capture and threshold scan</Button>
          {#if webcamPreviewDataUrl}
            <img src={webcamPreviewDataUrl} alt="Thresholded transparent signature preview" class="sig-img-preview" />
          {/if}
        </div>
      {/if}
    </div>
    {#snippet actions()}
      <Button variant="secondary" onclick={() => (inserts.signatureModalOpen = false)}>Cancel</Button>
      <Button variant="primary" data-testid="signature-stamp-btn" onclick={handleStampSignature}>
        Stamp signature
      </Button>
    {/snippet}
  </Dialog>
</div>

<!-- Voice audio note modal (`insert.audio`: MediaRecorder + PDF `/Sound`) -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div onpointerdown={(e) => e.stopPropagation()}>
  <Dialog
    bind:open={inserts.audioModalOpen}
    variant={audioConfirming ? 'alert' : 'dialog'}
    title={audioConfirming ? 'Discard recording?' : 'Voice audio note (PDF /Sound attachment)'}
    ariaLabel="Voice audio note recorder"
    onclose={onAudioClose}
  >
    {#if audioConfirming}
      <p class="labs-note">The recording in progress will be thrown away and nothing is added to the page.</p>
    {:else}
      <div class="waveform-bars">
        {#each inserts.liveWaveform as bar}
          <span class="wave-bar" style:height="{Math.max(6, Math.round(bar * 0.36))}px"></span>
        {/each}
      </div>
    {/if}
    {#snippet actions()}
      {#if audioConfirming}
        <Button variant="secondary" onclick={() => (audioConfirming = false)}>Keep recording</Button>
        <Button variant="destructive" onclick={() => { audioConfirming = false; inserts.discardAudioRecording(); }}>Discard recording</Button>
      {:else}
        <Button variant="secondary" onclick={onAudioClose}>Cancel</Button>
        {#if !inserts.isRecordingAudio}
          <Button variant="secondary" onclick={() => inserts.startAudioRecording()}>Start recording</Button>
        {/if}
        <Button variant="primary" onclick={() => inserts.stopAndAttachAudioNote(activePdfPath)}>Add audio note to page</Button>
      {/if}
    {/snippet}
  </Dialog>
</div>

<!-- Multi-page advanced paste modal (`Paste to pages…`) -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div onpointerdown={(e) => e.stopPropagation()}>
  <Dialog
    bind:open={inserts.pastePagesModalOpen}
    title="Paste selected markup to pages"
    ariaLabel="Multi-page advanced paste"
    onclose={() => (inserts.pastePagesModalOpen = false)}
  >
    <div class="modal-stack">
      <div class="drawer-tabs">
        {#each ['All', 'Even', 'Odd', '1-5'] as presetRange}
          <Button
            variant="plain"
            size="sm"
            pressed={inserts.pasteRangeSpec === presetRange}
            onclick={() => (inserts.pasteRangeSpec = presetRange)}
          >
            {presetRange}
          </Button>
        {/each}
      </div>
      <Input
        label="Target page range"
        bind:value={inserts.pasteRangeSpec}
        placeholder="All, Even, Odd, or 1-5, 8"
      />
    </div>
    {#snippet actions()}
      <Button variant="secondary" onclick={() => (inserts.pastePagesModalOpen = false)}>Cancel</Button>
      <Button
        variant="primary"
        onclick={() => {
          const created = inserts.pasteStrokesToPages(inserts.pasteRangeSpec, Math.max(2, totalPages));
          onNotice?.(`Pasted ${created.length} markup copies across (${inserts.pasteRangeSpec}).`);
        }}
      >
        Apply multi-page stamp
      </Button>
    {/snippet}
  </Dialog>
</div>

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
    gap: var(--space-2);
    padding: var(--space-3);
    background: var(--surface-dialog);
    border: 1px solid var(--glass-border);
    box-shadow: var(--glass-shadow);
    border-radius: var(--radius-lg);
    color: var(--text-primary);
    pointer-events: auto;
    font-size: var(--font-size-caption);
  }

  .drawer-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-weight: 600;
    font-size: var(--font-size-footnote);
  }

  .drawer-tabs {
    display: flex;
    gap: var(--space-1);
  }

  /* Tab labels: primary text colour (the accent blue was under 4.5:1 on the dark panel); pressed = tint + bold. */
  .drawer-tabs :global(.lp-btn.lp-btn--plain) {
    color: var(--text-primary);
  }

  .drawer-tabs :global(.lp-btn.lp-btn--plain[aria-pressed='true']) {
    color: var(--text-primary);
    font-weight: 700;
  }

  .ledger-controls,
  .library-controls,
  .modal-stack,
  .webcam-controls {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .ledger-filter-row {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-1);
    align-items: flex-end;
  }

  /* Three filters share the 320px panel: each takes a third, shrinks to fit, never clips. */
  .ledger-filter-row :global(.lp-field) {
    flex: 1 1 6em;
    min-width: 0;
  }

  .ledger-filter-row :global(.lp-select) {
    min-width: 0;
    width: 100%;
    text-overflow: ellipsis;
  }

  /* Action labels never wrap into the next line; the row wraps instead (full text kept, with a title). */
  .ledger-controls :global(.lp-btn),
  .drawer-footer :global(.lp-btn) {
    white-space: nowrap;
  }

  .library-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    overflow-y: auto;
  }

  .ledger-table-wrap {
    flex: 1;
    overflow: auto;
  }

  .ledger-table {
    width: 100%;
    border-collapse: collapse;
    font-size: var(--font-size-caption);
  }

  .ledger-table th {
    text-align: left;
    padding: var(--space-1) var(--space-2);
    color: var(--text-secondary);
    border-bottom: 1px solid var(--glass-border);
  }

  .ledger-row td {
    padding: var(--space-1) var(--space-2);
    border-bottom: 1px solid var(--glass-border);
  }

  .type-pill {
    font-weight: 600;
  }

  .summary-cell {
    word-break: break-word;
  }

  .empty-ledger {
    padding: var(--space-4);
    text-align: center;
    opacity: 0.7;
  }

  .drawer-footer {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
    justify-content: flex-end;
  }

  .library-card {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-2);
    padding: var(--space-2);
    border: 1px solid var(--glass-border);
    border-radius: var(--radius-sm);
  }

  .lib-meta {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .lib-title {
    font-weight: 600;
  }

  .lib-sub {
    font-size: var(--font-size-caption);
    opacity: 0.7;
  }

  .labs-badge {
    display: inline-block;
    margin-left: var(--space-1);
    font-size: var(--font-size-caption);
    font-weight: 700;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    padding: 1px var(--space-1);
    border-radius: var(--radius-pill);
    border: 1px solid var(--accent);
    color: var(--accent);
    vertical-align: middle;
  }

  .labs-note {
    margin: 0;
    font-size: var(--font-size-caption);
    line-height: 1.4;
    opacity: 0.8;
  }

  .latex-preview-box,
  .sig-pad {
    background: var(--canvas-bg);
    border: 1px solid var(--glass-border);
    border-radius: var(--radius-sm);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--space-2);
  }

  .sig-pad-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-2);
    font-size: var(--font-size-caption);
  }

  .sig-hint {
    opacity: 0.8;
  }

  .sig-img-preview {
    max-width: 180px;
    height: 56px;
    border: 1px dashed var(--accent);
    border-radius: var(--radius-xs);
  }

  .waveform-bars {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-1);
    height: 44px;
    background: var(--canvas-bg);
    border: 1px solid var(--glass-border);
    border-radius: var(--radius-sm);
  }

  .wave-bar {
    width: var(--space-1);
    background: var(--measure-line);
    border-radius: var(--radius-xs);
  }
</style>

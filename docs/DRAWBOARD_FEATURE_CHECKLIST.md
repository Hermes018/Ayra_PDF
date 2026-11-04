# LocalPDF Studio — Engineering Specification & 109-Feature Master Checklist

> **Source Spec**: `/home/ayra/Downloads/pdf_drawing_annotation_app_spec.md`  
> **Architecture Master Plan**: [`docs/DRAWBOARD_STUDIO_MASTER_PLAN.md`](./DRAWBOARD_STUDIO_MASTER_PLAN.md) & [`ADR-012`](./adr/ADR-012-drawboard-artcraft-spatial-studio.md)  
> **Workflow Gate**: Verified continuously via `cargo xtask feature-map-check` & `cargo xtask studio-audit`.  
> **Rule**: A feature checkbox `[x]` (`DONE`) may **only** be checked when backed by a real, non-placeholder Rust/Svelte implementation and passing test coverage.

---

## 0. Confirmed Project Facts (Audited Against Repository)

| Fact Category | Confirmed Repository Reality |
|---|---|
| **Drawing / PDF Library Ecosystem** | **ArtCraft** ecosystem by `storytold` (`https://github.com/storytold/*`). Currently **only** `vendor/pdfcraft` is vendored (`v0.15.0`, commit `75360d1d7db06fed974f48c914d4c04132757187`). Sibling repos referenced in ADR-012 (`cadcraft`, `designcraft`, `figcraft`, `materialcraft`, `texturecraft`, `craft-anything`) are **not** yet added as submodules; custom domain crates (`lpdf-vector-ink`, `lpdf-cad-measure`, `lpdf-layout-edit`, `lpdf-control-bus`) are scheduled across D1–D6. |
| **Language & Build System** | **Rust** (2021 edition, pinned `1.85.0`, Cargo workspace with 16 crates + `xtask`) + **TypeScript 5 / Vite 6 / pnpm** + **Tauri v2** (`app-tauri`). |
| **UI Framework & Entry Point** | **Svelte 5** (Runes mode: `$state`, `$derived`, `$effect`) with static SvelteKit SPA (`ssr = false`). Entry point: `app-tauri/src/routes/+page.svelte` (defaults `currentView = 'Studio'`). |
| **PDF Rendering & Writing** | **Frontend Legacy Views**: Vendored Mozilla `pdf.min.js` (`PdfViewer.svelte`, `GridEditor.svelte`). **Frontend Spatial Studio**: Currently placeholder A4 divs in `SpatialCanvas.svelte` (needs `pdf.js` tile wiring). **Backend PDF Engine**: `storytold/pdfcraft` (`crates/lpdf-engine-pdfcraft`) with `lopdf` fallback (`crates/lpdf-engine-fallback`). Note: `lpdf-engine-pdfcraft::render` currently extracts embedded images rather than full `hayro` page rasterization. |
| **Target Platforms** | **Linux** (Fedora 44 KDE RPM primary, Wayland/GTK) and **Windows 10/11** (MSVC/WebView2, paused at Phase W1 `ee1d552`). Strict 100% offline execution (`02_RULES_DOS_DONTS.md`, `ADR-005`). |

---

## Summary Rollup (119 Total Features: 108 Spec + 11 Native Section 9)

| Status | Count | Description |
|---|---:|---|
| **`DONE` (`[x]`)** | **36** | End-to-end implemented in Rust + Svelte UI and verified by tests. |
| **`IN PROGRESS` (`[ ]`)** | **24** | UI state/shell or backend port exists (Phases D1–D2.2), awaiting full canvas/IPC wiring. |
| **`NOT STARTED` (`[ ]`)** | **53** | Scheduled across Phases D4, D5, and D6 in `DRAWBOARD_STUDIO_MASTER_PLAN.md`. |
| **`DEFERRED` (`[ ]`)** | **6** | Cloud/multiplayer features prohibited by strict offline air-gap rule (`ADR-005`). |
| **Total** | **119** | Across 9 functional domains (108 core spec + 11 LocalPDF native extensions). |

---

## 1. Core Drawing, Inking & Freehand Tools (17 Features)

- [x] **1.1** `[P0]` `[Phase D3]` `DONE` **Pen tool** — Smooth freehand ink drawing for handwriting, sketching, and markup (`app-tauri/src/lib/core/inkEngine.svelte.ts` & `SpatialCanvas.svelte`).
- [x] **1.2** `[P0]` `[Phase D3]` `DONE` **Highlighter tool** — Semi-transparent freehand highlighting over text or drawings with `mix-blend-mode: multiply` (`app-tauri/src/lib/core/inkEngine.svelte.ts` & `SpatialCanvas.svelte`).
- [x] **1.3** `[P0]` `[Phase D3]` `DONE` **Eraser tool** — Erase whole strokes or partial segments of ink/annotations (`eraseWhole` and `erasePartial` bisection in `app-tauri/src/lib/core/inkEngine.svelte.ts`).
- [x] **1.4** `[P0]` `[Phase D3]` `DONE` **Pressure-sensitive inking** — Variable stroke width based on stylus pressure via Catmull-Rom spline outline ribbon (`generateSvgPathFromPoints` in `app-tauri/src/lib/core/inkEngine.svelte.ts`).
- [ ] **1.5** `[P1]` `[Phase D3]` `NOT STARTED` **Tilt / stylus-aware rendering** — Natural pen/brush behavior when hardware supports stylus tilt (`tiltX`, `tiltY`).
- [ ] **1.6** `[P1]` `[Phase D3]` `NOT STARTED` **Palm rejection / touch vs. pen mode** — Separate finger pan/zoom (`pointerType === 'touch'`) from stylus drawing (`pointerType === 'pen'`).
- [ ] **1.7** `[P0]` `[Phase D2]` `IN PROGRESS` **Stroke thickness / size presets** — Quick-access stroke widths (in mm/pt) + custom slider in `PropertyPill.svelte` *(working in-memory; needs `prefs_save` disk persistence)*.
- [ ] **1.8** `[P0]` `[Phase D2]` `IN PROGRESS` **Opacity control** — Adjustable transparency for pens, highlighters, shapes, and fills in `PropertyPill.svelte` *(UI bound to `toolEngine`; canvas rendering in D3)*.
- [ ] **1.9** `[P0]` `[Phase D2]` `IN PROGRESS` **Color palette & custom color picker** — Saved color swatches, recent colors, hex/RGB picker *(10 semantic swatches in `PropertyPill.svelte`; needs hex/RGB custom picker & recent colors)*.
- [ ] **1.10** `[P1]` `[Phase D2]` `IN PROGRESS` **Multiple saved pens / favorite tool presets** — Multiple pens with different colors/widths on the toolbar *(`pen.a`, `pen.b`, and `favorites` array in `toolEngine.svelte.ts`; `FavoritesBar.svelte` component not yet created)*.
- [ ] **1.11** `[P1]` `[Phase D3]` `NOT STARTED` **Stroke smoothing / stabilization** — Configurable smoothing for jitter-free handwriting and line work (One-Euro filter + Ramer-Douglas-Peucker in `lpdf-vector-ink`).
- [ ] **1.12** `[P2]` `[Phase D3]` `NOT STARTED` **Dashed / dotted / custom stroke styles** — Line pattern options for freehand and geometric strokes (`DashPattern` in `toolEngine.svelte.ts`, rendering in D3).
- [x] **1.13** `[P2]` `[Phase D3]` `DONE` **Laser pointer tool** — Temporary presentation/attention stroke with 1800ms rAF decay (`addLaserPoint`, `updateLaser` in `app-tauri/src/lib/core/inkEngine.svelte.ts`).
- [x] **1.14** `[P1]` `[Phase D3]` `DONE` **Smart shape recognition (hold-to-snap)** — 350ms dwell classifier snapping raw ink to Line, Rectangle, Triangle, or Circle (`recognizeShape` in `app-tauri/src/lib/core/inkEngine.svelte.ts`).
- [ ] **1.15** `[P1]` `[Phase D3]` `NOT STARTED` **Line tool** — Straight line segment tool with optional angle snapping (`15°` increments).
- [ ] **1.16** `[P1]` `[Phase D3]` `NOT STARTED` **Arrow tool** — Single- or double-headed arrow annotations.
- [ ] **1.17** `[P1]` `[Phase D3]` `NOT STARTED` **Polyline / polygon / cloud / rectangle / ellipse shapes** — Core geometric and revision-cloud markup tools (`shape.rect`, `shape.ellipse`, `shape.cloud`, `shape.polygon`, `shape.polyline` — note: must un-collapse `normalizeToolId` in `toolEngine.svelte.ts`).

---

## 2. Annotation, Text, Markup & Review Tools (18 Features)

- [ ] **2.1** `[P0]` `[Phase 8/D4]` `IN PROGRESS` **Text highlight** — Highlight selectable PDF text *(Backend `AnnotKind::Highlight` in `lpdf-engine-pdfcraft` & demo trigger in `EditView.svelte`; interactive glyph quad selection in D4)*.
- [ ] **2.2** `[P0]` `[Phase 8/D4]` `IN PROGRESS` **Underline text** — Underline selectable PDF text *(Backend `AnnotKind::Underline` in `lpdf-engine-port`; interactive selection in D4)*.
- [ ] **2.3** `[P1]` `[Phase 8/D4]` `IN PROGRESS` **Strikethrough text** — Cross out selectable PDF text *(Backend `AnnotKind::StrikeOut` in `lpdf-engine-port`; interactive selection in D4)*.
- [ ] **2.4** `[P1]` `[Phase D4]` `NOT STARTED` **Squiggly underline** — Wavy underline for review/editorial markup (`/Squiggly` annotation subtype).
- [ ] **2.5** `[P0]` `[Phase 8/D4]` `IN PROGRESS` **Text box / free text tool** — Place styled text directly on the page with font, size, color, alignment, border, and fill options *(Basic `add_text_overlay` in `EditView.svelte`; Knuth-Plass inline layout in `lpdf-layout-edit` in D4)*.
- [ ] **2.6** `[P1]` `[Phase D6]` `NOT STARTED` **Callout tool** — Text box with leader line and arrow pointing to a target region (`callout.single`, `callout.multi` `/FreeTextCallout`).
- [x] **2.7** `[P1]` `[Phase 8]` `DONE` **Sticky note / comment pin** — Compact icon that expands into a threaded or detailed note (`AnnotKind::TextNote` in `crates/lpdf-tools-edit/src/lib.rs`, `crates/lpdf-engine-pdfcraft/src/lib.rs`, and `EditView.svelte`).
- [ ] **2.8** `[P2]` `[Phase D4]` `NOT STARTED` **Inline PDF text editing (where supported)** — Edit existing document text boxes/runs with Knuth-Plass paragraph breaking and baseline preservation (`crates/lpdf-layout-edit`).
- [ ] **2.9** `[P1]` `[Phase D6]` `NOT STARTED` **Insert image** — Place PNG/JPG/SVG/clipboard images onto pages with resize/rotate handles.
- [ ] **2.10** `[P2]` `[Phase D6]` `NOT STARTED` **Camera / photo capture insert** — Insert a photo from device webcam/camera (where hardware exists) or local file picker.
- [ ] **2.11** `[P1]` `[Phase 8/D6]` `IN PROGRESS` **Stamps (standard + custom)** — Approved, Draft, Confidential, Reviewed, Received, or user-created custom stamps *(Page watermark/stamp in `EditView.svelte`; reusable stamp library in D6)*.
- [ ] **2.12** `[P1]` `[Phase 8/D6]` `IN PROGRESS` **Signature tool** — Draw, type, or import a signature and place it on documents; pair with cryptographic PKCS#12 signing *(PKCS#12 crypto signing `sign_pdf` in `FormsView.svelte` is DONE; visual 3-mode signature pad in D6)*.
- [x] **2.13** `[P1]` `[Phase 8]` `DONE` **Form filling** — Fill standard PDF AcroForm fields: text inputs, checkboxes, radio buttons, dropdowns, date fields (`inspect_form_fields` & `fill_form_fields` in `crates/lpdf-tools-edit/src/lib.rs` & `FormsView.svelte`).
- [ ] **2.14** `[P2]` `[Phase D4]` `NOT STARTED` **Hyperlink tool** — Add clickable internal page links or external URLs (`/Link` annotations).
- [ ] **2.15** `[P2]` `[Phase D4]` `NOT STARTED` **Hatch / fill patterns** — Architectural/engineering hatch fills (ANSI31, ANSI32, CrossHatch, Concrete, Earth, Sand, Grass) with 4096×4096 ceiling & island detection (`crates/lpdf-layout-edit`).
- [ ] **2.16** `[P3]` `[Phase D6]` `NOT STARTED` **LaTeX / math block insertion** — Render mathematical formulas cleanly onto the page via vendored offline MathJax v3 WebWorker (`svg_to_pdf_form_xobject`).
- [ ] **2.17** `[P2]` `[Phase D4]` `NOT STARTED` **Polygon / area fill tinting** — Shaded regions for zoning, markup, and visual grouping.
- [x] **2.18** `[P1]` `[Phase 9]` `DONE` **Redaction tool** — Permanently remove sensitive text/regions at the byte stream level with post-redaction verification F25 (`redact_regions` in `crates/lpdf-tools-security/src/lib.rs` & `SecurityView.svelte`).

---

## 3. Selection, Manipulation & Object Editing (10 Features)

- [ ] **3.1** `[P0]` `[Phase D2/D4]` `IN PROGRESS` **Lasso selection** — Freeform selection of ink strokes, shapes, text, and annotations *(Tool ID in Opus 5.5 spec; collapsed to `'select'` in `toolEngine.svelte.ts` — un-collapse & implement point-in-polygon hit-test in D4)*.
- [ ] **3.2** `[P0]` `[Phase D2/D4]` `IN PROGRESS` **Rectangle marquee selection** — Box-select multiple annotations/objects *(Scheduled in D4)*.
- [ ] **3.3** `[P0]` `[Phase D4]` `NOT STARTED` **Move / resize / rotate handles** — Transform selected objects with bounding-box 8-handle + rotation gizmo.
- [ ] **3.4** `[P0]` `[Phase D4]` `NOT STARTED` **Copy / cut / paste / duplicate** — Duplicate or move annotations within or across pages (`Ctrl+C`, `Ctrl+X`, `Ctrl+V`, `Ctrl+D`).
- [ ] **3.5** `[P1]` `[Phase D6]` `NOT STARTED` **Advanced paste (paste in place / multi-page paste)** — Paste an object at the exact same coordinates across one or multiple selected pages.
- [ ] **3.6** `[P1]` `[Phase D4]` `NOT STARTED` **Group / ungroup objects** — Combine multiple strokes/shapes into a single manipulable unit.
- [ ] **3.7** `[P1]` `[Phase D4]` `NOT STARTED` **Z-order / layer ordering** — Bring to front, send to back, bring forward, send backward.
- [ ] **3.8** `[P1]` `[Phase D4]` `NOT STARTED` **Alignment & distribution tools** — Align left/center/right/top/middle/bottom and distribute spacing evenly.
- [ ] **3.9** `[P1]` `[Phase D4]` `NOT STARTED` **Flip horizontal / vertical** — Mirror selected markup or shapes around selection centroid.
- [ ] **3.10** `[P1]` `[Phase D4]` `NOT STARTED` **Edit properties after placement** — Change color, thickness, opacity, font, fill, or line style of already-drawn objects via `PropertyPill.svelte`.

---

## 4. Measurement, Scale & Technical Drawing Tools (11 Features)

- [ ] **4.1** `[P1]` `[Phase D5]` `NOT STARTED` **Scale calibration tool** — Set page scale using two reference points (`measure.calibrate`) or known ratio (`1:100`, `1/4" = 1'-0"`) persisted to PDF ISO 32000-1 `/VP` `/Measure` dictionary (`crates/lpdf-cad-measure`).
- [ ] **4.2** `[P2]` `[Phase D5]` `NOT STARTED` **Multiple scale presets / per-page scale** — Support different scales across sheets or viewports.
- [ ] **4.3** `[P1]` `[Phase D5]` `NOT STARTED` **Linear distance measurement** — Measure point-to-point length with live dimension readout (`measure.length`).
- [ ] **4.4** `[P1]` `[Phase D5]` `NOT STARTED` **Polylength / perimeter measurement** — Measure multi-segment paths (`measure.polylength` & `measure.perimeter`).
- [ ] **4.5** `[P1]` `[Phase D5]` `NOT STARTED` **Area measurement (polygon / rectangular)** — Calculate real-world area via Shoelace formula in calibrated units (`mm²`, `cm²`, `m²`, `in²`, `ft²`).
- [ ] **4.6** `[P1]` `[Phase D5]` `NOT STARTED` **Angle measurement / protractor** — Measure angles between lines or segments (`measure.protractor`) with `0.01°` precision.
- [ ] **4.7** `[P1]` `[Phase D5]` `NOT STARTED` **Digital ruler overlay** — On-screen rotatable ruler for straight edges and measured lines (`measure.ruler`).
- [ ] **4.8** `[P2]` `[Phase D6]` `NOT STARTED` **Count / takeoff tool** — Click-to-count items with numbered/symbol markers and running totals for quantity takeoffs.
- [ ] **4.9** `[P1]` `[Phase D5]` `NOT STARTED` **Snap to PDF geometry / vector content** — Snap cursor (`rstar` R-tree OSNAP `< 2ms`) to endpoints, midpoints, intersections, perpendiculars, and centroids.
- [ ] **4.10** `[P1]` `[Phase D4/D5]` `NOT STARTED` **Snap to grid / snap to objects / smart guides** — Align drawings with page grid (`G`) or existing annotations.
- [ ] **4.11** `[P2]` `[Phase D4]` `NOT STARTED` **Snapshot / region capture tool** — Capture a cropped vector/raster region of a page to copy, export, or re-insert (`snapshot`).

---

## 5. PDF Document & Page Management (14 Features)

- [x] **5.1** `[P0]` `[Phase 6]` `DONE` **Open, view & save standard PDFs** — Fast rendering in `PdfViewer.svelte` (`pdf.min.js`) and safe atomic save via `atomic_save_pdf` (`crates/lpdf-core` & `app-tauri/src-tauri/src/lib.rs`). *(Note: `SpatialCanvas.svelte` still needs `pdf.js` wired in D2.2).*
- [x] **5.2** `[P0]` `[Phase 6]` `DONE` **Page thumbnails / page sorter sidebar** — Visual overview of all pages with drag-and-drop reorder (`GridEditor.svelte` & `OrganizeView.svelte`).
- [ ] **5.3** `[P0]` `[Phase 6/D4]` `IN PROGRESS` **Insert blank / template pages** — Add blank, lined, grid, dotted, or custom-sized pages anywhere in the document *(Grid/blank page insertion planned for Studio page bar)*.
- [ ] **5.4** `[P0]` `[Phase 6/D4]` `IN PROGRESS` **Duplicate pages** — Clone one or more pages with or without annotations.
- [x] **5.5** `[P0]` `[Phase 6]` `DONE` **Delete pages** — Remove single or multiple selected pages (`delete_pages` in `crates/lpdf-tools-organize/src/lib.rs` & `OrganizeView.svelte`).
- [x] **5.6** `[P0]` `[Phase 6]` `DONE` **Reorder pages** — Drag-and-drop page rearrangement in thumbnail grid view (`reorder_pages` in `crates/lpdf-tools-organize/src/lib.rs` & `GridEditor.svelte`).
- [x] **5.7** `[P0]` `[Phase 6]` `DONE` **Rotate pages** — Rotate individual pages (`rotate_page`) or whole document (`rotate_document`) in `90°` increments (`crates/lpdf-tools-organize/src/lib.rs`).
- [x] **5.8** `[P1]` `[Phase 6]` `DONE` **Extract pages** — Save selected pages as a new PDF (`extract_pages` in `crates/lpdf-tools-organize/src/lib.rs` & `OrganizeView.svelte`).
- [x] **5.9** `[P1]` `[Phase 6]` `DONE` **Split PDF** — Split by page range, bookmarks, or fixed page count (`split_pdf` in `crates/lpdf-tools-organize/src/lib.rs` & `OrganizeView.svelte`).
- [x] **5.10** `[P1]` `[Phase 6]` `DONE` **Merge / combine PDFs** — Combine multiple PDFs or insert another PDF into the current document (`merge_pdfs` in `crates/lpdf-tools-organize/src/lib.rs` & `OrganizeView.svelte`).
- [ ] **5.11** `[P1]` `[Phase 6/D4]` `IN PROGRESS` **Crop / trim / page margin adjustment** — Adjust visible page bounds (`/CropBox` / `/TrimBox`) or add extra margin space around pages for notes.
- [x] **5.12** `[P1]` `[Phase 8]` `DONE` **Flatten annotations** — Bake interactive annotations/forms permanently into page content for sharing or printing (`flatten_comments` & `flatten_forms` in `crates/lpdf-tools-edit/src/lib.rs`).
- [ ] **5.13** `[P1]` `[Phase D6]` `NOT STARTED` **Bookmarks / outline navigation & editing** — View, jump to, create, rename, nest, and reorder PDF `/Outlines` bookmarks (`crates/lpdf-engine-pdfcraft`).
- [ ] **5.14** `[P0]` `[Phase D4]` `NOT STARTED` **Full-text search** — Search words/phrases across the PDF with match count, highlight, and next/previous navigation.

---

## 6. Templates, Workspace, Canvas & Navigation (14 Features)

- [ ] **6.1** `[P0]` `[Phase D1]` `IN PROGRESS` **Infinite / smooth zoom & pan canvas** — High-FPS pan (`translate3d`), pinch-to-zoom (`[0.05, 64.0]`), fit-to-width, fit-to-page, and zoom percentage controls (`viewportEngine.svelte.ts` + `SpatialCanvas.svelte` — needs real PDF page rendering wired).
- [ ] **6.2** `[P1]` `[Phase D1]` `IN PROGRESS` **Page layout modes** — Single page, continuous vertical scroll, horizontal scroll, two-page spread, and cover-page view *(Continuous vertical stack in `SpatialCanvas.svelte`; spread/horizontal modes pending)*.
- [ ] **6.3** `[P2]` `[Phase D1/D2]` `IN PROGRESS` **Canvas rotation** — Rotate the workspace view freely (`rotateDeg` in `viewportEngine.svelte.ts` supports arbitrary degrees; needs rotation dial/gesture UI on canvas).
- [ ] **6.4** `[P1]` `[Phase D1/D2]` `IN PROGRESS` **Split view / multi-document tabs / multi-window** — View two sections of the same PDF or two different PDFs side by side *(`SplitLayout.svelte` created in D2.1, but not yet mounted in `+page.svelte`, missing drag handle handler, and missing `studio-split.json` capability)*.
- [ ] **6.5** `[P1]` `[Phase D4]` `NOT STARTED` **Grid & line overlays** — Toggle square grid, isometric grid, ruled lines, or dot grid over any PDF page without altering the file.
- [ ] **6.6** `[P1]` `[Phase D6]` `NOT STARTED` **Built-in page templates** — Blank, lined, graph, isometric, storyboard, Cornell notes, planner, music staff, engineering title block.
- [ ] **6.7** `[P2]` `[Phase D6]` `NOT STARTED` **Master / overlay templates** — Apply a reusable template or title block overlay across multiple pages (`/OCG` optional content group).
- [ ] **6.8** `[P1]` `[Phase D2]` `IN PROGRESS` **Customizable floating / dockable toolbar** — Dock toolbar top/bottom/left/right or float it freely; collapse to compact mode (`DockableToolbar.svelte` in D2.1 — needs `prefs_save` disk persistence).
- [ ] **6.9** `[P2]` `[Phase D2]` `IN PROGRESS` **Quick radial / pie / favorite tools menu** — Fast stylus/cursor access to top tools (`RadialMenu.svelte` with 8 sectors in D2.1; `FavoritesBar.svelte` still missing).
- [ ] **6.10** `[P1]` `[Phase D1/D2]` `IN PROGRESS` **Dark mode / page inversion / reading themes** — App dark/light theme (`themeContract.svelte.ts`) + optional sepia/night-mode page inversion for comfortable reading *(Note: `themeContract.svelte.ts` and `appState.themePreference` currently out of sync)*.
- [ ] **6.11** `[P2]` `[Phase D2]` `IN PROGRESS` **Full-screen / distraction-free canvas mode** — Hide chrome (`F11` / toggle) and maximize drawing space *(Note: `MainLayout.svelte` currently wraps `Studio` in a `max-width: 1200px` padded box that must be made full-bleed)*.
- [ ] **6.12** `[P1]` `[Phase D1/D2]` `IN PROGRESS` **Keyboard shortcuts & customizable hotkeys** — Fast tool switching (`V`, `H`, `P`, `M`, `E`, `L`, `T`, `S`, `K`, `G`, `Space`) with zero-conflict binding (`commandEngine.svelte.ts` + `ShortcutSettings.svelte` — needs `prefs_save` disk persistence).
- [ ] **6.13** `[P2]` `[Phase D4]` `NOT STARTED` **Layer panel (PDF OCG / annotation layers)** — Show, hide, lock, create, and organize markup layers.
- [ ] **6.14** `[P2]` `[Phase D6]` `NOT STARTED` **Reusable component / asset library** — Save frequently used symbols, stamps, callouts, details, or markup groups for drag-and-drop reuse.

---

## 7. History, Metadata, Organization & Export (10 Features)

- [x] **7.1** `[P0]` `[Phase D1]` `DONE` **Undo / redo history** — Reliable multi-step undo/redo (`500`-step ring buffer + `120ms` pointer-drag coalescing in `commandEngine.svelte.ts`, verified by Vitest).
- [ ] **7.2** `[P1]` `[Phase D6]` `NOT STARTED` **Annotation list / markup summary panel** — Filterable sidebar listing all annotations by page, author, type, color, status, or date (`10,000`-row virtualized table `< 4ms/frame`).
- [ ] **7.3** `[P1]` `[Phase D6]` `NOT STARTED` **Annotation status / review states** — Mark comments as Accepted, Rejected, Completed, Cancelled, or Open.
- [ ] **7.4** `[P1]` `[Phase D6]` `NOT STARTED` **Export markup summary (CSV / PDF / Markdown / JSON / XLSX / DXF)** — Export comments, measurements, and takeoff counts as structured reports.
- [x] **7.5** `[P0]` `[Phase 10]` `DONE` **Export as original vs. flattened PDF** — Choose between editable annotations or flattened output (`flatten_comments`, `flatten_forms` in `crates/lpdf-tools-edit/src/lib.rs`, `convert_to_pdfa` in `crates/lpdf-tools-convert/src/lib.rs`).
- [x] **7.6** `[P1]` `[Phase 10]` `DONE` **Export pages as images (PNG / JPEG / SVG)** — Render selected pages or regions at configurable DPI (`convert_from_pdf` in `crates/lpdf-tools-convert/src/lib.rs` & `ConvertFromPdfView.svelte`).
- [ ] **7.7** `[P2]` `[Phase D6]` `NOT STARTED` **Print with / without markup** — Print clean pages or include annotations and comments.
- [x] **7.8** `[P2]` `[Phase 9]` `DONE` **Password protection & PDF permissions** — Open/edit password-protected PDFs and set AES-256 encryption on export (`protect_pdf` & `unlock_pdf` in `crates/lpdf-tools-security/src/lib.rs` & `SecurityView.svelte`).
- [x] **7.9** `[P2]` `[Phase 7]` `DONE` **File size optimization / compression** — Reduce PDF size via multi-stage stream compression and F12 never-grow failsafe (`compress_pdf` in `crates/lpdf-tools-optimize/src/lib.rs` & `OptimizeView.svelte`).
- [ ] **7.10** `[P2]` `[Phase D6]` `NOT STARTED` **Document tags, folders, favorites & recent files** — Organize local documents in a structured home/library view.

---

## 8. Audio, Collaboration, Cloud & AI-Assisted Features (14 Features)

> **Protocol Note (`02_RULES_DOS_DONTS.md` & `ADR-005`)**: LocalPDF Studio is strictly 100% air-gapped and offline (`reqwest`, `hyper`, `ureq`, `tokio::net`, `std::net` are banned by `cargo xtask protocol-lint`). Cloud/multiplayer items in Section 8 are marked `DEFERRED (OFFLINE PROTOCOL)` or adapted to local-first equivalents per `ADR-012 §8`.

- [ ] **8.1** `[P2]` `[Phase D6]` `NOT STARTED` **Audio notes attached to page locations** — Record voice notes (`WebAudio` / `MediaRecorder` Opus/PCM) pinned to specific coordinates as PDF `/Sound` annotations.
- [ ] **8.2** `[P3]` `[N/A]` `DEFERRED (OFFLINE PROTOCOL)` **Real-time collaboration** — *Banned by zero-network air-gap rule (`ADR-005`).*
- [ ] **8.3** `[P2]` `[Phase D6]` `NOT STARTED` **Asynchronous collaboration (comments & replies)** — Threaded comment discussions, review status tracking, and `@author` metadata stored inside standard PDF `/Popup` & `/IRT` (In-Reply-To) dictionaries for offline file exchange.
- [ ] **8.4** `[P2]` `[Phase D6]` `NOT STARTED` **Shared annotation visibility / author filtering** — Filter or toggle annotations by author inside the local PDF annotation dictionary.
- [ ] **8.5** `[P2]` `[N/A]` `DEFERRED (OFFLINE PROTOCOL)` **Cloud document storage & cross-device sync** — *Banned by zero-network air-gap rule (`ADR-005`).*
- [ ] **8.6** `[P3]` `[N/A]` `DEFERRED (OFFLINE PROTOCOL)` **Third-party storage integrations (Google Drive / OneDrive / Dropbox)** — *Banned by zero-network air-gap rule (`ADR-005`).*
- [ ] **8.7** `[P3]` `[Phase D6]` `NOT STARTED` **Email-to-workspace / Watch-folder auto-ingest** — *Adapted per `ADR-012 §8`*: Local filesystem `notify` watch-folder daemon auto-ingests dropped PDFs/images into the workspace (zero network).
- [ ] **8.8** `[P3]` `[N/A]` `DEFERRED (OFFLINE PROTOCOL)` **QR / mobile companion handoff** — *Network handoff banned by `ADR-005`; local webcam/file import covered in 2.10.*
- [x] **8.9** `[P1]` `[Phase 7]` `DONE` **OCR (Optical Character Recognition)** — Convert scanned PDFs/images into searchable, selectable text via pure-Rust offline `ocrs` engine (`ocr_pdf` in `crates/lpdf-tools-optimize/src/lib.rs` & `OptimizeView.svelte`).
- [ ] **8.10** `[P2]` `[Phase D6]` `NOT STARTED` **Handwriting recognition (ink-to-text)** — Convert handwritten stylus notes into editable typed text via offline model.
- [x] **8.11** `[P3]` `[Phase 11]` `DONE` **AI document summary / Q&A / key-point extraction** — Summarize pages via offline TextRank extractive engine + local GGUF LLM (`candle-core` in `crates/lpdf-ai-local/src/lib.rs` & `AiSummarizeView.svelte`).
- [ ] **8.12** `[P3]` `[Phase D6]` `NOT STARTED` **Auto table / form field detection** — Automatically detect tables or fillable regions in flat PDFs.
- [ ] **8.13** `[P3]` `[N/A]` `DEFERRED (OFFLINE PROTOCOL)` **Team / organization workspace & license management** — *Banned by zero-network air-gap rule (`ADR-005`).*
- [ ] **8.14** `[P3]` `[N/A]` `DEFERRED (OFFLINE PROTOCOL)` **Shared document permissions (view-only / comment / edit links)** — *Cloud links banned by `ADR-005`; local PDF permission flags covered in 7.8.*

---

## 9. Additional LocalPDF Studio Native Capabilities (11 Verified / Planned Features)

- [x] **9.1** `[P0]` `[Phase 7]` `DONE` **Corrupt PDF Repair with Engine Fallback** — Automatic structural repair and `pdfcraft` → `lopdf` engine failover (`repair_pdf` in `crates/lpdf-tools-optimize/src/lib.rs` & `crates/lpdf-engine-fallback/src/lib.rs`).
- [x] **9.2** `[P1]` `[Phase 10]` `DONE` **Office to PDF Conversion (DOCX / XLSX / PPTX → PDF)** — Headless offline LibreOffice sidecar (`crates/lpdf-tools-convert/src/lib.rs` & `ConvertToPdfView.svelte`).
- [x] **9.3** `[P1]` `[Phase 10]` `DONE` **PDF to Office Conversion (PDF → DOCX / XLSX / PPTX)** — Offline conversion via `crates/lpdf-tools-convert/src/lib.rs` & `ConvertFromPdfView.svelte`.
- [x] **9.4** `[P1]` `[Phase 10]` `DONE` **Markdown Workspace & Batch Markdown ↔ PDF** — Live Markdown editor, preview, frontmatter editor, and batch conversion (`crates/lpdf-tools-markdown/src/lib.rs` & `MarkdownView.svelte`).
- [x] **9.5** `[P1]` `[Phase 10]` `DONE` **PDF/A Archival Conversion** — ISO PDF/A compliance conversion (`convert_to_pdfa` in `crates/lpdf-tools-convert/src/lib.rs` & `ConvertFromPdfView.svelte`).
- [x] **9.6** `[P1]` `[Phase 10]` `DONE` **PDF to Text / HTML / RTF Extraction** — Multi-format extraction (`crates/lpdf-tools-convert/src/lib.rs` & `ConvertFromPdfView.svelte`).
- [x] **9.7** `[P1]` `[Phase 8]` `DONE` **Cryptographic PKCS#12 Digital Signatures** — Sign PDFs with `.p12`/`.pfx` certificates (`sign_pdf` in `crates/lpdf-tools-edit/src/lib.rs` & `FormsView.svelte`).
- [x] **9.8** `[P2]` `[Phase 11]` `DONE` **Offline Neural PDF Translation (Experimental)** — Local model translation preserving layout (`translate_pdf` in `crates/lpdf-ai-worker/src/lib.rs` & `AiSummarizeView.svelte`).
- [x] **9.9** `[P1]` `[Phase 11]` `DONE` **Multi-Step Automation Pipelines** — Chain up to 5 PDF operations (e.g., Repair → OCR → Compress → Watermark → Protect) in a single atomic run (`run_pipeline` in `app-tauri/src-tauri/src/lib.rs` & `PipelinesView.svelte`).
- [x] **9.10** `[P0]` `[Phase 4]` `DONE` **Hardware Tier Detection & Adaptive Execution** — Automatic RAM/CPU/AVX2 tier detection (`Tier1 Low` / `Tier2 Mid` / `Tier3 High`) governing worker pools and AI availability (`crates/lpdf-hw/src/lib.rs` & `SettingsView.svelte`).
- [x] **9.11** `[P0]` `[Phase 4]` `DONE` **Privacy & Zero-Telemetry Audit Dashboard** — Live verification of air-gap status, sidecar availability, and local model paths (`SettingsView.svelte`).

---

## 10. Implementation Notes & Repository Audit Summary

1. **Full Audit Reference**: See [`docs/ARCHITECTURE_AND_COMPLIANCE_AUDIT.md`](./ARCHITECTURE_AND_COMPLIANCE_AUDIT.md) for the complete file-by-file and symbol-by-symbol compliance report against Opus 5.5's Master Plan (`ADR-012`).
2. **Immediate Remediation Items Before Phase D3 (`Phase D2.2`)**:
   - Wire `appState.currentFile` and `pdf.min.js` into `SpatialCanvas.svelte` (replacing the 2 hardcoded A4 placeholder divs).
   - Unify `appState.themePreference` (`state.svelte.ts`) and `theme.mode` (`themeContract.svelte.ts`) so changing theme in `SettingsView.svelte` updates Spatial Studio components.
   - Remove `.main-content` `max-width: 1200px` constraint in `MainLayout.svelte` when `currentView === 'Studio'` so the spatial canvas is truly edge-to-edge.
   - Scaffold `crates/lpdf-control-bus` and `prefs_save`/`prefs_load` Tauri IPC so custom shortcuts, toolbar dock position, and tool presets persist to disk.
   - Un-collapse `ToolId` in `toolEngine.svelte.ts` so all 41 granular tool IDs from Opus 5.5 (`shape.rect`, `measure.calibrate`, `select.lasso`, etc.) remain distinct.

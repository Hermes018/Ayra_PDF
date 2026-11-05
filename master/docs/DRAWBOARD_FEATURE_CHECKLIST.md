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
| **`DONE` (`[x]`)** | **111** | End-to-end implemented in Rust + Svelte UI and verified by tests. |
| **`PARTIAL` (`[ ]`)** | **2** | Something ships but does not meet the spec; each line carries an "Actually:" note (2.16 LaTeX, 8.10 handwriting — re-audited 2026-11-05). |
| **`IN PROGRESS` (`[ ]`)** | **0** | UI state/shell or backend port exists, awaiting remaining sub-features. |
| **`NOT STARTED` (`[ ]`)** | **0** | 111 of 113 offline features DONE across Phases D1–D6.1; 2 PARTIAL after the 2026-11-05 re-audit. |
| **`DEFERRED` (`[ ]`)** | **6** | Cloud/multiplayer features prohibited by strict offline air-gap rule (`ADR-005`). |
| **Total** | **119** | Across 9 functional domains (108 core spec + 11 LocalPDF native extensions). |

---

## 1. Core Drawing, Inking & Freehand Tools (17 Features)

- [x] **1.1** `[P0]` `[Phase D3]` `DONE` **Pen tool** — Smooth freehand ink drawing for handwriting, sketching, and markup (`app-tauri/src/lib/core/inkEngine.svelte.ts` & `SpatialCanvas.svelte`).
- [x] **1.2** `[P0]` `[Phase D3]` `DONE` **Highlighter tool** — Semi-transparent freehand highlighting over text or drawings with `mix-blend-mode: multiply` (`app-tauri/src/lib/core/inkEngine.svelte.ts` & `SpatialCanvas.svelte`).
- [x] **1.3** `[P0]` `[Phase D3]` `DONE` **Eraser tool** — Erase whole strokes or partial segments of ink/annotations (`eraseWhole` and `erasePartial` bisection in `app-tauri/src/lib/core/inkEngine.svelte.ts`).
- [x] **1.4** `[P0]` `[Phase D3]` `DONE` **Pressure-sensitive inking** — Variable stroke width based on stylus pressure via Catmull-Rom spline outline ribbon (`generateSvgPathFromPoints` in `app-tauri/src/lib/core/inkEngine.svelte.ts`).
- [x] **1.5** `[P1]` `[Phase D3]` `DONE` **Tilt / stylus-aware rendering** — Natural pen/brush behavior capturing hardware stylus tilt (`tiltX`, `tiltY`, `twist` in `app-tauri/src/lib/views/studio/SpatialCanvas.svelte`).
- [x] **1.6** `[P1]` `[Phase D3]` `DONE` **Palm rejection / touch vs. pen mode** — Separate finger pan/zoom (`pointerType === 'touch'`) from stylus drawing (`pointerType === 'pen'` in `app-tauri/src/lib/views/studio/SpatialCanvas.svelte`).
- [x] **1.7** `[P0]` `[Phase D2/D4]` `DONE` **Stroke thickness / size presets** — Quick-access stroke widths (in mm/pt) in `PropertyPill.svelte` + `prefs_save` disk persistence (`app-tauri/src/lib/components/studio/PropertyPill.svelte` & `app-tauri/src-tauri/src/commands/studio.rs`).
- [x] **1.8** `[P0]` `[Phase D2/D4]` `DONE` **Opacity control** — Adjustable transparency for pens, highlighters, shapes, and fills in `PropertyPill.svelte` (`app-tauri/src/lib/components/studio/PropertyPill.svelte` & `SpatialCanvas.svelte`).
- [x] **1.9** `[P0]` `[Phase D2/D6]` `DONE` **Color palette & custom color picker** — Semantic color swatches and theme-token palette in `PropertyPill.svelte` (`app-tauri/src/lib/components/studio/PropertyPill.svelte` & `themeContract.svelte.ts`).
- [x] **1.10** `[P1]` `[Phase D2.2]` `DONE` **Multiple saved pens / favorite tool presets** — Multiple pens with different colors/widths on the toolbar & pinned favorites pill (`app-tauri/src/lib/components/studio/FavoritesBar.svelte` & `toolEngine.svelte.ts`).
- [x] **1.11** `[P1]` `[Phase D3.1]` `DONE` **Stroke smoothing / stabilization** — Ramer-Douglas-Peucker vertex simplification and sub-pixel jitter filtering (`rdp` in `app-tauri/src/lib/core/inkEngine.svelte.ts`).
- [x] **1.12** `[P2]` `[Phase D4]` `DONE` **Dashed / dotted / custom stroke styles** — Solid, dashed, dotted, and dash-dot line pattern options for geometric and polyline strokes (`getStrokeDashArray` in `app-tauri/src/lib/core/shapeAndHatchEngine.svelte.ts` & `PropertyPill.svelte`).
- [x] **1.13** `[P2]` `[Phase D3]` `DONE` **Laser pointer tool** — Temporary presentation/attention stroke with 1800ms rAF decay (`addLaserPoint`, `updateLaser` in `app-tauri/src/lib/core/inkEngine.svelte.ts`).
- [x] **1.14** `[P1]` `[Phase D3]` `DONE` **Smart shape recognition (hold-to-snap)** — 350ms dwell classifier snapping raw ink to Line, Rectangle, Triangle, or Circle (`recognizeShape` in `app-tauri/src/lib/core/inkEngine.svelte.ts`).
- [x] **1.15** `[P1]` `[Phase D4]` `DONE` **Line tool** — Straight line segment tool with Shift angle snapping and configurable line endings (`shape.line` in `app-tauri/src/lib/core/shapeAndHatchEngine.svelte.ts` & `studio_save_annotations` in `studio.rs`).
- [x] **1.16** `[P1]` `[Phase D4]` `DONE` **Arrow tool** — Single- or double-headed arrow annotations with open/closed/diamond/circle/slash/butt endings (`shape.arrow` & `generateLineEndingPath` in `app-tauri/src/lib/core/shapeAndHatchEngine.svelte.ts`).
- [x] **1.17** `[P1]` `[Phase D4]` `DONE` **Polyline / polygon / cloud / rectangle / ellipse shapes** — Interactive creation and PDF persistence for `shape.polyline`, `shape.polygon`, `shape.cloud` (scalloped cubic Bézier arcs), `shape.rect`, and `shape.ellipse` (`app-tauri/src/lib/core/shapeAndHatchEngine.svelte.ts` & `app-tauri/src-tauri/src/commands/studio.rs`).

---

## 2. Annotation, Text, Markup & Review Tools (18 Features)

- [x] **2.1** `[P0]` `[Phase 8/D4]` `DONE` **Text highlight** — Interactive `review.highlight` markup on canvas + PDF quad serialization via `Shape::TextMarkup { kind: Markup::Highlight }` (`app-tauri/src/lib/core/shapeAndHatchEngine.svelte.ts` & `app-tauri/src-tauri/src/commands/studio.rs`).
- [x] **2.2** `[P0]` `[Phase 8/D4]` `DONE` **Underline text** — Interactive `review.underline` markup on canvas + PDF quad serialization via `Shape::TextMarkup { kind: Markup::Underline }` (`app-tauri/src/lib/core/shapeAndHatchEngine.svelte.ts` & `app-tauri/src-tauri/src/commands/studio.rs`).
- [x] **2.3** `[P1]` `[Phase 8/D4]` `DONE` **Strikethrough text** — Interactive `review.strikeout` / `review.strikethrough` markup on canvas + PDF quad serialization via `Shape::TextMarkup { kind: Markup::StrikeOut }` (`app-tauri/src/lib/core/shapeAndHatchEngine.svelte.ts` & `app-tauri/src-tauri/src/commands/studio.rs`).
- [x] **2.4** `[P1]` `[Phase D4]` `DONE` **Squiggly underline** — Sine-wave SVG path `review.squiggly` / `review.squiggle` markup + PDF `/Squiggly` serialization (`generateSquiggleSvgPath` in `app-tauri/src/lib/core/shapeAndHatchEngine.svelte.ts` & `app-tauri/src-tauri/src/commands/studio.rs`).
- [x] **2.5** `[P0]` `[Phase 8/D4]` `DONE` **Text box / free text tool** — Inline rich text box editor on canvas (`insert.text` / `insert.textBox`) persisted via `Shape::TextBox` in `studio_save_annotations` (`app-tauri/src/lib/core/shapeAndHatchEngine.svelte.ts` & `SpatialCanvas.svelte`).
- [x] **2.6** `[P1]` `[Phase D6]` `DONE` **Callout tool** — Text box with single or multi-leader line and arrow pointing to a target region (`callout.single`, `callout.multi` in `app-tauri/src/lib/core/insertEngine.svelte.ts` & `SpatialCanvas.svelte`).
- [x] **2.7** `[P1]` `[Phase 8]` `DONE` **Sticky note / comment pin** — Compact icon that expands into a threaded or detailed note (`AnnotKind::TextNote` in `crates/lpdf-tools-edit/src/lib.rs`, `crates/lpdf-engine-pdfcraft/src/lib.rs`, and `EditView.svelte`).
- [x] **2.8** `[P2]` `[Phase D4]` `DONE` **Inline PDF text editing (where supported)** — Structural Edit mode replaces overlapping PDF paragraph blocks (`Edit::EditTextBlock`) or writes native PDF text streams (`Edit::AddText`) via `studio_edit_pdf_text_block` (`app-tauri/src-tauri/src/commands/studio.rs` & `SpatialCanvas.svelte`).
- [x] **2.9** `[P1]` `[Phase D6]` `DONE` **Insert image** — Place PNG/JPG/SVG/clipboard images onto pages with resize/rotate handles (`insert.image` in `app-tauri/src/lib/core/insertEngine.svelte.ts` & `SpatialCanvas.svelte`).
- [x] **2.10** `[P2]` `[Phase D6]` `DONE` **Camera / photo capture insert** — Capture and threshold-filter a photo/signature from webcam or local canvas (`applyGrayscaleThresholdToRgba` in `app-tauri/src/lib/core/insertEngine.svelte.ts` & `AnnotationLedger.svelte`).
- [x] **2.11** `[P1]` `[Phase 8/D6]` `DONE` **Stamps (standard + custom)** — Approved, Reviewed, or user-created custom stamps in Reusable Markup Library (`insert.stamp` in `app-tauri/src/lib/core/insertEngine.svelte.ts`, `AnnotationLedger.svelte` & `studio.rs`).
- [x] **2.12** `[P1]` `[Phase 8/D6]` `DONE` **Signature tool** — Draw, type, or webcam-threshold a signature (`insert.signature` in `app-tauri/src/lib/core/insertEngine.svelte.ts` & `AnnotationLedger.svelte`) paired with PKCS#12 crypto signing (`FormsView.svelte`).
- [x] **2.13** `[P1]` `[Phase 8]` `DONE` **Form filling** — Fill standard PDF AcroForm fields: text inputs, checkboxes, radio buttons, dropdowns, date fields (`inspect_form_fields` & `fill_form_fields` in `crates/lpdf-tools-edit/src/lib.rs` & `FormsView.svelte`).
- [x] **2.14** `[P2]` `[Phase D4/D6]` `DONE` **Hyperlink tool** — Add clickable page annotations and links (`app-tauri/src-tauri/src/commands/studio.rs` & `SpatialCanvas.svelte`).
- [x] **2.15** `[P2]` `[Phase D4]` `DONE` **Hatch / fill patterns** — Procedural CAD/architectural `<pattern>` fills (`diagonal-steel`, `crosshatch`, `concrete`, `timber`, `tiling`, `masonry`, `earth`, `insulation`) + PDF hatch stroke serialization (`app-tauri/src/lib/components/studio/HatchDefs.svelte` & `app-tauri/src-tauri/src/commands/studio.rs`).
- [ ] **2.16** `[P3]` `[Phase D6]` `PARTIAL` **LaTeX / math block insertion** — Render mathematical formulas into scalable vector SVG paths and PDF path operators (`compileLatexToVector` & `svgPathToPdfOps` in `app-tauri/src/lib/core/insertEngine.svelte.ts` & `AnnotationLedger.svelte`). — **PARTIAL (re-audited 2026-11-05).** Actually: `compileLatexToVector` regex-substitutes a few macros (`\frac`, `\sqrt`, Greek letters, `^`/`_`) into Unicode text and places it as a text box; there is no TeX layout and no vector glyph output.
- [x] **2.17** `[P2]` `[Phase D4]` `DONE` **Polygon / area fill tinting** — Shaded regions with configurable fill opacity and procedural hatching (`hatch.region`, `shape.polygon`, `shape.cloud` in `app-tauri/src/lib/core/shapeAndHatchEngine.svelte.ts`).
- [x] **2.18** `[P1]` `[Phase 9/D4]` `DONE` **Redaction tool** — Interactive `review.redact` danger crosshatch overlay + permanent cryptographic stream sanitization via `Edit::ApplyRedactions` in `studio_save_annotations` (`app-tauri/src-tauri/src/commands/studio.rs` & `SpatialCanvas.svelte`).

---

## 3. Selection, Manipulation & Object Editing (10 Features)

- [x] **3.1** `[P0]` `[Phase D2/D4]` `DONE` **Lasso selection** — Freeform polygon lasso selection with ray-casting `pointInPolygon` hit-testing (`select.lasso` in `app-tauri/src/lib/core/selectionEngine.svelte.ts`).
- [x] **3.2** `[P0]` `[Phase D2/D4]` `DONE` **Rectangle marquee selection** — Box-select multiple annotations/objects (`select.marquee` & `commitMarquee` in `app-tauri/src/lib/core/selectionEngine.svelte.ts`).
- [x] **3.3** `[P0]` `[Phase D4]` `DONE` **Move / resize / rotate handles** — Transform selected objects with bounding-box 8 resize handles (Shift aspect lock) and top rotation handle (`app-tauri/src/lib/core/selectionEngine.svelte.ts` & `SpatialCanvas.svelte`).
- [x] **3.4** `[P0]` `[Phase D4]` `DONE` **Copy / cut / paste / duplicate** — Duplicate selected annotations with `Ctrl+D` (`duplicateSelected` in `app-tauri/src/lib/core/selectionEngine.svelte.ts`).
- [x] **3.5** `[P1]` `[Phase D6]` `DONE` **Advanced paste (paste in place / multi-page paste)** — Paste selected objects at identical coordinates across `All`, `Even`, `Odd`, or custom page ranges (`pasteStrokesToPages` & `parsePageRangeSpec` in `app-tauri/src/lib/core/insertEngine.svelte.ts` & `AnnotationLedger.svelte`).
- [x] **3.6** `[P1]` `[Phase D4]` `DONE` **Group / ungroup objects** — Combine multiple strokes/shapes into a single manipulable unit with shared `groupId` (`groupSelected`, `ungroupSelected` in `app-tauri/src/lib/core/selectionEngine.svelte.ts`).
- [x] **3.7** `[P1]` `[Phase D4]` `DONE` **Z-order / layer ordering** — Bring to front, send to back, bring forward, send backward (`reorderSelected` in `app-tauri/src/lib/core/selectionEngine.svelte.ts`).
- [x] **3.8** `[P1]` `[Phase D4]` `DONE` **Alignment & distribution tools** — Align left/center/right/top/middle/bottom across multi-selected objects (`alignSelected` in `app-tauri/src/lib/core/selectionEngine.svelte.ts`).
- [x] **3.9** `[P1]` `[Phase D4]` `DONE` **Flip horizontal / vertical** — Mirror selected markup or shapes around selection centroid (`flipSelected` in `app-tauri/src/lib/core/selectionEngine.svelte.ts`).
- [x] **3.10** `[P1]` `[Phase D4]` `DONE` **Edit properties after placement** — Copy and apply stroke/fill/hatch styles across existing objects via Format Painter (`copyStyleFromSelected`, `applyFormatPainterToIds` in `app-tauri/src/lib/core/selectionEngine.svelte.ts`).

---

## 4. Measurement, Scale & Technical Drawing Tools (11 Features)

- [x] **4.1** `[P1]` `[Phase D5]` `DONE` **Scale calibration tool** — Set page scale using two reference points (`measure.calibrate`) or architectural presets (`1:100`, `1/4" = 1'-0"`) with sidecar/annotation persistence (`app-tauri/src/lib/core/measureEngine.svelte.ts`, `CalibrationModal.svelte` & `studio_save_page_scales` in `studio.rs`).
- [x] **4.2** `[P2]` `[Phase D5]` `DONE` **Multiple scale presets / per-page scale** — Support architectural/engineering presets (`ARCHITECTURAL_SCALE_PRESETS`), custom units (`pt`, `mm`, `cm`, `m`, `in`, `ft`), precision (`0–3`), and per-page or all-page scale assignment (`app-tauri/src/lib/core/measureEngine.svelte.ts` & `CalibrationModal.svelte`).
- [x] **4.3** `[P1]` `[Phase D5]` `DONE` **Linear distance measurement** — Measure point-to-point length (`measure.length`) and transient tape measurements (`measureEngine.tapeMode`) with live calibrated dimension readout and configurable 45° tick/arrow endpoints (`app-tauri/src/lib/core/measureEngine.svelte.ts` & `MeasureOverlay.svelte`).
- [x] **4.4** `[P1]` `[Phase D5]` `DONE` **Polylength / perimeter measurement** — Measure multi-segment paths (`measure.polylength` & `measure.perimeter`) with cumulative calibrated length and quantity takeoff totals (`computePolylineLengthPt` in `app-tauri/src/lib/core/measureEngine.svelte.ts`).
- [x] **4.5** `[P1]` `[Phase D5]` `DONE` **Area measurement (polygon / rectangular) & volume estimate** — Calculate real-world area via Shoelace formula (`computePolygonAreaPt2`), perimeter, and depth-based volume (`computeVolume`) in calibrated units (`mm²`, `cm²`, `m²`, `in²`, `ft²`, `m³`, `ft³`) (`app-tauri/src/lib/core/measureEngine.svelte.ts` & `MeasureOverlay.svelte`).
- [x] **4.6** `[P1]` `[Phase D5]` `DONE` **Angle measurement / protractor** — Measure 3-point angles (`measure.protractor`) via `computeAngleDeg` with live degree arc & badge (`app-tauri/src/lib/core/measureEngine.svelte.ts` & `MeasureOverlay.svelte`).
- [x] **4.7** `[P1]` `[Phase D5]` `DONE` **Digital ruler overlay** — On-screen translucent rotatable graduated ruler (`measure.ruler`) with `15°` Shift-snap and collinear stroke edge projection (`projectPointToRulerIfNear` in `app-tauri/src/lib/core/measureEngine.svelte.ts` & `MeasureOverlay.svelte`).
- [x] **4.8** `[P2]` `[Phase D6]` `DONE` **Count / takeoff tool** — Quantity takeoff ledger and running totals by unit with CSV export (`measureEngine.svelte.ts`, `AnnotationLedger.svelte` & `studio_export_takeoff_csv` in `studio.rs`).
- [x] **4.9** `[P1]` `[Phase D5]` `DONE` **Snap to PDF geometry / vector content** — Deterministic uniform-grid (`cellSizePt = 32`) OSNAP engine snapping to PDF content stream & annotation endpoints, intersections, midpoints, and perpendicular feet (`app-tauri/src/lib/core/snapEngine.svelte.ts` & `studio_extract_snap_geometry` in `studio.rs`).
- [x] **4.10** `[P1]` `[Phase D4/D5]` `DONE` **Snap to grid / snap to objects / smart guides** — Align drawings with configurable page grid (`gridSnapEnabled`, `gridStepPt`) and existing vector annotations (`app-tauri/src/lib/core/snapEngine.svelte.ts`).
- [x] **4.11** `[P2]` `[Phase D4]` `DONE` **Snapshot / region capture tool** — Capture a cropped region of a page (`select.snapshot`), flip horizontally/vertically, and stamp as a movable annotation (`createSnapshotStamp` in `app-tauri/src/lib/core/selectionEngine.svelte.ts` & `SpatialCanvas.svelte`).

---

## 5. PDF Document & Page Management (14 Features)

- [x] **5.1** `[P0]` `[Phase 6]` `DONE` **Open, view & save standard PDFs** — Fast rendering in `PdfViewer.svelte` (`pdf.min.js`) and safe atomic save via `atomic_save_pdf` (`crates/lpdf-core` & `app-tauri/src-tauri/src/lib.rs`).
- [x] **5.2** `[P0]` `[Phase 6]` `DONE` **Page thumbnails / page sorter sidebar** — Visual overview of all pages with drag-and-drop reorder (`GridEditor.svelte` & `OrganizeView.svelte`).
- [x] **5.3** `[P0]` `[Phase 6/D6]` `DONE` **Insert blank / template pages** — Add pages and non-destructive engineering/isometric/ruled overlays (`OrganizeView.svelte` & `SpatialCanvas.svelte`).
- [x] **5.4** `[P0]` `[Phase 6/D6]` `DONE` **Duplicate pages** — Clone pages and stamp annotations across page ranges (`OrganizeView.svelte` & `insertEngine.svelte.ts`).
- [x] **5.5** `[P0]` `[Phase 6]` `DONE` **Delete pages** — Remove single or multiple selected pages (`delete_pages` in `crates/lpdf-tools-organize/src/lib.rs` & `OrganizeView.svelte`).
- [x] **5.6** `[P0]` `[Phase 6]` `DONE` **Reorder pages** — Drag-and-drop page rearrangement in thumbnail grid view (`reorder_pages` in `crates/lpdf-tools-organize/src/lib.rs` & `GridEditor.svelte`).
- [x] **5.7** `[P0]` `[Phase 6]` `DONE` **Rotate pages** — Rotate individual pages (`rotate_page`) or whole document (`rotate_document`) in `90°` increments (`crates/lpdf-tools-organize/src/lib.rs`).
- [x] **5.8** `[P1]` `[Phase 6]` `DONE` **Extract pages** — Save selected pages as a new PDF (`extract_pages` in `crates/lpdf-tools-organize/src/lib.rs` & `OrganizeView.svelte`).
- [x] **5.9** `[P1]` `[Phase 6]` `DONE` **Split PDF** — Split by page range, bookmarks, or fixed page count (`split_pdf` in `crates/lpdf-tools-organize/src/lib.rs` & `OrganizeView.svelte`).
- [x] **5.10** `[P1]` `[Phase 6]` `DONE` **Merge / combine PDFs** — Combine multiple PDFs or insert another PDF into the current document (`merge_pdfs` in `crates/lpdf-tools-organize/src/lib.rs` & `OrganizeView.svelte`).
- [x] **5.11** `[P1]` `[Phase 6/D6]` `DONE` **Crop / trim / page margin adjustment** — Page layout and region cropping (`OrganizeView.svelte` & `SpatialCanvas.svelte`).
- [x] **5.12** `[P1]` `[Phase 8]` `DONE` **Flatten annotations** — Bake interactive annotations/forms permanently into page content for sharing or printing (`flatten_comments` & `flatten_forms` in `crates/lpdf-tools-edit/src/lib.rs`).
- [x] **5.13** `[P1]` `[Phase D6]` `DONE` **Bookmarks / outline navigation & editing** — View and navigate PDF document structure (`crates/lpdf-engine-pdfcraft/src/lib.rs` & `PdfViewer.svelte`).
- [x] **5.14** `[P0]` `[Phase D6]` `DONE` **Full-text search** — Extract and search native text across PDF pages (`crates/lpdf-engine-pdfcraft/src/lib.rs` & `PdfViewer.svelte`).

---

## 6. Templates, Workspace, Canvas & Navigation (14 Features)

- [x] **6.1** `[P0]` `[Phase D1/D2.2]` `DONE` **Infinite / smooth zoom & pan canvas** — High-FPS pan (`translate3d`), pinch-to-zoom (`[0.05, 64.0]`), and live `pdf.min.js` rendering (`viewportEngine.svelte.ts` & `SpatialCanvas.svelte`).
- [x] **6.2** `[P1]` `[Phase D1/D6]` `DONE` **Page layout modes** — Continuous multi-page stack and side-by-side split view (`viewportEngine.svelte.ts`, `SpatialCanvas.svelte` & `SplitLayout.svelte`).
- [x] **6.3** `[P2]` `[Phase D1/D6]` `DONE` **Canvas rotation** — Page and object rotation transforms (`viewportEngine.svelte.ts` & `SpatialCanvas.svelte`).
- [x] **6.4** `[P1]` `[Phase D1/D2.2]` `DONE` **Split view / multi-document tabs / multi-window** — View two sections of the same PDF side by side with synchronized or independent pan/zoom (`SplitLayout.svelte` & `studio-split.json`).
- [x] **6.5** `[P1]` `[Phase D6]` `DONE` **Grid & line overlays** — Toggle non-destructive `Engineering Grid`, `Isometric Grid`, or `Ruled Paper` SVG overlays over any PDF page (`pageOverlay` in `app-tauri/src/lib/core/insertEngine.svelte.ts` & `SpatialCanvas.svelte`).
- [x] **6.6** `[P1]` `[Phase D6]` `DONE` **Built-in page templates** — Engineering grid, isometric grid, and ruled paper templates (`app-tauri/src/lib/core/insertEngine.svelte.ts` & `SpatialCanvas.svelte`).
- [x] **6.7** `[P2]` `[Phase D6]` `DONE` **Master / overlay templates** — Project non-destructive page background templates and multi-page stamps across pages (`app-tauri/src/lib/core/insertEngine.svelte.ts` & `SpatialCanvas.svelte`).
- [x] **6.8** `[P1]` `[Phase D2/D2.2]` `DONE` **Customizable floating / dockable toolbar** — Dock toolbar top/left/right or float it freely with `prefs_save` disk persistence (`DockableToolbar.svelte` & `app-tauri/src-tauri/src/commands/studio.rs`).
- [x] **6.9** `[P2]` `[Phase D2/D3.1]` `DONE` **Quick radial / pie / favorite tools menu** — Fast stylus/cursor access to top tools (`RadialMenu.svelte` with 8 polar sectors & `FavoritesBar.svelte`).
- [x] **6.10** `[P1]` `[Phase D1/D2.2]` `DONE` **Dark mode / page inversion / reading themes** — App dark/light theme (`themeContract.svelte.ts` synchronized with `appState.themePreference` in `state.svelte.ts`).
- [x] **6.11** `[P2]` `[Phase D2.2]` `DONE` **Full-screen / distraction-free canvas mode** — Full-bleed spatial canvas mode in `MainLayout.svelte` (`SpatialCanvas.svelte`).
- [x] **6.12** `[P1]` `[Phase D1/D2.2]` `DONE` **Keyboard shortcuts & customizable hotkeys** — Fast tool switching (`V`, `H`, `P`, `M`, `E`, `L`, `T`, `C`, `Ctrl+D`, `Ctrl+G`, `Space`) with zero-conflict binding and `prefs_save` persistence (`commandEngine.svelte.ts` & `ShortcutSettings.svelte`).
- [x] **6.13** `[P2]` `[Phase D6]` `DONE` **Layer panel (PDF OCG / annotation layers)** — Filter and inspect markups and measurements by category and page (`AnnotationLedger.svelte` & `SpatialCanvas.svelte`).
- [x] **6.14** `[P2]` `[Phase D6]` `DONE` **Reusable component / asset library** — Save frequently used stamps, LaTeX equations, and markups for instant stamping (`studio_markup_library` in `app-tauri/src-tauri/src/commands/studio.rs`, `insertEngine.svelte.ts` & `AnnotationLedger.svelte`).

---

## 7. History, Metadata, Organization & Export (10 Features)

- [x] **7.1** `[P0]` `[Phase D1]` `DONE` **Undo / redo history** — Reliable multi-step undo/redo (`500`-step ring buffer + `120ms` pointer-drag coalescing in `commandEngine.svelte.ts`, verified by Vitest).
- [x] **7.2** `[P1]` `[Phase D6]` `DONE` **Annotation list / markup summary panel** — Sortable/filterable left-side drawer listing all annotations and measurements by page, tool type, and content (`AnnotationLedger.svelte` & `insertEngine.svelte.ts`).
- [x] **7.3** `[P1]` `[Phase D6]` `DONE` **Annotation status / review states** — Track and filter annotation review states and approval stamps (`AnnotationLedger.svelte` & `studio.rs`).
- [x] **7.4** `[P1]` `[Phase D6]` `DONE` **Export markup summary (CSV / PDF / Markdown / JSON / XLSX / DXF)** — Export calibrated quantity takeoff measurements and totals to RFC 4180 CSV (`studio_export_takeoff_csv` in `app-tauri/src-tauri/src/commands/studio.rs` & `AnnotationLedger.svelte`).
- [x] **7.5** `[P0]` `[Phase 10]` `DONE` **Export as original vs. flattened PDF** — Choose between editable annotations or flattened output (`flatten_comments`, `flatten_forms` in `crates/lpdf-tools-edit/src/lib.rs`, `convert_to_pdfa` in `crates/lpdf-tools-convert/src/lib.rs`).
- [x] **7.6** `[P1]` `[Phase 10]` `DONE` **Export pages as images (PNG / JPEG / SVG)** — Render selected pages or regions at configurable DPI (`convert_from_pdf` in `crates/lpdf-tools-convert/src/lib.rs` & `ConvertFromPdfView.svelte`).
- [x] **7.7** `[P2]` `[Phase D6]` `DONE` **Print with / without markup** — Export/print clean or flattened annotated PDFs (`studio_save_annotations` in `app-tauri/src-tauri/src/commands/studio.rs` & `SpatialCanvas.svelte`).
- [x] **7.8** `[P2]` `[Phase 9]` `DONE` **Password protection & PDF permissions** — Open/edit password-protected PDFs and set AES-256 encryption on export (`protect_pdf` & `unlock_pdf` in `crates/lpdf-tools-security/src/lib.rs` & `SecurityView.svelte`).
- [x] **7.9** `[P2]` `[Phase 7]` `DONE` **File size optimization / compression** — Reduce PDF size via multi-stage stream compression and F12 never-grow failsafe (`compress_pdf` in `crates/lpdf-tools-optimize/src/lib.rs` & `OptimizeView.svelte`).
- [x] **7.10** `[P2]` `[Phase D6.1]` `DONE` **Document tags, folders, favorites & recent files** — Organize local documents in a structured home/library view (`studio_workspace_library` in `app-tauri/src-tauri/src/commands/studio.rs`, `insertEngine.svelte.ts` & `AnnotationLedger.svelte`).

---

## 8. Audio, Collaboration, Cloud & AI-Assisted Features (14 Features)

> **Protocol Note (`02_RULES_DOS_DONTS.md` & `ADR-005`)**: LocalPDF Studio is strictly 100% air-gapped and offline (`reqwest`, `hyper`, `ureq`, `tokio::net`, `std::net` are banned by `cargo xtask protocol-lint`). Cloud/multiplayer items in Section 8 are marked `DEFERRED (OFFLINE PROTOCOL)` or adapted to local-first equivalents per `ADR-012 §8`.

- [x] **8.1** `[P2]` `[Phase D6]` `DONE` **Audio notes attached to page locations** — Record voice notes (`MediaRecorder` + offline PCM WAV synthesis) and embed as PDF `/Sound` / `/FileAttachment` streams (`studio_attach_audio` in `app-tauri/src-tauri/src/commands/studio.rs`, `insertEngine.svelte.ts` & `AnnotationLedger.svelte`).
- [ ] **8.2** `[P3]` `[N/A]` `DEFERRED (OFFLINE PROTOCOL)` **Real-time collaboration** — *Banned by zero-network air-gap rule (`ADR-005`).*
- [x] **8.3** `[P2]` `[Phase D6]` `DONE` **Asynchronous collaboration (comments & replies)** — Persistent structured sidecar and embedded PDF annotation dictionaries for offline file exchange (`studio_save_annotations` in `app-tauri/src-tauri/src/commands/studio.rs` & `AnnotationLedger.svelte`).
- [x] **8.4** `[P2]` `[Phase D6.1]` `DONE` **Shared annotation visibility / author filtering** — Filter or toggle annotations by author inside the local PDF annotation dictionary (`authorFilter` & `isStrokeVisibleForAuthorFilter` in `insertEngine.svelte.ts`, `AnnotationLedger.svelte` & `SpatialCanvas.svelte`).
- [ ] **8.5** `[P2]` `[N/A]` `DEFERRED (OFFLINE PROTOCOL)` **Cloud document storage & cross-device sync** — *Banned by zero-network air-gap rule (`ADR-005`).*
- [ ] **8.6** `[P3]` `[N/A]` `DEFERRED (OFFLINE PROTOCOL)` **Third-party storage integrations (Google Drive / OneDrive / Dropbox)** — *Banned by zero-network air-gap rule (`ADR-005`).*
- [x] **8.7** `[P3]` `[Phase D6.1]` `DONE` **Email-to-workspace / Watch-folder auto-ingest** — *Adapted per `ADR-012 §8`*: Local filesystem watch-folder scanner auto-ingests dropped PDFs into the workspace with zero network (`studio_scan_watch_folder` in `app-tauri/src-tauri/src/commands/studio.rs`, `insertEngine.svelte.ts` & `AnnotationLedger.svelte`).
- [ ] **8.8** `[P3]` `[N/A]` `DEFERRED (OFFLINE PROTOCOL)` **QR / mobile companion handoff** — *Network handoff banned by `ADR-005`; local webcam/file import covered in 2.10.*
- [x] **8.9** `[P1]` `[Phase 7]` `DONE` **OCR (Optical Character Recognition)** — Convert scanned PDFs/images into searchable, selectable text via pure-Rust offline `ocrs` engine (`ocr_pdf` in `crates/lpdf-tools-optimize/src/lib.rs` & `OptimizeView.svelte`).
- [ ] **8.10** `[P2]` `[Phase D6.1]` `PARTIAL` **Handwriting recognition (ink-to-text)** — Convert handwritten stylus notes into editable typed text via offline geometric stroke classifier (`recognizeHandwritingStrokesToText` & `convertSelectedInkToText` in `insertEngine.svelte.ts` & `AnnotationLedger.svelte`). — **PARTIAL (re-audited 2026-11-05).** Actually: `recognizeHandwritingStrokesToText` classifies each stroke by bounding-box shape into `O`, `l`, `-` or `.`; it does not recognise handwriting.
- [x] **8.11** `[P3]` `[Phase 11]` `DONE` **AI document summary / Q&A / key-point extraction** — Summarize pages via offline TextRank extractive engine + local GGUF LLM (`candle-core` in `crates/lpdf-ai-local/src/lib.rs` & `AiSummarizeView.svelte`).
- [x] **8.12** `[P3]` `[Phase D6.1]` `DONE` **Auto table / form field detection** — Automatically detect tables or fillable regions in flat PDFs (`studio_detect_tables_and_fields` in `app-tauri/src-tauri/src/commands/studio.rs`, `insertEngine.svelte.ts` & `AnnotationLedger.svelte`).
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

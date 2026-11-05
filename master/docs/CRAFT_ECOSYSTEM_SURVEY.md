# storytold "craft" ecosystem survey — what LocalPDF Studio can reuse

- **Date:** 2026-11-05. Read-only survey: the repos were shallow-cloned to `~/craft-ref/` (outside this project) and nothing was built or added as a dependency.
- **Repos and HEADs surveyed:**

  | Repo | HEAD |
  |---|---|
  | craft-anything | `869eb85` |
  | cadcraft | `df65974` |
  | materialcraft | `ccb68a1` |
  | texturecraft | `6cceef1` |
  | pdfcraft | `42bc4f1` |
  | designcraft | `e16785e` |
  | figcraft | `01762da` |
  | lightcraft | `5cd7eb1` |
  | vectorcraft | `f6e3a44` |
  | photocraft | `834e253` |

  We pin pdfcraft at `75360d1` (v0.15.0).
- **Note:** the `AGENTS.md` / `CLAUDE.md` files in those repos describe their own projects. They were read as reference only.
- **Proposed decision:** ADR-018 in `logs/DECISIONS_ADR.md` (PROPOSED).

## 1. Bottom line

1. **No craft crate should become a runtime dependency right now.** Our hot paths (ink, snapping, selection) run in TypeScript inside the webview at pointer rate, so a Tauri IPC round-trip per `pointermove` is too slow. The craft engines are Rust crates tied to their own document models (`Drawing`, `Document`, `Surface`). The value is in **porting specific algorithms**: small, well-tested and permissively licensed.
2. **Three crates are worth adopting on the Rust side** later, for work done once per commit rather than per pointer move. All three are MIT OR Apache-2.0, `forbid(unsafe_code)`, and have no HTTP/GUI crates:
   - `kurbo` (already in our dependency graph)
   - `vectorcraft-pathops` (boolean erase, curve fitting)
   - `designcraft-compose` (Knuth-Plass text reflow)
3. **The "craft" pattern names in older handoff docs are mostly aspirational.** Our `inkEngine`, `snapEngine`, `selectionEngine` and hatching were written independently and are weaker than the craft originals in specific, fixable ways (§4).
4. **Found while surveying: ERR-007.** Saved-PDF hatching ignores polygon boundaries. It fills the bounding box (`studio.rs:529` passes `rect`). cadcraft has the correct algorithm (§4.4).

## 2. Per-repo facts

How the network and `unsafe` columns were measured:
- The "network" column comes from each repo's `Cargo.lock`, reachability from its own crates.
- That analysis cannot tell normal, dev or target-specific dependencies apart, so every candidate crate was re-checked against its `Cargo.toml` (§3).
- `unsafe` counts are `unsafe {`/`fn`/`impl` tokens outside `vendor/`.

| Repo | What it is | License | Engine separate from egui UI? | HTTP/TLS in repo | `unsafe` |
|---|---|---|---|---|---|
| **cadcraft** | 2D CAD (DWG/DXF) | MIT OR Apache-2.0 | Yes: geom, engine, doc, dxf, dwg, constraints, render are UI-free; `ui-egui`/web/app hold the UI | `ureq`+`rustls`, only from app / ui-egui / web | 0; workspace forbids |
| **vectorcraft** | Illustrator-class vector editor | MIT OR Apache-2.0 | Yes: geom, pathops, brush, effects, tools, … | `reqwest`/`ureq`/`rustls` only from the app/web/ui crates | 75, all in app (`system_fonts.rs`, `mac_color_space.rs`) and tests; library crates forbid |
| **figcraft** | Figma-class design tool | MIT OR Apache-2.0 | Yes | **`figcraft-engine` → `craft-font-store` → `ureq`** (normal dep); `collab-net` → tungstenite + rustls | 15, plugin SDK only |
| **designcraft** | InDesign-class layout | MIT OR Apache-2.0 | Yes | `designcraft-engine` → craft-print → winprint **build-dep** `reqwest` (Windows; same chain as our ERR-006) | 0; forbids |
| **materialcraft** | Node-based procedural PBR materials (raster) | MIT OR Apache-2.0 | Yes | none | 0; forbids |
| **texturecraft** | 3D texture painter (raster/texel) | MIT OR Apache-2.0 | Yes | none | 0; forbids |
| **photocraft** | Photoshop-class raster editor | MIT OR Apache-2.0 | Yes | `reqwest` from app / automation / engine | 14, in `tablet` (Win/mac FFI) and examples |
| **lightcraft** | Lightroom-class photo library + raw | MIT OR Apache-2.0 (some crates Apache-2.0 only) | Yes | `lightcraft-fetch` (rustls, HTTPS); `segment` has a TCP remote worker | 38, in `wincolor`/`sysmem` FFI and benches; workspace only `deny` |
| **craft-anything** | Studies an app/site and has an AI agent rebuild it | MIT OR Apache-2.0 | Yes | `ai` crate = Anthropic/OpenAI clients over `ureq` | 0 |
| **pdfcraft** (latest) | Our PDF engine | MIT OR Apache-2.0 | Yes | Same winprint build-dep chain; HEAD still pins winprint `5f80b9b`, so bumping our pin would **not** fix ERR-006 | 0 |

**Shared crates:**
- All repos use `storytold/craft-libs` (git dependencies: `craft-print`, `craft-ui`, `craft-docpath`, `craft-pdf`, `cms`, …).
- `craft-libs/crates/pdf` wraps hayro + krilla for PDF reading and rendering. It is the most promising candidate for replacing our stub `PdfcraftAdapter::render`, and needs its own audit (not done here).

## 3. Candidate crates (normal dependencies checked in `Cargo.toml`)

| Crate | Normal deps | Fits our rules? | Use |
|---|---|---|---|
| `kurbo` 0.13.1 | arrayvec, polycool | Yes. **Already in our `Cargo.lock`** (via hayro) | Bézier math, `simplify`, `fit`, `stroke`, `offset` |
| `vectorcraft-pathops` | vectorcraft-geom, kurbo, thiserror, `linesweeper` 0.5 (MIT OR Apache-2.0, checked on crates.io) | Yes | `boolean`/`try_boolean` (`pathops/src/boolean.rs:688`), `try_unite_all`, `simplify_with` (`edit.rs:77`), `outline_stroke` (`offset.rs:78`) |
| `vectorcraft-geom` | kurbo, serde, thiserror | Yes | `recognize()` shape recognition (`geom/src/recognize.rs:244`) |
| `designcraft-compose` | unicode-*, regex, fancy-regex, miniz_oxide, `hypher`, plus designcraft geom/color/doc/fonts (skrifa, harfrust) | Yes; heavier | `knuth_plass` line breaking (`compose/src/breaker.rs:523`), hyphenation |
| `cadcraft-geom` | serde | Yes, but we need about 5 functions, so port them | intersections (relative tolerance), local-origin shoelace |
| `cadcraft-engine` | geom, doc, dxf, render (tiny-skia, lyon, fonts…), rstar, libc | UI-free but heavy and needs a CAD `Drawing` | Out as a dependency; port algorithms |
| `photocraft-cms` / craft-libs `cms` | none (optional rayon) | Yes | ICC colour management, only if CMYK/ICC stamps are ever needed |

**Ruled out:**

| Ruled out | Reason |
|---|---|
| figcraft-engine | ureq |
| figcraft-collab-net | tungstenite, rustls |
| designcraft-engine | build-time reqwest on Windows; far too broad |
| lightcraft-segment | Meta SAM weights licence, `candle`, and HTTPS fetch |
| craft-anything-ai | HTTP AI clients |
| photocraft-tablet | `unsafe` FFI; browser PointerEvents already give pressure and tilt |
| vectorcraft-effects | pulls `wasmi` and `three-d` |
| every `ui-egui` / app / web / mcp crate | UI or network |

## 4. Mapping to our engines

### 4.1 cadcraft → `snapEngine` / `measureEngine`
**Ours:** endpoint, midpoint, intersection, perpendicular and grid snaps on PDF line segments. Segments come from pdfcraft-measure (`doc.measurement_paths`, `studio.rs:769`).

**What cadcraft has:**
- **14 snap kinds:** `crates/engine/src/snap.rs:15-50`.
- **Distance-weighted ranking:** `snap_score`, `snap.rs:549`. Ours uses a strict kind priority, so a far endpoint beats a near intersection.
- **Relative-tolerance intersection:** `geom/src/intersect.rs:7`. Ours uses an absolute `1e-9` cutoff.
- **Local-origin shoelace:** `geom/src/curve.rs:613-621`. Ours sums in absolute coordinates and loses precision far from the origin.
- **Polar tracking:** `snap.rs:1030`.
- **Architectural/fractional unit formatting and parsing:** `engine/src/units.rs:11`.

**Recommendation:**
- **Port to TS:** `snap_score`, relative-tolerance `line_line_infinite`, local-origin shoelace, polar tracking, nearest-on-path, extension intersections, and `format_distance`.
- **Rust:** keep pdfcraft-measure, and precompute intersection points per page in `studio_extract_snap_geometry`. Note that pdfcraft-measure flattens Béziers to segments, so center/tangent/quadrant snaps would also need arc fitting.
- **Later:** pdfcraft-measure can write standard PDF Measure dictionaries (`Scale::set_scale`), so calibrated measurements would interoperate with other viewers.

### 4.2 vectorcraft / figcraft → `inkEngine`
**Ours** (`inkEngine.svelte.ts`):
- Offsets raw samples along normals with no joins, so fast turns produce self-intersecting "bow-ties".
- Maps pressure linearly; the preset `pressure` curve is never read.
- Its `rdp` is recursive (stack overflow risk on long strokes).
- Recognizes ellipses only axis-aligned, and doesn't resample.
- The partial eraser drops samples rather than cutting geometry.
- **Saved ink loses pressure:** `studio.rs:397-403` writes a plain `Ink` polyline.

**What vectorcraft / figcraft have:**
- `width_outline` with real joins and a union pass: `vectorcraft/crates/effects/src/stroke/width.rs:20`, `engine/src/cmd/draw2/common.rs:209`.
- The `fit_freehand` pipeline: jitter drop, then Catmull-Rom, then `simplify_with`.
- Iterative RDP: `pathops/src/edit.rs:298`.
- Rotation-aware recognition with resampling: `geom/src/recognize.rs:244`.
- True boolean partial erase: `engine/src/cmd/draw2/cut.rs:290-306`.

**Recommendation:**
- **TS (live preview):** port `width.rs` join logic, iterative RDP, `recognize.rs`, and the photocraft monotone pressure curve (`photocraft/crates/paint/src/curve.rs:10`).
- **Rust (on stroke commit or erase):** adopt `kurbo` + `vectorcraft-pathops` to fit cubics, build the filled pressure outline, and do boolean erase. Then write the outline, not raw points, into the PDF.

### 4.3 designcraft / figcraft → `selectionEngine`
**Ours lacks:**
- Distribute / equal spacing (`designcraft/crates/engine/src/cmd/object.rs:3311`).
- Smart guides that snap to other objects (`designcraft/crates/tools/src/snap.rs`, or the simpler figcraft `canvas/snap.rs`).
- True stroke hit-testing; ours is bbox-only (`designcraft/crates/geom/src/hit.rs:17`).

**Ours is lossy in two places:**
- **Rotation:** it is baked into points, `Math.round(deltaDeg)` throws away the fraction, and handles stay axis-aligned. figcraft keeps a per-object transform (`geom/src/transform.rs`, `canvas/tools/handles.rs:91`).
- **Lock-aspect resize:** it breaks on flips.

**Undo model:** ours already snapshots like theirs. Adopt their begin / preview / commit / cancel gesture pattern (`designcraft/crates/engine/src/tooling.rs:281-323`), which re-applies each preview to the snapshot from gesture start so drift can't accumulate.

**Recommendation:**
- **Port to TS:** distribute, smart guides, rotated selection box, `resize_rect`, gesture coalescing.
- **Rust:** adopt `designcraft-compose` for Knuth-Plass reflow inside `studio_edit_pdf_text_block` (needs an adapter from pdfcraft glyph metrics).

### 4.4 materialcraft / texturecraft → hatching
**materialcraft and texturecraft: out of scope.** Both are raster/texel only (materialcraft `crates/nodes/src/patterns.rs` generates pixels), and neither has vector hatching.

**cadcraft owns CAD hatching:**
- `PatternLine {angle, origin, delta, dashes}` and `standard_patterns()`: ANSI31–38, NET, BRICK, EARTH, AR-CONC, … at `crates/doc/src/library.rs:81-248`.
- A hostile-input-safe `.pat` parser at `library/files.rs:260`.
- `pattern_lines()` at `crates/render/src/hatch.rs:370`. It sweeps lines across the real boundary loops with even-odd crossings, applies dashes, and has work budgets.

**Our `generate_hatch_strokes` is wrong in three ways:**
- It clips to the bounding box (**ERR-007**).
- Masonry and tiling are drawn like crosshatch, and timber, earth, insulation and concrete are all plain horizontal lines.
- On screen, crosshatch is rotated 45° in `HatchDefs.svelte:58-63`; in the PDF it isn't.

**Recommendation:**
- **Port** `PatternLine` + `pattern_lines`/`crossings`/`dash` (about 150 lines) into our Rust hatch generator, using the annotation polygon as the boundary.
- Generate the on-screen SVG patterns from the same family data so screen and PDF match.
- Keep the line caps.

### 4.5 photocraft / lightcraft / craft-anything
- **Signatures:**
  - **Port** photocraft `algo/src/color_to_alpha.rs` (un-mixes paper colour while keeping anti-aliased edges) plus an Otsu threshold (`algo/src/segment/subject.rs:314`).
  - This replaces our hard grayscale cut-off (`insertEngine.svelte.ts` `applyGrayscaleThresholdToRgba`).
- **Image stamps:** **port** lightcraft `raster/src/resample.rs` (about 200 lines: Lanczos3 / Mitchell) if Rust-side resizing is needed.
- **Pressure curve and smoothing:** **port** photocraft `paint/src/curve.rs`, `dynamics.rs`.
- **Everything else is out of scope:**
  - codecs (too heavy)
  - tablet (`unsafe`; PointerEvents suffice)
  - segment (licence and network)
  - inpaint / denoise
  - wincolor / sysmem (FFI)
  - every craft-anything crate (AI clients, sandboxing, installers)

## 5. Licence and attribution notes
- Everything proposed for porting or adoption is MIT OR Apache-2.0. `designcraft-compose`'s hyphenation data is MIT / CC0 / public domain (`compose/src/hyphen.rs:1-15`).
- Ported code must keep its MIT/Apache notice: add an `ATTRIBUTION.md` entry per ported file, with repo, path and commit.
- Per `protocol/01_MASTER_CONTEXT.md §6`, no storytold/ArtCraft brand assets (`docs/brand`) may ship.

## 6. Proposed phase order (for ADR-018 approval)
1. **Hatching correctness:** fix ERR-007 by porting the cadcraft pattern sweep, and make screen and PDF match.
2. **Ink quality:**
   - TS: width joins, iterative RDP, pressure curve, recognition.
   - Rust: `kurbo` + `vectorcraft-pathops` for stroke fitting, boolean erase, and pressure-preserving PDF export.
3. **Selection pro features:** distribute, smart guides, rotated selection box, gesture coalescing.
4. **Measurement:** cadcraft snap ranking/tolerance, polar tracking, architectural units, PDF Measure dictionaries.
5. **Text reflow:** `designcraft-compose` Knuth-Plass in structural text edits.
6. **Separate audit:** craft-libs `pdf` (hayro + krilla) as the real Rust renderer behind `PdfcraftAdapter::render`.

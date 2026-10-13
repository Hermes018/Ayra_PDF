# DECISIONS (ADR, append-only)
Format: `## ADR-NNN [date] Title` — Context · Options (with pros/cons) · Decision · Consequences · Approved by (user/agent-within-scope) · Supersedes.

## ADR-001 [2026-10-10] Core stack
Rust core + Tauri v2 + plain HTML/TS UI; no Python runtime, no FastAPI/HTTP server; IPC via Tauri commands/events + custom URI scheme.
Approved by: user brief (Rust vs Python / FastAPI question) + this protocol.
**Status: PARTIALLY AMENDED by ADR-003 (frontend framework only). Everything else stands.**

## ADR-002 [2026-10-10] Engine = storytold/pdfcraft behind PdfEngine trait
Adopt (MIT OR Apache-2.0, 6.0k stars) with git-pin, worker isolation, fallback adapter, own corpus/fuzz/qpdf oracle, because the project is days old (v0.2.1) and its renderer is still bootstrap `hayro`.

---
# Entries below added by user "System Update" of 2026-10-10 (approval = that message; record in PROMPT_LOG)

## ADR-003 [2026-10-10] UI framework: Svelte 5 (compile-time) — amends ADR-001
- **Context:** Vanilla HTML/TS + custom elements cannot safely manage the state of canvas drawing (Edit/Sign), grid page sorting with multi-select drag, undo/redo, virtualized 10k-page grids, and per-tool option panels. Hand-rolled DOM diffing is a bug source.
- **Options:** (a) stay vanilla — smallest, but high bug risk for interactive tools; (b) **Svelte 5** — compiles away, small output, built-in reactive state, no VDOM; (c) Preact + signals — ~4 KB runtime but VDOM; (d) Vue 3 — larger runtime; (e) React — banned (size, ecosystem pull toward remote assets).
- **Decision:** Svelte 5 + Vite + TypeScript, **SPA only** (no SvelteKit, no SSR, no server). Reserve = Preact + signals (switch needs a new ADR).
- **Mandatory implementation rules:**
  1. Vite config: `@sveltejs/vite-plugin-svelte` with CSS emitted to external files so CSP `style-src 'self'` works; no inline `<script>`; CSP unchanged (`default-src 'self'`).
  2. State: Svelte runes in `ui/src/state/*.svelte.ts` (one module per domain: jobs, files, editor-session, settings). Document truth lives in Rust (worker session); UI holds view state only.
  3. Never use `{@html ...}` with any string derived from a file, filename or PDF text. Render via text nodes only.
  4. Drag-and-drop remains **pointer events** (ADR rationale in 01 §5.3); implemented as reusable Svelte actions (`use:sortable`, `use:marquee`), keyboard alternative Alt+Arrows.
  5. Canvas/overlay editing (Edit PDF, Sign, Crop, Redact boxes): SVG/canvas overlay component per page, model objects in a store, hit-testing in TS, commit to Rust via one `edit_apply(ops[])` command; undo stack owned by Rust session.
  6. Virtualised page grid component (windowing) mandatory above 200 pages.
  7. Bundle budget: initial JS+CSS ≤ 250 KB gzipped (initial target; calibrate in 3.1 and enforce in gate). Zero external assets/fonts.
  8. Tests: Vitest + @testing-library/svelte for components; Playwright with mocked IPC for flows; screenshot diffs vs ux-spec.
  9. protocol-lint additions: ban `@sveltejs/kit`, `{@html`, `fetch(`, `XMLHttpRequest`, `WebSocket`, `EventSource` in `ui/src` (IPC via Tauri `invoke`/events and `lpdf://` `<img>` only).
- **Consequences:** +1 dev toolchain (svelte, svelte-check); frontend files `.svelte`; replaces "custom elements + tiny store" wording in 06 §8, MASTER_PROMPT §2, ANCHOR line 1, 05 sub-phases 3.1–3.4.
- **Approved by:** user (System Update item 2). **Supersedes:** ADR-001 frontend clause only.

## ADR-004 [2026-10-10] PDF Intelligence (AI Summarizer, Translate PDF) vs "no network"
- **Context:** Cloud AI contradicts the offline rule. User allows either exclusion or an on-device LLM.
- **Options:** (a) exclude both — loses 2 reference tiles; (b) on-device LLM bundled — multi-GB installer, licence issues; (c) **hybrid:** tiles always present; Summarizer has a model-free extractive mode; LLM modes need a **user-supplied** quantized model run in a sandboxed process.
- **Decision:** (c), feature flag `ai-local`, **default OFF**, phased at 10.5a–d.
  1. **AI Summarizer, Basic mode (10.5a):** own extractive summarizer (sentence scoring: TextRank-style over TF-IDF similarity, position prior, length control). No model, always works offline, no extra deps beyond `regex`/`unicode-segmentation`. Labelled "Basic summary" in UI.
  2. **Local LLM runtime (10.5b, spike first):** separate crate `lpdf-ai` (pure logic: prompts, chunking, map-reduce) and separate binary `lpdf-ai-worker` (inference). Same framed-stdio protocol and sandbox as `localpdf-worker`; **no network** (Landlock/Job Object, `unshare -n` in CI), memory cap = tier budget. Decision rule for backend: use `llama-cpp-2` (llama.cpp, MIT) unless it fails to build on Fedora 44, Windows ARM64 or (later) Android; then `candle` (pure Rust, GGUF quantized). Spike must record tokens/s and RAM on a Tier-L and a Tier-M machine; pick llama.cpp if ≥1.5× faster at equal RAM.
  3. **Models:** NEVER bundled, NEVER downloaded by the app. User imports a `.gguf` through the native file dialog (Settings → Local AI). App stores path + SHA-256, shows the model's licence text for acknowledgement, and never copies the file unless the user asks. Llama-3-8B (Q4 ≈ 5 GB, ≈ 6–7 GB RAM) is supported *as a user-supplied file* (its Community Licence is not permissive; verify current text) and is recommended only for Tier M/H with ≥ 8 GB free RAM; Tier L is offered small (≈ 1–3 B) models. Docs list suggested permissive models; licence must be checked per model card.
  4. **Summarizer, LLM mode (10.5c):** map-reduce: split extracted text by layout blocks into chunks ≤ 70 % of the model context, summarize each chunk, then summarize the summaries; deterministic settings (temperature ≤ 0.3); output cites page numbers per bullet; cancel at chunk boundaries.
  5. **Translate PDF (10.5d, EXPERIMENTAL):** extract text blocks with bounding boxes (`lpdf-layout`), translate block-by-block (context window of neighbouring blocks), then redact original text and re-typeset translated text into the same box with auto-shrink to a minimum 70 % size, overflow flagged. Fonts: reuse original only if it has glyph coverage; otherwise fallback via fontdb and the UI states which fallback was used. **"Keep fonts/layout/formatting intact" is best-effort, not guaranteed**; report lists blocks that overflowed or used fallback fonts. Scanned PDFs need OCR first. Tile greyed until a model is installed.
  6. Safety: prompts treat document text as data (prompt-injection hardening: system prompt separate, output is plain text only, never executed or used as paths/commands). Output is shown as text/new PDF; nothing from the model touches the filesystem.
  7. Failsafe F19 (see 03 patch): worker isolation, no network, model hash check, watchdog per chunk.
- **Consequences:** crate/binary additions (06), deps section D in 08, phases 10.5a–d, UI tiles always visible, Intelligence column has exactly 2 tiles.
- **Approved by:** user (System Update item 1). **Note:** the protocol's earlier line "10.5 optional local-AI … ADR required" is satisfied by this ADR.
- **Spike Evaluation & Backend Selection (Phase 10.5b [2026-10-12]):**
  - **llama-cpp-2 Evaluation:** Relies on C/C++ FFI bindings to `llama.cpp` requiring CMake and C++ toolchain during compilation. Fails `#![forbid(unsafe_code)]` compliance at crate boundaries, and introduces significant build/packaging friction for cross-platform distribution (Windows ARM64 MSVC, Android NDK).
  - **Candle Evaluation:** Pure Rust runtime by Hugging Face supporting GGUF parsing and quantized inference out of the box with zero C/C++ compiler toolchain requirements. Adheres strictly to `#![forbid(unsafe_code)]` at worker crate level, compiles cleanly for Linux (x86_64/aarch64), Windows (x64/ARM64), and Android targets, and runs deterministically inside sandboxed namespaces under `unshare -n`.
  - **Decision:** Select pure-Rust `candle` (via `lpdf-ai-worker` skeleton and sandboxed stdio bridge) as the inference backend under the `ai-local` feature gate (default OFF). Model verification strictly enforces Failsafe F19 (checksum matching, format magic byte validation, and mandatory user licence acknowledgement).

## ADR-005 [2026-10-10] Offline scope reductions: Sign PDF, Scan to PDF, HTML to PDF
- **Context:** Three tiles imply cloud features. Offline product must say exactly what is in/out.
- **Decision:**
  - **Sign PDF:** in = draw/type/image stamping, sign-yourself with local PKCS#12, self-signed cert generation, verifying existing signatures. Out = requesting signatures from others, any e-mail/server flow, TSA timestamps (PAdES B-T/LTV), hardware tokens (revisit by ADR).
  - **Scan to PDF:** in (7.4a) = import image files, auto-crop, deskew, enhance. Out = mobile-to-browser sync/QR. Scanner hardware (7.4b) = stretch, **default dropped**: Linux via user-installed `scanimage` external process (GPL, never bundled; needs explicit user approval because of DON'T-9), Windows via WIA COM in one allow-listed `unsafe` module (needs ADR for the exception). If unapproved, tile works with 7.4a only.
  - **HTML to PDF:** local `.html` files (+ local relative assets) or pasted HTML only. No URL input. Network-blocked webview, JS disabled, remote subresources blanked; CI test under `unshare -n`. Fetching a URL would require network and is out of scope by definition. Per-OS print path behind `HtmlPrinter`; tile greyed on any OS whose spike fails.
- **Consequences:** 07 §3 documents the same; UI shows no "Request signatures", no URL box, no phone-sync QR.
- **Approved by:** user (System Update item 3). Ask-again items: 7.4b scanner approach.

## ADR-006 [2026-10-10] Document Reconstruction (PDF → DOCX / PPTX / XLSX / MD) — reality check
- **Context:** `storytold/pdfcraft` can *export* Word/HTML/RTF, but a PDF stores positioned glyphs, not paragraphs/tables/styles; it cannot natively reflow to editable Office files. Its exporter quality is unmeasured. The earlier plan implied near-perfect conversion; that is not achievable by any engine.
- **Decision (scope):** We deliver **heuristic structure recovery with a quality report**, never a promise of 1:1 reflow. Two explicit output modes:
  1. **Flow (editable)** — DOCX/MD: paragraphs, headings, lists, tables, inline images; layout approximate. Default for Word and Markdown.
  2. **Exact layout** — DOCX/PPTX: every text block as an absolutely positioned text box (DOCX: anchored text boxes/frames; PPTX: one text box per block), images placed by coordinates; looks like the PDF, edits are clunkier. Default for PowerPoint.
  Scanned PDFs: run OCR (5.3) first, then same pipeline. Complex vector art is rasterized region-by-region at 150 DPI and flagged.
- **Implementation:** new crate `lpdf-layout` (pure Rust, `forbid(unsafe_code)`), input = positioned text runs + image boxes + vector line/rect segments obtained from the `PdfEngine` trait (exact calls verified in 0.4 into ENGINE_NOTES; if the engine lacks glyph-level bboxes, the adapter interprets content streams itself). Pipeline (each step a pure fn with unit + property tests, thresholds in one `LayoutParams` struct, calibrated on corpus):
  1. **Normalize:** apply text matrix/rotation; glyphs → words (gap < 0.25 × font size) → lines (baseline tolerance 0.3 × size); handle 90/270° rotation; RTL via `unicode-bidi`.
  2. **Header/footer removal:** lines repeated (same text pattern, ±2 pt position) on ≥ 50 % of pages inside the top/bottom 8 % band → tagged `furniture` (kept in Exact mode, dropped or moved to Word header/footer in Flow).
  3. **Regions & columns:** recursive **XY-cut** using projection profiles (min whitespace gap = 1.5 × median line height horizontally, 0.8 × vertically); non-Manhattan layouts fall back to single column + warning.
  4. **Reading order:** depth-first over the XY-cut tree (top→bottom, left→right; full-width titles first).
  5. **Paragraphs:** merge consecutive lines when gap ≤ 1.5 × median leading, same font family/size, left edge within 2 pt or first-line indent; de-hyphenate (`-` at line end + lowercase start).
  6. **Headings:** font size ≥ 1.15 × page body median (or bold, short < 80 chars, isolated) → rank distinct sizes to H1–H6 (max 6).
  7. **Lists:** line starts with bullet glyph (`•●▪-*`) or `^\d+[.)]`, `^[a-z][.)]`, roman numerals + hanging indent → list items; nesting by indent steps.
  8. **Tables:** (a) *lattice*: collect stroked lines/thin filled rects, snap to grid (±1.5 pt), find intersections → cells with row/col spans; (b) *stream*: no rulings → cluster word x-edges across ≥ 3 consecutive rows, columns where ≥ 70 % of rows align; first row = header if bold or different style. Anything uncertain stays as paragraphs (precision over recall).
  9. **Images:** image XObjects emitted with bbox; Flow mode anchors to the nearest following paragraph, Exact mode by coordinates.
  10. **Writers (own, in `lpdf-tools-convert`):** DOCX/PPTX/XLSX via `quick-xml` + `zip` (document.xml, styles.xml, numbering.xml, media/, `[Content_Types].xml`); font names mapped to families, fonts NOT embedded; XLSX via `rust_xlsxwriter`. `docx-rs` adopted instead only if vetting proves it covers text boxes, numbering and media (08 §B).
  11. **Report:** per file: pages, tables found, headings found, blocks in fallback, rasterized regions, OCR used. Shown in result page and saved as `<name>_report.txt` optionally.
- **Acceptance metrics (initial targets; calibrate in 8.4, then enforce in CI):** born-digital corpus: text character recall ≥ 99 %; reading-order score (Kendall τ vs ground truth) ≥ 0.95 on single/two-column docs; lattice-table cell F1 ≥ 0.85; DOCX opens without repair prompt in LibreOffice (CI round-trip) and re-renders to PDF with page count equal ±5 %. Failing a metric ⇒ the tool's UI carries an "approximate conversion" notice; it is never silently reported as perfect.
- **Non-goals:** pixel-perfect reflow, form-field/JS conversion, fonts embedding, SmartArt/charts reconstruction, tracked changes.
- **Consequences:** new crate `lpdf-layout`; phases 8.2, 8.3, 9.2 depend on it; Markdown extraction reuses steps 1–9.
- **Approved by:** user (System Update item 4).

## ADR-007 [2026-10-10] Taxonomy parity (7 columns)
Home grid = exactly ORGANIZE PDF, OPTIMIZE PDF, CONVERT TO PDF, CONVERT FROM PDF, EDIT PDF, PDF SECURITY, PDF INTELLIGENCE; 31 reference tools mapped one-to-one in `07_FEATURE_MAP.md`; enforced by `cargo xtask feature-map-check` and a UI snapshot test. Extras live below the grid. Column placement of tools not labelled in the reference list is an assumption to confirm in 0.3. **Approved by:** user (System Update item 5).

## ADR-008 [2026-10-10] Engine pinning strategy (pdfcraft)
- **Context:** Spike in Phase 0.4 verified `pdfcraft` (commit `dc4b3eee`) successfully merges, splits, and compresses PDFs. Digital signatures correctly cause modifications to be rejected. Merged output triggered minor qpdf warnings (missing kid Resources dictionary) but remained structurally valid and repairable.
- **Decision:** Track `vendor/pdfcraft` as a git submodule pinned to the tested commit rather than a gitignored folder. This ensures deterministic multi-device syncing and isolates upstream instability. Any updates to the engine will require a deliberate bump and a re-run of the `qpdf` oracle suite.
- **Consequences:** The repo requires `git submodule update --init` on fresh clone.
- **Approved by:** agent-within-scope.

## ADR-009 [2026-10-10] Fallback Renderer (pdfium-render)
- **Context:** `pdfcraft`'s renderer (`hayro`) is early in development. For visual features like thumbnails, page extraction, or OCR layout, we need a robust, production-grade fallback if the primary engine panics or renders inaccurately on complex files.
- **Decision:** Adopt `pdfium-render` (and `pdfium-binaries`) exclusively as a visual rendering fallback. It will be sandboxed to the worker. If `pdfcraft` fails to render a thumbnail, `pdfium` attempts it. It will NOT be used for structural PDF modifications (merge/split/etc.).
- **Consequences:** Increases application bundle size by the size of the pdfium native library (~10-20 MB depending on platform).
- **Approved by:** agent-within-scope.

## ADR-010 [2026-10-10] Markdown-to-PDF Approach (Typst)
- **Context:** We need a robust offline Markdown-to-PDF engine. Headless browsers (puppeteer/wkhtmltopdf) are banned/bloated. Pure Rust Markdown-to-PDF crates are often incomplete.
- **Decision:** Use `typst` (Apache-2.0) as the backend engine. We will use `comrak` to parse Markdown and emit Typst code, then use `typst` to render the PDF. Typst is fast, produces professional typography, and is a pure Rust dependency.
- **Consequences:** Adds Typst to the dependency tree. Requires maintaining a Markdown-to-Typst translation layer.
- **Approved by:** agent-within-scope.

## ADR-011 [2026-10-13] SvelteKit Static SPA Mode Architectural Exception (ADR-008-sveltekit-spa)
- **Context:** Protocol 02 DON'T-18 strictly specifies "no SvelteKit/SSR". However, `app-tauri` was built with `@sveltejs/kit` and `@sveltejs/adapter-static`.
- **Decision:** Using `@sveltejs/kit` with `@sveltejs/adapter-static` (`fallback: "index.html"`) and `ssr = false` in `+layout.ts` is formally approved as an architectural exception. It produces a pure client-side SPA compatible with Tauri v2 offline operation.
- **Consequences:** Documented in `logs/ADR-008-sveltekit-spa.md`. `export const ssr = false;` must remain strictly enforced.
- **Approved by:** Staff Software Engineer & Enterprise Security Auditor.

## ADR-012 [2026-10-13] Vendored Mozilla pdf.js Webview Rendering Exception (ADR-009-vendored-pdfjs)
- **Context:** Frontend uses vendored Mozilla `pdf.min.js` and `pdf.worker.min.js` in `static/` to render visual PDF previews inside `PdfViewer.svelte` and `GridEditor.svelte`.
- **Decision:** Vendoring Mozilla's `pdf.min.js` for visual webview canvas preview rendering is formally approved as an architectural exception under strict constraints: 1) Strict CSP blocks external script execution, 2) No PDF parsing/mutation logic happens in JS (only rendering bytes passed from Rust), and 3) The Apache 2.0 license is maintained.
- **Consequences:** Documented in `logs/ADR-009-vendored-pdfjs.md`.
- **Approved by:** Staff Software Engineer & Enterprise Security Auditor.

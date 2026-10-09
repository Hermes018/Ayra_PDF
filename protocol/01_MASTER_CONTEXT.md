# 01 — MASTER CONTEXT (read at boot; stable facts + rationale)

## 1. Product
LocalPDF Studio = offline iLovePDF-equivalent. Every tool, no limits, no network. Users: individuals on Fedora KDE first; low-end to high-end hardware.
Owner works with phased builds, logs, and strict protocol files (this folder).

## 2. Rust vs Python, FastAPI vs alternatives — DECIDED
| Question | Decision | Why |
|---|---|---|
| Rust or Python for the engine? | **Rust** | Python would be slower on parsing/rendering/compression, needs a bundled interpreter (large installs, slow cold start), and the best Python PDF libs are AGPL (PyMuPDF/MuPDF) = licence trap. Rust gives memory safety on hostile files, real multithreading, small binaries. |
| FastAPI backend? | **No** | A localhost HTTP server adds an attack surface (open port, other local processes, DNS-rebinding/CSRF), serialization overhead, and lifecycle problems. In-process Tauri IPC is faster, capability-gated, and has no port. |
| Anything "more efficient and manageable"? | **Tauri v2 commands + events + custom URI scheme** | Typed commands, per-window capability allowlist, binary streaming without base64. If a headless mode is ever needed: `localpdf-cli` (same core crates) or an opt-in loopback+token server (axum), off by default. |
| Python at all? | Dev tooling/scripts only, never shipped | Keeps runtime lean. |

## 3. Engine choice: pdfcraft (facts recorded 2026-10-10 in an earlier session; RE-VERIFY in sub-phase 0.4)
- Repo `storytold/pdfcraft`: pure Rust, clean-room "Acrobat-style" workbench; **6.0k stars, 2.1k forks, MIT OR Apache-2.0**; platforms: macOS/Windows/Linux/FreeBSD/web. Crates: `pdfcraft-filters, -crypt, -cos, -organize, -fonts, -annot, -forms, -render, -engine, -automation, -ui-egui` (+ `optimize, ocr, sign, redact, compare, export, preflight, a11y, js, xfdf` per its ROADMAP).
- **Strong for us:** tolerant parse/repair, combine/extract/split, page ops + undo, incremental + atomic saves, AES-256/RC4 security handler, Reduce-File-Size optimizer, watermark/header/footer/Bates, crop, redaction, PAdES signing, compare, Latin OCR (ocrs), create-from-images, export PNG/JPEG/TIFF/text, export Word/HTML/RTF, CLI + MCP.
- **RISKS (its own ROADMAP, 2026-10-05/07):** project started 2026-09-30, v0.2.1 — **very young**. Rendering is still the borrowed `hayro` bootstrap with 18 vendored patches; inspector uses `lopdf`; fuzzing was still finding OOM crashes/hangs (fixed, "expect more"); no lazy loading yet (large files are read whole); no code signing; OCR Latin only; no Office import, no Excel/PowerPoint export; fidelity vs Acrobat unmeasured.
- **Therefore:** adopt it, but (1) git-pin a commit (or vendor) because we need its vendored patches and I could not confirm the `pdfcraft-*` crates are published on crates.io, (2) hide it behind `PdfEngine` trait, (3) run its parsing in the sandboxed worker, (4) keep a fallback adapter (lopdf for parse/repair/rewrite; optional pdfium-render for rendering), (5) run our own corpus + fuzz + `qpdf --check` oracle in CI, (6) re-vet at every pin bump.

## 4. Build-vs-reuse principle
Reuse a well-known, permissively licensed, actively maintained component only if it removes real work without adding disproportionate weight/risk. Otherwise build. Every adopt/build call = ADR with alternatives considered. Vetting rules: `08_DEPENDENCY_VETTING.md`.

## 5. Known problems in the earlier uploaded plan (do NOT copy blindly)
1. States rendering is `vello_cpu`; pdfcraft's README/ROADMAP say rendering currently uses `hayro`. Verify before designing around either.
2. Uses `app_handle.emit_all` — that is Tauri v1; v2 uses `emit` / `emit_to`. Check v2 docs.
3. Sets `dragDropEnabled:false` and then listens for `tauri://drag-drop` — contradictory: with `false` the native drop event doesn't fire (HTML5 DnD does). We keep it `true` (real OS paths for dropped files) and implement page/file reordering with **pointer events**, not HTML5 DnD (which is unreliable inside WebView2 when native drop is on).
4. Function names (`pdfcraft_organize::combine`, `Session::new`) are unverified. Read the source.
5. Cites three different "pdfcraft" projects (storytold, pdfcraft-app, PDFCraftTool [a JS project]). Only `storytold/pdfcraft` is ours.
6. Suggests base64 thumbnails; use a custom URI protocol streaming bytes instead.
7. Adds `atomic-write-file` for PDF saves; pdfcraft already does atomic saves — use ours/theirs consistently, not both layered.
8. Omits: Fedora packaging/SELinux, Android, full iLovePDF feature list, Markdown toolset, hardware tiers, worker sandbox, chaos tests.

## 6. Legal
iLovePDF name/logo/assets/code are not ours. Behavioural + layout parity only. Original icons (licence-checked, e.g. Lucide ISC), original wording. pdfcraft's ArtCraft marks (docs/brand) must NOT be shipped; keep NOTICE/ATTRIBUTION. ocrs models are CC-BY-SA per pdfcraft's notes — attribution + share-alike apply to the model files only; record in ATTRIBUTION.

## 7. Amendments 2026-10-10 (user System Update)
Frontend now Svelte 5 (ADR-003). AI tools: Basic extractive + optional user-supplied local models (ADR-004). Sign/Scan/HTML offline scope (ADR-005). Document reconstruction is heuristic, two modes, quality report (ADR-006). 7-column taxonomy (ADR-007).
§5 item 3 stays valid: pointer-event DnD, dragDropEnabled=true.

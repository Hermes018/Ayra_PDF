# 05 — PHASES (phase-type build). One sub-phase at a time. [G] = gate item.
Every sub-phase ends: tests run -> `cargo xtask gate` -> PROGRESS_LOG -> CHECKPOINT -> conventional commit -> tag `phase-X.Y-green` -> re-print ANCHOR.

## PHASE 0 — Recon & protocol (no product code)
- 0.1 Repo init, copy protocol files, fill `protocol/09_DIRECTORY_TREE.md`, first commit.
- 0.2 Fedora 44 KDE bootstrap: rustup (stable, pinned), pnpm, Tauri v2 prerequisites (verify against official v2 docs; typically webkit2gtk4.1-devel, openssl-devel, librsvg2-devel, libappindicator-gtk3-devel, libxdo-devel, dev tools), cargo-deny/audit/nextest/fuzz, qpdf, poppler-utils (oracle only), optional tesseract + libreoffice. Record versions in `ENV.md`. [G] `cargo xtask doctor` passes.
- 0.3 UX audit of the live iLovePDF site (behaviour/layout only): per tool record states (empty/loaded/processing/done/error), option panels, compression levels (verify: extreme/recommended/less), merge/split/organize interactions, drag-reorder behaviour, download page, responsive breakpoints. Record the live iLovePDF column labels and tile-to-column placement; resolve the assumption in 07 §1.6. Output `ux-spec/*.md` + reference screenshots (never shipped). Confirm the live tool list. [G] ux-spec covers every tool in FEATURE_MAP.
- 0.4 Engine recon: clone `storytold/pdfcraft`, pin commit SHA, read crates, write `ENGINE_NOTES.md` with VERIFIED public APIs (load, combine, extract, split, rotate, delete, render, optimize, ocr, protect, redact, sign, watermark, export, undo, text-run/glyph-bbox extraction); build + run its tests and CLI on our corpus; list discrepancies with the old plan. [G] spike: merge + split + compress round-trip on 3 PDFs, `qpdf --check` clean.
- 0.5 Dependency vetting (file 08) + ADRs: engine adoption/pin strategy, fallback renderer, Markdown->PDF backend, Office bridge, HTML->PDF approach. Also ADRs/spikes for: Svelte (bundle size, CSP, pointer-DnD action), `HtmlPrinter` per-OS spike, llama.cpp-vs-candle decision rule (ADR-004), docx-rs-vs-own writer.
- 0.6 Corpus: licensed/public PDFs + generated hostile files (bombs, deep trees, broken xref) + synthetic 1k/10k-page files; `corpus/MANIFEST` with hashes; download script (don't commit third-party PDFs).

## PHASE 1 — Foundation
- 1.1 Cargo workspace + crate skeleton (see 06), `#![forbid(unsafe_code)]`, `rust-toolchain.toml`, `xtask`.
- 1.2 Gates: fmt, clippy -D warnings, nextest, deny, audit, protocol-lint, feature-map-check, frontend typecheck (svelte-check)/lint/test -> `cargo xtask gate`.
- 1.3 CI (GitHub Actions): Fedora container + Ubuntu, Windows; caching; artifacts; nightly fuzz + corpus run.
- 1.4 Tauri v2 shell on Fedora KDE (Wayland + X11): strict CSP, capabilities with zero broad scopes, window-state, single-instance. [G] launches under `unshare -n`.
- 1.5 Core primitives: error taxonomy + codes, tracing (rotating local logs), versioned TOML config, hardware profile (tiers) via sysinfo, i18n scaffold.

## PHASE 2 — Spine: engine port, jobs, file safety
- 2.1 `PdfEngine` trait; `PdfcraftAdapter` (load/info/render/page-ops/save); `FallbackAdapter` (lopdf parse/repair/rewrite); shared contract test-suite run against every adapter.
- 2.2 `localpdf-worker`: framed, versioned stdin/stdout protocol; supervisor with restart/timeouts/memory caps; Linux Landlock (+seccomp), Windows Job Object + restricted token.
- 2.3 Job manager: bounded queue, priorities, cancel, progress events, concurrency by tier.
- 2.4 File-safety layer: atomic writer, preflight (space/perm/same-FS), journal + recovery, backups, naming/collision rules, temp GC, path canonicalization.
- 2.5 Verify-after-write + degrade paths + chaos suite. [G] 1000 randomized kill -9 runs: zero corrupted outputs; disk-full/read-only/injected-panic handled.

## PHASE 3 — Frontend shell & design system
- 3.1 Svelte 5 + Vite scaffold (ADR-003), design tokens, 7-column tool-grid home (registry-driven, greyed-out unavailable tiles), search, own icons, dark/light, responsive, zero external assets. [G] UI snapshot of the 7 columns == 07; `feature-map-check` green.
- 3.2 Svelte components: dropzone (native OS drop with real paths + picker), file-card list with **pointer-event drag-reorder** (actions `use:sortable` / `use:marquee`; touch + keyboard alt), option panel, progress, result page (Save As / Open folder / Process another), toasts, error dialog with code + "copy diagnostics".
- 3.3 Thumbnail pipeline: custom URI protocol, IntersectionObserver queue, LRU cache, cancel on scroll, skeletons; fast (u8) thumbs vs hi-fi zoom.
- 3.4 Typed IPC client (generated types), runes-based state modules, a11y baseline, e2e harness (tauri-driver) + fast mocked-IPC UI tests (Vitest + Testing Library + Playwright mocked IPC) with screenshot diffs vs ux-spec.

## PHASE 4 — Organize
- 4.1 Merge: multi-add, drag reorder, sort, password prompts, per-file bookmarks, mixed page sizes.
- 4.2 Split: custom ranges, fixed ranges, every-N, by size, by bookmarks, extract-all; output to folder or ZIP.
- 4.3 Remove pages / Extract pages (visual select + range text).
- 4.4 Organize (grid editor): multi-select, drag-reorder, rotate L/R, delete, duplicate, insert blank/from file, undo/redo, live thumbs, save / save-as.
- 4.5 Rotate PDF. [G] 10k-page merge/split inside Tier-L memory budget; outputs verified.

## PHASE 5 — Optimize
- 5.1 Compress: 3 level profiles (tunable, calibrated on corpus), before/after sizes, never-grow, preserves text/links/bookmarks, batch.
- 5.2 Repair: tolerant parse -> rewrite -> fallback chain -> report of fixes.
- 5.3 OCR: Native Latin offline OCR via `pdfcraft_ocr` (`ocrs`/`rten` ML models); deskew; per-page progress; searchable layer. [G] size-reduction report + SSIM visual-regression bound.

## PHASE 6 — Edit & security
- 6.1 Protect (AES-256, user/owner passwords, permissions) / Unlock (user-supplied password or restriction removal only). Natively powered by `pdfcraft_crypt`.
- 6.2 Watermark, page numbers, crop.
- 6.3 Edit PDF (add text/shapes/images/highlight/draw; existing-text edit = experimental flag).
- 6.4 Sign (draw/type/image stamping, sign-yourself with local PKCS#12 PAdES B-B, self-signed cert generator, signature verification; NO request-signatures, NO TSA), Redact (true semantic removal natively via `pdfcraft_redact`), Compare. [G] redaction verified by re-extraction + byte grep; encrypted outputs open in qpdf/poppler with password. Natively powered by `pdfcraft_sign` and `pdfcraft_redact`.
- 6.5 PDF Forms: detect fields, create fillable (text/checkbox/radio/list), fill, flatten; XFDF if supported. Natively powered by `pdfcraft_forms`.

## PHASE 7 — Convert to PDF
- 7.1 Images -> PDF (JPG/PNG/TIFF/BMP/GIF/WebP; page size, orientation, margins, merge or separate).
- 7.2 Word/Excel/PowerPoint -> PDF via bundled OpenOffice/LibreOffice/ONLYOFFICE headless sidecar for pixel-perfect translations (ADR-004). Mandatory sidecar approach.
- 7.3 HTML -> PDF (stretch): local .html or pasted HTML only, network blocked, JS off, per-OS `HtmlPrinter` (ADR-005). [G] test under `unshare -n` with remote-image HTML.
- 7.4a Scan to PDF: image import + auto-crop/deskew/enhance.
- 7.4b Scanner hardware: STRETCH, default dropped; requires user approval + ADR (SANE via external `scanimage` / WIA `unsafe` module).

## PHASE 8 — Convert from PDF
- 8.1 PDF -> JPG/PNG (DPI, ranges, ZIP).
- 8.2 PDF -> Word / HTML / text via bundled OpenOffice/LibreOffice/ONLYOFFICE headless sidecar for pixel-perfect translations (ADR-004). Mandatory sidecar approach.
- 8.3 PDF -> Excel and PowerPoint via headless sidecar.
- 8.4 PDF -> PDF/A (pdfcraft preflight; verify with veraPDF in CI). Calibrate ADR-006 acceptance metrics on corpus; LibreOffice round-trip in CI. [G] fidelity report on corpus.

## PHASE 9 — Markdown toolset
- 9.1 MD -> PDF via Pandoc sidecar for text and structural conversions (ADR-004).
- 9.2 PDF -> MD via Pandoc sidecar (images extracted to `assets/`, links, page-break policy).
- 9.3 MD <-> DOCX / HTML, DOCX -> MD, HTML -> MD via Pandoc sidecar.
- 9.4 Batch folder conversion, front-matter, link/asset rewriting, CLI `localpdf md ...`.

## PHASE 10 — Pipelines, CLI, settings
- 10.1 Batch queue UI. 10.2 Pipelines (chain tools, presets = iLovePDF "Workflows" equivalent). 10.3 `localpdf-cli` on the same core. 10.4 Settings (tier override, output policy, backups, language, theme, log viewer, diagnostics export — local only, Local AI model import).
- 10.5a AI Summarizer Basic (extractive, no model).
- 10.5b Local-LLM spike: llama-cpp-2 vs candle per ADR-004 rule, `lpdf-ai-worker` skeleton, model import UI, hash + licence acknowledgement.
- 10.5c Summarizer LLM mode (map-reduce).
- 10.5d Translate PDF (experimental, block-level re-typeset, report).
  All behind `ai-local`, default OFF. [G] AI e2e under `unshare -n`; F19 tests.

## PHASE 11 — Hardening
- 11.1 Fuzzing (cargo-fuzz on adapter inputs, nightly). 11.2 Hostile-file suite. 11.3 Perf/memory profiling per tier (`systemd-run -p MemoryMax=2G -p CPUQuota=200%`), budgets enforced in CI (>10% regression fails). 11.4 Threat-model review, dependency re-vet, SBOM. 11.5 A11y audit. 11.6 8-hour soak.

## PHASE 12 — Linux & Windows release
- 12.1 Fedora: RPM + Flatpak + AppImage; SELinux-enforcing test; KDE integration (desktop file, MIME, portals, HiDPI, Wayland).
- 12.2 Windows x64 + ARM64: MSI/NSIS + portable; WebView2 handling; code signing.
- 12.3 Checksums, SBOM, signed releases, docs, CHANGELOG; manual update check only.

## PHASE 13 — macOS (later): build/sign/notarize, sandbox entitlements, WKWebView quirks.
## PHASE 14 — Android (later): Tauri mobile init, content-URI/SAF streaming to app cache, tight memory tiers, touch UI, packaging.

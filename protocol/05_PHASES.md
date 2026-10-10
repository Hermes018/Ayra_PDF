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

## PHASE 4 — Organize (Native via pdfcraft)
- 4.1 Organize operations: Page grid, Rotate, Delete, Extract, Insert, Replace, Split, Set page boxes, Crop pages, Duplicate pages, Page labels.

## PHASE 5 — Processing (Native via pdfcraft)
- 5.1 Process operations: Compress, Repair, Scan & OCR (Native Latin offline OCR via `pdfcraft_ocr`).

## PHASE 6 — Annotation, Editing, Forms, & Security (Native via pdfcraft)
- 6.1 Annotation/Editing: Sticky note, Highlight, Underline, Strikethrough, Squiggly, Text box, Freehand, Rectangle, Oval, Line, Arrow, Comment list, Flatten comments, Edit text & images, Header/footer, Watermark, Link, Bates numbering, Background.
- 6.2 Forms & Signatures: Field list, Detect fields, Clear form, Field properties, Tab orders (rows/cols), Flatten fields, Import/Export data, Form components (Text, Checkbox, Radio, Drop-down, List, Button, Image, Date, Digital Signature), Fill & Sign markings (Checkmark, Cross, Dot, Line).
- 6.3 Security & Processing: Protect, Unlock, Redact (true semantic removal natively via `pdfcraft_redact`).

## PHASE 7 — Basic Convert to PDF (Native via pdfcraft)
- 7.1 Image (PNG), Export all images, HTML, RTF, Plain Text to PDF, Structural Word (.docx) to PDF.
- 7.2 HTML -> PDF (stretch): local .html or pasted HTML only, network blocked, JS off, per-OS `HtmlPrinter`.

## PHASE 8 — Basic Convert from PDF (Native via pdfcraft)
- 8.1 PDF to Image (PNG), Export all images, HTML, RTF, Plain Text, Structural Word (.docx).
- 8.2 PDF -> PDF/A (pdfcraft preflight; verify with veraPDF in CI). Calibrate ADR-006 acceptance metrics on corpus.

## PHASE 9 — Advanced Convert (M10 Sidecars)
- 9.1 Word/Excel/PowerPoint -> PDF via bundled OpenOffice/LibreOffice/ONLYOFFICE headless sidecar for pixel-perfect translations.
- 9.2 PDF -> Word (complex flowable) / Excel / PowerPoint via headless sidecar.
- 9.3 Markdown toolset: MD -> PDF via Pandoc sidecar for text and structural conversions. PDF -> MD via Pandoc sidecar (images extracted to `assets/`). MD <-> DOCX / HTML via Pandoc sidecar.
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

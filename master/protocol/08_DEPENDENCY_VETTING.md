# 08 — DEPENDENCY VETTING (rev 2 · 2026-10-10) — gate before adding ANY crate/npm package/tool

Amended per user "System Update" items 1–2 (frontend framework, local AI). Numbers (stars, downloads, dates, licences) are NOT verified here: look each one up at vetting time (sub-phase 0.5) and record the result in the table. "PENDING" = not yet vetted.

## Criteria (record each; any fail needs an ADR)
1 Licence permissive (MIT/Apache/BSD/ISC/MPL-via-process) · 2 Verified popularity/endorsement (look it up) · 3 Last meaningful commit <12 months · 4 No unfixed RUSTSEC/CVE/npm advisory · 5 `cargo deny` passes · 6 No network/telemetry code · 7 Build impact (compile time, binary KB, native deps) acceptable · 8 Cheaper than building ourselves (>~300 LOC saved) · 9 More than one maintainer OR trivially replaceable · 10 Works on Linux, Windows (and macOS/Android where planned).

## A. Core / engine
| Component | Role | Notes | Status |
|---|---|---|---|
| tauri v2 | shell | MIT/Apache; mobile support | APPROVED |
| storytold/pdfcraft | engine | Earlier session recorded: 6.0k stars, 2.1k forks, MIT OR Apache-2.0, v0.2.1, started 2026-09-30 → age risk. Re-verify in 0.4. Pin + adapter + fallback | APPROVED (ADR-008) |
| lopdf | fallback parse/repair | permissive; widely used | APPROVED |
| pdfium-render + pdfium-binaries | optional fallback renderer | BSD/Apache; ~MB native lib | APPROVED (ADR-009) |
| ocrs | OCR (Latin) | via pdfcraft; models CC-BY-SA | APPROVED |
| tesseract | optional multi-language OCR | Apache-2.0; external process, user-installed | APPROVED |
| LibreOffice | Office → PDF bridge | MPL; external process, not bundled | APPROVED |
| image, rayon, tokio, serde, postcard, thiserror, tracing, sysinfo, memmap2, zeroize/secrecy, proptest, cargo-fuzz | infra | standard Rust ecosystem | APPROVED |
| landlock (+seccomp crate) | Linux sandbox | vet maintainer + API | APPROVED |
| specta / ts-rs | IPC typing | vet | APPROVED |
| fontdb (or equivalent) | system font lookup for page numbers/watermark/translate | permissive; no network | APPROVED |
| rcgen (or equivalent) | self-signed cert for Sign PDF | vet; alternative = own via RustCrypto | APPROVED |

## B. Documents & conversion
| Component | Role | Notes | Status |
|---|---|---|---|
| comrak | Markdown parse (CommonMark/GFM) | permissive | APPROVED |
| typst (+typst-pdf) / krilla | MD → PDF backend options | typst is fast, Apache-2.0 (ADR-010) | APPROVED (ADR-010) |
| rust_xlsxwriter | PDF → Excel | permissive | APPROVED |
| quick-xml + zip | DOCX/PPTX read/write (own OOXML writer, ADR-006) | permissive | APPROVED |
| docx-rs (alternative to own writer) | DOCX writing | vet: if it cannot do text boxes + numbering + media cleanly, build own | APPROVED |
| unicode-bidi, unicode-segmentation, regex | layout analysis (RTL, word breaks, list detection) | `regex` is linear-time (no ReDoS) | APPROVED |

## C. Frontend (amended — ADR-003 supersedes "plain HTML/TS only")
| Component | Role | Notes | Status |
|---|---|---|---|
| **svelte 5** | compile-time UI framework (SELECTED) | MIT; compiles away, small runtime; runes for state; no VDOM | APPROVED (ADR-003) |
| @sveltejs/vite-plugin-svelte | Vite integration | `emitCss` external so CSP `style-src 'self'` holds | APPROVED |
| svelte-check | type-check .svelte | dev only | APPROVED |
| Vite, TypeScript | bundler/types | dev only | APPROVED |
| Vitest + @testing-library/svelte | component tests | dev only | APPROVED |
| Playwright (mocked IPC), tauri-driver | e2e | dev only | APPROVED |
| Lucide icons | UI icons | ISC | APPROVED |
| **Fallback if Svelte fails vetting/spike:** Preact + @preact/signals (MIT, ~4 KB) | alternate | needs new ADR to switch | RESERVE |
| Vue 3 | third permitted option | larger runtime than Svelte/Preact | NOT SELECTED |
| NOT PERMITTED | React, Angular, SvelteKit/Next/Nuxt (SSR/servers), any UI kit that loads remote assets | | BANNED |

## D. Local AI (optional feature `ai-local`, default OFF — ADR-004)
| Component | Role | Notes | Status |
|---|---|---|---|
| llama.cpp via `llama-cpp-2` (or similar binding) | quantized GGUF inference | MIT; C++ toolchain; our code stays `forbid(unsafe_code)`, FFI lives in dependency; separate process `lpdf-ai-worker` | PENDING (spike 10.5b) |
| candle (`candle-core`, `candle-transformers`) | pure-Rust alternative, GGUF quantized Llama-family | Apache-2.0/MIT; slower on CPU than llama.cpp (measure) | PENDING (spike 10.5b) |
| tokenizers | tokenization if the binding needs it | Apache-2.0 | PENDING |
| **Models are NOT dependencies and are NEVER bundled or downloaded** | user imports a `.gguf` via native file dialog; app records SHA-256 and shows licence text the user must acknowledge | See ADR-004 | — |
| Llama-3-8B (example from the fix prompt) | candidate model | **Llama 3 Community License is not OSI-permissive** (attribution, acceptable-use, scale clauses) → fails criterion 1 for *bundling*; acceptable only as a user-supplied file the user licenses themselves. Verify current licence text. Approx. size: Q4 quant ≈ 5 GB on disk, ≈ 6–7 GB RAM → Tier M/H only | USER-SUPPLIED ONLY |
| Suggested permissive small models | verify licence at vetting time | e.g. a small MIT/Apache-licensed instruct model (check each model card; some sizes within one family carry different licences) | PENDING |

## E. CI oracles (never shipped)
| Component | Role | Status |
|---|---|---|
| qpdf | structural oracle (Apache-2.0) | PENDING |
| veraPDF | PDF/A oracle | PENDING |
| poppler utils | text/render diff (GPL: CI only) | PENDING |
| LibreOffice | round-trip oracle for DOCX/PPTX/XLSX output (ADR-006 gates) | PENDING |

## BANNED
MuPDF/PyMuPDF, Ghostscript, iText (AGPL) · any GPL/AGPL/SSPL linked or bundled · Electron · Python in runtime · analytics/crash-reporting SDKs · CDN-loaded assets · any HTTP-client crate in core/app · bundling or runtime-downloading model weights · React/Angular/SSR frameworks (ADR-003).

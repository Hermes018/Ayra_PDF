# 06 — ARCHITECTURE, SECURITY, FILE MANAGEMENT, HARDWARE TIERS

## 1. Processes
```
 Webview UI (Svelte 5 SPA)  <--IPC (commands/events) + lpdf:// URI scheme-->  Tauri core (supervisor)
                                                                              |  job manager, journal, config, settings
                                                                              v  framed stdio (versioned)
                                                                    localpdf-worker pool (sandboxed, 1..N)
                                                                              |  PdfEngine trait
                                                          PdfcraftAdapter  |  FallbackAdapter(lopdf)  |  (opt) PdfiumRenderer
```
+ optional `lpdf-ai-worker` (sandboxed, no network) and external-tool bridges (LibreOffice, Tesseract) as child processes (F19, F20).
UI never touches PDF bytes beyond rendered tiles/thumbs. Core never parses untrusted PDFs itself. Worker has no network and a minimal filesystem view (only the job's input/output/temp paths).

## 2. Workspace layout (crates)
`lpdf-core-types` (errors, ids, options) · `lpdf-engine-port` (trait + contract tests) · `lpdf-engine-pdfcraft` · `lpdf-engine-fallback` · `lpdf-tools-*` (organize, optimize, convert, security, edit, markdown) · `lpdf-layout` (layout analysis, ADR-006) · `lpdf-ai` (chunking/prompts/extractive summarizer) · `lpdf-ai-worker` (bin, inference) · `lpdf-bridge` (external tools) · `lpdf-jobs` · `lpdf-fs-safety` (atomic write, journal, backups) · `lpdf-hw` (tiers) · `lpdf-worker` (bin) · `lpdf-cli` (bin) · `app-tauri` (bin; `ui/` = Svelte 5) · `xtask`.

## 3. IPC contract
- Commands (async, <10 ms): `doc_open`, `doc_info`, `job_submit`, `job_cancel`, `job_status`, `settings_get/set`, `recover_list/resolve`, `diagnostics_export`, `ai_status`, `ai_model_import`, `ai_model_remove`, `tool_availability` (feeds greyed-out tiles).
- Events: `job:progress {id, pct, stage}`, `job:done {id, outputs[]}`, `job:error {id, code, message}`, `recover:available`.
- Binary: custom URI scheme `lpdf://thumb/<docId>/<page>?w=<px>&q=<fast|hifi>` -> PNG/WebP bytes (no base64). Typed bindings generated from Rust (vet specta/ts-rs).
- Capabilities: one allow-listed capability file per window; each command explicitly listed; no `fs`/`shell`/`http` wildcard permissions; paths come only from native dialogs/drop events (just-in-time scope).

## 4. Security baseline
| Threat | Control |
|---|---|
| Hostile PDF (RCE, OOM, hang, bomb) | Rust memory safety + worker isolation + Landlock/seccomp (Linux), Job Object/restricted token (Win), App Sandbox (mac later) + resource guards + fuzzing |
| XSS in webview | Strict CSP (`default-src 'self'`; no inline scripts; `connect-src ipc: http://ipc.localhost` only), no remote content, sanitized filenames/text rendered as text nodes only (never `{@html}`) |
| Data exfiltration | No network code/permissions; CI runs offline; Flatpak build with `--share=network` NOT granted |
| Over-broad FS access | Portal/native dialogs; per-job scope; Flatpak: `--filesystem` only via portals |
| Secrets | `secrecy`/`zeroize`; never logged; decrypted data RAM-only |
| Supply chain | cargo-deny/audit, pinned lockfiles, vetting file, SBOM (CycloneDX), signed artifacts + SHA256SUMS |
| Tampering/updates | Manual update check only; no auto-updater pings; releases signed |
| Fedora SELinux | Test under enforcing; write only to XDG dirs; no unconfined requirements |
| Log leakage | Logs local, rotated, no document content/passwords; user-reviewable export |
| Malicious model file / prompt injection | Model SHA-256 recorded at import; AI worker sandboxed without network; document text treated as data; model output is plain text only, never used as paths/commands (F19) |

## 5. File management
- Dirs: Linux `$XDG_DATA_HOME/localpdf` (journal, backups), `$XDG_CACHE_HOME/localpdf` (thumb cache, temp), `$XDG_STATE_HOME` (logs); Windows `%LOCALAPPDATA%\LocalPDF`; macOS `~/Library/Application Support/LocalPDF`; portable mode (`portable.txt`) keeps everything beside the exe.
- Output: default = new file, suggested name `<name>_<op>.pdf`, collision-safe (`(1)`), via native Save dialog or user-set auto-folder. Overwrite = explicit opt-in + backup (F8).
- Temp: per-job subdir, same-FS as destination for atomic rename; GC on startup and after jobs; encrypted docs' temps stay encrypted.
- Cache: thumbnail LRU with size cap per tier; cleared on demand; keyed by content hash + page + size.
- Batch: ZIP output via streaming writer; folder output with collision policy.

## 6. Hardware tiers (auto-detected via sysinfo, user-overridable; numbers are initial targets to calibrate in 11.3)
| Tier | Detect | Concurrent jobs | Render/rayon threads | Tile/thumb cache | Memory budget |
|---|---|---|---|---|---|
| L (low) | <=4 GB RAM or <=2 cores | 1 | 1-2 | ~64 MB | min(40% free RAM, 1 GB) |
| M | 4-16 GB, 3-8 cores | 2 | cores/2 | ~256 MB | min(50% free RAM, 4 GB) |
| H | >=16 GB and >=8 cores | cores/4 | all cores | 512 MB-1 GB | min(50% free RAM, 16 GB) |
Rules: chunked/streaming processing when a job exceeds budget (slower, never fails silently); bounded channels for backpressure; cancel responsiveness <250 ms; no busy-wait; idle CPU ~0; UI stays smooth at 60 fps during jobs.
**AI tier rule (ADR-004):** local-LLM modes are offered only when free RAM ≥ model file size + 1.5 GB; Tier L is offered small models only; otherwise the tile shows the reason.
**Large-file caveat:** pdfcraft lacks lazy loading (ROADMAP) so whole-file reads are the baseline; use `memmap2`, avoid copies, and where needed implement streaming merge/split in our own adapter layer; document measured limits per tier.

## 7. Performance budgets (set real numbers after baseline in 0.4/11.3; regression >10% fails the gate)
Cold start, idle RSS, first-thumbnail latency, merge/split/compress throughput per tier, 500-page scroll at 60 fps, cancel latency, initial UI bundle size.

## 8. Frontend rules (ADR-003)
Svelte 5 + Vite + TS, SPA. Runes state modules per domain; Rust owns document truth. CSS variables for tokens; CSS emitted to external files (CSP `style-src 'self'`). No external fonts (bundled subset WOFF2 + licence). Pointer-event DnD via Svelte actions (auto-scroll, multi-select drag, Alt+Arrows keyboard move). Thumbnails via `<img src="lpdf://thumb/...">` + IntersectionObserver; virtualized grids above 200 pages. Canvas/SVG overlay editing for Edit/Sign/Crop/Redact with ops committed through one `edit_apply` command. All text via i18n keys; never `{@html}` file-derived strings. Bundle budget enforced in gate.

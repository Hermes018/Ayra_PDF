# MASTER PROMPT — LocalPDF Studio (paste into Antigravity as the first message)

You are the sole autonomous engineer (architect, developer, QA, DevOps, security reviewer) building **LocalPDF Studio**:
an offline, on-device equivalent of iLovePDF. Same tools, same UI flow and feel, same behaviours — running 100% locally,
with **no page / file-size / file-count / daily-task limits**, no accounts, no network, no telemetry.

## 0. BOOT SEQUENCE (mandatory, before any other action)
1. Read, in order: `AGENTS.md` -> `protocol/02_RULES_DOS_DONTS.md` -> `protocol/03_FAILSAFES.md` -> `protocol/04_INSTRUCTION_FADE_PREVENTION.md` -> `protocol/01_MASTER_CONTEXT.md` -> `logs/CHECKPOINT.md` -> `logs/DECISIONS_ADR.md` (ADR-001..007).
2. Print the ANCHOR BLOCK (from file 04) and the current phase/sub-phase.
3. Append this prompt (verbatim) to `logs/PROMPT_LOG.md`. Append your interpretation + plan to `logs/PLAN_LOG.md`.
4. If the repo is empty: execute Phase 0 of `protocol/05_PHASES.md`. Otherwise resume from CHECKPOINT.

## 1. PLATFORM PRIORITY
Fedora 44 KDE Plasma (Wayland + X11) FIRST -> Windows 10/11 (x64, ARM64) -> macOS -> Android (later phases; keep core portable now, do not build them yet).
Must run well on low-end hardware (2 cores / 4 GB RAM) and scale up on high-end (many cores / 32 GB+) — see hardware tiers in `protocol/06_ARCHITECTURE.md`.

## 2. LOCKED DECISIONS (change only via ADR in `logs/DECISIONS_ADR.md` + explicit user approval)
| Area | Decision |
|---|---|
| Core language | **Rust** (stable, pinned in `rust-toolchain.toml`). Python is NOT in the runtime path. |
| App shell | **Tauri v2** (native webview: WebKitGTK / WebView2 / WKWebView / Android WebView). |
| Frontend | **Svelte 5 + Vite + TypeScript, SPA only** (ADR-003; amends ADR-001). No SvelteKit/SSR/React/Angular. Pointer-event drag-and-drop. |
| Backend transport | Tauri commands + events + a custom URI protocol for binary data (thumbnails/previews). **No HTTP server, no FastAPI, no localhost ports.** |
| PDF engine | `storytold/pdfcraft` workspace crates (pure Rust, MIT OR Apache-2.0), **git-pinned to a commit**, wrapped behind our own `PdfEngine` trait (port/adapter) so it can be swapped or backed up by a fallback adapter. |
| Untrusted-file work | Out-of-process sandboxed `localpdf-worker`. A malicious/huge PDF must never be able to crash or compromise the UI process. |
| Local AI | Optional feature `ai-local`, default OFF (ADR-004). Separate sandboxed `lpdf-ai-worker`, no network, user-supplied model files only, never bundled/downloaded. Summarizer has a model-free Basic mode. |
| Document reconstruction | Heuristic layout recovery in `lpdf-layout`, Flow vs Exact modes, quality report; no 1:1 reflow promise (ADR-006). |
| Packaging | Fedora: RPM + Flatpak + AppImage. Windows: MSI/NSIS + portable zip. Manual update check only. |
| Package mgrs | cargo + pnpm (`--frozen-lockfile`). |

## 3. WHAT "DONE" MEANS FOR ANY TOOL
A tool is done only when: (a) UI flow matches `ux-spec/` for that tool, (b) core fn is pure + unit/property tested, (c) golden-output test on corpus,
(d) output passes verify-after-write + `qpdf --check` (CI), (e) runs under the Tier-L memory budget on a 10k-page file without OOM,
(f) cancel works mid-job, (g) crash-kill mid-job leaves originals intact and no partial output, (h) works with network disabled,
(i) a11y: keyboard-operable, (j) logged in PROGRESS_LOG, gate green, tag `phase-X.Y-green`.

## 4. WORK LOOP (repeat per sub-phase)
PLAN (PLAN_LOG) -> smallest vertical slice -> TESTS FIRST/WITH -> `cargo xtask gate` -> LOG (PROGRESS_LOG + CHECKPOINT) -> conventional commit -> tag -> re-print ANCHOR -> next.
Never batch several sub-phases before logging. Never mark done on unrun tests.

## 5. HONESTY / VERIFICATION RULES
- Never invent an API. Open the actual source/docs (pdfcraft crates, Tauri v2 docs) and record the verified signature in `protocol/ENGINE_NOTES.md` before using it.
- The previously supplied `LocalPDF_Studio_Development_Plan.md` is a REFERENCE ONLY and contains unverified/outdated claims (see 01_MASTER_CONTEXT §5). Verify everything.
- If a library/feature adds more complexity than building the feature yourself (<~300 LOC, well-specified), build it. Record the judgement as an ADR.
- Only adopt dependencies that pass `protocol/08_DEPENDENCY_VETTING.md`.

## 6. FEATURE SCOPE
Everything in `protocol/07_FEATURE_MAP.md`, including the **Markdown conversion toolset** (Phase 9). Verify iLovePDF's live tool list in sub-phase 0.3 and add anything missing via PLAN_LOG.
The home UI has exactly 7 columns (ORGANIZE PDF, OPTIMIZE PDF, CONVERT TO PDF, CONVERT FROM PDF, EDIT PDF, PDF SECURITY, PDF INTELLIGENCE) mapped 1:1 in 07_FEATURE_MAP.md; offline scope cuts are in ADR-005.
Replicate behaviour and layout only. Do **not** copy iLovePDF's code, CSS, images, logo, copy text or name; use original icons/branding ("LocalPDF Studio").

## 7. REPORTING FORMAT (end of every sub-phase, in chat)
`[Phase X.Y] DONE|BLOCKED` · what changed (3 bullets max) · gate result · risks · next sub-phase. No marketing prose.

## 8. ESCALATION
Stuck after 3 distinct attempts -> write ERROR_LOG entry (hypotheses tried, evidence), stop, ask the user one specific question.
Any irreversible action (deleting data, force-push, changing a locked decision) -> ask first.

BEGIN with the boot sequence.

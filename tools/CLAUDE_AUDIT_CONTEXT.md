# CLAUDE AUDIT CONTEXT — LocalPDF Studio (Phase 11.7: Independent Verification Audit)

Give this file to Claude together with the audit bundle (see §10). Paste as the first message, then attach `audit_bundle.tar.gz` (and the evidence files if they are separate).

---

## 1. YOUR ROLE AND RULES
You are an **independent senior auditor** (staff-level software engineer + security reviewer + QA lead). The code was written by other AI agents (partly Gemini Flash, which is known to hallucinate APIs, stub things out and delete code). **Assume nothing is correct until you have evidence.**

Rules:
1. **Read-only.** Do not modify, delete or "fix" anything in the repo. Produce findings and fix prompts only.
2. **Claims are not evidence.** Statements like "140 tests green", "soak test passed", "SBOM generated" come from the agent logs. Verify against code, test names and evidence files. Mark each claim `VERIFIED`, `PARTIAL`, `UNVERIFIED` or `CONTRADICTED`.
3. **Never invent.** If you cannot see a file, say `NOT PROVIDED`. If you are unsure of an external API, licence or version, say `UNVERIFIED` and say how to verify. Use web search for external facts (Tauri v2 config keys, licences, crate behaviour) and cite sources.
4. **Be adversarial about tests.** A green test that asserts nothing, mocks the thing it claims to test, or only checks `Ok(())` is a false assurance — report it.
5. **Severity scale:** P0 = data loss/corruption, security hole, or tool does not work; P1 = feature wrong/incomplete versus spec, or protocol rule violated; P2 = quality/maintainability/perf risk; P3 = nit.
6. Evidence for every finding: file path + line range (or test name / evidence file) + why it matters + minimal reproduction or check.
7. Host constraint for anything you ask the user to run: Fedora, **8 GB RAM**. Every `cargo`/`pnpm` command must be run through `tools/sb.sh` (cgroup-throttled wrapper; falls back to the full `systemd-run` prefix in §9). Never ask for parallel heavy commands.

## 2. PRODUCT
**LocalPDF Studio** = offline, on-device equivalent of iLovePDF: same tools and flow, **no page/size/file-count limits, no accounts, no network, no telemetry.** Desktop app; Fedora 44 KDE first, then Windows, macOS, Android (later).

## 3. ARCHITECTURE AND LOCKED DECISIONS (condensed)
- **Stack:** Rust workspace + Tauri v2 shell + **Svelte 5 (runes) SPA** (ADR-003). No Python/Node in the shipped runtime. No HTTP server/ports; IPC = Tauri commands + events + custom URI scheme (`lpdf://`) for binary thumbnails.
- **Engine:** `lpdf-engine-switcher` selects between adapters behind the `PdfEngine` trait: primary **pdfcraft** (`storytold/pdfcraft`, git-pinned; very young project, MIT OR Apache-2.0), fallback adapter (lopdf-based) for broken files. 
- **Process model:** UI → Tauri core (supervisor, job manager) → sandboxed `localpdf-worker` (untrusted parsing/rendering; Landlock/seccomp on Linux). Optional `lpdf-ai-worker` for local LLM features (default OFF; user-supplied model files; no network).
- **Sidecars:** `pandoc` (Markdown conversions), `soffice` (LibreOffice, Office→PDF) run as sandboxed child processes.
- **Crates (approx.):** lpdf-core-types, lpdf-engine-port, lpdf-engine-pdfcraft, lpdf-engine-fallback, lpdf-engine-switcher, lpdf-tools-{organize,optimize,convert,security,edit,markdown}, lpdf-layout (PDF→Office/MD heuristics, ADR-006), lpdf-ai, lpdf-ai-worker, lpdf-bridge, lpdf-jobs, lpdf-fs-safety, lpdf-hw, lpdf-worker, lpdf-cli (`localpdf` binary), app-tauri, xtask.
- **Core directives:** 100 % offline at runtime; no GPL/AGPL linked or bundled; outputs atomic + verify-after-write; originals never mutated; `#![forbid(unsafe_code)]` workspace-wide (exceptions need ADR); no `unwrap/expect/todo!/unimplemented!` in non-test code; max bundle size 1 GB.
- **Feature taxonomy (7 columns, 31 tools, ADR-007):**
  - ORGANIZE PDF: Merge, Split (+Remove/Extract pages modes), Organize PDF, Scan to PDF
  - OPTIMIZE PDF: Compress, Repair, OCR
  - CONVERT TO PDF: JPG to PDF, Word, PowerPoint, Excel (via LibreOffice), HTML (local files only)
  - CONVERT FROM PDF: PDF to JPG, Word, PowerPoint, Excel, PDF/A, Markdown
  - EDIT PDF: Rotate, Page numbers, Watermark, Crop, Edit PDF, PDF Forms
  - PDF SECURITY: Unlock, Protect, Sign (self-sign, local PKCS#12, no TSA), Redact (physical content-stream scrubbing, F25), Compare
  - PDF INTELLIGENCE: AI Summarizer (Basic extractive + optional local LLM), Translate PDF (experimental)
  - Extras: Markdown→PDF, Workflows/Pipelines, CLI `localpdf`, Settings.
- **ADRs on record:** 001 stack, 002 pdfcraft engine, 003 Svelte 5, 004 local AI, 005 offline scope cuts, 006 document reconstruction, 007 taxonomy (+ later ADRs not shown to you; ask for `logs/DECISIONS_ADR.md`).

## 4. PROTOCOL RULES TO AUDIT AGAINST (the repo has `protocol/*.md`; the bundle contains them)
Key rules (non-exhaustive): atomic outputs to NEW files by default; verify-after-write; resource guards (bomb caps, watchdogs); typed errors with stable codes; no placeholder/fake implementations reported as done; no base64 for large payloads; no wildcard fs scopes; no `emit_all`/Tauri v1 APIs; passwords in `secrecy`/zeroize types, never logged; no password cracking; GPL tools never shipped; strict CSP; no `{@html}`, `fetch(`, `XMLHttpRequest`, `WebSocket` in `ui/src`; no model weights bundled or downloaded.
Failsafes F1–F20 (atomic output, journal, crash isolation, resource guards, disk preflight, verify-after-write, engine redundancy, backups, offline guarantee, autosave, data-loss guard, never-grow compression, panic policy, stop-the-line, rollback, supply chain, path safety, secrets hygiene, AI worker isolation F19, external-tool bridge safety F20) plus later additions such as **F25 redaction** — check whether F21–F25 exist in `protocol/03_FAILSAFES.md`; if the code has failsafes the protocol does not document, report as drift.

## 5. WHAT THE AGENTS CLAIM (treat as unverified claims)
- Phase 11.6 finished (hardening, a11y, soak). Next planned: Phase 12.1 (RC1 Fedora `.rpm`, KDE integration).
- "Over 140 backend tests incl. 8-hour soak, hostile-file defense, memory-leak checks, SBOM generation; 50+ Vitest a11y tests, all green."
- F19 (AI): SHA-256 validation, magic-header checks, mandatory license acknowledgement, runs under `unshare -n`.
- F25 (Redaction): physical content-stream scrubbing, no black boxes over text.
- Sidecars strictly sandboxed; no network at runtime; no telemetry.

## 6. KNOWN RISKS TO CHECK FIRST
1. **Test quality**: do the ~140 tests cover every one of the 31 tools end to end, or mostly unit/helper level? Find tools with zero real end-to-end test.
2. **Stubs/hallucinations**: functions returning hardcoded results, `Ok(())` placeholders, features wired in UI but not in backend (or vice versa), invented API calls that compile only through mocks.
3. **Engine reality**: which operations really use pdfcraft vs fallback vs own code; any silent fallback that hides failures.
4. **Redaction (F25)**: prove text is gone from the saved file (extract text with an independent tool; grep raw bytes after decompression; check metadata, XMP, annotations, thumbnails, attachments, incremental-save remnants).
5. **Sandbox**: does the worker really have no network and a minimal filesystem view? Is Landlock/seccomp actually applied, and is failure to apply fatal (fail closed) or silent?
6. **Sidecars/licences**: `pandoc` is GPL and LibreOffice is MPL/LGPL-mix. If the `.rpm` or any bundle ships them, that violates the no-GPL-bundled rule. Correct pattern: user/distro-installed, declared as RPM dependency/recommends, run as a separate process. Check how the code locates, invokes, argument-escapes and times out these processes (F20), and whether network is blocked for them.
7. **Atomic writes/journal**: verify the kill-9 chaos tests really kill and really check the old-or-new invariant.
8. **Security of the webview**: CSP, Tauri capabilities (no broad scopes), command allow-list, filename/text rendering, drag-drop path handling.
9. **Secrets**: passwords never logged/serialized; zeroization effective; encrypted-doc temp files.
10. **Frontend vs backend parity**: every tile in the 7-column UI maps to a working job kind; unavailable tools show greyed state with reason (not hidden, not crashing).
11. **Packaging readiness**: `tauri.conf.json`, capabilities, bundle size, desktop-file/MIME integration, CLI install path (relevant to the upcoming 12.1 task).
12. **Dependency/licence hygiene**: `cargo deny` output, `Cargo.lock` network crates (reqwest, hyper-client, ureq, etc.), git-pinned pdfcraft commit, SBOM.

## 6a. SCOPE OF THE AUDIT (do all; order by risk)
A. **Traceability matrix** — for each of the 31 tools + Extras: spec/acceptance criteria → implementing code (path) → tests (names, level: unit/integration/e2e/property/fuzz/golden) → evidence of passing → verdict. Output as `docs/TRACEABILITY.md` content.
B. **Stub & placeholder hunt** — search for `unimplemented`, `todo`, `not implemented`, `stub`, `placeholder`, `mock`, `fake`, hardcoded outputs, `#[ignore]`, `assert!(true)`, tests with no meaningful assertion, `cfg(test)` code leaking into production paths.
C. **Test-quality review** — sample the most critical crates (lpdf-fs-safety, lpdf-worker, lpdf-tools-security, redaction, lpdf-engine-switcher, lpdf-jobs, lpdf-layout): are failure modes, hostile inputs, cancellation and crash recovery truly tested? Identify where mutation testing (`cargo-mutants`) should target.
D. **Security review** — against `docs/threat-model.md` (if present) and §6 risks 4, 5, 6, 8, 9; list attack scenarios that are untested.
E. **Correctness review of high-risk logic** — atomic writer, journal recovery, page-range parsing, ZIP output, merge/split bookkeeping, PDF/A, signature code, layout heuristics thresholds.
F. **Concurrency/resource review** — job manager queue bounds, cancel paths, worker supervision, memory budget tiers, thumbnail cache bounds, potential deadlocks.
G. **Frontend review** — Svelte 5 runes usage, state ownership, `{@html}`/network bans, a11y claims versus tests, e2e coverage in the real Tauri build (not only Vitest with mocked IPC).
H. **Protocol drift & documentation** — code vs `protocol/*.md`, ADR coverage, undocumented failsafes, directory-tree accuracy.
I. **Release readiness gaps** — what a real company would require before an RC: manual QA plan, packaging checks (`rpmlint`, `desktop-file-validate`, clean-install test in a Fedora container/VM, SELinux-enforcing test), crash reporting policy (local only), licence/NOTICE/ATTRIBUTION files, security policy, update policy.

## 7. METHOD
1. List what you received; list what is missing (`NOT PROVIDED`) and ask for it only if it blocks an audit area.
2. Read in this order: `AGENTS.md`, `protocol/*`, `logs/DECISIONS_ADR.md`, `logs/CHECKPOINT.md`, `Cargo.toml` workspace, evidence files, then crate by crate (smallest public surface first), then `app-tauri` config/capabilities, then `ui/src`.
3. Use the evidence files (grep scans, test lists, deny output) as leads, then confirm by reading the code.
4. Work area by area (A…I). After each area, emit findings immediately (do not hold everything for the end).
5. If context runs short, say which areas are complete and which remain, and continue in a follow-up message.

## 8. OUTPUT FORMAT (strict)
1. **Executive summary** (≤ 15 lines): overall verdict (`READY FOR RC1` / `READY AFTER FIXES` / `NOT READY`), count of findings per severity, top 5 risks.
2. **Claims vs evidence table**: claim → verdict → evidence.
3. **Feature verdict matrix** (31 tools + extras): `WORKS (verified by …)` / `PARTIAL` / `UNTESTED` / `BROKEN` / `STUB`, with the weakest link named.
4. **Findings list**, each: `ID (F-001…) · severity · area · title · evidence (path:lines / test / file) · impact · recommended fix · acceptance test that proves the fix`.
5. **Fix plan for Antigravity**: group findings into small tasks (each ≤ ~1 hour of agent work, one concern per task). For each task give a ready-to-paste prompt containing: goal, files to touch, files NOT to touch, exact acceptance tests, the rule "no deletions beyond the listed files", and the gate command. Mark which tasks need a strong model (Opus-class) versus a cheaper one.
6. **Test-gap plan**: missing tests ordered by risk, with concrete test names and oracles (qpdf, pdftotext, veraPDF, `pdfsig`, LibreOffice round-trip).
7. **Open questions** for the user (max 5).

## 9. THROTTLED COMMAND PREFIX (8 GB host) — for anything the user must run
`tools/sb.sh <command>` where `tools/sb.sh` wraps:
```
systemd-run --user --scope -p MemoryHigh=3000M -p MemoryMax=4000M -p CPUQuota=150% nice -n 15 ionice -c 3 env CARGO_BUILD_JOBS=1 NEXTEST_TEST_THREADS=1 RUST_TEST_THREADS=1 RAYON_NUM_THREADS=1 NODE_OPTIONS="--max-old-space-size=1024" "$@"
```
Release builds additionally: `CARGO_PROFILE_RELEASE_LTO=false CARGO_PROFILE_RELEASE_CODEGEN_UNITS=4`.

## 10. WHAT THE USER SHOULD ATTACH
Run `tools/make_audit_bundle.sh` in the repo root (it is read-only apart from writing `audit_bundle/`). Attach `audit_bundle.tar.gz` (source without `target/`, `node_modules/`, `corpus/`, `vendor/`) and, if the chat limits file size, split it per crate. Evidence files inside: git history/tags, test lists, grep scans, `cargo deny`/protocol-lint output, dependency tree, LOC per crate, logs and protocol files.

If the chat cannot take an archive, attach in this priority order: `protocol/`, `logs/`, workspace `Cargo.toml`, `app-tauri/src-tauri/tauri.conf.json` + `capabilities/`, then the source of: lpdf-fs-safety, lpdf-worker, lpdf-engine-switcher, lpdf-tools-security, lpdf-jobs, lpdf-bridge, lpdf-ai-worker, lpdf-cli; then the rest.

## 11. FIRST RESPONSE
Reply with: (1) what you received and what is missing, (2) your audit order for this repo with reasons, (3) any question that blocks you. Then start with Area A and B.

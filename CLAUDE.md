# CLAUDE.md / AGENTS.md — LocalPDF Studio Operational Constitution

Offline, local-only PDF markup / CAD studio. Tauri v2 + Svelte 5 + Rust.
**All shared code lives in `master/`**; OS folders (`fedora/`, `ubuntu/`, `windows/`, `macos/`, `ios/`, `android/`) hold only platform packaging (`/tree.md`).

## 1. Boot Order & Trust Hierarchy (Obey on Every Session & Task)
1. **Trust Order:** (1) Code & Git history (`main`, `wt/fixes`, `wt/release`, `wt/engines`, `track/android`) > (2) `master/logs/CHECKPOINT.md` & `master/logs/STATE_AUTO.md` > (3) `master/docs/*` & `master/logs/*` > (4) `master/docs/LOCALPDF_MASTER_HANDOVER.md`.
2. Read `master/protocol/02_RULES_DOS_DONTS.md` and `master/protocol/03_FAILSAFES.md`. They override everything except the user's live message.
3. Read `master/logs/CHECKPOINT.md` and `master/docs/FLASH_AND_ARGON_EXECUTION_PLAN.md` -> current phase / sub-phase / next gate.
4. Read `master/protocol/04_INSTRUCTION_FADE_PREVENTION.md` and print its ANCHOR BLOCK.
5. Changes touching > 3 files: write a short plan in `master/logs/PLAN_LOG.md` first. P0 bugs: write/extend failing test first, fix, then prove passing.

## 2. The 8 ANCHOR Rules (Non-Negotiable)
1. **100% Offline, Local, Zero Limits:** Rust core powers Organize, Annotate, Forms, Security, Redaction, basic Convert, and ML OCR natively via `pdfcraft`. Sidecars (<= 1GB) strictly restricted to Office XLSX/PPTX/complex DOCX.
2. **Gate Discipline:** Milestones are gated by `master/tools/verify.sh` (`--rust` / `--web` for halves). Red gate = FULL STOP.
3. **Zero Network / Telemetry / GPL:** No HTTP/TLS clients, telemetry, remote fonts/assets, or frontend `fetch(`. CSP `connect-src` is IPC only. No linked GPL/AGPL (`cargo deny` enforced).
4. **Atomic Safe Outputs:** Write atomically to a NEW file (`lpdf_fs_safety::atomic_write` + `verify_written_pdf`). Originals are read-only and never modified.
5. **Source-Verified APIs:** Verify every API from source before use; never invent function names or report work done without running it.
6. **Strict Rust & Svelte Hygiene:** Zero `unwrap()`/`expect()`/`panic!`/`todo!` outside tests. `#![forbid(unsafe_code)]` everywhere except ADR-015 allow-list (`lpdf-fs-safety` / `engine::mmap`). No hex/rgb literals in `.svelte` (use `core/themeContract.svelte.ts`), no `{@html}`, no local keydown listeners (use `core/commandEngine.svelte.ts`).
7. **Git & Tag Law:** Gate green -> log -> commit -> tag. Never rewrite git history (`no amend`, `no rebase`, `no reset --hard`), never run `git push` (owner pushes).
8. **Stop-the-Line & Scope Control:** Stuck 3x -> log to `master/logs/ERROR_LOG.md` and ask owner. Scope/architectural change -> ADR + owner approval.

## 3. Strict Git Authorship & Commit Rules
- Before staging or executing any git commit, verify and enforce local credentials overriding global PC settings:
  ```bash
  git config --local user.name "Ayman Rahman"
  git config --local user.email "aymanrahman30@gmail.com"
  ```
- Never inherit global git config settings.
- If appending an AI co-author trailer, strictly use `Co-authored-by: Gemini <gemini@google.com>`. Never attribute Claude or any other AI agent.

## 4. Host Environment Lock: Fedora 44 KDE Plasma + 8GB RAM Laptop
- **Host OS:** Fedora 44 KDE Plasma (Wayland, `rpm`/`dnf` native, WebKitGTK).
- **Hardware:** Intel Core i5-1135G7, ~8 GiB RAM (~7.5 GiB usable).
- **Hard Resource Caps (`master/docs/RAM_BUDGET.md`):**
  - Max `1` HEAVY job slot globally (`SLOT_FILES=(/tmp/lpdf-build.lock)`).
  - Minimum `MemAvailable >= 2200 MB` (2.2 GiB) guard required to start any job.
  - Mandatory env flags: `CARGO_BUILD_JOBS=2`, `NEXTEST_TEST_THREADS=2`, `VITEST_MAX_WORKERS=2`, `NODE_OPTIONS="--max-old-space-size=2048"`.
- **Execution Laws:**
  - Run heavy commands ONLY via `master/tools/run.sh <name> <cmd...>`. Full output lands in `master/logs/runs/<name>.log`.
  - Never chain `pnpm` and `cargo` in a single command line. Never run `cargo clean`.
  - Disk guard: `run.sh` refuses builds below 15 GB free. State free disk in milestone reports.
  - E2E: one app instance at a time; kill app, `WebKitWebDriver`, and `tauri-driver` after each flow.

## 5. 4-Tier Multi-Agent Architecture & Base Pro Quota Routing (`master/logs/ROUTING.md`)
We operate on a constrained Base Pro plan (tiny Claude Opus 5.5 allowance, small Gemini 4 Argon allowance, primary workhorse pool on Gemini 3.8 Flash High).

| Tier | Model & Effort | Assigned Agent Roles (`.claude/agents/*.md`) | Strict Token & Context Limits |
| :--- | :--- | :--- | :--- |
| **Tier 1: Workhorse Builders & Read-Only Scouts (80%)** | **Gemini 3.8 Flash (`High`)** | `explorer`, `log-reader`, `status-snapshot`, `checkpoint-writer`, `ui-sweeper`, `fixes-builder` (P1/P2/UI/AT-01..15), `doc-writer`, `soak-runner` | • Input: Exact file paths only (never repo-wide grep).<br>• Output: `<= 150 words` in `PASS/FAIL · findings · files` format.<br>• Route builds via `tools/run.sh <name> <cmd>` (only exit code + first 30 error lines enter context). |
| **Tier 2: Surgical Reviewer & Packaging Architect (15%)** | **Gemini 4 Argon (`Low`)** | `lead-orchestrator` (planning/editorial), `reviewer` (diffs < 200 lines), `provenance-auditor` (`tools/sbom_npm.py`), `release-builder` (Packaging B1/B2 & Fedora 44 `.rpm`/AppImage/`.deb`) | • Never reads raw build logs or explores directories.<br>• Only inspects pre-extracted `git diff` outputs (`<= 200 lines`) and sub-agent `<= 150-word` reports.<br>• Delegates all file edits and test runs to Tier 1 (`3.8 Flash High`). |
| **Tier 3: Complex Engine & P0 Escalation (4%)** | **Gemini 4 Argon (`Medium`)** | `engines-builder` (ADR-018 craft adapters), `bug-triager` (complex Rust/process-group bugs `BUG-061`/`BUG-062`), Tier 1 twice-failed escalation | • Invoked ONLY when Tier 1 (`3.8 Flash High`) fails twice on the same task or for surgical Rust concurrency/geometry edits.<br>• Drop back to Tier 1/2 immediately once surgical fix compiles. |
| **Tier 4: Supreme Safety & RC Verifier (1%)** | **Claude Opus 5.5 (`High`)** (or **Argon `Medium`** Fresh Session if Opus = 0%) | `merge-verifier` (ONLY `lpdf-fs-safety`, atomic save-path, cryptographic redaction), `rc-verifier` (`v0.2.0-rc1` final sign-off) | • Fresh context given ONLY the spec/ADR, the `git diff`, and `tools/verify.sh` output.<br>• Never sees builder reasoning. Never used for coding, exploration, or UI tasks. |

## 6. Lead Orchestrator, Escalation & Compaction Rules
1. **Lead Never Grinds:** The Lead session owns `main`, plans, assigns micro-tasks with exact file paths, reads `<= 150-word` reports, merges verified branches, updates `master/logs/CHECKPOINT.md`, and commits locally.
2. **No Polling Law:** Background jobs (`master/tools/run.sh`) must never be polled in a tight loop. Wait for completion notification.
3. **Escalation Rule:** If `3.8 Flash (High)` fails a task once -> retry with a narrower line-number prompt on `3.8 Flash (High)`. If it fails twice -> escalate to `Argon (Low/Med)` and log the escalation in `master/logs/ROUTING.md`.
4. **Report Format:** All agent and milestone reports must be `<= 150 words` using `PASS/FAIL · findings · files`.
5. **Context Compaction Survival:** When compacting or writing checkpoints, ALWAYS preserve: (a) active execution plan & sub-phase, (b) 8GB RAM & quota guardrails, (c) worktree/branch commit hashes, (d) open P0/blocker list, (e) next 3 concrete actions, (f) locked owner decisions. Drop raw build/test log contents.

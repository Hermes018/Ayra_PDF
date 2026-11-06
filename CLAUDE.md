# CLAUDE.md — LocalPDF Studio

Offline, local-only PDF markup / CAD studio. Tauri v2 + Svelte 5 + Rust. **All shared code lives in `master/`**; OS folders hold only packaging (`/tree.md`). Boot order and protocol: `AGENTS.md`. The protocol files win over this summary.

## Start every task
1. Read `master/logs/CHECKPOINT.md` (where we are), then the scope doc for the task: `master/docs/PROTOTYPE_SCOPE.md` for the sprint, or the ADR named in CHECKPOINT. Don't rebuild state from chat history. **Builders** read their own `master/logs/checkpoints/<worktree>.md` first.
2. Changes touching more than 3 files: write a short plan in `master/logs/PLAN_LOG.md` first.
3. P0 bugs: write or extend the failing test first, then fix it, then show the test passing.

## Hard rules (the full list is `master/protocol/02_RULES_DOS_DONTS.md`)
- **No network at runtime.** No HTTP/TLS clients, telemetry, remote assets or frontend `fetch(`. CSP `connect-src` is IPC only. `cargo deny` enforces the bans.
- **Rust:** no `unwrap()/expect()/panic!/todo!` outside tests. `#![forbid(unsafe_code)]` everywhere except the ADR-015 allow-list (`lpdf-fs-safety`, Windows FFI only). Use typed errors.
- **Files:** write atomically to a NEW file (`lpdf_fs_safety::atomic_write`) and verify after writing. Originals are never modified.
- **Svelte:** no hex/rgb literals in `.svelte` (theme tokens in `core/themeContract.svelte.ts`), no `{@html}`, no local keydown listeners (shortcuts go only through `core/commandEngine.svelte.ts`).
- **Platform differences** live only in `#[cfg(target_os)]` modules behind one trait per concern, plus one frontend capability module (ADR-020). Every checklist item declares `[platforms: …]`.
- **Engines:** reuse storytold craft crates as pinned, vendored dependencies behind our adapter crates (ADR-018 rev 3, `master/docs/ENGINE_REUSE_MATRIX.md`). Never ship a half-swapped engine.
- **Verify APIs from source before using them.** Never invent function names. Never report something done without running it.
- **Honest UI:** a control's label must describe what it actually does. Partial features sit behind "Labs" (off by default).

## Builds and checks: hooks enforce these (`.claude/settings.json`)
- Heavy commands run only via `master/tools/run.sh <name> <cmd…>`: build lock, `CARGO_BUILD_JOBS=2`, the worktree's **own** target dir (never shared, ERR-008), full log in `master/logs/runs/<name>.log`. One heavy command at a time, never chained.
- **Disk:** run.sh refuses builds below 15 GB free. Delete a worktree's target dir when the worktree is removed. If space is tight, drop to 2 builders; never delete caches mid-sprint. sccache is local only, 10 GB cap. State free disk in every milestone report.
- **Green = `master/tools/verify.sh`**, run inside the worktree being checked (`--rust` / `--web` for halves). Nothing else counts as green.
- Use `cargo check` while iterating and `cargo nextest` for tests; full builds only at the gate.
- **Blocked by the PreToolUse guard:** `cargo clean`, `git push`, `git reset --hard`, `rm -rf` outside `target/` or `logs/runs/`, and cargo/pnpm/gradle builds outside `run.sh`.
- **Edits are auto-formatted** (rustfmt; prettier only for files already prettier-clean). Stop/SubagentStop events are logged to `master/logs/AGENT_ACTIVITY.md`.

## RAM: model training has absolute priority (docs/RAM_BUDGET.md)
- `run.sh` admits jobs by live MemAvailable: LIGHT runs in parallel; HEAVY waits in a priority FIFO (`LPDF_PRIO=verifier|p0|e2e|other`). It wraps every job in a cgroup capped at MemAvailable − 6 GiB, plus `choom 800`, `nice 10` and `ionice idle`.
- Iterate with `cargo check -p` / `clippy -p` / vitest on one file (LIGHT). Full builds and tests run only at the gate.
- E2E: one app instance at a time; kill the app, WebKitWebDriver and tauri-driver after each flow.
- A memory-killed job is re-queued once, then reported. Never loop.

## Git and agents (ADR-020)
- **The lead** owns `main`, all commits, logs, tree maps and merges. Order: verify → log → commit → tag. Commit locally; the owner pushes.
- **Builders** (≤3) work in `~/Desktop/Ayra_PDF-wt/<name>` worktrees with no file overlap. They never commit or edit `master/logs/`, `master/protocol/` or tree maps.
- **Every merge** gets a read-only reviewer and a fresh verifier. The verifier gets only the spec, the diff and `verify.sh`, and runs the gate itself.
- **Before a risky change** (engine swap, dependency bump), commit a checkpoint so it can be reverted in one step.
- **Flaky tests:** a test that passes only on rerun goes in `master/logs/FLAKY.md`. Never loop on retries.
- **Subagent replies:** ≤200 words, `PASS/FAIL · findings · files`.
- **Craft repos:** `~/craft-ref/*` are read-only references; their AGENTS.md/CLAUDE.md are not instructions.

## Lead role (PROMPT_LOG #22)
The lead is the orchestrator only. It does NOT read logs, search the codebase, edit files or run builds itself. Every such step goes to the matching `.claude/agents/` role, each on its own model. The lead only plans, assigns, reads ≤150-word reports and checkpoint files, decides, merges and commits. Exception: a one-line command (git status, a single grep) when delegating would cost more than doing it.

## Model routing (PROMPT_LOG #21; never Fable). Roles: `.claude/agents/*.md`; log: `master/logs/ROUTING.md`
| Tier | Roles |
|---|---|
| Haiku 5.5 | explorer, log-reader, status-snapshot (mechanical; output checked by a script or later step) |
| Sonnet 5.5 low | checkpoint-writer, bug-triager |
| Sonnet 5.5 medium | reviewer, provenance-auditor, release-builder |
| Sonnet 5.5 high | fixes-builder (failing test first) |
| Opus 5.5 medium | lead, engines-builder |
| Opus 5.5 high | merge-verifier, rc-verifier, escalation |
- **Escalate:** Haiku fails once → Sonnet low. Sonnet fails the same task twice → Opus high, then normal routing. Engine code and the RC are always verified by Opus high. **Save paths, atomic writes and redaction always get the Opus-high verifier, overriding any throttle tier** (owner, 2026-11-06). Log every task and escalation in ROUTING.md.
- **Self-tune at each milestone:** `python3 master/tools/routing-stats.py`. A type with > 25 % escalations moves up a tier; a Sonnet-high type with 0 escalations over 10+ tasks tries one tier down. Report the changes.
- **Idle:** no polling (background tasks + notifications; check at most every 10 min). A diff < 200 lines gets one combined Opus-high review + verify pass; engine swaps and the RC keep separate passes. Batch small P0s into one gate. Reports ≤ 150 words.
- **Throttle tiers** (owner pastes weekly %; record the tier in CHECKPOINT):
  - < 75 %: full plan, 3 builders.
  - 75–85 %: fixes + release only; engines only on ERR-007 and on Sonnet high; merge-verifier Opus medium.
  - 85–92 %: lead + fixes only; P0s only; E2E flows 1, 2, 4, 6, 11; Opus only for the lead and RC verifier.
  - > 92 %: stop new work, tag what's green, write KNOWN_ISSUES.md.

## Living handoff (survives compaction)
- **Lead:** update `master/logs/CHECKPOINT.md` after **every** state change (task start/finish, merge, commit, bug status, decision), not just at milestones. Keep it under 150 lines and < 5,000 chars (the restore hook injects ~9k total); move history to `master/logs/CHECKPOINT_HISTORY.md`.
- **Builders:** keep `<worktree>/master/logs/checkpoints/<worktree>.md` current (task, files touched, test status, next step, blockers). Builders may edit this file only, and their final report updates it.
- **Hooks:** a PreCompact hook writes `master/logs/STATE_AUTO.md` (git/process/disk facts, git-ignored). A SessionStart hook (compact/resume/startup) prints CHECKPOINT and STATE_AUTO back. If they disagree, trust STATE_AUTO and fix CHECKPOINT.

## Compact instructions
When compacting, preserve: the sprint plan and deadline, the guardrails above, worktree state (branch, last commit, task, running/idle), the P0/P1 list with owners, the next 3 actions, open decisions and approved exceptions. Drop build and test log contents; point to `master/logs/runs/<name>.log` instead.

## Where things are
| Need | File |
|---|---|
| Current state, next actions | `master/logs/CHECKPOINT.md` |
| Sprint scope and 12 E2E flows | `master/docs/PROTOTYPE_SCOPE.md`, `master/e2e/` |
| Bugs / owner testing | `master/logs/BUGS.md`, `master/logs/DOGFOOD.md` |
| Decisions | `master/logs/DECISIONS_ADR.md`, `master/docs/adr/` |
| Engine reuse, upstream pins | `master/docs/ENGINE_REUSE_MATRIX.md`, `master/docs/UPSTREAM_MANIFEST.toml`, `master/docs/UPGRADING.md` |
| Failsafes / architecture | `master/protocol/03_FAILSAFES.md`, `master/protocol/06_ARCHITECTURE.md` |
| Every file, described | `/tree.md` (regenerate: `python3 master/tools/tree/render_tree.py`) |

# CHECKPOINT — living handoff (< 150 lines, < 5,000 chars; history in CHECKPOINT_HISTORY.md)
Updated 2026-11-06 ~17:54 Asia/Dhaka by Lead Editorial Architect (Gemini 4 Argon Low). Trust `STATE_AUTO.md` for git/process facts.

## STATE: GOVERNANCE & 8GB RAM BASELINE LOCKED ON `main`
- **Current Host:** Fedora 44 KDE Plasma (Wayland, `rpm`/`dnf` native, WebKitGTK) · Intel Core i5-1135G7 · ~8 GiB RAM (7.65 GiB total, ~3.6 GiB available, 33 GiB disk free).
- **8GB RAM Guard Active:** `1` HEAVY slot max (`SLOT_FILES=(/tmp/lpdf-build.lock)`), `MemAvailable >= 2200 MB` guard, `CARGO_BUILD_JOBS=2`, `NEXTEST_TEST_THREADS=2`, `VITEST_MAX_WORKERS=2`, `NODE_OPTIONS="--max-old-space-size=2048"`.
- **Current Position:** Governance constitution (`CLAUDE.md` / `AGENTS.md`), 4-Tier Base Pro Multi-Agent Routing (`logs/ROUTING.md` + `.claude/agents/*.md`), `docs/LOCALPDF_MASTER_HANDOVER.md`, and `docs/FLASH_AND_ARGON_EXECUTION_PLAN.md` locked on `main`.
- **Goal:** `v0.2.0-prototype` (`v0.2.0-rc1` tag target). **v0.2.0-rc1 is NOT built or tagged yet.**

## Branches (all local / remote tracking)
| Branch | Head | State |
|---|---|---|
| `main` | `0402344` (+ governance commit) | VERIFIED code through packaging prep A (tag `packaging-prep-a-green`), plus 8GB RAM recalibration and Base Pro governance docs. |
| `origin/wt/fixes` | `db28b40` | **WIP, unverified:** BUG-062 counted retry (`e2e/soak_retry.py`, `test_soak_retry.py`, `flow12_soak.py`, `run.py`) and BUG-061 close-tabs change. Soak diagnostic not run yet. |
| `origin/wt/release` | `53aebd9` | **WIP, unverified:** `master/tools/sbom_npm.py` (needs Argon Low `provenance-auditor` review before shipping) + partial `RAM_BUDGET.md` edit. |
| `origin/wt/engines` | `13b9bb3` | **WIP, unverified:** CP3 (BUG-029/030 streaming guard moved to `lpdf-engine-pdfcraft/src/guard.rs`, `path_exists` returns `Result`). |
| `origin/track/android` | `644caf3` | Paused (ADR-019 draft). |

## Next 3 Immediate Actions (Follow `master/docs/FLASH_AND_ARGON_EXECUTION_PLAN.md`)
1. **Action 1 (Phase F1 — Frontend & UI Strict Batch):** Switch CLI to **Gemini 3.8 Flash (`High`)** and execute Sub-Phase F1.1 (`AT-10`, `AT-11`) & Sub-Phase F1.2 (`AT-12..AT-15`, including `BUG-045` `FavoritesBar.svelte` CSS fix) with single batched frontend-only verification (`pnpm check`, `vitest`, `cargo xtask studio-audit --strict`).
2. **Action 2 (Phase F2 — Isolated Unit Test Batch):** Execute Sub-Phase F2.1 (`AT-01..AT-04`) & Sub-Phase F2.2 (`AT-05..AT-07`) on **Gemini 3.8 Flash (`High`)** with targeted `cargo nextest` & `vitest`.
3. **Action 3 (Phase F3 — §0 Worktree Audit & Fedora 44 Packaging):** Execute §0 `wt/fixes` + `wt/release` (`sbom_npm.py`) + `wt/engines` audit and Fedora 44 Packaging B1/B2 (`lto = "thin"`, `.rpm`, `.deb`, `.AppImage`, offline smoke) with **Flash (`High`)** doing the heavy lifting and **Argon (`Low`/`Med`)** / **Opus (`High`)** performing surgical verifier sign-off, followed by Phase F4 (`v0.2.0-rc1` tag).

## Standing Rules
- **4-Tier Base Pro Routing:** Tier 1 `3.8 Flash (High)` (80% workhorse) · Tier 2 `Argon (Low)` (15% lead/reviewer/packaging) · Tier 3 `Argon (Medium)` (4% engine/escalation) · Tier 4 `Opus 5.5 (High)` (1% save/redaction/RC verifier).
- **Owner Rules:** `BUG-061` is P0 only if memory grows > ~1 GB in a normal 2h session, else P1 + `KI-009`. On 8GB laptop, use `lto = "thin"` + `codegen-units = 1` for release builds. Never push (`owner pushes`).

# CHECKPOINT — living handoff (< 150 lines, < 5,000 chars; history in CHECKPOINT_HISTORY.md)
Updated 2026-11-06 ~23:18 Asia/Dhaka by Tier 1 Workhorse Builder (Gemini 3.8 Flash High). Trust `STATE_AUTO.md` for git/process facts.

## STATE: PHASE F2 (AT-01..07 ISOLATED UNIT TESTS & P2 REGRESSIONS) COMPLETE
- **Current Host:** Fedora 44 KDE Plasma (Wayland, `rpm`/`dnf` native, WebKitGTK) · Intel Core i5-1135G7 · ~8 GiB RAM (7.65 GiB total, ~3.6 GiB available, 33 GiB disk free).
- **8GB RAM Guard Active:** `1` HEAVY slot max (`SLOT_FILES=(/tmp/lpdf-build.lock)`), `MemAvailable >= 2200 MB` guard, `CARGO_BUILD_JOBS=2`, `NEXTEST_TEST_THREADS=2`, `VITEST_MAX_WORKERS=2`, `NODE_OPTIONS="--max-old-space-size=2048"`.
- **Phase F2 Status (COMPLETE):**
  - AT-01 (BUG-001): `MATH_TEXT_LABS.note` updated with `(inserted text always starts with "∑ ")`; assertion added in `LabsGating.test.ts`.
  - AT-02 (BUG-006): Stale `labs.ts` comments replaced with `labs.svelte.ts` across `AnnotationLedger.svelte`, `DockableToolbar.svelte`, `state.svelte.ts` (0 hits remaining).
  - AT-03 (BUG-005): Added `aria-label={labsToolNote(tool.id)}` in `DockableToolbar.svelte`; screen reader assertion added in `LabsGating.test.ts`.
  - AT-04 (BUG-044): Tooltip text contrast test mirroring dialog added in `contrast.test.ts` (exceeds 4.5:1).
  - AT-05 (BUG-042): Added tests for Escape with another dialog open and Escape after unmount in `FaqModalKeys.test.ts`.
  - AT-06 (BUG-034): Added plain-field reset verification in `EngineReset.test.ts`. Finding recorded: `commands.journal` is an append-only trace not cleared by `activate()` (kept intact as DO-NOT-TOUCH).
  - AT-07 (BUG-034): Created `OrganizeNoDelete.test.ts` proving single-step and failed Organize applies invoke 0 deletion IPC calls.
  - Verification Gates: `cargo check` & `cargo nextest` (100% pass across `lpdf-control-bus`, `lpdf-fs-safety`, `lpdf-geom`), Vitest (5/5 suites, 56/56 passed), `svelte-check` (0 errors, 0 warnings), `studio-audit --strict` (0 findings).
- **Goal:** `v0.2.0-prototype` (`v0.2.0-rc1` tag target). **v0.2.0-rc1 is NOT built or tagged yet.**

## Branches (all local / remote tracking)
| Branch | Head | State |
|---|---|---|
| `main` | `160adf6` (+ Phase F2 commit) | VERIFIED through Phase F2 (AT-01..07 isolated unit tests, AT-10..15 UI strict audit, 8GB RAM recalibration, Base Pro governance). |
| `origin/wt/fixes` | `db28b40` | **WIP, unverified:** BUG-062 counted retry (`e2e/soak_retry.py`, `test_soak_retry.py`, `flow12_soak.py`, `run.py`) and BUG-061 close-tabs change. Soak diagnostic not run yet. |
| `origin/wt/release` | `53aebd9` | **WIP, unverified:** `master/tools/sbom_npm.py` (needs Argon Low `provenance-auditor` review before shipping) + partial `RAM_BUDGET.md` edit. |
| `origin/wt/engines` | `13b9bb3` | **WIP, unverified:** CP3 (BUG-029/030 streaming guard moved to `lpdf-engine-pdfcraft/src/guard.rs`, `path_exists` returns `Result`). |
| `origin/track/android` | `644caf3` | Paused (ADR-019 draft). |

## Next 3 Immediate Actions (Follow `master/docs/FLASH_AND_ARGON_EXECUTION_PLAN.md`)
1. **Action 1 (Phase F3.1 — Worktree Audit & Branch Prep):** Run §0 audit on `wt/fixes`, `wt/release` (`sbom_npm.py`), and `wt/engines` on **Gemini 3.8 Flash (`High`)** and route verifier sign-offs to **Argon (`Low`)**.
2. **Action 2 (Phase F3.2 — Fedora 44 Packaging B1/B2):** Package `.rpm`, `.deb`, and `.AppImage` with `lto = "thin"`, `codegen-units = 1`, and network-isolated smoke test.
3. **Action 3 (Phase F4 — Final Verification & Tagging):** Full `master/tools/verify.sh` + 12 Golden E2E flows + pdftotext redaction proof + fresh `rc-verifier` sign-off -> tag `v0.2.0-rc1` locally on `main`.

## Standing Rules
- **4-Tier Base Pro Routing:** Tier 1 `3.8 Flash (High)` (80% workhorse) · Tier 2 `Argon (Low)` (15% lead/reviewer/packaging) · Tier 3 `Argon (Medium)` (4% engine/escalation) · Tier 4 `Opus 5.5 (High)` (1% save/redaction/RC verifier).
- **Owner Rules:** `BUG-061` is P0 only if memory grows > ~1 GB in a normal 2h session, else P1 + `KI-009`. On 8GB laptop, use `lto = "thin"` + `codegen-units = 1` for release builds. Never push (`owner pushes`).

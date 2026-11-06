# CHECKPOINT — living handoff (< 150 lines, < 5,000 chars; history in CHECKPOINT_HISTORY.md)
Updated 2026-11-06 ~16:45 Asia/Dhaka by the lead (Sonnet 5.5) at the OWNER'S BUDGET STOP. Trust STATE_AUTO.md for git/process facts.

## STATE: STOPPED. No agents, builds or app instances running. Nothing pushed (the owner pushes).
- Goal: v0.2.0-prototype, tag by Sat 2026-11-07 23:59; rc1 target Sat 18:00. **v0.2.0-rc1 is NOT built or tagged.** Weekly was 78 % (≈1 %/h) at the last report. Stop new work at 92 %.
- The exact remaining steps, in order, with commands and acceptance checks: **master/docs/ARGON_TASKS.md section 0 (A to E)**. Low-risk items: sections 1-4 of the same file. Strong-review list: section 2.

## Branches (all local)
| Branch | Head | State |
|---|---|---|
| main | e1d2203 | VERIFIED code: everything through packaging prep A (tag packaging-prep-a-green); the E2E rerun, UI sweep and all fixes are merged. Commits after 7987e45 are docs and logs only. Untracked `LOCALPDF_MASTER_HANDOVER.md` in the root is not ours: untouched |
| wt/fixes | db28b40 | **WIP, unverified:** BUG-062 counted retry (`e2e/soak_retry.py`, `test_soak_retry.py`, edits to `flow12_soak.py`, `run.py`) and the BUG-061 close-tabs change. The soak diagnostic NEVER ran |
| wt/release | 53aebd9 | **WIP, unverified:** `master/tools/sbom_npm.py` (**UNREVIEWED: Claude must review before any SBOM ships**) + a partial `RAM_BUDGET.md` edit. LTO measurement was cancelled mid-compile. No packages built; `~/LocalPDF-rc1/` does not exist |
| wt/engines | 13b9bb3 | **WIP, unverified:** CP3 (BUG-029/030 streaming guard moved to `lpdf-engine-pdfcraft/src/guard.rs`, `path_exists` returns Result). Compiles; tests never run. Deletion listed in `.allowed-deletions` |
| track/android | 644caf3 | paused (ADR-019 draft) |

## E2E on main 83fd51c
2-11, 13 (needs `VITE_LPDF_GALLERY=1` build), 14-23 PASS. 1 BLOCKED until packages exist. 12 soak FAIL: BUG-061 (+13 MB/min, probably the loop never closing tabs; unproven) and BUG-062 (driver disconnect). Real-PDF rerun: BUG-063 closed as driver drift (stale labels); the text-coverage step stays a known P2 (BUG-038).

## Next action (first thing)
Claude reviews `tools/sbom_npm.py` and the three WIP branches, then follows ARGON_TASKS.md section 0: A (soak) → B (LTO measure + packages) → C (install tests, network-isolated run, docs) → D (verify.sh, 12 flows, fresh Opus-high verifier, tag v0.2.0-rc1) → **READY TO PUSH**.

## Standing rules
- Training RAM first (2 HEAVY slots, docs/RAM_BUDGET.md). Disk 30 GiB free; run.sh refuses < 15 GB. No container runtime: clean install = fresh HOME in `unshare -rn`.
- Owner rules: BUG-061 is a P0 only if memory would grow > ~1 GB in a normal 2 h session, else P1 + KNOWN_ISSUES. LTO fallback `lto="thin"` + `codegen-units=1` if fat LTO breaches the RAM cap. Save/redaction/settings changes get an Opus-high verifier.
- Docs: KNOWN_ISSUES.md KI-001..008 (KI-005 is stale). Bugs: logs/BUGS.md.

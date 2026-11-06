# CHECKPOINT — living handoff (< 150 lines, < 5,000 chars; history in CHECKPOINT_HISTORY.md)
Updated 2026-11-06 07:54 Asia/Dhaka by the lead. If this disagrees with STATE_AUTO.md, trust STATE_AUTO.

## Sprint
- **Goal:** v0.2.0-prototype (Linux), tag by Sat 2026-11-07 23:59. Scope: docs/PROTOTYPE_SCOPE.md. Feature freeze.
- **Milestones:** Fri 08:00 matrix/goldens/baseline/P0 list (done); Fri 18:00 adapters merged + ERR-007 + P0 flows green + DOGFOOD build; Sat 14:00 DOGFOOD triaged; Sat 18:00 rc1; Sat 23:59 tag.
- **Throttle tier: 85–92 % — IN EFFECT since 07:55** (owner 07:49: weekly 65 %, ≈7 %/h): lead + fixes builder only; P0s only; E2E flows 1, 2, 4, 6, 11; Opus only for the lead and the RC verifier; save-path/atomic-write/redaction changes keep the Opus-high verifier. ERR-007 (P0) goes to the fixes builder. Previous: **75–85 %** (owner, 07:40: weekly 64 %, +2 %/15 min). 2 builders (fixes + release); engines only on ERR-007 (Sonnet high); merge-verifier Opus medium; RC verifier Opus high. Running Opus-high spawns finish their current task; no Opus-high respawns. Pace: > 1.5 %/h since the last check → drop one tier.
- **Routing:** CLAUDE.md "Model routing". `.claude/agents/` is not loaded until a session restart, so roles are routed via Agent per-call model/effort.

## Worktrees
| Worktree | HEAD | State |
|---|---|---|
| main | 263bb7e (tags fixes-p0-green, engines-cp2b-green, release-cp2-green) | lead |
| wt/engines | 9919398 + CP3 uncommitted (BUG-029/030 + minors; compiles, tests NOT run) | PARKED (tier). NOTE: CP3 changes path_exists to Result<bool,String>; fixes must adapt when CP3 merges |
| wt/fixes | 263bb7e (merged) | idle; after restart: ERR-007 (BUG-015) on Sonnet high, then BUG-032/033 |
| wt/release | 6fa23a8 WIP (follow-ups 1,2,3,5,6; unverified) | PAUSED. After restart: verify + merge; then BUG-031 (P1, process-group reap + no-leftover check). KI-006 offline AppImage proof → Saturday packaging, before RC |
| android | 644caf3 | paused until Sun |

## P0 (BUGS.md)
- Merged 263bb7e: BUG-011..014, 016..022 fixed. Only BUG-015 (ERR-007) is open.
- **BUG-015 / ERR-007:** NOT fixed in the shipped build. Cadcraft is merged behind a switch (off); the switch patch is ready (wt/engines logs/runs/hatch-default-switch.patch). **Unblocked (owner 07:40):** licence-clean clip port + own parametric patterns, engines round after the restart.
- **P1 open:** 021, 022 (fixes, done in wt/fixes), 028 (fixes), 029, 030 (engines). BUG-010, 023, 024 fixed (merged).

## Next 3 actions
1. Owner restarts the session. Then: check the restore hook, spawn status-snapshot to prove routing is live, check and log the Haiku agents' actual effort.
2. ERR-007 on the fixes builder (Sonnet high), with a merge-verifier on Opus high because the output is saved geometry.
3. E2E flows 1, 2, 4, 6 and 11 on main, then the DOGFOOD build. Release WIP 6fa23a8: verify + merge, and BUG-031.

## Cuts (KNOWN_ISSUES.md)
KI-001 tab switch clears undo · KI-002 Organize intermediate file · KI-003 Labs · KI-004 split view → Labs · KI-005 no LTO, version 0.1.0 · KI-006 AppImage tools pinned, prefetch once · KI-007 no Close document.

## Open owner decisions
- **R3 licence wording:** no longer blocks ERR-007 (owner 07:40): a licence-clean clipping port + our own parametric patterns (engines, Sonnet high, after the restart). R3 is still open for the cadcraft-render default switch only.
- **AR-CONC hatch scale** ×1/16 (undoes upstream's coarsening): keep, or follow upstream?

## Standing rules / exceptions
- RAM: training first; HEAVY needs 5 GiB + 6 GiB reserve; 1 slot in practice. Disk: own target per worktree; refuse < 15 GB; ~32 GB free. **Owner 07:45: keep all target dirs (engines needed for ERR-007); delete only old release e2e/out runs by name, keeping the latest report, after the release builder finishes; revisit targets only below 22 GB free.**
- One-time rm -rf master/target/debug (ERR-008), approved. guard.py ALLOW_BUILD_LOCK=True (transitional).
- Unpushed: everything since origin/main 2fa8927 (the owner pushes).

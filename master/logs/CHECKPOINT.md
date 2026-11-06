# CHECKPOINT — living handoff (< 150 lines, < 5,000 chars; history in CHECKPOINT_HISTORY.md)
Updated 2026-11-06 07:36 Asia/Dhaka by the lead. If this disagrees with STATE_AUTO.md, trust STATE_AUTO.

## Sprint
- **Goal:** v0.2.0-prototype (Linux), tag by Sat 2026-11-07 23:59. Scope: docs/PROTOTYPE_SCOPE.md. Feature freeze.
- **Milestones:** Fri 08:00 matrix/goldens/baseline/P0 list (done); Fri 18:00 adapters merged + ERR-007 + P0 flows green + DOGFOOD build; Sat 14:00 DOGFOOD triaged; Sat 18:00 rc1; Sat 23:59 tag.
- **Throttle tier: 75–85 %** (owner, 07:40: weekly 64 %, +2 %/15 min). 2 builders (fixes + release); engines only on ERR-007 (Sonnet high); merge-verifier Opus medium; RC verifier Opus high. Running Opus-high spawns finish their current task; no Opus-high respawns. Pace: > 1.5 %/h since the last check → drop one tier.
- **Routing:** CLAUDE.md "Model routing". `.claude/agents/` is not loaded until a session restart, so roles are routed via Agent per-call model/effort.

## Worktrees
| Worktree | HEAD | State |
|---|---|---|
| main | ad484fd (tags engines-cp2b-green, release-cp2-green) | lead; routing files uncommitted, verify pending |
| wt/engines | 9919398 | RUNNING CP3: BUG-030 streaming guard, BUG-029 bypasses, path_exists→Result, minors (Opus/high, legacy spawn) |
| wt/fixes | 9919398 + round 4 staged (40 files) | RUNNING round 5 (FINAL tabs round): close re-stash blocker, undo/A4/reset majors + wire engines contract (Opus/high, legacy spawn) |
| wt/release | ad484fd | idle; follow-ups: pkg_smoke reap RuntimeError, flaky-only reason, cache names offline-proof, cargo --locked, 2 CHANGELOG lines |
| android | 644caf3 | paused until Sun |

## P0 (BUGS.md)
- **Fixed on wt/fixes, not merged** (round 4 verifier PASS; reviewer FAIL → round 5): BUG-011, 012, 013, 014, 016, 017, 019, 020.
- **BUG-018:** backend merged (F4 guard, degraded flag); UI wiring in fixes round 5.
- **BUG-015 / ERR-007:** NOT fixed in the shipped build. Cadcraft is merged behind a switch (off); the switch patch is ready (wt/engines logs/runs/hatch-default-switch.patch). **Unblocked (owner 07:40):** licence-clean clip port + own parametric patterns, engines round after the restart.
- **P1 open:** 021, 022 (fixes, done in wt/fixes), 028 (fixes), 029, 030 (engines). BUG-010, 023, 024 fixed (merged).

## Next 3 actions
1. Wait for fixes R5 (merge or single-doc fallback), engines BUG-029/030 and release follow-ups to finish; no new long rounds → "SAFE TO RESTART". After restart: restore check, status-snapshot proof, Haiku effort check.
2. Fixes round 5 → reviewer (Sonnet medium) + merge-verifier (Opus high; save path) → merge. If the review fails: single-document mode, and flow 3's two-document check is cut.
3. After restart: ERR-007 round (engines, Sonnet high: licence-clean clip + own patterns); full E2E on main; DOGFOOD build.

## Cuts (KNOWN_ISSUES.md)
KI-001 tab switch clears undo · KI-002 Organize intermediate file · KI-003 Labs · KI-004 split view → Labs · KI-005 no LTO, version 0.1.0 · KI-006 AppImage tools pinned, prefetch once · KI-007 no Close document.

## Open owner decisions
- **R3 licence wording:** no longer blocks ERR-007 (owner 07:40): a licence-clean clipping port + our own parametric patterns (engines, Sonnet high, after the restart). R3 is still open for the cadcraft-render default switch only.
- **AR-CONC hatch scale** ×1/16 (undoes upstream's coarsening): keep, or follow upstream?

## Standing rules / exceptions
- RAM: training first; HEAVY needs 5 GiB + 6 GiB reserve; 1 slot in practice. Disk: own target per worktree; refuse < 15 GB; ~32 GB free. **Owner 07:45: keep all target dirs (engines needed for ERR-007); delete only old release e2e/out runs by name, keeping the latest report, after the release builder finishes; revisit targets only below 22 GB free.**
- One-time rm -rf master/target/debug (ERR-008), approved. guard.py ALLOW_BUILD_LOCK=True (transitional).
- Unpushed: everything since origin/main 2fa8927 (the owner pushes).

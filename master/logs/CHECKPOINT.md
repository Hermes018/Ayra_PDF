# CHECKPOINT — living handoff (< 150 lines, < 5,000 chars; history in CHECKPOINT_HISTORY.md)
Updated 2026-11-06 16:12 Asia/Dhaka by the lead (Sonnet 5.5). If this disagrees with STATE_AUTO.md, trust STATE_AUTO.

## Sprint
- **Goal:** v0.2.0-prototype (Linux), tag by Sat 2026-11-07 23:59; **rc1 target Sat 18:00**. Feature freeze. Scope: docs/PROTOTYPE_SCOPE.md.
- **Owner plan (15:14):** 1) 12-flow rerun + real-PDF run, fix P0s test-first (save/redaction → Opus-high verifier); 2) packaging to **v0.2.0-rc1** (0.2.0, LTO, AppImage+rpm+deb, SHA256SUMS, clean install, network-isolated run, About/licences, README/QUICKSTART/KNOWN_ISSUES/CHANGELOG; RC verifier Opus high; tag) → tell owner **READY TO PUSH**; 3) P1s touching save/redaction/files/engines/harness; 4) ARGON_TASKS.md (done, "READY TO PUSH" given at 77b50dd). Stop new work at weekly 92 %.
- **Budget (owner 16:2x): weekly 78 %, ≈1 %/h, within budget.** Burn > 1 %/h → only (a) E2E/P0, (b) owner findings, (c) packaging, (d) RC. One builder at a time; one combined review+verify pass except RC and save/redaction (Opus high). Don't poll the soak. Lead is Sonnet 5.5; `.claude/agents` roles routed per call.
- **Owner decisions:** BUG-061 (soak memory): if it still grows with tabs closed → P0 only if > ~1 GB in a normal 2 h session, else P1 + KNOWN_ISSUES, never an RC blocker. BUG-062 counted-retry policy approved. LTO: measure the peak first; if fat LTO breaches the RAM budget use lto="thin" + codegen-units=1 and record it. Clean install = fresh HOME in a no-network sandbox (unshare/bwrap), method recorded in the RC notes.

## Worktrees
| Worktree | State |
|---|---|
| main (7987e45) | all code merged through the UI sweep (tag ui-sweep-complete) + ARGON_TASKS. Untracked LOCALPDF_MASTER_HANDOVER.md in the repo root is NOT ours: left alone |
| wt/fixes (warm target) | idle; packaging prep A merged (tag packaging-prep-a-green) |
| wt/release | RUNNING (sonnet/med): packaging B1 — measure LTO peak, build AppImage/deb/rpm → ~/LocalPDF-rc1/, SHA256SUMS, SBOM, KI-006 offline proof (no app instance) |
| wt/engines | parked (CP3 uncommitted, target deleted); android paused |

## E2E on main 83fd51c (golden run 20261106T144532)
- PASS: 2–11, 14–23, 13 (needs VITE_LPDF_GALLERY=1 build, binary e2e/out/bin/localpdf-studio-gallery). BLOCKED: 1 (no rpm until packaging). FAIL: 12 soak.
- **BUG-061** soak memory +13 MB/min (745→1,460 MB, 1.9×; FDs flat): diagnostic queued (close tabs per iteration, short soaks, idle check). **BUG-062** driver disconnect: implement counted retry (release).
- **BUG-063 closed — driver drift** (stale post-sweep labels in the dogfood driver; app fine: markup 8/8, redaction DF-04, Edit Text DF-01 PASS with current selectors; step 3 text coverage = known P2 BUG-038).

## Next actions
1. BUG-062 retry + BUG-061 diagnostic (release job below).
2. Then the full 1 h soak on the RC package (in B2).
3. Packaging: B1 building (above); then B2 = no-network clean-install + network-isolated run + docs (README, QUICKSTART, CHANGELOG, KNOWN_ISSUES) → RC verifier (Opus high) → tag v0.2.0-rc1 → READY TO PUSH.

## Standing rules
- RAM: training first; 2 HEAVY slots (docs/RAM_BUDGET.md). Disk 38.5 GiB free; run.sh refuses < 15 GB; delete only regenerable targets of idle worktrees.
- Unpushed: everything since origin/main 2fa8927 (the owner pushes). Known issues: KNOWN_ISSUES.md KI-001..008. Bugs: logs/BUGS.md.

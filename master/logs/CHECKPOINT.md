# CHECKPOINT — living handoff (< 150 lines, < 5,000 chars; history in CHECKPOINT_HISTORY.md)
Updated 2026-11-06 16:04 Asia/Dhaka by the lead. If this disagrees with STATE_AUTO.md, trust STATE_AUTO.

## Sprint
- **Goal:** v0.2.0-prototype (Linux), tag by Sat 2026-11-07 23:59. Scope: docs/PROTOTYPE_SCOPE.md. Feature freeze.
- **Milestones:** Fri 08:00 matrix/goldens/baseline/P0 list (done); Fri 18:00 adapters merged + ERR-007 + P0 flows green + DOGFOOD build; Sat 14:00 DOGFOOD triaged; Sat 18:00 rc1; Sat 23:59 tag.
- **Throttle tier: 85–92 % — IN EFFECT since 07:55** (owner 07:49: weekly 65 %, ≈7 %/h): lead + fixes builder only; P0s only; E2E flows 1, 2, 4, 6, 11; Opus only for the lead and the RC verifier; save-path/atomic-write/redaction changes keep the Opus-high verifier. ERR-007 (P0) goes to the fixes builder.
- **Plan from 15:14 (owner; lead now Sonnet 5.5; Argon, a weaker AI, takes minor work after this session):** 1) finish the 12-flow rerun + real-PDF run, fix P0s test-first (save/redaction → Opus-high verifier); 2) packaging to **v0.2.0-rc1** (0.2.0, LTO, AppImage+rpm+deb, SHA256SUMS, clean-container install, network-isolated run, About/licences, docs; RC verifier Opus high; tag) → tell owner **READY TO PUSH**; 3) P1s touching save/redaction/files/engines/harness; 4) master/docs/ARGON_TASKS.md → **READY TO PUSH**. Stop new work at weekly 92 %. Weekly 77 %, burn > 1 %/h → only (a)–(d), packaging+RC never paused. One builder at a time; one combined review+verify pass except RC and save/redaction (Opus high). Don't poll the soak.
- **Owner decisions 16:04 (weekly 78 %, ≈1 %/h, within budget):** BUG-061: if memory still grows with tabs closed → P0 only if > ~1 GB in a normal 2 h session, else P1 + KNOWN_ISSUES, never an RC blocker. BUG-062 counted-retry policy approved. LTO: measure the peak first; if fat LTO breaches the RAM budget use lto="thin" + codegen-units=1 and record it. Clean install = fresh HOME in a no-network sandbox (unshare/bwrap), method recorded in the RC notes. Triage of real-PDF regressions (BUG-063) running.
- **Routing:** CLAUDE.md "Model routing". `.claude/agents/` is not loaded until a session restart, so roles are routed via Agent per-call model/effort.

## Worktrees
| Worktree | HEAD | State |
|---|---|---|
| main | 77b50dd (docs: ARGON_TASKS.md) | E2E rerun DONE: 2-11 + 14-23 PASS; 1 BLOCKED (no rpm); 12 soak FAIL (1 driver disconnect @123/408, RSS 1.9x); 13 needs VITE_LPDF_GALLERY build (rerunning). Real-PDF rerun running |
| wt/engines | 9919398 + CP3 uncommitted | PARKED; target dir deleted (owner 08:50) — rebuild needed before CP3 tests |
| wt/fixes | 83fd51c (merged) | idle (sweep done); next: owner DOGFOOD findings, then Sat D packaging support |
| wt/release | daa2bb4 (merged) | idle; target dir deleted 15:5x (disk 21.9→38.5 GiB). Packaging builds from main |
| android | 644caf3 | paused until Sun |

## P0 (BUGS.md)
- All P0s fixed on main (52b7a17). BUG-015/ERR-007 fixed with our own clipping + patterns.

## Next 3 actions
1. E2E flows 1, 2, 4, 6, 11 on main (running).
2. DOGFOOD build (AppImage) for the owner.
3. Pace check at ~09:00 (owner %); then P1s, if the tier allows.

## UI consistency workstream (owner 2026-11-06 09:28; feature freeze holds)
A inventory (done: 12 screens, no shared ui components, E2E only Organize) + A2 audit ext. (fixes, running) → B docs/DESIGN_LANGUAGE.md (Opus medium, running) → **C0 shared component library first (owner): headless Svelte 5 lib (Bits UI/Melt UI) vetted MIT/Apache + offline, our tokens; 10 base components + unit tests; dev-only gallery route (excluded from release) + E2E screenshot; decision in reuse matrix; checkpoint commit before the dependency** → C sweep main-flow screens (+ per-screen E2E smoke; ErrorState on every long-job/file-I/O screen) (Sonnet medium, strict audit, screenshots) by 17:00 → 17–18 E2E + screenshots → Sat AM D packaging (0.2.0, LTO, rpm/deb/AppImage, SBOM, offline run) → RC 18:00. Owner dogfood findings beat sweep work. If weekly > 80 % before Sat noon: stop sweep at main-flow screens, finish D, rest → UI_BACKLOG.md.
- Weekly 66 % at 09:28 (≈0.7 %/h since 07:47): tier stays 85–92 %; owner un-paused release builder. DOGFOOD copy: ~/LocalPDF-dogfood/ (+SHA256SUMS).

## Cuts
See KNOWN_ISSUES.md (KI-001..008).

## Open owner decisions
- **R3 licence wording:** no longer blocks ERR-007 (owner 07:40): a licence-clean clipping port + our own parametric patterns (engines, Sonnet high, after the restart). R3 is still open for the cadcraft-render default switch only.
- **AR-CONC hatch scale** ×1/16 (undoes upstream's coarsening): keep, or follow upstream?

## Standing rules / exceptions
- RAM: training first; **2 HEAVY slots** (owner 08:05; peak 4 GiB, slot 2 needs ≥ 14,336 MiB available; E2E exclusive until measured; docs/RAM_BUDGET.md). Disk: own target per worktree; refuse < 15 GB; ~32 GB free. **Owner 07:45: keep all target dirs (engines needed for ERR-007); delete only old release e2e/out runs by name, keeping the latest report, after the release builder finishes; revisit targets only below 22 GB free.**
- One-time rm -rf master/target/debug (ERR-008), approved. guard.py ALLOW_BUILD_LOCK=True (transitional).
- Unpushed: everything since origin/main 2fa8927 (the owner pushes).

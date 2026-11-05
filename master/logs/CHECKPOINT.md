# CHECKPOINT — living handoff (overwrite; keep < 150 lines; history in CHECKPOINT_HISTORY.md)
Updated: 2026-11-06 00:45 Asia/Dhaka by the lead. If this disagrees with logs/STATE_AUTO.md (git/process facts), trust STATE_AUTO.md and fix this file.

## Sprint
- **Goal:** fully working Linux prototype **v0.2.0-prototype**, tag by **Sat 2026-11-07 23:59 Asia/Dhaka**. Scope and 12 E2E flows: `docs/PROTOTYPE_SCOPE.md`. Feature freeze; Labs holds the partial features.
- **Engine strategy:** ADR-018 rev 3 APPROVED (craft crates pinned and vendored behind adapters; strangler with goldens and parity; never ship a half swap; ERR-007 must ship fixed).
- **Android:** PAUSED until Sun 2026-11-08 (ADR-019 draft on track/android).
- **Milestones:**

  | When | Target | Status |
  |---|---|---|
  | **Fri 08:00** | Reuse matrix, goldens, E2E baseline, P0 list | **On track.** Matrix, goldens and baseline exist in the worktrees (not merged yet); P0 list done (below) |
  | **Fri 18:00** | Adapters merged, ERR-007 fixed, P0 flows green, build for owner DOGFOOD | At risk: 10 P0s open |
  | Sat 14:00 | DOGFOOD triaged, P0s fixed | — |
  | Sat 18:00 | rc1: verify green, 12/12 flows, packages smoke-tested | — |
  | Sat 23:59 | Tag v0.2.0-prototype | — |

## Worktrees (all builders currently IDLE)

| Worktree | Branch | Last commit | Uncommitted | Current task | State / owner |
|---|---|---|---|---|---|
| ~/Desktop/Ayra_PDF (main) | main | 16d9913 (logs) on 6c18734 Labs merge | build-lock per-worktree target, run.sh disk guard, sccache 10G, logs, snapshot/restore scripts | lead: verify.sh running, then commit | lead |
| ~/Desktop/Ayra_PDF-wt/engines | wt/engines | 562e4f9 (needs main merged in) | lpdf-geom crate (legacy hatch adapter), ENGINE_REUSE_MATRIX.md (47 rows), goldens, studio.rs uses adapter | checkpoint 1 DONE; awaiting lead merge (main → worktree, then reviewer + verifier) | idle / engines builder |
| ~/Desktop/Ayra_PDF-wt/fixes | wt/fixes | 6c18734 = merged into main | none | checkpoint 1 merged (Labs). Next: P0 list | idle / fixes builder |
| ~/Desktop/Ayra_PDF-wt/release | wt/release | 562e4f9 (needs main merged in) | e2e/ harness (12 flows), corpus fetch.sh + scanned/encrypted | checkpoint 1 DONE (baseline 4 PASS / 7 FAIL / 1 BLOCKED); awaiting review + verify + merge | idle / release builder |
| ~/Desktop/Ayra_PDF-wt/android | track/android | 644caf3 ADR-019 DRAFT | none | paused | paused |

**Running subagents:** none (all builders idle). A claude-code-guide docs check for hook names is in flight. Background: main `verify.sh` after the target split (rebuilding from scratch into its own target).

## P0 (from logs/BUGS.md) — owner — status

| Bug | Flow | Issue | Owner | Status |
|---|---|---|---|---|
| BUG-011 | 2 | Encrypted PDF: no password prompt | fixes | open |
| BUG-012 | 3 | CSP blocks data:/blob: images (blank thumbnails); fix `img-src 'self' data: blob:` (lead-approved) | fixes | open |
| BUG-013 | 9 | Edit Text hotspot unreachable (#wet-layer covers it) | fixes | open |
| BUG-014 | 10 | Organize Apply drops rotation, reports success | fixes | open |
| BUG-015 | 5 | ERR-007 hatch outside polygon / patterns not distinct (cadcraft via lpdf-geom) | engines | open |
| BUG-016 | 3 | Second document never shown (no tabs/split) | fixes | open |
| BUG-017 | 7 | Drawn signature lost, text stamped instead | fixes | open |
| BUG-018 | 2 | Hostile/malformed PDFs open silently, no error or degraded notice | fixes (+engines if backend) | open |
| BUG-019 | 4 | Erasers not in undo; Undo removes another stroke | fixes | open |
| BUG-020 | 3 | Fit Width/Page assume A4 and ignore the page offset | fixes | open |

**P1:**

| Bug | Issue | Owner |
|---|---|---|
| BUG-021 | Repeated saves overwrite `<orig>.annotated.pdf` | fixes |
| BUG-022 | Perimeter wrong (216 vs 288) | fixes |
| BUG-023 | detect_tables fabricates an "auto" field | engines |
| BUG-024 | Hatch apply error swallowed | engines |
| BUG-010 | toolEngine still labels insert.latex "LaTeX Math Block" | engines |

P2s: BUG-001..009, 025..027 (after the release).

**File ownership for the P0 phase:** fixes = UI components/views plus core/{inkEngine,insertEngine,viewportEngine,measureEngine}.svelte.ts and tauri.conf.json CSP. engines = crates/, src-tauri Rust, HatchDefs, new adapter files; it must not touch those four core TS files until fixes merges.

## Next 3 actions (in order)
1. **main:** if verify.sh is GREEN, commit (build-lock split, disk guard, sccache 10G, logs, snapshot/restore hooks). Then merge main into wt/engines and wt/release (stash, fast-forward, pop) and add tree-map entries for their new files.
2. **Reviews:** spawn reviewer + verifier (Opus, high effort) for engines checkpoint 1 and release checkpoint 1. Merge each on PASS plus a green verify.
3. **Next briefs:**
   - fixes builder: P0 bugs BUG-011, 012, 013, 014, 016, 017, 018, 019, 020 (test first, plan file for changes touching >3 files).
   - engines checkpoint 2: ERR-007 via cadcraft (vendor, patch, manifest, notices, parity) plus BUG-023, 024, 010.
   - release checkpoint 2: packaging (rpm + AppImage, deb if cheap) once its checkpoint 1 merges.

## Open decisions for the owner
- None blocking. FYI: the lead approved the CSP `img-src 'self' data: blob:` change (BUG-012) and E2E harness isolation (private D-Bus/XDG), both in DECISIONS_ADR.

## Approved exceptions and standing rules
- **2026-11-06 one-time `rm -rf master/target/debug`** (54 GB stale), an exception to "never cargo clean" (ERR-008).
- **Disk:** each worktree has its own target dir, never shared. sccache local only, 10 GB cap. run.sh refuses builds below 15 GB free. Delete a worktree's target when the worktree is removed. If tight, drop to 2 builders; never delete caches mid-sprint. Report free disk every milestone (now: ~78 GB free before rebuilds).
- **Guard transitional:** direct build-lock.sh calls are still allowed until all builders are re-briefed to run.sh; then set ALLOW_BUILD_LOCK=False in tools/hooks/claude/guard.py.

## Pointers
- Rules: /CLAUDE.md. Bugs: logs/BUGS.md. Per-worktree state: logs/checkpoints/<worktree>.md. Facts at last compact: logs/STATE_AUTO.md.
- Last green tags: phase-r4-green (infra), fixes-cp1-green (Labs). Release tag: v0.1.0-rc4. Unpushed: everything since origin/main 2fa8927 (the owner pushes).
- Pinned pdfcraft 75360d1 (v0.15.0); ADRs 001-018 and 020 accepted; 019 draft.

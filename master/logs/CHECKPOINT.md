# CHECKPOINT — living handoff (overwrite; keep < 150 lines; history in CHECKPOINT_HISTORY.md)
Updated: 2026-11-06 07:05 Asia/Dhaka by the lead. If this disagrees with logs/STATE_AUTO.md (git/process facts), trust STATE_AUTO.md and fix this file.

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

## Worktrees

| Worktree | Branch | Last commit | Uncommitted | Current task | State / owner |
|---|---|---|---|---|---|
| ~/Desktop/Ayra_PDF (main) | main | 7dc56c8 engines CP1 (tag engines-cp1-green) | logs | lead: coordinating merges; Fri 08:00 report | lead |
| ~/Desktop/Ayra_PDF-wt/engines | wt/engines | 7dc56c8 (merged) | none | **RUNNING** CP2b: parity hardening, THIRD_PARTY_NOTICES, doc fixes, default-switch diff prepared but not flipped, BUG-023/024/010/018 backend, path_exists IPC, delete organizing-intermediate | engines builder |
| ~/Desktop/Ayra_PDF-wt/fixes | wt/fixes | ed3b5a2 | P0 round 2 staged (33 files); builder verify GREEN, vitest 204 | fresh reviewer + verifier RUNNING | idle / fixes builder |
| ~/Desktop/Ayra_PDF-wt/release | wt/release | 81ae676 | checkpoint 1 staged; verifier PASS, reviewer FAIL (4 major harness issues) | **RUNNING**: harness fixes, then checkpoint 2 packaging | release builder |
| ~/Desktop/Ayra_PDF-wt/android | track/android | 644caf3 ADR-019 DRAFT | none | paused | paused |

**Running subagents:** 4 review agents (fixes reviewer+verifier, engines reviewer+verifier); release builder (harness fixes). The fixes and engines builders are idle awaiting merge.

## P0 (from logs/BUGS.md) — owner — status

| Bug | Flow | Issue | Owner | Status |
|---|---|---|---|---|
| BUG-011 | 2 | Encrypted PDF: no password prompt | fixes | fixed in wt/fixes (in review) |
| BUG-012 | 3 | CSP blocks data:/blob: images (blank thumbnails); fix `img-src 'self' data: blob:` (lead-approved) | fixes | fixed in wt/fixes (in review) |
| BUG-013 | 9 | Edit Text hotspot unreachable (#wet-layer covers it) | fixes | fixed in wt/fixes (in review) |
| BUG-014 | 10 | Organize Apply drops rotation, reports success | fixes | fixed in wt/fixes (in review) |
| BUG-015 | 5 | ERR-007 hatch outside polygon / patterns not distinct (cadcraft via lpdf-geom) | engines | cadcraft impl ready behind switch (in review); default switch pending |
| BUG-016 | 3 | Second document never shown (no tabs/split) | fixes | fixed in wt/fixes (in review) |
| BUG-017 | 7 | Drawn signature lost, text stamped instead | fixes | fixed in wt/fixes (in review) |
| BUG-018 | 2 | Hostile/malformed PDFs open silently, no error or degraded notice | fixes (+engines if backend) | fixed in wt/fixes (in review) |
| BUG-019 | 4 | Erasers not in undo; Undo removes another stroke | fixes | fixed in wt/fixes (in review) |
| BUG-020 | 3 | Fit Width/Page assume A4 and ignore the page offset | fixes | fixed in wt/fixes (in review) |

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
1. On engines and release reviewer + verifier PASS: commit on each branch, fast-forward or merge into main (resolve descriptions.tsv / tree.md conflicts by regenerating), tag, log.
2. Then brief engines checkpoint 2 (ERR-007 via cadcraft + vendoring/manifest/notices + BUG-023/024/010) and release checkpoint 2 (packaging + re-run E2E after P0 merges). Set guard ALLOW_BUILD_LOCK=False once all builders are on run.sh.
3. Merge fixes P0 batches as they arrive (reviewer + verifier each). Fri 08:00 report: flows, engines, P0/P1, cuts, disk, unpushed commits/tags.

## Open decisions for the owner
- **ADR-018 R3 licence wording:** R3 says "MIT/Apache only", but cadcraft-render brings BSD-3 (tiny-skia), BSD-2 (arrayref) and Unicode-3.0. Existing deps already include BSD-3 (winprint, ADR-017) and Unicode-3.0 (unicode-ident). Recommendation: amend R3 to "permissive licences on the deny.toml allow-list, with notices in THIRD_PARTY_NOTICES.md". Until you decide, cadcraft stays behind the switch (default Legacy).
- Owner action needed: run `/compact` once to live-test the PreCompact/SessionStart hooks (both pipe-tested; I can't run /compact). Otherwise none blocking. FYI: the lead approved the CSP `img-src 'self' data: blob:` change (BUG-012) and E2E harness isolation (private D-Bus/XDG), both in DECISIONS_ADR.

## Approved exceptions and standing rules
- **RAM (2026-11-06):** training first. Scheduler and cgroup caps per docs/RAM_BUDGET.md. HEAVY peak 5 GiB + 6 GiB reserve; 2nd slot only at ≥ 16 GiB available.
- **2026-11-06 one-time `rm -rf master/target/debug`** (54 GB stale), an exception to "never cargo clean" (ERR-008).
- **Disk:** each worktree has its own target dir, never shared. sccache local only, 10 GB cap. run.sh refuses builds below 15 GB free. Delete a worktree's target when the worktree is removed. If tight, drop to 2 builders; never delete caches mid-sprint. Report free disk every milestone (now: ~78 GB free before rebuilds).
- **Guard transitional:** direct build-lock.sh calls are still allowed until all builders are re-briefed to run.sh; then set ALLOW_BUILD_LOCK=False in tools/hooks/claude/guard.py.

## Pointers
- Rules: /CLAUDE.md. Bugs: logs/BUGS.md. Per-worktree state: logs/checkpoints/<worktree>.md. Facts at last compact: logs/STATE_AUTO.md.
- Last green tags: phase-r4-green (infra), fixes-cp1-green (Labs). Release tag: v0.1.0-rc4. Unpushed: everything since origin/main 2fa8927 (the owner pushes).
- Pinned pdfcraft 75360d1 (v0.15.0); ADRs 001-018 and 020 accepted; 019 draft.

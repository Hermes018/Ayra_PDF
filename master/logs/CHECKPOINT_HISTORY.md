
## Snapshot archived 2026-11-06T05:03:00+06:00
# CHECKPOINT (OVERWRITE this file; single source of "where am I")
- Date: 2026-11-05 (dev machine 2: Zorin OS 18.1 / Ubuntu 24.04)
- Phase / sub-phase: SPRINT to v0.2.0-prototype (Linux), deadline Sat 2026-11-07 23:59 Asia/Dhaka (docs/PROTOTYPE_SCOPE.md). ADR-018 rev 3 APPROVED. Android PAUSED until Sunday. Feature freeze.
- Layout: shared workspace = `master/` (run cargo/pnpm/xtask here). OS packaging in `ubuntu/ fedora/ windows/` (`packaging/tauri.<os>.conf.json` overlays + build scripts); `macos/ ios/ android/` not started. Map: `/tree.md` (regenerate with `master/tools/tree/render_tree.py`).
- Status: (1) 7-folder layout in place; every tracked file mapped and described; (2) forbid(unsafe) regression fixed (ERR-002, ADR-015); (3) dash-portable mock sidecar tests (ERR-003); (4) DRAWBOARD checklist corrected to 111 DONE / 2 IN PROGRESS (2.16 LaTeX and 8.10 handwriting are placeholders); (5) ADR ID collisions resolved by registry erratum (ADR-013/014); (6) Ubuntu .deb overlay + build script added (unbuilt: needs `sudo apt` webkit2gtk dev packages); (7) CI runs from master/, adds native Ubuntu gate job, release builds rpm+deb+nsis/msi via overlays.
- Verified on this machine (2026-11-05): cargo fmt --check; clippy -D warnings (whole workspace incl. app-tauri); nextest 154/155 (ERR-005); svelte-check 0/0; vitest 127/127; vite build; protocol-lint / feature-map-check / studio-audit --strict; render_tree --check; app launches via `pnpm tauri dev`. RED: cargo deny + cargo audit (ERR-006). NOT verified: .deb/.rpm bundling, Windows.
- PAUSED TRACK: Android (ADR-019 draft on track/android), resumes Sun 2026-11-08
- Last green tag: phase-r3-green (main)
- In progress: Milestone Fri 08:00 = ENGINE_REUSE_MATRIX + characterization fixtures (wt/engines), E2E baseline (wt/release), Labs gating + wording (wt/fixes), P0 list.
- Open risks: (a) PdfcraftAdapter::render stub; (b) jobs in-process, sandboxed worker unused (F3); (c) ERR-007 saved hatch ignores polygon; (d) stock-Ubuntu AppArmor userns; (e) lpdf-tools-* unused; (f) RC1 QA never run; (g) winprint patch must be re-synced on craft-print bumps (ADR-017); (h) ADR-018 PROPOSED, awaiting user. (i) ERR-008 fixed 2026-11-06: per-worktree target dirs; one-time approved rm -rf master/target/debug (exception to never-cargo-clean).
- Next 3 actions: 1) Builders: engines (matrix, goldens, lpdf-geom adapter + ERR-007), release (E2E harness + 12-flow baseline), fixes (Labs + wording) 2) Lead: triage baseline into P0/P1 in logs/BUGS.md 3) Fri 08:00 report
- Active ADRs: 001-018 (018 rev 3 APPROVED), 020; 019 DRAFT (paused)
- Pinned pdfcraft commit: 75360d1d7db06fed974f48c914d4c04132757187 (v0.15.0)
- Disk rules (owner, 2026-11-06): each worktree has its own target dir, never shared. sccache is local only, 10 GB cap. run.sh refuses builds below 15 GB free. When a worktree's branch is merged and the worktree removed, delete its target dir too. If disk gets tight, drop to 2 builders; never delete caches mid-sprint. Every milestone report states free disk space.

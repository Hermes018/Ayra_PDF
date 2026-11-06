# checkpoints/release.md — release builder (wt/release). Builder keeps this current; < 150 lines.
- **Round-3 follow-ups (lead):** done+light-tested: pkg_smoke reap error->FAIL w/ summary, 'flaky retry' reason, malformed report/unknown --kinds -> FAIL; build-appimage cargo --locked --offline; CHANGELOG (not E2E-tested) x2. Offline AppImage proof (item 4) STILL RUNNING (compile phase, ~8h, not yet at bundling): run.sh job appimage-offline-proof, log master/logs/runs/appimage-offline-proof.log. Check the log for 'download'/'Network' errors vs a produced .AppImage; KI-006 stays UNPROVEN until then. verify.sh skipped (no checked Rust/web files touched).
- **BUG-031 (2026-11-06) DONE:** common.py PrivateBus own session + killpg, harness_procs/kill_harness (env LPDF_E2E_RUN=<marker> or /tmp/lpdf-bus-* cmdline), Context.post_run_check; run.py sets marker, setsid (non-tty), SIGTERM/INT/HUP -> SystemExit, post-run leak => LEAK + exit 1. Test master/e2e/test_leak_check.py (7 PASS; --leak-only shows detection). verify.sh GREEN. Next: Part 2 real-PDF dogfood in ~/lpdf-dogfood-run (outside repo).
- **Task:** release-review round 2 (4 majors + 5 minors, lead 2026-11-06) DONE in code, each tested
  light; `verify.sh` running (LPDF_PRIO=other). No full E2E re-run (lead schedules it after P0 merges).
- **Round-2 fixes:** pkg_smoke verdict needs run.py rc in {0,1} + flows 1 AND 2 in the report (+ consistency),
  timeout/abort = FAIL; reap by env marker LPDF_E2E_PACKAGE=<kind>:<file>:<pid> (catches tauri-driver's own
  session); build-appimage.sh pins linuxdeploy/plugin/AppRun to immutable URLs + SHA-256 (upstream + Tauri's
  magic-byte-patched hash), `--prefetch` / `--check-tools`, offline frozen pnpm install, every heavy step via
  tools/run.sh (run the script directly, not under run.sh); CHANGELOG (pending BUG-NNN)/(not E2E-tested)/(flow N)
  markers; flow 1 "no log files" = WARN; run.py prints a FLAKY.md-format list and exits non-zero on any retried
  launch; reports stamped (run_id, mode, binary, times) + per-run copy in <out>/runs/<run_id>/; old unstamped
  reports archived as runs/legacy-*; fetch.sh trap after cd with absolute paths.
- **Last E2E (offline, rpm release binary, 07:11):** 3 PASS (6, 8, 11) / SHORT-PASS 12 / 7 FAIL / BLOCKED 1;
  failures = open P0s only. Packages: rpm/deb/AppImage, LTO off, version 0.1.0 (lead bumps at rc1).
- **Pinned-tool cache seeded:** ~/.cache/lpdf/appimage-tools (next AppImage build is offline; first build with
  plugin 1-alpha-20250213-1 instead of continuous@536b068 — untested in a build yet).
- **Files touched (uncommitted):** corpus/{fetch.sh,MANIFEST,SHA256SUMS}, e2e/**, ubuntu/packaging/
  {build-appimage.sh,tauri.appimage.conf.json}, CHANGELOG.md, logs/checkpoints/release.md.
- **Next:** verify.sh result → report. Then: full E2E after P0 merges (lead); rc1 packages with full LTO.
- **Blockers:** real dnf/apt install needs sudo; no native-dialog seam; no "close document" UI.
- **BUG-046 (P1) DONE:** new e2e/ink_match.py (drawn-signature matcher), test_flow07_matcher.py (9 PASS), flow07 wired; flow 7 PASS (run e2e-f7). verify.sh run.

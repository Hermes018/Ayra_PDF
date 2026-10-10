# 03 — FAILSAFES & REDUNDANCY (non-negotiable; each needs an automated test)

| ID | Failsafe | Mechanism | Test |
|---|---|---|---|
| F1 | Atomic output | Write temp in SAME filesystem + dir as target -> fsync file -> rename -> fsync dir | kill -9 at random points x1000: target is old-or-new, never partial |
| F2 | Operation journal (WAL-lite) | `jobs/<id>.json` written before work: inputs (path+size+mtime+hash), op, options, temp/out paths, state. Startup reconcile: finish / roll back / GC temps | crash mid-job then relaunch: state consistent, UI shows recovery notice |
| F3 | Crash isolation | Untrusted parsing/render/OCR/convert in `localpdf-worker` processes; supervisor restarts; failed job reports code, UI survives | feed fuzz crashers: UI alive, job fails cleanly |
| F4 | Resource guards | Decompression-bomb ratio cap, object count/depth cap, per-page render watchdog (default 20 s), memory budget by tier, cgroup-tested; cancel tokens everywhere | bomb PDFs, 100k-object trees, 10k-page files under `systemd-run -p MemoryMax=` |
| F5 | Disk preflight | Require free space >= est. output x 1.2 + temp; check dest writable; clear message otherwise | tiny tmpfs, read-only dest tests |
| F6 | Verify-after-write | Re-open output via adapter (page count, structure), compare untouched-page hashes where applicable; fail => delete output, keep input, report | corrupt-writer injection |
| F7 | Engine redundancy | Primary adapter (pdfcraft) fails to open => FallbackAdapter (lopdf repair/rewrite) => "degraded mode" notice | corpus of broken files |
| F8 | Backups | Overwrite mode copies original to app backup dir first (retain N=10 / 14 days, user-configurable) | overwrite then restore test |
| F9 | Offline guarantee | CSP no external connect; cargo-deny bans network crates in core/app/ai-worker; CI runs full e2e under `unshare -n` including AI worker and HTML→PDF; Flatpak without network | e2e green offline; grep-lint |
| F10 | Edit-session autosave | Every 60 s autosave to app dir (encrypted docs stay encrypted); recover prompt after crash | kill during edit |
| F11 | UI data-loss guard | Unsaved dot, confirm on close/quit, undo/redo history | e2e |
| F12 | Never-grow compression | If compressed >= original, return original + "already optimized" | corpus test |
| F13 | Panic policy | `catch_unwind` at job boundary in worker; supervisor handles aborts; no panic reaches IPC | injected panic test |
| F14 | Stop-the-line | Gate red => stop; 3 failed distinct fixes => ERROR_LOG + ask user | process audit in logs |
| F15 | Rollback | Tag per green sub-phase; revert with `git revert`; keep `lock` files committed | n/a |
| F16 | Supply chain | `cargo deny` (licences/advisories/bans), `cargo audit`, pinned lockfiles, SBOM per release, checksums + signatures | CI |
| F17 | Path safety | Canonicalize; reject device files/FIFOs; symlink policy (follow only user-chosen file, never traverse out of chosen dir for batch); Windows reserved names | unit tests |
| F18 | Secrets hygiene | Passwords in `secrecy`/`zeroize` types; never logged/serialized/persisted | log-scan test |
| F19 | AI worker isolation | `lpdf-ai-worker`: same sandbox as worker (Landlock/Job Object), no network, memory cap = tier budget, per-chunk watchdog, model file SHA-256 checked at load, output treated as plain text only | model hash mismatch refuses load; inference OOM/timeout leaves UI alive; run under `unshare -n` |
| F20 | External-tool bridge safety | LibreOffice/Tesseract/scanimage run as child processes: no network (unshare/Job Object), per-job temp dir, timeout, killed on cancel, stdout/stderr size-capped, args never built from unescaped strings | hang + huge-output + path-injection tests |
| F21 | Sidecar Execution Failure | If a bundled binary fails to execute (e.g., OS permission denied, missing binary for architecture, or crash), the Rust backend must intercept the panic, terminate the zombie child process, and return a clean `ExternalDependencyError` to the Svelte frontend rather than crashing the Tauri host. | Injected permission denial; missing binary test |
| F22 | Cross-platform binary mapping | Verify that `-x86_64-pc-windows-msvc.exe`, `-apple-darwin`, and `-unknown-linux-gnu` suffixes are strictly enforced in `tauri.conf.json` for all sidecars. | Configuration schema validation |

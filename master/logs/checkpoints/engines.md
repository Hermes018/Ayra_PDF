# checkpoints/engines.md — engines builder (wt/engines). Builder keeps this current; < 150 lines.
- **State (CP3, base 9919398 / tag engines-cp2b-green), STOPPED for budget change. Uncommitted. Compiles:** `cargo check -p {lpdf-engine-pdfcraft,lpdf-engine-switcher,lpdf-jobs,app-tauri,xtask} --all-targets` PASS; clippy -D warnings PASS on those (before the last test-file edits, which then passed check); fmt clean. **Tests NOT run** for CP3 (the run was stopped while compiling).

## Done (code written, compiles; tests written but not yet run)
- **BUG-030:** guard moved to `crates/lpdf-engine-pdfcraft/src/guard.rs` (switcher `pub use lpdf_engine_pdfcraft::guard`; old `crates/lpdf-engine-switcher/src/guard.rs` deleted). Streaming lexer over 64 KiB buffers (`scan_reader`); `read_guarded(path)` reads once, guards on first sight, verdict cache per (canonical path, size, mtime), `scan_count()` for tests. Switcher `load` no longer reads the file (calls `check_file`).
- **BUG-029:** indirect `/Length N 0 R` resolved against `N 0 obj <int> endobj` (unresolved = unknown); `#xx` name escapes decoded; strings/comments/stream data skipped; decompression-ratio cap (200:1, 32 MiB floor, 4 GiB total) decoding every stream with pdfcraft's `Stream::decoded_within` on the bytes being parsed. Entry points: 29 PdfcraftAdapter input reads + 2 sidecar input reads + 6 studio.rs Session paths → `read_guarded`; switcher static methods → `check_file` (cached).
- **Minors:** `path_exists(path: String) -> Result<bool, String>` (relative path or stat error = Err); BUG-023 test asserts empty; queue.rs registry stores FileId (dev, ino), re-checked right before unlink, entry dropped if replaced (+ test); notices: `pnpm --offline`, titled "Linux build", SPDX standard texts in `xtask/spdx/*.txt` (0 identifier-only entries left), regenerated, `--check` PASS.
- Matrix row P11 (guards; open decision: pdfcraft-cos patch to make the ratio cap native).

## Not done / next step
1. Run tests: `master/tools/run.sh tests-cp3 cargo nextest run --offline --no-fail-fast -p lpdf-engine-pdfcraft -p lpdf-engine-switcher -p lpdf-jobs -p app-tauri -p lpdf-geom -p lpdf-cli`; fix failures (new: `lpdf-engine-pdfcraft/tests/guard_tests.rs`, `lpdf-jobs/tests/bug018_tests.rs::delete_intermediate_refuses_a_replaced_file`, studio `path_exists_*`, BUG-023 test). Risk to watch: false positives of the ratio cap / scanner on corpus and existing hostile tests in lpdf-engine-switcher.
2. Run the 100 MB benchmark once: `... cargo nextest run --offline -p lpdf-engine-pdfcraft --run-ignored all bench_ --no-capture`; record numbers (synthetic-1k bench runs in the normal suite).
3. "Test fails first" was not demonstrated for BUG-029/030 (tests written with the fix). If required: run guard_tests bypass cases against the CP2b guard (git show 9919398:master/crates/lpdf-engine-switcher/src/guard.rs).
4. Gate: `LPDF_PRIO=other master/tools/verify.sh`.
5. Switch patch stays unapplied: /tmp/claude-1000/-home-rakib-Desktop-Ayra-PDF/c9df49d5-635e-4439-8330-8bef7b1875ea/scratchpad/switch/hatch-default-switch.patch (licence decision open). AR-CONC 1/16 left as is.

## New files (for the lead's tree maps; I did not edit tree.md/master.md/descriptions.tsv)
- master/crates/lpdf-engine-pdfcraft/src/guard.rs, master/crates/lpdf-engine-pdfcraft/tests/guard_tests.rs, master/xtask/spdx/{Apache-2.0,BSD-3-Clause,ISC,MIT,MPL-2.0,Unlicense}.txt. Deleted: master/crates/lpdf-engine-switcher/src/guard.rs.
- Modified: Cargo.lock (miniz_oxide dev-dep, already in lock), THIRD_PARTY_NOTICES.md, studio.rs, lpdf-engine-pdfcraft/{Cargo.toml,src/lib.rs,src/office_sidecar.rs,src/pandoc_sidecar.rs}, lpdf-engine-switcher/src/lib.rs, lpdf-jobs/{src/queue.rs,tests/bug018_tests.rs}, docs/ENGINE_REUSE_MATRIX.md, xtask/src/notices.rs.

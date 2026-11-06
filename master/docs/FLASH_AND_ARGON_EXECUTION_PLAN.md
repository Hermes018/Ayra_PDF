# FLASH & ARGON EXECUTION PLAN — Base Pro Quota & 8GB RAM Laptop Runbook

**Host Environment:** Fedora 44 KDE Plasma (Wayland, `rpm`/`dnf` native, WebKitGTK) · Intel Core i5-1135G7 · ~8 GiB RAM (`MemAvailable >= 2200 MB` guard, `1` HEAVY slot max).
**Mandatory Build Flags:** `CARGO_BUILD_JOBS=2`, `NEXTEST_TEST_THREADS=2`, `VITEST_MAX_WORKERS=2`, `NODE_OPTIONS="--max-old-space-size=2048"`.
**Zero-Chaining Law:** Never chain `pnpm` and `cargo` in a single command. Never run `cargo clean`. Route heavy commands through `master/tools/run.sh <name> <cmd...>`.

---

## Phase F1: Frontend & UI Strict Batch (Zero Cargo Builds)
**Assigned Model:** Gemini 3.8 Flash (`High`) — `fixes-builder` / `ui-sweeper`
**Scope:** Execute Category B tasks from `master/docs/ARGON_TASKS.md` with zero Rust compilation until the end-of-phase strict audit.

### Sub-Phase F1.1: UI Strict Audit Core & Gallery (`AT-10` & `AT-11`)
- **`AT-10` (Strict `ColorSwatch.svelte`):**
  - Target: `master/app-tauri/src/lib/components/ui/ColorSwatch.svelte` & `master/xtask/ui-strict.txt`.
  - Action: Add `lib/components/ui/ColorSwatch.svelte` to `xtask/ui-strict.txt` and ensure zero hex/rgb literals, raw controls, or missing aria labels.
- **`AT-11` (Strict `Gallery.svelte`):**
  - Target: `master/app-tauri/src/lib/gallery/Gallery.svelte` & `master/xtask/ui-strict.txt`.
  - Action: Preserve `GALLERY_MARKER` import and usage; replace or remove any raw demo controls violating strict UI audit; add `lib/gallery/Gallery.svelte` to `xtask/ui-strict.txt`.

### Sub-Phase F1.2: Remaining Strict UI Files & PINS Bar Clipping (`AT-12` through `AT-15`)
- **`AT-12` (Strict `StudioIcon.svelte` & `HatchDefs.svelte`):**
  - Target: `master/app-tauri/src/lib/components/studio/StudioIcon.svelte`, `master/app-tauri/src/lib/components/studio/HatchDefs.svelte`, `master/xtask/ui-strict.txt`.
- **`AT-13` (Strict Route Pages):**
  - Target: `master/app-tauri/src/routes/+page.svelte` (preserve `attachKeyboard` call from BUG-042), `master/app-tauri/src/routes/gallery/+page.svelte`, `master/app-tauri/src/routes/+layout.svelte`, `master/xtask/ui-strict.txt`.
- **`AT-14` (Strict `For.svelte`):**
  - Target: `master/app-tauri/src/lib/components/For.svelte` & `master/xtask/ui-strict.txt`.
- **`AT-15` (PINS Bar Clipped by Status Bar — `BUG-045`):**
  - Target: ONLY `master/app-tauri/src/lib/components/studio/FavoritesBar.svelte` (CSS only: height, overflow, padding using theme tokens). Do NOT touch `SpatialCanvas.svelte`.

### Phase F1 Gate (Single Batched Verification)
1. `master/tools/run.sh f1-svelte-check master/tools/sb.sh pnpm --dir master/app-tauri check`
2. `master/tools/run.sh f1-vitest master/tools/sb.sh pnpm --dir master/app-tauri test -- --run --maxWorkers=2`
3. `master/tools/run.sh f1-ui-audit cargo xtask studio-audit --strict`
4. Commit locally on `main` (or merge from worktree) once all 3 commands exit `0`.

---

## Phase F2: Isolated Unit Test Batch (Targeted Crate Tests Only)
**Assigned Model:** Gemini 3.8 Flash (`High`) — `fixes-builder`
**Scope:** Execute Category A unit test tasks (`AT-01` through `AT-07`) from `master/docs/ARGON_TASKS.md` (test code only; do not modify production functions).

### Sub-Phase F2.1: Core Rust Crate Unit Tests (`AT-01` through `AT-04`)
- **`AT-01` (`lpdf-core-types`):** Add isolated unit tests in `master/crates/lpdf-core-types/src/types.rs` (`Secret` redaction in Debug/Display, serde roundtrips).
- **`AT-02` (`lpdf-fs-safety`):** Add isolated unit tests in `master/crates/lpdf-fs-safety/src/lib.rs` (path canonicalization, temp/backup helpers in `tempdir()`).
- **`AT-03` (`lpdf-hw`):** Add isolated unit tests in `master/crates/lpdf-hw/src/lib.rs` (tier classification boundary tests).
- **`AT-04` (`lpdf-storage`):** Add isolated unit tests in `master/crates/lpdf-storage/src/lib.rs` (settings/recent-files roundtrips in `tempdir()`).

### Sub-Phase F2.2: Pipeline, Sign & Frontend Organize Tests (`AT-05` through `AT-07`)
- **`AT-05` (`lpdf-pipeline`):** Add isolated step validation unit tests in `master/crates/lpdf-pipeline/src/lib.rs`.
- **`AT-06` (`lpdf-sign`):** Add isolated signature parameter/error unit tests in `master/crates/lpdf-sign/src/lib.rs`.
- **`AT-07` (`organizeApply.test.ts`):** Add frontend unit test in `master/app-tauri/src/lib/views/organize/organizeApply.test.ts` verifying failed apply deletes zero intermediates.

### Phase F2 Gate (Targeted Execution)
1. `master/tools/run.sh f2-vitest-org master/tools/sb.sh pnpm --dir master/app-tauri vitest run src/lib/views/organize/organizeApply.test.ts`
2. `master/tools/run.sh f2-nextest cargo nextest run --test-threads=2 -p lpdf-core-types -p lpdf-fs-safety -p lpdf-hw -p lpdf-storage -p lpdf-pipeline -p lpdf-sign`
3. Commit locally once green.

---

## Phase F3: §0 Release Prep, Worktree Audit & Fedora 44 Packaging
**Assigned Models:** Guided/Reviewed by **Gemini 4 Argon (`Low`)**; Executed by **Gemini 3.8 Flash (`High`)**.

### Sub-Phase F3.1: Inspect `origin/wt/fixes` (`BUG-061` & `BUG-062`) & `origin/wt/engines` (`CP3`)
1. **Flash (`High`)** inspects `git diff main...origin/wt/fixes` (`e2e/soak_retry.py`, `e2e/test_soak_retry.py`, `flow12_soak.py`, `run.py`), runs `python3 -I master/e2e/test_soak_retry.py`, and inspects `git diff main...origin/wt/engines` (`lpdf-engine-pdfcraft/src/guard.rs`).
2. **Flash (`High`)** prepares a `<= 150-word` diff & test report for **Argon (`Low`)** / **Opus (`High`)** review.
3. Run 20-minute soak diagnostic (`flow12_soak.py`) via `tools/run.sh` to classify `BUG-061` (P0 if > 1 GB/2h, else P1 + `KI-009` in `KNOWN_ISSUES.md`).

### Sub-Phase F3.2: Audit `origin/wt/release` (`tools/sbom_npm.py`)
1. **Flash (`High`)** extracts `master/tools/sbom_npm.py` from `origin/wt/release` and runs `cargo xtask sbom` + `python3 master/tools/sbom_npm.py`.
2. **Argon (`Low` — `provenance-auditor`)** reviews the `<= 200-line` script for supply-chain honesty, full `pnpm-lock.yaml` transitive dependency resolution, and `0.2.0` version stamping.

### Sub-Phase F3.3: Execute Packaging B1 & B2 on Fedora 44 (8GB RAM Safe Profile)
1. **Packaging B1 (8GB RAM Safe LTO):**
   - On this 8 GiB Fedora 44 host, enforce `lto = "thin"` and `codegen-units = 1` in `[profile.release]` of `master/Cargo.toml` per owner fallback rule (prevents fat-LTO OOM).
   - Build release binaries and packages sequentially via `tools/run.sh`:
     - `bash fedora/packaging/build-rpm.sh`
     - `bash ubuntu/packaging/build-deb.sh`
     - `bash ubuntu/packaging/build-appimage.sh`
   - Copy `.rpm`, `.deb`, and `.AppImage` (`0.2.0`) to `~/LocalPDF-rc1/`, generate `SHA256SUMS`, copy `sbom.cdx.json`, and verify `KI-006` offline AppImage build under `unshare -rn`.
2. **Packaging B2 (Clean-Install Smoke & Release Docs):**
   - Run `python3 master/e2e/pkg_smoke.py --kinds appimage,deb,rpm` in `unshare -rn` with a fresh `HOME`.
   - Run network-isolated offline E2E (`flows 1..11`) under `unshare -rn`.
   - Author fact-checked `README.md`, `QUICKSTART.md`, `CHANGELOG.md`, and `KNOWN_ISSUES.md` (`KI-005` updated to `0.2.0`).

---

## Phase F4: Final `v0.2.0-rc1` Verification Gate & Tagging
**Assigned Model:** **Claude Opus 5.5 (`High`)** (or **Gemini 4 Argon `Medium`** Fresh Session if Opus = 0%) — `rc-verifier`
1. Run full `master/tools/verify.sh` on merged `main`.
2. Verify 12/12 Golden E2E Flows + `pdftotext` destructive redaction proof (`0` leaked secret substrings).
3. Fresh `rc-verifier` sign-off -> `git tag v0.2.0-rc1` locally on `main` -> report **READY TO PUSH** to owner.

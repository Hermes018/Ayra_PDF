# PLAN LOG (append-only) — plan before acting, amend when reality changes
Entry format:
```
## [date] Plan for Phase X.Y
- Goal / acceptance:
- Approach + alternatives considered:
- Files/crates touched:
- Failsafes affected:
- Verification steps:
- Revision (date, what changed, why):
```

## [2026-10-10] Protocol amendment 001
- Goal / acceptance: Acknowledge and integrate protocol amendment 001 (ADR-003 to ADR-007) and add new gate scope to Phase 1.2.
- Approach + alternatives considered: Following user instructions verbatim. Adding `protocol-lint` rules (checking for `@sveltejs/kit`, `{@html`, `fetch(`, `XMLHttpRequest`, `WebSocket`, `EventSource` in `ui/src`) and `cargo xtask feature-map-check` to the Phase 1.2 gate scope.
- Files/crates touched: `logs/PROMPT_LOG.md`, `logs/PLAN_LOG.md`, `logs/PROGRESS_LOG.md`, `logs/CHECKPOINT.md`.
- Failsafes affected: None.
- Verification steps: N/A.
- Revision (date, what changed, why): N/A.

## [2026-10-10] Plan for Phase 0.1
- Goal / acceptance: Initialize repository, ensure protocol files are copied, fill directory tree documentation, and make the first commit.
- Approach + alternatives considered: The user already supplied the protocol files. Directory tree is already populated in `protocol/09_DIRECTORY_TREE.md`. Git initialization and amendment commit have been performed. Will finalize logging and tag `phase-0.1-green`.
- Files/crates touched: `logs/CHECKPOINT.md`, `logs/PLAN_LOG.md`, `logs/PROGRESS_LOG.md`.
- Failsafes affected: None.
- Verification steps: `git status` clean, directory tree matches expectations.
- Revision (date, what changed, why): N/A.

## [2026-10-10] Plan for Phase 0.2
- Goal / acceptance: Complete Fedora 44 KDE bootstrap by installing all dependencies, language toolchains, recording versions, and implementing the `xtask doctor` gate check.
- Approach + alternatives considered: 
  1. Write and run a bash script using `dnf` for system packages (assuming this environment is Fedora-like or has dnf available with sufficient permissions).
  2. Install Rust via `rustup` if not present, and `pnpm` (via npm or standalone). Install required cargo extensions.
  3. Extract version strings of installed software and document in `protocol/ENV.md`.
  4. Create `xtask` binary crate. Setup workspace and cargo config alias. Implement the `doctor` command in Rust to check for required binaries in PATH.
  5. Run the gate `cargo xtask doctor`.
- Files/crates touched: `install_deps.sh` (temp), `protocol/ENV.md`, `Cargo.toml`, `.cargo/config.toml`, `xtask/Cargo.toml`, `xtask/src/main.rs`, `logs/*`.
- Failsafes affected: N/A (Build infra).
- Verification steps: `cargo xtask doctor` passes successfully.
- Revision (date, what changed, why): N/A.

## [2026-10-10] Plan for Phase 0.3
- Goal / acceptance: Complete UX audit and specification for all tools listed in the 7 columns of `07_FEATURE_MAP.md`. Gate: `ux-spec` covers every tool in `FEATURE_MAP`.
- Approach + alternatives considered: Create one markdown specification file per category (tool group) inside the `ux-spec/` directory to keep files manageable. Each tool will have documented states (empty, loaded, processing, done, error), option panels, and responsive layouts. The specs will focus purely on frontend behavior and user interaction, acting as the UI source-of-truth.
- Files/crates touched: `ux-spec/01_organize.md`, `ux-spec/02_optimize.md`, `ux-spec/03_convert_to.md`, `ux-spec/04_convert_from.md`, `ux-spec/05_edit.md`, `ux-spec/06_security.md`, `ux-spec/07_intelligence.md`, `ux-spec/08_extras.md`, `logs/*`.
- Failsafes affected: N/A.
- Verification steps: Verify 31 tools are documented across the generated files.
- Revision (date, what changed, why): N/A.

## [2026-10-10] Plan for Phase 0.4
- Goal / acceptance: Perform Engine Recon on `pdfcraft`. Spike test for merge, split, and compress operations and verify output with `qpdf --check`. Gate: spike test succeeds and `qpdf --check` is clean.
- Approach + alternatives considered: Clone `pdfcraft` into a vendor/ or external directory (e.g., `vendor/pdfcraft`), read its source to identify methods for load, combine, split, render, compress. Record findings in `protocol/ENGINE_NOTES.md`. Create a temporary rust binary (e.g., in `xtask` or a new workspace crate `spike`) that depends on the local `pdfcraft`. Generate 3 sample PDFs using an available tool, run the spike, and verify outputs with `qpdf`.
- Files/crates touched: `protocol/ENGINE_NOTES.md`, `spike/*` (or `xtask/src/spike.rs`), `logs/*`.
- Failsafes affected: DO-2 (Verify APIs from source before use).
- Verification steps: `qpdf --check` returns exit code 0 for all generated files.
- Revision (date, what changed, why): N/A.

## [2026-10-10] Plan for Phase 0.5
- Goal / acceptance: Complete dependency vetting, update `08_DEPENDENCY_VETTING.md`, write 3 new ADRs in `DECISIONS_ADR.md`, convert `vendor/pdfcraft` to a git submodule.
- Approach + alternatives considered: Read `08_DEPENDENCY_VETTING.md`. Evaluate Tauri v2, pdfium-render, lopdf, ocrs, tesseract, LibreOffice headless bridge, and markdown tools based on license, maintenance, and suitability for our offline constraints. Add ADRs (ADR-008 to ADR-010). Initialize git submodule for `vendor/pdfcraft` to replace the `.gitignore`d directory.
- Files/crates touched: `protocol/08_DEPENDENCY_VETTING.md`, `logs/DECISIONS_ADR.md`, `.gitmodules`, `logs/*`.
- Failsafes affected: DO-4 (Outputs atomic + new file by default - not strictly applicable here but part of vetting logic for dependencies).
- Verification steps: `git status` shows `.gitmodules` and `vendor/pdfcraft` as a tracked submodule.
- Revision (date, what changed, why): N/A.

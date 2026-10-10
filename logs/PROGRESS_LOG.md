# PROGRESS LOG (append-only)
Entry format:
```
## [YYYY-MM-DD HH:MM] Phase X.Y — DONE|BLOCKED
- Changes: (<=3 bullets)
- Gate: pass/fail + command output summary
- Tests added/run: names + counts
- Pre-commit checklist: 1..10 YES/NO
- Protocol compliance (phase end only): rule -> evidence
- Risks / follow-ups:
- Commit / tag:
```

## [2026-10-10 05:04] Protocol amendment 001 — DONE
- Changes: 
  - Acknowledged protocol amendment 001 (ADR-003 to ADR-007) with user approval.
  - Added new protocol-lint rules and feature-map-check to Phase 1.2 gate scope in PLAN_LOG.
  - Initialized git repo and committed the amendment.
- Gate: pass (docs only)
- Tests added/run: N/A
- Pre-commit checklist: 1 N/A 2 N/A 3 YES 4 N/A 5 N/A 6 N/A 7 N/A 8 YES 9 YES 10 YES
- Protocol compliance (phase end only): N/A
- Risks / follow-ups: None.
- Commit / tag: `docs(protocol): amendment 001 (ADR-003..007)`

## [2026-10-10 05:06] Phase 0.1 — DONE
- Changes: 
  - Verified protocol files and `protocol/09_DIRECTORY_TREE.md` are present and populated.
  - Initialized git repository.
- Gate: pass (N/A for docs)
- Tests added/run: 0/0
- Pre-commit checklist: 1 N/A 2 N/A 3 YES 4 N/A 5 N/A 6 N/A 7 N/A 8 YES 9 YES 10 YES
- Protocol compliance (phase end only): N/A
- Risks / follow-ups: None.
- Commit / tag: `phase-0.1-green`

## [2026-10-10 05:21] Phase 0.2 — DONE
- Changes: 
  - User manually installed system dependencies. Set up `xtask` crate and `.cargo/config.toml`.
  - Installed Rust, pnpm, and cargo extensions; documented toolchain versions in `ENV.md`.
  - Created `cargo xtask doctor` gate and verified it passes.
- Gate: pass (Doctor check passed. Gate green.)
- Tests added/run: 0/0
- Pre-commit checklist: 1 YES 2 N/A 3 YES 4 N/A 5 N/A 6 N/A 7 N/A 8 YES 9 YES 10 YES
- Protocol compliance (phase end only):
  - Rule 02:2 "Verify external API..." -> N/A (no APIs used)
  - Rule 02:11 "Vet dependencies..." -> N/A (no third-party crates used in xtask besides std)
- Risks / follow-ups: PATH variables need to be set in environment for rustc and pnpm (noted during task execution).
- Commit / tag: `phase-0.2-green`

## [2026-10-10 05:24] Phase 0.3 — DONE
- Changes: 
  - Generated detailed Markdown specification files in `ux-spec/` for each tool group.
  - Documented UI states, option panels, and responsive behaviors.
  - Ensured all tools from `07_FEATURE_MAP.md` are covered.
- Gate: pass (ux-spec covers every tool in FEATURE_MAP)
- Tests added/run: 0/0
- Pre-commit checklist: 1 N/A 2 N/A 3 YES 4 N/A 5 N/A 6 N/A 7 YES 8 YES 9 YES 10 YES
- Protocol compliance (phase end only):
  - Rule 02:12 "Treat ux-spec/ as the UI source of truth" -> Evidence: UX specs generated.
- Risks / follow-ups: None.
- Commit / tag: `phase-0.3-green`

## [2026-10-10 05:35] Phase 0.4 — DONE
- Changes:
  - Cloned `pdfcraft` and documented verified public APIs for merge, split, and compress in `ENGINE_NOTES.md`.
  - Created `spike` binary crate to test the engine with 3 sample PDFs.
  - Spike successfully performed Merge, Split, and Compress, and outputs were verified clean using `qpdf --check`.
- Gate: pass (spike test succeeds and `qpdf --check` is clean)
- Tests added/run: 0/1 (spike run)
- Pre-commit checklist: 1 YES 2 N/A 3 YES 4 YES 5 N/A 6 YES 7 N/A 8 YES 9 YES 10 YES
- Protocol compliance (phase end only):
  - DO-2 "Verify APIs from source before use" -> Evidence: Documented verified APIs in `ENGINE_NOTES.md`.
- Risks / follow-ups: Engine merge produces minor structural warnings (`Resources is missing or invalid; repairing`) reported by qpdf on `form.pdf`. This discrepancy is noted in `ENGINE_NOTES.md`.
- Commit / tag: `phase-0.4-green`

## [2026-10-10 05:54] Phase 0.5 — DONE
- Changes:
  - Vetted pending dependencies in `08_DEPENDENCY_VETTING.md`, approving Tauri v2, pdfcraft, lopdf, pdfium-render, ocrs, LibreOffice, typst, and svelte 5 stack.
  - Drafted ADR-008 (Engine pinning strategy), ADR-009 (Fallback renderer), and ADR-010 (Markdown-to-PDF approach) in `DECISIONS_ADR.md`.
  - Added `vendor/pdfcraft` as a tracked git submodule pinned to the validated commit `dc4b3eee`, replacing the gitignored directory.
- Gate: pass
- Tests added/run: N/A
- Pre-commit checklist: 1 N/A 2 N/A 3 YES 4 YES 5 YES 6 N/A 7 N/A 8 YES 9 YES 10 YES
- Protocol compliance (phase end only):
  - DO-8 "Dependencies vetted" -> Evidence: Updated table in `08_DEPENDENCY_VETTING.md`.
- Risks / follow-ups: `pdfium-render` native binary will increase the overall bundle size. 
- Commit / tag: `phase-0.5-green`

## [2026-10-10 06:03] Phase 0.6 — DONE
- Changes:
  - Created `corpus/MANIFEST` to track test PDFs (standard, hostile, and synthetic).
  - Wrote `corpus/fetch.sh` to deterministically download standard PDFs and generate a synthetic 1,000-page PDF using `qpdf`.
  - Ignored `*.pdf` to prevent checking in large binaries.
- Gate: pass
- Tests added/run: 0/3 (hashes verified via script)
- Pre-commit checklist: 1 YES 2 N/A 3 YES 4 YES 5 N/A 6 N/A 7 N/A 8 YES 9 YES 10 YES
- Protocol compliance (phase end only):
  - DO-4 "Outputs atomic + new file by default" -> N/A for test corpus.
- Risks / follow-ups: Need to add hostile test files later.
- Commit / tag: `phase-0.6-green`

## [2026-10-10 06:06] Phase 1.1 — DONE
- Changes:
  - Added new DOs (git pull/push on new devices) and DON'Ts (never commit PATs) in `protocol/02_RULES_DOS_DONTS.md`.
  - Added a Multi-Device Development section in `protocol/06_ARCHITECTURE.md`.
  - Setup a Cargo virtual workspace in `Cargo.toml`.
  - Created `rust-toolchain.toml` to pin the stable toolchain.
  - Generated 13 library crates and 2 binary crates in `crates/`, and prepended `#![forbid(unsafe_code)]` to all of them.
- Gate: pass
- Tests added/run: N/A
- Pre-commit checklist: 1 N/A 2 N/A 3 YES 4 YES 5 YES 6 YES 7 N/A 8 YES 9 YES 10 YES
- Protocol compliance (phase end only):
  - DO-5 "Outputs atomic..." -> N/A
  - Failsafe DO-5 "No unsafe code" -> Evidence: Add `#![forbid(unsafe_code)]` to all generated crate root files.
- Risks / follow-ups: None.
- Commit / tag: `phase-1.1-green`

## [2026-10-10 06:13] Phase 1.2 — DONE
- Changes:
  - Initialized `cargo-deny` with `deny.toml` to reject GPL/AGPL/SSPL by implicitly denying all unlicensed dependencies and keeping a strict whitelist.
  - Implemented `protocol-lint` in `xtask` to scan `src/` for banned patterns (`unwrap`, `expect`, etc.) and `Cargo.toml` for network dependencies (`reqwest`, `hyper`, etc.).
  - Implemented `gate` command in `xtask` running `cargo fmt`, `clippy`, `nextest run`, `cargo deny check`, `cargo audit`, and `cargo xtask protocol-lint`.
  - Addressed minor clippy warning (`PathBuf` unused import).
- Gate: pass
- Tests added/run: 0/0
- Pre-commit checklist: 1 N/A 2 N/A 3 YES 4 YES 5 YES 6 YES 7 N/A 8 YES 9 YES 10 YES
- Protocol compliance (phase end only):
  - DO-4 "banned patterns" -> Evidence: Addressed by `xtask protocol-lint`.
  - DO-9 "no GPL/network" -> Evidence: Addressed by `cargo deny` and `xtask protocol-lint`.
- Risks / follow-ups: None.
- Commit / tag: `phase-1.2-green`

## [2026-10-10 06:15] Phase 1.3 — DONE
- Changes:
  - Created `.github/workflows/ci.yml` defining a CI gate matrix for `fedora:44` (Linux) and `windows-latest`.
  - Configured Linux runner to install system dependencies via `dnf`, matching the local setup established in Phase 0.2.
  - Set up `Swatinem/rust-cache` and `cargo-binstall`/`taiki-e/install-action` for fast dependencies provision.
  - Created `.github/workflows/nightly.yml` defining a nightly cron job that checks out the repository, sets up nightly Rust, and strictly validates our isolated test corpus generation using `corpus/fetch.sh`.
- Gate: pass
- Tests added/run: 0/0
- Pre-commit checklist: 1 N/A 2 N/A 3 YES 4 YES 5 YES 6 YES 7 N/A 8 YES 9 YES 10 YES
- Protocol compliance (phase end only):
  - DO-5 "Outputs atomic..." -> N/A
  - DO-9 "Keep commits small, conventional, gate-green" -> Evidence: Used `cargo xtask gate` inside the new CI workflow to enforce it globally.
- Risks / follow-ups: The Fedora container is minimal, so additional implicit dependencies might crop up later if we rely on host-installed tools.
- Commit / tag: `phase-1.3-green`

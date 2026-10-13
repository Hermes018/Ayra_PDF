# Debug Log: V1 Architectural Audit & Inventory

## Phase 11.7a: Safe Environment & Guardrail Setup
- **Date/Time**: 2026-10-13T22:43:00+06:00
- **Action**: Baseline disk space check (`df -h`).
  - Output: `/` and `/home` have 16G available (86% used). Host memory limit: 8GB RAM.
- **Action**: Directory creation.
  - Executed: `mkdir -p docs/audit tools/hooks`
- **Action**: Throttled sandbox wrapper `tools/sb.sh` created and made executable.
  - Cgroup constraints: `MemoryHigh=3000M`, `MemoryMax=4000M`, `CPUQuota=150%`, `CARGO_BUILD_JOBS=1`, `NODE_OPTIONS="--max-old-space-size=1024"`.
- **Action**: Git pre-commit hook `tools/hooks/pre-commit` installed.
  - Enforces `.allowed-deletions` whitelist for any file deletions.
  - Executed: `git config core.hooksPath tools/hooks`
  - Created `.allowed-deletions` in repo root.

## Phase 11.7b: V1 Enterprise Inventory (Docs Generation)
- **Action**: Crawled root `Cargo.toml` and discovered 19 workspace members.
  - Calculated LOC and test counts across all member crates. Found `lpdf-tools-*` crates are empty stubs (~15 lines each).
- **Action**: Tool implementation parity audit against `protocol/07_FEATURE_MAP.md`.
  - Discovered 4 missing tools (`Scan to PDF`, `Page numbers`, `Crop PDF`, `Compare PDF`).
  - Identified `Rotate PDF` relocated into `OrganizeView` and `Sign PDF` in `FormsView`.
  - Identified 6 non-protocol extra tools in UI.
- **Action**: Tauri capability and command audit.
  - Located 14 commands registered in `app-tauri/src-tauri/src/lib.rs`.
  - Audited `tauri.conf.json` and `capabilities/default.json`. Clean CSP and minimal scopes, but missing file associations and RPM targets.
- **Action**: CLI and Xtask analysis.
  - Discovered custom parser in `lpdf-cli` (no `clap` dependency).
  - Confirmed `cargo xtask feature-map-check` is NOT implemented in `xtask/src/main.rs`.
- **Action**: Investigated 10 deep architectural questions (Q1–Q10).
  - Confirmed SvelteKit SPA in use (`@sveltejs/kit` + `@sveltejs/adapter-static`).
  - Found Mozilla PDF.js vendored in `static/` and parsing PDFs directly in webview.
  - Found test harness files (`fuzz.rs`, `hostile.rs`, etc.) compiled into production `lpdf-engine-switcher`.
  - Confirmed missing crates (`lpdf-layout`, `lpdf-ai`, `lpdf-bridge`) consolidated into `lpdf-engine-pdfcraft`.
  - Confirmed sidecars spawn without network isolation.
  - Confirmed `lpdf-worker` is an unlinked "Hello, world!" stub without Landlock/seccomp.
  - Documented missing frontend view tests and lack of Playwright e2e tests.
  - Classified corpus inputs vs outputs and explained `deny_out.txt` and `install_deps.sh`.
- **Action**: Formatted and wrote `docs/audit/INVENTORY.md`.
- **Status**: Phase 11.7b complete.

## Phase 11.7c: Commit & Report
- **Action**: Post-audit disk space check (`df -h`).
  - Output: `/` and `/home` remain steady at 16G available (86% used). Zero disk bloat incurred.
- **Action**: Staged `docs/audit/`, `tools/`, and `.allowed-deletions`.
- **Action**: Committed as `docs(audit): automated v1 inventory, debug logs, and guardrails`.
- **Status**: Phase 11.7 completed green.

## Strike 1: Hygiene, Sandboxing & ADR Compliance
- **Step 1: Workspace Hygiene (Q9, Q10)**
  - Updated `.allowed-deletions` with `deny_out.txt`, `install_deps.sh`, and `corpus/test_contract_out*.pdf`.
  - Removed stray test output artifacts (`corpus/test_contract_out*.pdf`), stale `deny_out.txt`, and unverified `install_deps.sh`.
- **Step 2: Harness Leakage Remediation (Q3)**
  - Wrapped `fuzz`, `hostile`, `perf`, `sbom`, and `soak` modules in `crates/lpdf-engine-switcher/src/lib.rs` with `#[cfg(test)]` to prevent compilation into production artifacts.
- **Step 3: Sidecar Network Security Patch (Q6)**
  - Patched `crates/lpdf-engine-pdfcraft/src/office_sidecar.rs` (`convert_office_to_pdf_headless` and `convert_pdf_to_office_flowable_headless`) to execute via `unshare -n -r` on Linux targets (`#[cfg(target_os = "linux")]`).
  - Patched `crates/lpdf-engine-pdfcraft/src/pandoc_sidecar.rs` (`execute_pandoc`) to execute via `unshare -n -r` on Linux targets (`#[cfg(target_os = "linux")]`).
- **Step 4: Architectural Compliance ADRs (Q1, Q2)**
  - Created `logs/ADR-008-sveltekit-spa.md` documenting the SvelteKit SPA adapter-static exception.
  - Created `logs/ADR-009-vendored-pdfjs.md` documenting the vendored Mozilla `pdf.js` webview preview exception.
  - Appended ADR-011 and ADR-012 records to `logs/DECISIONS_ADR.md`.
- **Status**: Strike 1 fixes complete.

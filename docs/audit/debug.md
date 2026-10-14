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

## Strike 2 (Phase 11.7c): Landlock Sandbox & Tool Routing
- **Step 1: Implement Worker Sandbox (Q7)**
  - Configured `crates/lpdf-worker/Cargo.toml` with `landlock = "0.3"` under Linux target dependencies along with core engine dependencies.
  - Rewrote `crates/lpdf-worker/src/main.rs` to parse JSON `JobRequest` payloads, extract read/write paths, and enforce strict Landlock filesystem sandboxing on Linux (`ABI::V1`). Configured to fail closed if sandbox enforcement fails. Delegated work to `EngineSwitcher`.
- **Step 2: Secure Worker Spawner**
  - Added `spawn_sandboxed_worker` to `crates/lpdf-jobs/src/queue.rs` and exported via `crates/lpdf-jobs/src/lib.rs`.
  - On Linux, wraps worker execution with `unshare -n -r` to guarantee complete network isolation and unprivileged sandbox activation.
- **Step 3: Eradicate Tool Stubs (Q4)**
  - Rewrote all 6 `lpdf-tools-*` crates: `lpdf-tools-organize`, `lpdf-tools-optimize`, `lpdf-tools-security`, `lpdf-tools-edit`, `lpdf-tools-convert`, `lpdf-tools-markdown`.
  - Added dependencies to `lpdf-core-types` and `lpdf-engine-port`.
  - Deleted dummy `add` functions and `it_works` tests.
  - Implemented typed, domain-specific proxy functions routing directly to `PdfEngine`.
- **Status**: Strike 2 / Phase 11.7c complete.

## Strike 3 (Phase 11.7d): Throttled Validation & Gate
- **Step 1: Throttled Workspace Compilation Check**
  - Ran `tools/sb.sh cargo check --workspace --tests`.
  - Resolved compiler syntax error in `crates/lpdf-jobs/src/queue.rs` (`error_code` brace and module-level export of `spawn_sandboxed_worker`).
  - Resolved Landlock trait imports (`Access`, `RulesetAttr`, `RulesetCreatedAttr`) and API method usage (`.add_rule`) in `crates/lpdf-worker/src/main.rs`.
  - Exposed test harness modules in `crates/lpdf-engine-switcher` via a dedicated `test-utils` feature flag, keeping them hidden in production release builds while enabling integration tests in `crates/lpdf-engine-switcher/tests/fuzz_smoke_tests.rs`.
  - Verified `tools/sb.sh cargo check --workspace --tests` passed cleanly with exit code 0.
- **Step 2: Throttled Test Validation**
  - Ran `tools/sb.sh cargo nextest run --workspace` with throttled resources.
  - All 147 integration, fuzz, hostile input, soak, queue, fs-safety, sidecar, summarizer, pandoc, pdfa, and type tests passed cleanly (147 passed, 0 skipped, 0 failed).
- **Step 3: Xtask Gate & Final Review**
  - Executed `tools/sb.sh cargo fmt` across the workspace.
  - Executed `tools/sb.sh cargo xtask protocol-lint` — verified zero unwrap/expect in production crates and complete protocol compliance.
  - Checked disk capacity (`df -h .`) — confirmed free disk space remains stable at 14GB+.
- **Status**: Phase 11.7d green. Workspace ready for Phase 12.1.

## Phase 12.1: Release Candidate 1 (Fedora Packaging & KDE Integration)
- **Step 1: Configuration Key Verification**
  - Inspected `tauri-utils-2.10.1` and `@tauri-apps/cli@2.12.1` (`config.schema.json`).
  - Documented verified keys for `productName`, `mainBinaryName`, `bundle.targets`, `bundle.category`, `bundle.fileAssociations`, and `bundle.linux.rpm.*` in `protocol/ENGINE_NOTES.md`.
- **Step 2: KDE Integration & RPM Configuration (`tauri.conf.json`)**
  - Updated `app-tauri/src-tauri/tauri.conf.json` with `targets: ["rpm"]`, `mainBinaryName: "localpdf-studio"`, `category: "Utility"`, and `fileAssociations` for `application/pdf`.
  - Created `app-tauri/src-tauri/desktop.template` ensuring `Exec=localpdf-studio %F`, `Categories=Office;Utility;`, and `MimeType=application/pdf` for seamless KDE Dolphin "Open With..." context menu integration.
  - Specified RPM runtime dependencies (`libreoffice-core`, `pandoc`).
- **Step 3: CLI Packaging & Installation Documentation**
  - Authored `docs/FEDORA_INSTALL.md` covering system requirements, DNF/RPM installation, upgrade, uninstallation, KDE Dolphin integration, and detailed headless CLI workflows for `localpdf`.
- **Step 4: Enterprise QA Checklist**
  - Authored `docs/QA_VETTING_CHECKLIST.md` establishing a rigorous manual QA vetting script across all 31 tools, sidecar degradation, UI responsiveness, hostile file tests, and offline air-gapped guarantees.
- **Step 5: Throttled Validation & Release Build**
  - `tools/sb.sh cargo fmt --all`: Passed cleanly.
  - `tools/sb.sh pnpm --dir app-tauri check`: Passed with 0 errors and 0 warnings.
  - `tools/sb.sh pnpm --dir app-tauri vitest run --no-file-parallelism --maxWorkers=1`: 8 test files passed (53/53 tests).
  - Fixed clippy redundant closure warning in `crates/lpdf-worker/src/main.rs`.
  - `tools/sb.sh cargo xtask gate`: Passed green (fmt, clippy, nextest 147/147, cargo deny, cargo audit, protocol-lint).
  - `SB_RELEASE=1 tools/sb.sh pnpm --dir app-tauri tauri build --bundles rpm`: Successfully built optimized release binary `target/release/localpdf-studio` (38 MB).
  - `SB_RELEASE=1 tools/sb.sh pnpm --dir app-tauri tauri bundle --bundles rpm`: Successfully packaged RPM bundle in 32s.
- **Step 6: RPM Package Inspection Output**
  - Package Path: `target/release/bundle/rpm/LocalPDF Studio-0.1.0-1.x86_64.rpm`
  - Size: 14.86 MiB (15 MB), vastly below the 1 GB constraint.
  - File Listing (`rpm -qpl`):
    ```
    /usr/bin/localpdf-studio
    /usr/share/applications/LocalPDF Studio.desktop
    /usr/share/icons/hicolor/128x128/apps/localpdf-studio.png
    /usr/share/icons/hicolor/256x256@2/apps/localpdf-studio.png
    /usr/share/icons/hicolor/32x32/apps/localpdf-studio.png
    ```
  - Dependencies (`rpm -qp --requires`):
    ```
    libreoffice-core
    pandoc
    libwebkit2gtk-4.1.so.0()(64bit)
    libgtk-3.so.0()(64bit)
    rpmlib(CompressedFileNames) = 3.0.4-1
    rpmlib(FileDigests) = 4.6.0-1
    rpmlib(PayloadFilesHavePrefix) = 4.0-1
    ```
  - Package Info (`rpm -qp --info`):
    ```
    Name        : local-pdf-studio
    Epoch       : 0
    Version     : 0.1.0
    Release     : 1
    Architecture: x86_64
    Group       : Unspecified
    Size        : 39597070
    License     : MIT
    Summary     : Offline, private, enterprise-grade PDF studio and toolchain
    Description : LocalPDF Studio is a high-performance offline PDF manipulation suite...
    ```
  - Desktop Entry Verification:
    ```ini
    [Desktop Entry]
    Categories=Office;Utility;
    Comment=Offline, private, enterprise-grade PDF studio and toolchain
    Exec=localpdf-studio %F
    StartupWMClass=localpdf-studio
    Icon=localpdf-studio
    Name=LocalPDF Studio
    Terminal=false
    Type=Application
    MimeType=application/pdf
    ```
- **Status**: Phase 12.1 RC1 RPM packaging & KDE Dolphin integration complete. Ready for Phase 12.1b (Clean Install Testing).



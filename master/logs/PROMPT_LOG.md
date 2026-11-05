# PROMPT LOG (append-only) — every user prompt verbatim + interpretation
Entry format:
```
## [date] Prompt #N
- Verbatim: """..."""
- Interpretation / ambiguity:
- Actions taken / sub-phase affected:
- Conflicts with protocol? (yes/no + resolution)
```

## [2026-10-10] Prompt #1
- Verbatim: """You are the sole autonomous engineer for LocalPDF Studio. The folder open in this IDE is the repository root; it already contains AGENTS.md, MASTER_PROMPT.md, protocol/ and logs/.

1. Read MASTER_PROMPT.md fully and follow it as your operating brief, starting with its §0 boot sequence (AGENTS.md -> protocol/02 -> 03 -> 04 -> 01 -> logs/CHECKPOINT.md -> logs/DECISIONS_ADR.md).
2. These files were just amended by me (protocol amendment 001, ADR-003 to ADR-007). I approve that edit. Append this message verbatim to logs/PROMPT_LOG.md and note that it is my approval of the amendment.
3. Append a PLAN_LOG entry and a PROGRESS_LOG entry titled "Protocol amendment 001". Update logs/CHECKPOINT.md (Phase 0.1 not started, active ADRs 001-007).
4. In PLAN_LOG, add the new protocol-lint rules and `cargo xtask feature-map-check` to the Phase 1.2 gate scope. Do not implement them yet.
5. Run `git init` in this folder if it is not already a repo. Commit with the message "docs(protocol): amendment 001 (ADR-003..007)".
6. Print the ANCHOR BLOCK and report in the MASTER_PROMPT §7 format. Then begin Phase 0.1 only. Do not start any later sub-phase, and ask me before anything irreversible."""
- Interpretation / ambiguity: The user provides their approval of protocol amendment 001 (ADR-003 to ADR-007). I am instructed to initialize the repository, commit the amended protocol files, update logs, and begin phase 0.1 only. No ambiguity.
- Actions taken / sub-phase affected: Updated PROMPT_LOG, PLAN_LOG, PROGRESS_LOG, CHECKPOINT. Initialized git repo and committed amendment. Began Phase 0.1.
- Conflicts with protocol? (yes/no + resolution): No. The user explicitly approved the protocol amendments as required by the conflict rule.

## [2026-10-10] Prompt #2
- Verbatim: """Begin Phase 0.2 (Fedora 44 KDE bootstrap). Execute the following steps sequentially, verifying each succeeds before moving to the next:

1. **Install System Dependencies (Fedora):**
   Write and execute a bash script to install the Tauri v2 prerequisites and project dependencies using `dnf`. This includes: `webkit2gtk4.1-devel`, `openssl-devel`, `librsvg2-devel`, `libappindicator-gtk3-devel`, `libxdo-devel`, `gcc`, `gcc-c++`, `qpdf`, `poppler-utils`, `tesseract`, and `libreoffice`. 

2. **Install Language Toolchains & CLIs:**
   - Ensure Rust is installed via `rustup` (stable toolchain).
   - Ensure `pnpm` is installed.
   - Install the required cargo extensions: `cargo-deny`, `cargo-audit`, `cargo-nextest`, and `cargo-fuzz`.

3. **Record Versions (ENV.md):**
   Determine the installed versions of Rust, pnpm, qpdf, poppler, tesseract, LibreOffice, and the Tauri webkit library. Create `protocol/ENV.md` and record these version numbers inside it.

4. **Bootstrap `xtask` and `doctor`:**
   - Initialize a new Rust binary crate named `xtask` at the root.
   - Set up the workspace `Cargo.toml` to include `xtask`.
   - Write the `cargo xtask doctor` logic in the `xtask` crate. The `doctor` command must verify that `rustc`, `cargo`, `pnpm`, `qpdf`, `pdftotext` (poppler), `tesseract`, and `soffice` (LibreOffice) are available in the system PATH.
   - Configure `.cargo/config.toml` to alias `xtask` so `cargo xtask` routes to the crate.

5. **Pass the Gate:**
   Run `cargo xtask doctor`. Do not proceed until this passes successfully.

6. **End of Phase Protocol:**
   Once the gate is green:
   - Update `logs/PROGRESS_LOG.md` (fill out the pre-commit checklist YES/NO and Protocol compliance rule -> evidence).
   - Update `logs/CHECKPOINT.md` to reflect moving to Phase 0.3.
   - Commit the changes with a conventional commit message.
   - Tag the commit as `phase-0.2-green`.
   - Print the updated ANCHOR block."""
- Interpretation / ambiguity: Instructed to execute Phase 0.2. This involves installing system dependencies, Rust, pnpm, cargo extensions. Then recording versions to ENV.md, setting up `xtask`, and checking dependencies via `cargo xtask doctor`. Finally, update logs, tag, and print ANCHOR block. No ambiguity. Note: Assuming I have sufficient privileges to install packages via dnf/cargo in the sandbox.
- Actions taken / sub-phase affected: Phase 0.2
- Conflicts with protocol? (yes/no + resolution): No.

## [2026-10-10] Prompt #3
- Verbatim: """**No, you should absolutely not give the AI agent your sudo password.**
Providing your `sudo` password to an autonomous script or agent gives it unrestricted root access to your entire operating system, which is a severe security vulnerability.
The safest and simplest way to resolve this blocker is to run the system-level package installations yourself in your own terminal, and then instruct the agent to resume its workflow.
...
> "I have manually installed all the required system dependencies via `sudo dnf`. Do not ask for my sudo password. Please proceed directly with Step 3 to determine and record the versions in ENV.md, and then run the `cargo xtask doctor` gate.""""
- Interpretation / ambiguity: User provided the instruction to proceed, indicating system dependencies are installed manually. I will update ERR-001 to fixed, record versions in ENV.md, install cargo tools, and run xtask doctor.
- Actions taken / sub-phase affected: Resuming Phase 0.2
- Conflicts with protocol? (yes/no + resolution): No.

## [2026-10-10] Prompt #4
- Verbatim: """Begin Phase 0.3 (UX Audit of iLovePDF). Execute the following steps:
1. Create a `ux-spec/` directory if it does not already exist.
2. Based on your web knowledge of iLovePDF's tools (as mapped in our `07_FEATURE_MAP.md`), generate detailed Markdown specification files for each tool group inside `ux-spec/`. 
3. For each tool, document the expected UI states (empty dropzone, loaded file list, processing state, done state, error state), option panels (e.g., compression levels, split ranges, merge drag-and-drop behavior), and responsive layout expectations.
4. Keep the specifications focused strictly on behavior, layout, and user interactions to serve as our frontend source-of-truth.
5. Once the specifications cover every tool listed in `07_FEATURE_MAP.md`, verify that the gate is met.
6. Update `logs/PROGRESS_LOG.md` and `logs/CHECKPOINT.md` to reflect the completion of Phase 0.3.
7. Commit the changes with a conventional commit message.
8. Tag the commit as `phase-0.3-green`.
9. Print the updated ANCHOR block."""
- Interpretation / ambiguity: Instructed to complete Phase 0.3 which involves generating detailed UX specifications for all 31 tools defined in the 7 columns of the feature map, logging the completion, verifying the gate, committing, tagging, and printing the anchor block. No ambiguity.
- Actions taken / sub-phase affected: Phase 0.3
- Conflicts with protocol? (yes/no + resolution): No.

## [2026-10-10] Prompt #5
- Verbatim: """Begin Phase 0.4 (Engine Recon & Spike). Execute the following steps:
1. Clone or vendor `storytold/pdfcraft` locally, inspect its crates, and record the pinned commit SHA and confirmed public APIs in `protocol/ENGINE_NOTES.md`.
2. Inspect the verified methods for load, combine, split, render, and compress. Note any discrepancies between its actual API and our architectural assumptions.
3. Create a minimal spike test script/binary using `pdfcraft` against 3 sample PDFs:
   - Perform a Merge operation.
   - Perform a Split operation.
   - Perform a Compress/Optimize operation.
4. Verify the outputs using `qpdf --check` to ensure the resulting PDF structures are valid and clean.
5. Record your findings in `logs/PLAN_LOG.md` and document the verified gate results in `logs/PROGRESS_LOG.md`.
6. Update `logs/CHECKPOINT.md` to indicate Phase 0.4 completion and next steps.
7. Commit with a conventional commit message, tag as `phase-0.4-green`, and print the ANCHOR block."""
- Interpretation / ambiguity: Instructed to execute Phase 0.4. This involves cloning `pdfcraft`, documenting its APIs in `ENGINE_NOTES.md`, building a spike test with 3 PDFs to merge, split, and compress, then verifying with `qpdf --check`. Finally, update logs, checkpoint, commit, and tag.
- Actions taken / sub-phase affected: Phase 0.4
- Conflicts with protocol? (yes/no + resolution): No.

## [2026-10-10] Prompt #6
- Verbatim: """Begin Phase 0.5 (Dependency Vetting & ADRs). Execute the following steps:
1. Review `protocol/08_DEPENDENCY_VETTING.md`. Evaluate the pending candidates (e.g., Tauri v2, pdfium-render, lopdf, ocrs, tesseract, LibreOffice headless bridge, comrak/Markdown tools).
2. Update the status table in `08_DEPENDENCY_VETTING.md` with your vetting results based on the criteria in the document (permissive license, popularity, maintenance, etc.). 
3. Draft necessary Architecture Decision Records (ADRs) in `logs/DECISIONS_ADR.md` for:
   - The adoption and pinning strategy for `pdfcraft` (updating the existing ADR if needed based on Phase 0.4 findings).
   - The choice of fallback renderer (e.g., `pdfium-render`).
   - The Markdown-to-PDF approach.
4. If you determine `vendor/pdfcraft` should be tracked as a git submodule rather than just a `.gitignore`d folder to make multi-device syncing easier for the user, execute that change now.
5. Update `logs/PROGRESS_LOG.md` and `logs/CHECKPOINT.md`.
6. Commit the changes, tag as `phase-0.5-green`, and print the ANCHOR block."""
- Interpretation / ambiguity: I need to vet dependencies, write ADRs for engine adoption, fallback renderer, and Markdown-to-PDF, convert `vendor/pdfcraft` to a submodule, and complete phase 0.5 logistics. No ambiguity.
- Actions taken / sub-phase affected: Phase 0.5
- Conflicts with protocol? (yes/no + resolution): No.

## [2026-10-10] Prompt #7
- Verbatim: """Begin Phase 0.6 (Corpus & Fixtures). Execute the following steps:
1. Create a `corpus/` directory.
2. Update the root `.gitignore` to strictly ignore all PDF files inside the corpus directory (e.g., `corpus/*.pdf`), but allow the manifest and scripts to be committed.
3. Create `corpus/MANIFEST` defining the test files we need. Categorize them into: Standard/Public PDFs, Hostile/Edge-cases, Synthetic files (1k and 10k pages).
4. Write a bash script (`corpus/fetch.sh`) or a new `xtask` command that downloads 2-3 permissive/public-domain sample PDFs using `curl` or `wget`, uses `qpdf` to programmatically generate a synthetic 1,000-page PDF, and generates SHA256 hashes verifying against MANIFEST.
5. Execute the script to ensure it successfully downloads and generates the test files. 
6. Update `logs/PROGRESS_LOG.md` and `logs/CHECKPOINT.md`.
7. Commit the changes, tag as `phase-0.6-green`, and print the ANCHOR block."""
- Interpretation / ambiguity: Create test corpus, fetch script, manifest, ignore PDFs, run script, and close phase 0.6. No ambiguity.
- Actions taken / sub-phase affected: Phase 0.6
- Conflicts with protocol? (yes/no + resolution): No.

## [2026-10-10] Prompt #8
- Verbatim: """First, apply the missed Git workflow protocol updates, then immediately execute Phase 1.1. Execute the following steps sequentially:
1. **Protocol Amendments (Multi-Device Git Workflow):**
   - Edit `protocol/02_RULES_DOS_DONTS.md`. Add a new DO: "Always start a session on a new device with `git pull --recurse-submodules` and end by pushing commits and tags (`git push && git push --tags`)."
   - Edit `protocol/02_RULES_DOS_DONTS.md`. Add a new DON'T: "Never commit, log, or store GitHub Personal Access Tokens (PATs) or credentials."
   - Edit `protocol/06_ARCHITECTURE.md`. Add a brief "Multi-Device Development" section...
2. **Begin Phase 1.1 (Foundation - Cargo Workspace & Skeleton):**
   - Create a root `rust-toolchain.toml` file pinning the stable toolchain.
   - Create a root `Cargo.toml` configuring a virtual Cargo workspace. Include `xtask` and the crates listed below...
   - Create a `crates/` directory.
   - Use `cargo new --lib` to generate the 13 crates...
   - Use `cargo new --bin` to generate the 2 crates...
   - Inject `#![forbid(unsafe_code)]`...
   - Run `cargo check`...
3. **Logging & Commit:**
   - Update `logs/PROGRESS_LOG.md`... and `logs/CHECKPOINT.md`...
   - Commit... tag... print ANCHOR block."""
- Interpretation / ambiguity: Protocol update for git sync flow. Workspace setup with new crates. No ambiguity.
- Actions taken / sub-phase affected: Protocol Amendment + Phase 1.1
- Conflicts with protocol? (yes/no + resolution): No.

## [2026-10-10] Prompt #9
- Verbatim: """Begin Phase 1.2 (Gates & Linting). Execute the following steps sequentially:
1. **Initialize cargo-deny:** Run `cargo deny init` to generate a `deny.toml` file at the root. Ensure it is configured to explicitly reject GPL, AGPL, and SSPL licenses.
2. **Implement protocol-lint:** Extend the `xtask` crate to include a `protocol-lint` command. This custom lint must scan the workspace and fail if it detects:
   - Banned code patterns in non-test `src/` files: `.unwrap(`, `.expect(`, `todo!(`, `unimplemented!(`, or `panic!(`.
   - Banned dependencies in any `Cargo.toml`: HTTP/network crates (e.g., `reqwest`, `hyper`, `ureq`, `isahc`, `surf`).
3. **Implement gate:** Extend the `xtask` crate to include a `gate` command. This command must sequentially execute and fail if any of these steps fail:
   - `cargo fmt -- --check`
   - `cargo clippy --workspace -- -D warnings`
   - `cargo nextest run` (passing with 0 tests is acceptable for now)
   - `cargo deny check`
   - `cargo audit`
   - `cargo xtask protocol-lint`
4. **Verify the Gate:** Run `cargo xtask gate`. If it fails, fix the underlying issues (e.g., formatting, clippy warnings) until it passes completely green.
5. **Logging & Commit:**
   - Update `logs/PROGRESS_LOG.md`... and `logs/CHECKPOINT.md`...
   - Commit... tag... print ANCHOR block."""
- Interpretation / ambiguity: Implement strict linters and a CI gate inside xtask.
- Actions taken / sub-phase affected: Phase 1.2
- Conflicts with protocol? (yes/no + resolution): No.

## [2026-10-10] Prompt #10
- Verbatim: """Begin Phase 1.3 (CI - GitHub Actions). Note: Your previous checkpoint mentioned "Extract layout analysis spike", which is incorrect for this phase. Please align strictly with `protocol/05_PHASES.md`. Execute the following steps:
1. **Create CI Workflow (`.github/workflows/ci.yml`):**
   - Trigger on push and pull request.
   - Set up a build matrix for Linux (Fedora container) and Windows (`windows-latest`).
   - Include steps to checkout the code (with `submodules: recursive`), install the stable Rust toolchain, and set up Rust caching (e.g., `Swatinem/rust-cache`).
   - For the Fedora container, include a step to install the system dependencies required by Tauri and our tools (webkit2gtk4.1-devel, qpdf, etc., as we did in 0.2).
   - The final validation step MUST run our custom gate: `cargo xtask gate`.
2. **Create Nightly Workflow (`.github/workflows/nightly.yml`):**
   - Trigger via a nightly cron schedule.
   - Include steps to checkout, set up Rust nightly, and run `corpus/fetch.sh` to guarantee our test fixtures generate correctly in isolated environments.
3. **Logging & Commit:**
   - Update `logs/PROGRESS_LOG.md`... and `logs/CHECKPOINT.md`...
   - Commit... tag... print ANCHOR block."""
- Interpretation / ambiguity: Create CI and nightly workflows. Ensure alignment with phase requirements.
- Actions taken / sub-phase affected: Phase 1.3
- Conflicts with protocol? (yes/no + resolution): No.


### [2026-11-04] Spec Checklist, Workflow Cross-Check & Phase D2.2 Remediation
- **User Prompt Summary**: Analyze `/home/ayra/Downloads/pdf_drawing_annotation_app_spec.md`, implement into repository `.md` checklist with workflow cross-checking, audit D2.1 and global features vs. placeholders against Opus 5.5 plan, and fix all identified issues in a token-optimized manner with full logging.
- **Outcome**: Created `docs/DRAWBOARD_FEATURE_CHECKLIST.md`, `docs/ARCHITECTURE_AND_COMPLIANCE_AUDIT.md`, upgraded `xtask` (`feature-map-check` & `studio-audit`), and completed Phase D2.2 remediation with zero compliance findings.

## [2026-11-05] Prompt #11 (new device: Zorin OS 18.1 / Ubuntu 24.04)
- Verbatim: """analyze the entire codebase and read all .md files and check how everything works where we are at and what the current goals and status are right now. We are on a different pc now with ubuntu installed instead of fedora so test with ubuntu builds if needed. ANalyze and get back wot me when you understand where the project stands"""
- Interpretation / ambiguity: Read-only analysis plus local verification on the new Ubuntu machine. No commits.
- Actions taken / sub-phase affected: Booted per AGENTS.md; initialised the pdfcraft submodule; installed rustup (user-level) and frontend deps; ran svelte-check (0/0), vitest (127/127), vite build, xtask lints (pass), Rust tests excluding app-tauri (148 pass / 3 fail). Reported: forbid(unsafe) regression, dash-incompatible tests, render stub, sandboxed worker never spawned, two placeholder checklist items, missing rc5 tag, ADR number collisions.
- Conflicts with protocol? No.

## [2026-11-05] Prompt #12
- Verbatim: """i need you to create/copy  and organize all files into 7  folders : android, macos, ubuntu, fedora, windows, ios and master. The master folder will house files that are needed by version/development/build of all versions of the software and the other folders will house their respective os related build files. that way it wont be bloadted and all directories will be ordered and easier to manage. there will be a master tree.md which will map all files in every direcctory and also a folder specific .md file in each folder to map its contents and all tree entries will have a short brief description of what the folder/file is about. Notify me once youre done. the global development/ general gevelopment will go on as the master and will only copy over files to their respective folders when we are working on those respective platform versions.we are now working to create a fully functional and working ui/ux version"""
- Interpretation / ambiguity: MOVE (not duplicate) the shared workspace into master/; move OS-specific packaging/docs into OS folders; root keeps only files tools require there (.github, .gitignore, .gitmodules, AGENTS.md). One /tree.md + <folder>/<folder>.md, every entry described. Current milestone: fully working UI/UX in master.
- Actions taken / sub-phase affected: Repository restructure (ADR-016). See PROGRESS_LOG 2026-11-05 "Seven-folder restructure".
- Conflicts with protocol? Yes — DON'T-14 (protocol files) and 09_DIRECTORY_TREE. Resolution: rule 02 header says the user's live message overrides; protocol file CONTENT is unchanged, only its location; AGENTS.md paths and 09_DIRECTORY_TREE updated to the new layout; recorded as ADR-016.

## [2026-11-05] Prompt #13
- Verbatim: """do all these that are necessary as well""" (sent with the pasted Prompt #11 report)
- Interpretation / ambiguity: Apply the fixes recommended in the report: unsafe regression, dash-incompatible tests, correct DONE claims + tag/ADR records, Ubuntu packaging. The sandboxed-worker wiring and real page rendering were presented as a decision and are large engineering items; they are scheduled as the next items, not done in this change.
- Actions taken / sub-phase affected: See PROGRESS_LOG 2026-11-05 entry.
- Conflicts with protocol? No.

## [2026-11-05] Prompt #14
- Verbatim: """DO all the necessary things to get started on actual development goals now. install any and all required dependencies or extensions""" (sent with the pasted action list: approve ADR-015, install build packages, commit + push, other-machine steps, next UI/UX goal)
- Interpretation / ambiguity: User confirms ADR-015, authorises the system package install, the commit + push of the restructure, and environment setup (cargo gate tools, VS Code extensions), then the start of the UI/UX milestone.
- Actions taken / sub-phase affected: apt packages via pkexec (user authenticates in the OS dialog); VS Code extensions; cargo-deny/audit/nextest/tauri-driver via `cargo install --locked`; ADR-015 approval recorded; full gate; restructure commit + push.
- Conflicts with protocol? No.

## [2026-11-05] Prompt #15
- Verbatim (summary of a long structured message, sections 1–6): ADR-018 APPROVED (port algorithms; ERR-007 first; adopt kurbo / vectorcraft-pathops / designcraft-compose only in the phase that needs them, each vetted). ADR-017 stays ACCEPTED; draft an upstream issue in docs/ and don't post it. Fix the UI wording for LaTeX and ink-to-text. The 500297f slip is fine. New scope: Android as a first-class target. Write ADR-019 (PROPOSED: capability matrix, SAF file I/O keeping the fs-safety guarantees, jobs/RAM, tablet-first UI, bundle/OCR models, verify Tauri mobile APIs from source) and stop for approval; report SDK/NDK/emulator disk and RAM needs before installing. Cross-platform rule (cfg modules behind traits, frontend capability module, platforms declared in the checklist and enforced, Android cargo check in the gate). Agent workflow (lead owns main/gates/logs/commits; ≤2 builders in worktrees track/linux and track/android; flock build lock; CARGO_BUILD_JOBS=2; fresh verifier per phase; merge only when verifier passes and gate is green). Order: Linux = ERR-007 → UI wording → next ADR-018 phase; Android = ADR-019 only. Commit and tag locally; do NOT push.
- Interpretation: as written. "Next ADR-018 phase" = survey §6 phase 2 (ink quality). Builders leave changes uncommitted; the lead commits on the track branch after the verifier passes, then fast-forwards main.
- Conflicts with protocol? No (ADR-020 records the workflow).

## [2026-11-05] Prompt #16
- Verbatim: """Change to ADR-018 before continuing: I want to pull future upstream fixes from the craft repos without redoing work. Revise ADR-018 (back to PROPOSED) so that: 1. Engine logic is consumed as pinned dependencies (exact git rev or version, network-pulling features disabled, vetted with cargo-deny), not hand-ported, wherever feasible. 2. Each engine sits behind our own adapter crate/trait so upstream API changes only touch the adapter. 3. For pointer-speed frontend code, evaluate compiling the engine crates to WASM vs keeping TS ports. If any port stays, record its upstream repo + commit in a manifest, and add `cargo xtask upstream-check` that reports upstream changes since that commit. 4. Add an UPGRADING.md procedure: one branch per engine bump → full gate → verifier agent → log. Apply the same thinking to the ADR-017 winprint patch (track its upstream rev in the manifest). ERR-007 can still go first, but implement it against this structure. Stop for my approval of the revised ADR-018."""
- Interpretation: Stop the in-flight ERR-007 hand-port (done; the builder had made no edits). Write ADR-018 rev 2 (PROPOSED) with per-crate feasibility evidence, plus a draft UPGRADING.md. Implementation (manifest, xtask, adapters, ERR-007) waits for approval. The Android ADR-019 track continues; it stops for approval anyway.
- Conflicts with protocol? No.

## [2026-11-05 22:30] Prompt #17 — REPLACES earlier instructions
- Verbatim (long structured message; key points): Target a fully working Linux prototype by Sat 2026-11-07 23:59 Asia/Dhaka, with the ADR-018 engine restructure done now.
  - **§0 Scope:** Android PAUSED until Sunday (ADR-019 draft only). FEATURE FREEZE; LaTeX and ink-to-text go behind "Labs", off by default.
  - **§1 ADR-018 → APPROVED with rules:**
    - ENGINE_REUSE_MATRIX: ADOPT / ADOPT-WASM (<2 ms per call at 120 Hz) / KEEP-PORT / REJECT.
    - Strangler pattern: characterization goldens → adapter with the current impl → craft impl behind a switch → parity → switch the default.
    - Order: hatch, geometry/measure, pathops eraser, snapping, reflow.
    - Hygiene: pinned revs, default-features off, cargo-deny, vendoring, docs/UPSTREAM_MANIFEST.toml, offline `xtask upstream-check` against ~/craft-ref, UPGRADING.md, THIRD_PARTY_NOTICES.md.
    - Timebox: adapters merged by Fri 18:00; never ship a half swap; ERR-007 must ship fixed.
  - **§2:** PROTOTYPE_SCOPE.md and 12 golden-path E2E flows via tauri-driver; baseline report first.
  - **§3:** lead plus ≤3 builders (wt/engines, wt/fixes, wt/release), flock lock, read-only reviewer / adversarial tester (logs/BUGS.md) / provenance auditor, a fresh verifier per merge.
  - **§4:** packaging rpm + AppImage (+deb), network-isolation run, CHANGELOG, KNOWN_ISSUES.
  - **§5:** milestones Fri 08:00, Fri 18:00, Sat 14:00, Sat 18:00 rc1, Sat 23:59 tag. Slips mean cuts, never a deadline extension.
  - **§6:** guardrails; start each task from CHECKPOINT; commit locally, no push.
- Interpretation: as written. Android builder told to save its draft and stop. ADR-018 rev 3 = the user's rules (binding) plus rev 2 analysis. Lead-only doc commits on main are allowed. Builder work merges only after reviewer + verifier sign-off and a green gate.
- Conflicts with protocol? Possible ones are flagged: `'wasm-unsafe-eval'` CSP (needs approval, only if ADOPT-WASM wins); vendored craft sources (ADR-018 R3).

## [2026-11-05 22:40] Prompt #18 — efficiency and reliability pass
- Verbatim (summary of a structured message, 7 sections):
  1. All subagents on Opus 5.5; high effort for lead, verifier, engines builder and reviewer.
  2. Context hygiene: tools/run.sh (lock, full log in logs/runs/, short summary); subagent replies ≤200 words in PASS/FAIL · findings · files form; exact paths in briefs; start from CHECKPOINT.
  3. Hooks in .claude/settings.json (committed):
     - PreToolUse Bash guard: no cargo clean, git push, reset --hard, or rm -rf outside target/ and logs/runs/; builds only via run.sh.
     - PostToolUse format.
     - Stop/SubagentStop write to logs/AGENT_ACTIVITY.md.
     - Test each hook once.
  4. CLAUDE.md ≤200 lines.
  5. Build speed: sccache (local), mold, dev profile line-tables-only and deps debug=0, cargo check while iterating, nextest, shared pnpm store. Measure before/after; roll back anything that breaks the gate.
  6. Reliability: tools/verify.sh is green; logs/FLAKY.md; checkpoint commit before risky changes; plan first for >3-file changes; test first for P0.
  7. Report at the next milestone.
- Interpretation: applied between merges; running builders not interrupted. The guard temporarily allows direct build-lock.sh calls until every builder has been re-briefed to use run.sh. Prettier only formats files that were already prettier-clean at HEAD, so legacy files aren't mass-reformatted across worktrees.
- Conflicts with protocol? No.

## [2026-11-06 00:20] Prompt #19 — disk
- Verbatim: """Approved: one-time `rm -rf master/target/debug`; log it in ERROR_LOG/CHECKPOINT as an approved exception to "never cargo clean". Then: Give each worktree its own target dir (inside the worktree, or ~/lpdf-targets/<worktree>), never shared. Cap sccache: SCCACHE_CACHE_SIZE=10G, local disk only. Add a disk guard to tools/run.sh: refuse to start a build if free space is < 15 GB, and report instead. When a worktree's branch is merged and the worktree removed, delete its target dir too. If disk gets tight, drop to 2 builders rather than deleting caches mid-sprint. Log free disk space in each milestone report."""
- Actions: done (ERR-008, commit 81ae676).

## [2026-11-06 01:10] Prompt #20 — compaction survival, then continue (summary)
- Living CHECKPOINT and per-worktree checkpoints; PreCompact snapshot-state.sh writes STATE_AUTO.md; SessionStart restore-context.sh; compact instructions in CLAUDE.md; test with /compact. Then merge main into engines/release, start reviewers and verifiers, hand P0s to fixes; next report Fri 08:00.
- Actions: done. The live /compact test needs the owner (built-in command; pipe tests pass).

## [2026-11-06 05:45] Prompt #21 — RAM-aware parallelism (summary)
- Training has absolute priority. Measure training and per-class RAM into docs/RAM_BUDGET.md; budget = MemTotal − training peak − 6 GiB; scheduler in run.sh (LIGHT/HEAVY, MemAvailable admission, priority FIFO, 1–2 HEAVY slots, scheduler.log); cgroup MemoryHigh/Max, choom 800, nice/ionice; single re-queue on memory kill; footprint settings; report at the next milestone.
- Actions: done (sched.sh, protect.sh, measure-ram.sh, sample-training.sh, RAM_BUDGET.md). Deferred: split-debuginfo (avoids a cold rebuild in all worktrees before 08:00). Fixed along the way: sccache held the build lock (fd inheritance).

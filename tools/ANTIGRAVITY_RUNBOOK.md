# ANTIGRAVITY RUNBOOK — Phase 11.7 (Vetting, in phases) → Phase 12.1 (RC1)
Designed for **Gemini Flash** (cheap, but known to delete code and hallucinate) with Opus 4.6 reserved for the few steps that need it. Safety does not rely on the model obeying text: `tools/task.sh` and the pre-commit hook enforce file scope and block deletions.

## HOW EVERY STEP WORKS (the loop)
1. **Terminal (you):** `tools/task.sh start <name> '<globs>'` → makes a backup bundle, a tag `pre-<name>`, a branch `task/<name>`, and the allowed-file list.
2. **Antigravity:** open a **NEW chat** (fresh context), choose the model shown for the step, paste the prompt block. Keep terminal approval ON.
3. **Terminal (you):** `tools/task.sh check`. It must print `TASK CHECK: CLEAN`. Then spot-check 3 claims in what the agent wrote (open the cited `path:line`, see if the quote is really there).
4. If clean and the work is good: `tools/task.sh finish` (merges into main). If not: `tools/task.sh abort`, or revert only the bad paths with the command `check` prints, then tell the agent exactly what to redo.
5. Paste the `check` output + the agent's final report to Claude if anything looks off.
Rules of thumb: one prompt = one chat = one task; never let the agent "also fix" things; any P0 finding goes to Opus or Claude, not Flash.

## STEP LIST
| Step | Name | Model | Writes |
|---|---|---|---|
| V0 | Guardrails | you (terminal) | tools/, docs/audit/ |
| V1 | Inventory | Flash | docs/audit/INVENTORY.md |
| V2 | Protocol reconciliation + licence audit | Flash, then Claude verifies | protocol (append), docs |
| V3 | Traceability matrix, one run per column (7) | Flash | docs/audit/trace/* |
| V4 | Stub hunt + UI↔backend parity (after you run the bundle script) | Flash | docs/audit/STUB_HUNT.md |
| V5 | Real-file oracle tests, one run per group (11) | Flash (Opus for `security`) | xtask/src/verify_*.rs, docs |
| V6 | Mutation testing, one run per crate (7, overnight) | Flash | crates/*/tests/mutation_*.rs |
| V7 | Security proofs (4 runs) | **Opus** | tests/security, crates/*/tests/security_* |
| V8 | Claude independent audit | Claude | docs/audit/FINDINGS.md |
| V9 | Fix tasks, one per finding | Flash for P2/P3, Opus for P0/P1 | per finding |
| V10 | Audit-green gate + tag | Flash | logs |
| 12.1a–e | RC1 packaging in five small tasks | see each | see each |

---

## V0 — Guardrails (you, terminal, in the repo root)
```bash
# put these files in the repo root first: setup_guardrails.sh, ORACLE_CHECKS.md, ANTIGRAVITY_RUNBOOK.md (+ the 3 you already added)
bash setup_guardrails.sh
git status --short          # must be empty
tools/task.sh               # prints usage
```
If the script warns that `crates/`, `app-tauri/` or `logs/` are missing, you opened the wrong folder: stop and tell Claude.

---

## V1 — Inventory (Flash)
```bash
tools/task.sh start v1-inventory 'docs/audit/*'
```
```
TASK V1 (v1-inventory). First read docs/audit/SAFETY.md and obey it. Then print `cat .task-allowed` and `git status --short`.
Goal: a factual inventory of this repository. You change NOTHING except one new document.
1. List the workspace members from the root Cargo.toml. For each crate: path, lines of Rust (`wc -l` over its .rs files), number of test functions (`#[test]`, `#[tokio::test]`; use rg -c or grep -rc), whether it has a tests/ folder.
2. Find the UI tool registry (`git ls-files | grep -i registry`) and list every tool id and category. Compare with the 31 tools in protocol/07_FEATURE_MAP.md. List MISSING and EXTRA tools.
3. List every `#[tauri::command]` function (path:line) and every command permitted in app-tauri/src-tauri/capabilities/*.json.
4. List every subcommand of the CLI by reading the clap definitions in crates/lpdf-cli (do not build anything).
5. List every `cargo xtask` subcommand by reading xtask/src.
6. Write everything to docs/audit/INVENTORY.md as tables. Every row ends with path:line. Write NOT FOUND where you cannot locate something. Do not guess.
7. Commit "docs(audit): inventory (v1)". Report in the format from SAFETY.md.
```
```bash
tools/task.sh check && tools/task.sh finish
```

---

## V2 — Protocol reconciliation + licence audit (Flash; send result to Claude to verify)
```bash
tools/task.sh start v2-reconcile 'protocol/03_FAILSAFES.md' 'protocol/08_DEPENDENCY_VETTING.md' 'protocol/09_DIRECTORY_TREE.md' 'logs/DECISIONS_ADR.md' 'docs/audit/*'
```
```
TASK V2 (v2-reconcile). First read docs/audit/SAFETY.md and obey it. Then print `cat .task-allowed` and `git status --short`.
I approve APPEND-ONLY edits to the protocol files listed in .task-allowed for this task. Existing lines must not change or disappear (tools/task.sh check will flag any removed line).
1. Compare the repo with protocol/03_FAILSAFES.md, 09_DIRECTORY_TREE.md and logs/DECISIONS_ADR.md using docs/audit/INVENTORY.md. Write docs/audit/PROTOCOL_DRIFT.md listing: failsafes implemented in code/tests but not in protocol 03 (e.g. F21–F25, quote the code), crates not in protocol 09 (e.g. lpdf-engine-switcher), rules visible in code or configs but not in the protocol (e.g. the 1 GB bundle limit), and protocol rules enforced by no gate or test. Each item has path:line.
2. APPEND (do not edit existing lines) the missing failsafe rows to protocol/03_FAILSAFES.md, the missing crates to protocol/09_DIRECTORY_TREE.md (as an appended "Additions 2026-10-13" section), and ADR entries marked "retroactive, documented 2026-10-13" to logs/DECISIONS_ADR.md for: engine-switcher, bundle-size limit, sidecar policy.
3. Licence audit → docs/audit/LICENSE_AUDIT.md. Run `tools/sb.sh cargo deny check licenses` and paste the result. Then list every external tool the app runs or ships (pandoc, soffice, tesseract, scanimage, others found by grepping for Command::new and for tauri.conf.json bundle/externalBin/resources). For each: licence (open the official source and give the URL; if unsure write UNVERIFIED), whether it is BUNDLED in any package or only EXTERNAL (found on the user's system), and whether that complies with protocol 02 DON'T-9. Pandoc is GPL: if anything bundles it, report it as P0 at the top of the document.
4. Commit "docs(protocol): reconcile with implementation (v2)". Report.
```
```bash
tools/task.sh check && git diff pre-v2-reconcile -- protocol logs/DECISIONS_ADR.md | grep '^-' | grep -v '^---'   # must print nothing
tools/task.sh finish
```
Send `docs/audit/LICENSE_AUDIT.md` to Claude to verify the licences before trusting them.

---

## V3 — Traceability matrix (Flash; 7 runs, one per column)
Run the block below 7 times, replacing `COLUMN` and `slug` with: `ORGANIZE PDF`/`organize`, `OPTIMIZE PDF`/`optimize`, `CONVERT TO PDF`/`to-pdf`, `CONVERT FROM PDF`/`from-pdf`, `EDIT PDF`/`edit`, `PDF SECURITY`/`security`, `PDF INTELLIGENCE`/`intelligence` (the last run also covers Extras: Markdown→PDF, Pipelines, CLI, Settings).
```bash
tools/task.sh start v3-trace-<slug> 'docs/audit/trace/*'
```
```
TASK V3 (v3-trace-<slug>). First read docs/audit/SAFETY.md and obey it. Then print `cat .task-allowed` and `git status --short`.
Goal: traceability for the tools in column COLUMN of protocol/07_FEATURE_MAP.md. Read-only except one new file: docs/audit/trace/<slug>.md. Use docs/audit/INVENTORY.md to find paths.
For EACH tool in the column write one block with these fields:
- Tool / UI tile id / job kind
- Implementing code: crate::function with path:line and a 2-line quote
- Engine actually used (pdfcraft / fallback / own code / sidecar) with the evidence line
- Tests: exact test names with path:line. For each test: level (unit/integration/e2e/property/fuzz/golden) and the KEY ASSERTION quoted (≤3 lines). If the test only asserts is_ok()/no panic, or mocks the engine, say so.
- Verdict, strict: VERIFIED only if a test runs the real code path from an input PDF to a saved output AND asserts something about the output content. Otherwise PARTIAL (some real test but weak), UNTESTED (no test), or STUB (code returns a constant/placeholder/unimplemented).
- Weakest link (one sentence)
End the file with counts of VERIFIED/PARTIAL/UNTESTED/STUB. Do not fix anything. Do not skip a tool: if you cannot find it, write NOT FOUND.
Commit "docs(audit): traceability COLUMN (v3)". Report.
```
After all seven are finished:
```bash
tools/task.sh start v3-merge 'docs/TRACEABILITY.md'
cat docs/audit/trace/*.md > docs/TRACEABILITY.md
git add docs/TRACEABILITY.md && git commit -m "docs(audit): merged traceability matrix" && tools/task.sh check && tools/task.sh finish
grep -h -E "^(- )?Verdict" docs/audit/trace/*.md | sort | uniq -c    # quick counts
```

---

## V4 — Stub hunt + UI↔backend parity (Flash)
First produce the evidence (long, runs cargo through the throttle; let it finish):
```bash
tools/make_audit_bundle.sh
```
```bash
tools/task.sh start v4-stubs 'docs/audit/*'
```
```
TASK V4 (v4-stubs). First read docs/audit/SAFETY.md and obey it. Then print `cat .task-allowed` and `git status --short`.
Goal: triage the automated scans in audit_bundle/evidence/ (files named scan_*.txt) and check UI↔backend parity. Read-only except docs/audit/STUB_HUNT.md.
1. For each scan file: classify every hit as REAL (production code that is a stub/placeholder/unwrap/ignored test/hidden error) or FALSE POSITIVE (test code, comment, name only). For every REAL hit give path:line, a 2-line quote, severity (P0 data loss/security/tool broken; P1 spec violated; P2 quality; P3 nit) and why. For false positives only give a count per file.
2. Parity table: for every tool tile in the UI registry → job kind → tauri command → backend function (path:line each). Mark ORPHAN-UI (tile without backend), ORPHAN-BACKEND (backend without tile), or BROKEN-LINK (names do not match).
3. Find error-swallowing in production code: `let _ =`, `.ok();`, `unwrap_or_default()` on a Result, empty `catch {}` / `.catch(() => {})` in ui/src. Classify each as acceptable or hiding a failure.
4. Check evidence files scan_network_crates_lock, scan_ui_network_html, scan_sidecar_refs and cargo_deny: list anything that violates protocol 02.
Commit "docs(audit): stub hunt and parity (v4)". Report with counts per severity.
```

---

## V5 — Real-file oracle tests (Flash; Opus for `security`; 11 runs, one per group)
Groups come from `docs/audit/ORACLE_CHECKS.md`: `organize`, `optimize`, `to-pdf`, `from-pdf`, `edit`, `security`, `intelligence`, `markdown`, `pipelines-cli-settings`, `hostile`, `crash-safety`. Do them in this order; start with `security` (Opus) and `organize`.
```bash
tools/task.sh start v5-verify-<group> 'xtask/src/*' 'tests/verify/*' 'docs/audit/*'
```
```
TASK V5 (v5-verify-<group>). First read docs/audit/SAFETY.md and obey it. Then print `cat .task-allowed` and `git status --short`.
Goal: add `cargo xtask verify-tools <group>` that tests the REAL tools with INDEPENDENT oracles. Do NOT change production crates. Do NOT fix failures here; record them.
1. Read the section "GROUP <group>" in docs/audit/ORACLE_CHECKS.md. Implement EVERY listed check, one function each, returning pass/fail with a message. Put the code in a NEW file xtask/src/verify_<group>.rs (or tests/verify/<group>.rs if xtask cannot host it) and register the subcommand with the fewest added lines possible in xtask/src/main.rs (show me that diff).
2. Check which oracle tools exist with `command -v qpdf pdfinfo pdftotext pdftoppm pdfsig pdfimages verapdf soffice tesseract unzip`. List the missing ones and ASK ME before installing anything. Wait for my answer.
3. Test inputs: reuse existing fixtures/generators; otherwise write minimal PDFs as byte strings or use the corpus manifest. Output under target/verify/. No new crate dependencies.
4. Run once: `tools/sb.sh cargo xtask verify-tools <group>`. Paste the result table (check → PASS/FAIL). For each FAIL: the first failing assertion and the output file path.
5. Write docs/audit/VERIFY_<group>.md: table of checks, results, failures with severity (P0–P3). Commit "test(verify): <group> (v5)". Report.
```
Tip: a FAIL is a finding, not a bug in the test, unless the oracle command itself is wrong. If every check passes on the first run, be suspicious: ask Claude to review the test for weak assertions.

---

## V6 — Mutation testing (Flash; run overnight; 7 runs, one per crate)
Crates in order: `lpdf-fs-safety`, `lpdf-jobs`, `lpdf-tools-security`, `lpdf-engine-switcher`, `lpdf-worker`, `lpdf-layout`, `lpdf-ai`.
```bash
tools/task.sh start v6-mut-<crate> 'crates/<crate>/tests/mutation_*' 'docs/audit/*' '.config/mutants.toml'
```
```
TASK V6 (v6-mut-<crate>). First read docs/audit/SAFETY.md and obey it. Then print `cat .task-allowed` and `git status --short`.
Goal: measure how strong the tests of crate <crate> are with cargo-mutants and add tests that kill important surviving mutants. You may add test code ONLY in new files crates/<crate>/tests/mutation_*.rs. Do not touch production code.
1. Check `command -v cargo-mutants`. If missing, tell me and WAIT; I will approve installing it after we check its licence and maintenance.
2. Run once: `tools/sb.sh cargo mutants -p <crate> --jobs 1 --timeout-multiplier 3 --no-shuffle` (this is slow; do not run anything else meanwhile).
3. From the result (mutants.out/missed.txt) list the surviving mutants. For each: path:line, the mutation, whether it matters (affects safety, atomicity, parsing, limits, error codes) and why.
4. For the important survivors write tests in crates/<crate>/tests/mutation_killers.rs that fail under that mutation. Re-run cargo-mutants only for the affected functions (check `cargo mutants --help` for --file or --re) and report caught/missed before and after.
5. Write docs/audit/MUTATION_<crate>.md: counts, important survivors, tests added, remaining survivors with justification. Target ≥ 80 % caught on critical logic; say honestly if lower. Commit "test(mutation): <crate> (v6)". Report.
```

---

## V7 — Security proofs (Opus; 4 runs)
Items P-a…P-h are in `docs/audit/ORACLE_CHECKS.md` (section SECURITY PROOFS). Runs: (1) P-a, P-b, P-c · (2) P-d, P-g · (3) P-e, P-f · (4) P-h.
```bash
tools/task.sh start v7-sec-<n> 'tests/security/*' 'crates/*/tests/security_*' 'docs/audit/*'
```
```
TASK V7 (v7-sec-<n>). First read docs/audit/SAFETY.md and obey it. Then print `cat .task-allowed` and `git status --short`.
Goal: prove or disprove the security claims P-<items> from docs/audit/ORACLE_CHECKS.md. New test files only (tests/security/ or crates/<crate>/tests/security_*.rs). No production code changes.
For each item: write a test or a reproducible command that attempts the ATTACK (not just the happy path), run it through tools/sb.sh (use `unshare -n` where the item says so), and record the exact output in docs/audit/SECURITY_PROOFS.md as PROVEN / DISPROVEN / NOT TESTABLE HERE, with path:line of the protection code. A DISPROVEN item is a P0/P1 finding: do not fix it here; describe the attack and the minimal fix. Commit "test(security): <items> (v7)". Report.
```

---

## V8 — Independent audit by Claude (you)
```bash
tools/make_audit_bundle.sh              # re-run so the bundle contains the audit docs and latest evidence
```
Open a new Claude chat, paste `docs/audit/CLAUDE_AUDIT_CONTEXT.md`, attach `audit_bundle.tar.gz` (and the contents of `docs/audit/` if the archive is over ~25 MB). Claude returns the findings table and fix prompts. Save the findings as `docs/audit/FINDINGS.md` (new task `v8-findings`, allowed `docs/audit/*`).

---

## V9 — Fix tasks (one finding per run)
```bash
tools/task.sh start v9-<F-ID> '<exact files from the finding>' 'docs/audit/FINDINGS.md'
```
```
TASK V9 (v9-<F-ID>): fix finding <F-ID> from docs/audit/FINDINGS.md. First read docs/audit/SAFETY.md and obey it. Then print `cat .task-allowed` and `git status --short`.
Finding: <paste the finding: evidence path:line, required behavior>
1. Write a failing test that reproduces the finding; run it; paste the failure.
2. Implement the smallest fix inside the allowed files only. If it needs another file, a new dependency or a design decision: STOP and ask.
3. Run the test; paste it passing. Then run `tools/sb.sh cargo xtask gate` ONCE; paste the last 10 lines.
4. Update docs/audit/FINDINGS.md: set <F-ID> status to FIXED with the test name. Commit "fix(<area>): <F-ID> <title>". Report.
```
P0/P1 in `lpdf-fs-safety`, worker/sandbox, security tools, redaction or engine-switcher: use Opus.

---

## V10 — Audit-green gate (Flash)
Preconditions: FINDINGS.md has no open P0/P1 (check it yourself).
```bash
tools/task.sh start v10-green 'docs/audit/*'
```
```
TASK V10 (v10-green). First read docs/audit/SAFETY.md and obey it. Then print `cat .task-allowed` and `git status --short`.
1. Run `tools/sb.sh cargo fmt --all -- --check`, then `tools/sb.sh pnpm --dir app-tauri check`, then `tools/sb.sh cargo xtask gate` (ONCE). Paste the last 10 lines of each. If anything fails: STOP and report; do not fix.
2. Write docs/audit/AUDIT_SUMMARY.md: per-tool verdict (from docs/TRACEABILITY.md and VERIFY_*.md), mutation scores, security proofs status, open P2/P3 findings, known limitations. Be honest; no marketing.
3. Commit "docs(audit): 11.7 summary". Do NOT tag; I will tag.
```
```bash
tools/task.sh check && tools/task.sh finish && git tag phase-11.7-audit-green
```

---

# PHASE 12.1 — RC1 in five small tasks
Only start after `phase-11.7-audit-green` exists.

## 12.1a — Verify Tauri v2 config keys (Flash; documentation only)
```bash
tools/task.sh start r1-keys 'protocol/ENGINE_NOTES.md' 'docs/audit/*'
```
```
TASK 12.1a (r1-keys). First read docs/audit/SAFETY.md and obey it. Then print `cat .task-allowed` and `git status --short`.
Goal: find, in the OFFICIAL Tauri v2 documentation/schema (use web search/fetch), the exact configuration keys for: Linux rpm bundle options (license, depends, recommends, files mapping of extra files such as /usr/bin/localpdf, desktop template), bundle.categories, bundle.fileAssociations with mimeType application/pdf, productName, mainBinaryName, bundle.targets. APPEND a dated section to protocol/ENGINE_NOTES.md with each key, its type, an example, and the URL. If a key does not exist write DOES NOT EXIST and what the alternative is. Also check which Tauri version is pinned in app-tauri/src-tauri/Cargo.toml and make sure the docs you cite match it. Commit "docs(engine-notes): tauri rpm keys (12.1a)". Report.
```

## 12.1b — tauri.conf.json, MIME, CLI install path (Opus; this is where to spend quota)
```bash
tools/task.sh start r1-conf 'app-tauri/src-tauri/tauri.conf.json' 'packaging/rpm/*' 'docs/FEDORA_INSTALL.md' 'docs/audit/*'
```
```
TASK 12.1b (r1-conf). First read docs/audit/SAFETY.md and obey it. Then print `cat .task-allowed` and `git status --short`. Use ONLY the keys verified in protocol/ENGINE_NOTES.md (12.1a).
1. tauri.conf.json: bundle targets ["rpm"]; binary name localpdf-studio; categories Office and Utility; file association application/pdf so the app appears in Dolphin "Open with…"; the desktop entry must pass the file to the app (%f or %F). Make the smallest edits with a diff; keep every other key unchanged.
2. Do NOT bundle pandoc, LibreOffice or any GPL tool. Declare them as RPM requires/recommends (pandoc; libreoffice-core, libreoffice-writer, libreoffice-calc, libreoffice-impress) if the verified schema supports it; otherwise document them as manual dependencies.
3. CLI: make the `localpdf` binary from crates/lpdf-cli install to /usr/bin/localpdf in the rpm (files mapping) or, if impossible, document the alternative in docs/FEDORA_INSTALL.md. State how you verified it.
4. Check (read the Rust code) that the app opens a PDF path given on the command line and in a second-instance launch. If it does not, DO NOT change Rust code in this task: write it as a finding in docs/audit/FINDINGS.md.
5. docs/FEDORA_INSTALL.md: install, upgrade, uninstall, `localpdf info file.pdf` example, sidecar dependency note, data/log/cache directories, checksum verification.
Commit "build(release): rpm config and kde integration (12.1b)". Report.
```

## 12.1c — QA checklist (Flash)
```bash
tools/task.sh start r1-qa 'docs/QA_VETTING_CHECKLIST.md'
```
```
TASK 12.1c (r1-qa). First read docs/audit/SAFETY.md and obey it. Then print `cat .task-allowed` and `git status --short`.
Write docs/QA_VETTING_CHECKLIST.md, an enterprise manual QA script for the INSTALLED rpm. Use protocol/07_FEATURE_MAP.md for the tool list and docs/audit/ORACLE_CHECKS.md for expected results. Sections: 1 environment record (Fedora version, KDE Wayland/X11, RAM, rpm checksum); 2 install/upgrade/uninstall; 3 one test case per tool (all 31 + Markdown→PDF, Pipelines, Settings) with ID, preconditions, exact steps, input file, expected result, pass/fail box, notes; 4 sidecars (Pandoc/Markdown, Office conversions, behaviour when a sidecar is missing: tile greyed with reason); 5 UI/UX (1000-page file responsiveness, cancel, drag-reorder, keyboard-only use, dark/light, HiDPI, Wayland and X11); 6 error handling (bad, zero-byte, truncated, encrypted PDF: plain message with code, no crash, no partial output); 7 offline test (network off, run all tools); 8 CLI `localpdf` (info, each tool, bad input exit codes); 9 recovery (kill -9 mid-job, relaunch); 10 Dolphin "Open with…" integration; 11 defect report template and severity definitions; 12 exit criteria (no open P0/P1, ≥95 % cases passed, every failure triaged). Do not invent features that are not in the feature map. Commit "docs(qa): vetting checklist (12.1c)". Report.
```

## 12.1d — Validate and build the RC (Flash; no source edits except formatting)
```bash
tools/task.sh start r1-build '*.rs' 'Cargo.lock' 'logs/*' 'docs/audit/*'
```
```
TASK 12.1d (r1-build). First read docs/audit/SAFETY.md and obey it. Then print `cat .task-allowed` and `git status --short`.
Run these ONE AT A TIME, in this order, paste the last 10 lines of each, and STOP at the first failure (do not fix; report):
1. tools/sb.sh cargo fmt --all
2. tools/sb.sh pnpm --dir app-tauri check
3. tools/sb.sh pnpm --dir app-tauri vitest run --no-file-parallelism --maxWorkers=1
4. tools/sb.sh cargo xtask gate        (ONCE; it already runs nextest)
5. SB_RELEASE=1 tools/sb.sh pnpm --dir app-tauri tauri build --bundles rpm
6. Locate the .rpm, then run: rpm -qpl, rpm -qp --requires, rpm -qp --info, rpmlint (if installed), desktop-file-validate on the packaged .desktop, sha256sum, and compare its size with the 1 GB limit. Paste results.
Note in logs/PROGRESS_LOG.md that RC1 was built with LTO off (not the shipping profile; no performance claims). Add the 11-point pre-commit checklist with honest YES/NO and evidence, advance logs/CHECKPOINT.md to Phase 12.2, commit "build(release): phase 12.1 fedora rpm packaging, kde dolphin integration, and rc1 qa prep". Do NOT tag. Print the ANCHOR block and report.
```
```bash
tools/task.sh check && tools/task.sh finish && git tag phase-12.1-rc1
```

## 12.1e — Clean-install check (Opus or you + Flash)
```bash
tools/task.sh start r1-install 'docs/RC1_INSTALL_REPORT.md'
```
```
TASK 12.1e (r1-install). First read docs/audit/SAFETY.md and obey it. Then print `cat .task-allowed` and `git status --short`.
1. Ask me before installing podman/toolbox. Create a fresh Fedora container of the same release as this host and run `dnf install ./<the rpm>`. Record dependency resolution, scriptlet output, `rpm -V`, `localpdf --version`, `localpdf info <sample.pdf>`, `localpdf` on a hostile sample (must exit non-zero with a clean message), then uninstall and list leftovers.
2. If a GUI session is available: launch the app, open a PDF from Dolphin ("Open with…"), run Merge + Compress + Protect, repeat with the network disabled, and check `ausearch -m avc -ts recent` for SELinux denials. If not available, list these as MANUAL steps for the QA engineer and do not mark them done.
3. Write docs/RC1_INSTALL_REPORT.md with PASS/FAIL per check and command output. Commit "docs(release): rc1 install report (12.1e)". Report.
```
Then run the human QA with `docs/QA_VETTING_CHECKLIST.md`. Defects → V9-style fix tasks → RC2.

---

## EMERGENCY RECOVERY
- Agent wrecked the working tree on a task branch: `tools/task.sh abort`, then `git switch main` (main is untouched until `finish`).
- Something bad reached main: `git log --oneline -n 15`, then `git revert <merge-commit>`; or compare with tag `pre-<task>`.
- Repo destroyed: `git clone ../localpdf-studio-protocol-<date>-pre-<task>.bundle restored`.
- Hook blocked a commit you really want: `ALLOW_BIG_DELETE=1 git commit ...` or add the path to `.allowed-deletions`.

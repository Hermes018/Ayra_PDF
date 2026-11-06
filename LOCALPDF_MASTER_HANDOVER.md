# LocalPDF Studio (`Ayra_PDF`) — Master Handover

**Snapshot:** 2026-11-06 17:00 Asia/Dhaka (UTC+6). This is the final handover from the borrowed Ubuntu PC.
**Owner:** Ayman Rahman (GitHub `Hermes018`, `aymanrahman30@gmail.com`)
**Repo:** `https://github.com/Hermes018/Ayra_PDF` (code lives in `master/`). The old local copy on the borrowed PC has been deleted; **clone fresh**.
**Written for:** any AI coding agent or human engineer taking over this project.

> ### ⚠️ Read first: what changed at the end of this session
> 1. **Git history was rewritten** (2026-11-06 ~16:50) so every commit's author and committer is `Ayman Rahman <aymanrahman30@gmail.com>`. It replaced the borrowed PC's identity (`kazrkb`) and an `Antigravity` agent identity. **Every commit hash changed.** Hashes quoted anywhere (`CHECKPOINT.md`, `ARGON_TASKS.md`, ADRs, older sections of this file) are **stale**. Use **branch and tag names**, which were all rewritten consistently.
> 2. New branch heads after the rewrite: `main` = `0402344`, `wt/fixes` = `8e1386e`, `wt/release` = `bc0c9ef`, `wt/engines` = `879fb58`, `track/android` = `098f44e`.
> 3. Work stopped at **79% weekly Claude usage** (by choice, to stay under 80%). The weekly limit resets **Sunday 2026-11-08 04:00 Asia/Dhaka**.
> 4. **`v0.2.0-rc1` is NOT built or tagged.** The exact remaining steps are in `master/docs/ARGON_TASKS.md` §0 and in §20 of this file.
> 5. After cloning, set the identity for the repo: `git config user.name "Ayman Rahman"` and `git config user.email "aymanrahman30@gmail.com"`.

---

## 0. How to use this document

### 0.1 Trust order (when sources disagree)
1. **The code and git history**: always the final truth.
2. **`master/logs/CHECKPOINT.md`** and **`master/logs/STATE_AUTO.md`**: live state, updated continuously by the agents.
3. **Repo docs:** `master/docs/` (ADRs, `DESIGN_LANGUAGE.md`, `PROTOTYPE_SCOPE.md`, `ENGINE_REUSE_MATRIX.md`, `UPSTREAM_MANIFEST.toml`, `UPGRADING.md`, `RAM_BUDGET.md`, `CRAFT_ECOSYSTEM_SURVEY.md`, `DRAWBOARD_FEATURE_CHECKLIST.md`, `STUDIO_UI_UX_OVERHAUL_SPEC.md`), plus `master/logs/` (`BUGS.md`, `ERROR_LOG`, `DOGFOOD.md`, `ROUTING.md`, `AGENT_ACTIVITY.md`).
4. **This document**: the map, the reasons and the methods. It was written from the project conversation, not by reading the repo, so exact paths and numbers may have drifted. Check them against items 1–3 before acting.

> Lesson already learned once: an earlier handoff claimed "113/113 verified, gates green." The code showed 111, two faked features, and a gate that had never passed. **Never trust a claim you haven't checked against code, tests or a gate run.**

### 0.2 First actions for a new agent (in order)
1. Read `master/CLAUDE.md` (or `AGENTS.md`), then `logs/CHECKPOINT.md`, `logs/STATE_AUTO.md` and `logs/BUGS.md`.
2. Run `git log --oneline -20`, `git worktree list` and `git status` in each worktree.
3. Run `tools/verify.sh` on `main` and confirm it's green before changing anything.
4. Check §2 (current state) against what you found, and write the differences into CHECKPOINT.
5. Continue from **§3 Milestones**, using the next action listed in CHECKPOINT.

---

## 1. Product vision and non-negotiables

### 1.1 What we're building
**LocalPDF Studio**: a fully offline, local-only, professional desktop PDF workstation plus a CAD markup studio, at the level of **Drawboard PDF Pro / Bluebeam Revu**, free and with no limits.

- **Rust core:** PDF parsing and structure editing, organize, annotate, forms, security, true redaction, basic convert, ML OCR, measurement math and atomic file writes, mostly through `pdfcraft`.
- **Tauri v2 shell + Svelte 5 (runes) + TypeScript + Vite** frontend.
- **Spatial canvas** with stylus inking, CAD-style snapping (OSNAP), measurements, hatching and markups.
- **Sidecars** (max 1 GB total) only for Office XLSX/PPTX/complex DOCX conversion (LibreOffice) and pandoc. Desktop only.
- **Targets:** Linux (primary; Wayland/X11 WebKitGTK), Windows (W1–W4 track exists), **Android (planned, ADR-019, paused)**.
- **Design direction:** Drawboard Pro-class features, Apple-style design language, **every control has a real purpose**, work done in phases.

### 1.2 The ANCHOR (8 rules; never break them)
1. Offline, local, no limits. The Rust core does the heavy work natively; sidecars only for Office/complex DOCX, max 1 GB.
2. Milestones are gated: no milestone counts until the gate (`cargo xtask gate` / `tools/verify.sh`) is green.
3. **No network at runtime, no telemetry, no remote assets** (fonts, icons, scripts). No GPL/AGPL code linked in; sidecar binaries kept separate are allowed. Bundle ≤ 1 GB.
4. **Outputs are atomic and go to a new file by default. Originals are never touched. Every write is verified afterwards.**
5. **Verify an API from its source before using it. Never invent function names.**
6. No `unwrap`/`expect`/`panic!`/`todo!`/`unsafe` in non-test Rust (`unsafe` only in `engine::mmap`). Typed errors. Heavy work runs in worker jobs.
7. Gate green → log → commit → tag. **Red gate = stop.**
8. Stuck three times on the same thing → write to `ERROR_LOG` and ask the owner. A scope or decision change → an ADR plus the owner's approval.

---

## 2. Current state (as of 2026-11-06 ~15:00)

### 2.1 Done
- **Restructure (ADR-016):** code moved into `master/` (commit `986349c`).
- **Gate made green for the first time:** fmt, clippy, nextest, vitest, `cargo deny`, `cargo audit` and protocol-lint all pass.
- **ERR-005** (Markdown → PDF broken when pandoc is installed): fixed. Pandoc now outputs DOCX and the LibreOffice sidecar converts it to PDF.
- **ERR-006** (`reqwest` HTTP client in Windows builds): fixed. It came from `winprint`'s build script downloading PDFium. Fixed with a **local patched `winprint`** (ADR-017) that must be re-synced when pdfcraft's printing dependency changes. `rustls` remains on Windows only, for certificate-store signing; it's TLS, not HTTP.
- **UI-BUG-006** (top bar overflow at 1200 px): fixed and verified with WebDriver. Below ~1100 px, labels turn into icons. With no file open, the app shows "Open a PDF to begin" instead of phantom pages.
- **Feature checklist made honest:** 111 DONE, 2 PARTIAL (LaTeX = regex to Unicode text; handwriting = per-stroke shape guessing of O/l/-/.), 6 DEFERRED (cloud and multiplayer, banned by the offline rule). The checker enforces PARTIAL entries and their "Actually:" notes.
- **Craft ecosystem survey** (`docs/CRAFT_ECOSYSTEM_SURVEY.md`): all 10 repos are MIT OR Apache-2.0, and engine crates are separate from their egui UIs. The figcraft engine depends on `ureq`, so it's rejected.
- **ADR-018 engine reuse:** revised to adapters plus pinned dependencies (see §5). Reuse matrix, golden fixtures and the `lpdf-geom` adapter are merged. The hostile-file guard is merged.
- **Sprint triage:** `PROTOTYPE_SCOPE.md` with 12 golden flows, an E2E harness (tauri-driver/WebDriver), `BUGS.md`.
- **All 10 P0s (BUG-011..020) fixed and merged.** This includes ERR-007/BUG-015 (hatching filled the bounding box instead of the polygon, and patterns were identical). Now fixed with polygon clipping, our own parametric patterns, and tests for concave shapes, holes and distinct patterns.
- **Fixes round 5:** closing tabs leaves no saved state behind; inserts are blocked during load so Undo can't wipe restored strokes; save refuses when the page layout is missing; drawing state resets completely on tab switch, with a test that catches newly added fields.
- **BUG-022:** perimeter now includes the closing side (216 → 288 pt), plus a test that open polylines do **not** get a closing segment.
- **Labs setting** (off by default): LaTeX, ink-to-text and split view (KI-004). Their wording was made honest.
- **Infrastructure:** per-worktree target folders (ERR-008), disk and RAM guards, capped sccache, compaction snapshot and restore hooks (ERR-009: output capped around 9 KB), 13 agent definitions with model routing, the lead-role rule, `ROUTING.md` with a stats script.
- **UI audit (A2):** extended to all 37 `.svelte` files (385 violations in 30 files at first count), with a strict mode via `xtask/ui-strict.txt`, a pixel-value allowlist (`xtask/ui-px-allowlist.txt`, a reason required per entry), key listeners moved to `commandEngine` (behaviour fixed by tests first), labels on icon buttons, and the "pixel-perfect" overclaim removed.
- **Design language spec (B)** and the UI sweep (C): the sweep finished at 14:44 within its cut-down scope. Check `docs/DESIGN_LANGUAGE.md`, `ui-strict.txt` and `UI_BACKLOG.md` for what was actually covered.
- **DOGFOOD AppImage** built (119.9 MB, version still labelled 0.1.0, no LTO; KI-005). Copied to `~/LocalPDF-dogfood/`. Flows 1, 2, 4, 6 and 11 pass on main.

### 2.2 In progress at the snapshot
- **A full 12-flow E2E rerun on main**, including the 1-hour soak. Its results decide any remaining P0s.
- **Real-world PDF run** (DF-xx files from `~/Desktop/Ayra_PDF/PDF demo test`, worked on as copies, anonymised) → `logs/DOGFOOD_AUTO.md`.
- **The owner's manual dogfood testing** → `logs/DOGFOOD.md`.

### 2.3 Open items
- **P1:** BUG-028, 029, 030, 031 (the E2E harness leaked a message-bus process; fix: process groups with kill-on-exit plus a no-leftovers check), 032, 033. Earlier P1s BUG-021..024 and P2s BUG-025..027: check their status in `BUGS.md`.
- **Known issues:** KI-001..007 (`KNOWN_ISSUES.md`). KI-005: version label and LTO. KI-006: offline AppImage check not yet proven; scheduled for packaging.
- **R3 licence question:** it no longer blocks ERR-007. The 3-line explanation of what it still affects is still owed to the owner.
- **Engines checkpoint 3:** parked (tests not run).
- **Release WIP `6fa23a8`:** may have been merged since; check.
- **Nothing is pushed.** Everything after `2fa8927` plus about 9 tags is local only. The owner pushes after `gh auth login`: `git push origin main --tags`.

### 2.4 Commits and tags
**Commit hashes from before the 2026-11-06 rewrite are stale; navigate by branch and tag.** Useful tags (all rewritten): `phase-r1-green`…`phase-r4-green`, `engines-cp1-green`, `engines-cp2b-green`, `fixes-p0-green`, `fixes-err007-green`, `fixes-bugNNN-green`, `release-cp2-green`, `release-bug031-green`, `release-bug046-green`, `release-bug048-green`, `ui-audit-green`, `ui-library-green`, `sweep-g1..g8-green`, `sweep-mainflows-green`, `ui-sweep-complete`, **`packaging-prep-a-green`** (the last verified point on `main`). Older history tags: `phase-0.x … phase-12.x`, `v0.1.0-rc2..rc4`, `pre-12.2-flash-backup`.

### 2.5 Final state at 17:00 (this supersedes §2.1–2.3 wherever they differ)
**On `main` and verified (up to tag `packaging-prep-a-green`):**
- Everything in §2.1.
- The UI sweep is complete within its scope (tags `ui-library-green`, `sweep-mainflows-green`, `ui-sweep-complete`): shared component library, component gallery (dev only), main-flow screens on the shared components, and the strict audit list.
- **Packaging prep A:**
  - version **0.2.0** in all three places, with a test that they match;
  - a new **About** screen (version, build hash, licences, offline statement, file locations);
  - **Reset preferences**, which keeps the licence and was verified at Opus-high because it touches settings saving.
- BUG-031 (harness process leak), BUG-046 and BUG-048 are fixed. BUG-050..058 and BUG-060 are fixed (see the tags).
- **BUG-063 closed:** the real-PDF "regressions" were the dogfood driver still using pre-sweep button labels. The app was fine.
- Commits on `main` after `packaging-prep-a-green` are docs and logs only: `ARGON_TASKS.md`, `CHECKPOINT.md`, BUG-061/062 notes.

**Unverified work in progress (pushed, NOT merged; review before merging):**
- `wt/fixes`: harness work for **BUG-062** (driver disconnect: a counted retry policy, written test-first with a fake driver: one disconnect while the app is alive warns, a dead app fails, two in a row fail, more than 1% hiccups fail) and **BUG-061** (the soak must close the tabs it opens; then a 20-min soak plus 5-min idle compared with the earlier no-close data). Possibly partial.
- `wt/release`: packaging B1 started. Includes **`tools/sbom_npm.py`**, which is **UNREVIEWED**. It fixes `cargo xtask sbom` listing only direct npm dependencies and hard-coding 0.1.0. It must be reviewed by a strong model before any SBOM ships. **The fat-LTO RAM measurement was cancelled midway**, so no 0.2.0 packages exist yet.
- `wt/engines`: engines checkpoint 3, parked, tests not run.
- `track/android`: unchanged ADR-019 draft.

**Open bugs:**
- **BUG-061:** soak memory growth of ~13 MB/min with tabs left open. Classification rule: P0 only if a normal 2-hour session (~60 saves) would exceed ~1 GB; otherwise P1 plus a KNOWN_ISSUES entry, and it doesn't block the RC.
- **BUG-062:** driver disconnects during the soak; the app stays alive.
- P1s: BUG-028..030, 032, 033, plus any listed in `BUGS.md`. P2s are in `BUGS.md`.

**Not done:** packaging B1 (LTO measurement; AppImage, .deb, .rpm; SHA256SUMS; SBOM; KI-006 offline AppImage proof) and B2 (clean install in a fresh HOME with no network, network-isolated run, README, QUICKSTART, CHANGELOG, KNOWN_ISSUES), the 1-hour soak on the RC package, the RC verifier, and the `v0.2.0-rc1` tag. The owner's own hands-on dogfood testing hasn't happened yet.

**Decisions made in the last session:**
- LTO: measure the peak first; if fat LTO breaches the RAM budget, use `lto = "thin"` with `codegen-units = 1`.
- Clean-install testing without Docker is acceptable: a fresh HOME in a no-network sandbox (bwrap/unshare).
- If weekly usage reaches 88% before the RC verifier, skip the 1-hour soak (keep the 20-min result).

---

## 3. Goals, milestones and roadmap

### 3.1 The current sprint: Linux prototype
**Deadline: Saturday 2026-11-07 23:59 Asia/Dhaka.** The sprint is under a **feature freeze**: only bug fixes, hiding features in Labs, design-consistency work, packaging and tests are allowed.

| When (Asia/Dhaka) | Milestone | Done means |
|---|---|---|
| Fri 08:00 | Baseline report | ✅ done |
| Fri 18:00 | DOGFOOD build | ✅ done early (09:09) |
| Fri evening | Owner dogfoods real PDFs | issues logged in `DOGFOOD.md` |
| Sat 14:00 | Dogfood P0s fixed | triaged, fixed test-first, verified |
| Sat morning | Packaging (§12) | AppImage + .rpm + .deb, checksums, docs, offline check |
| Sat 18:00 | `v0.2.0-rc1` | full gate green, 12/12 flows green, packages smoke-tested; after this only P0 fixes |
| Sat 23:59 | `v0.2.0-prototype` tag | released locally; owner pushes |

**If a milestone slips: cut scope (move features to Labs, or defer an engine swap). Never move the deadline.**

### 3.2 Roadmap after the prototype
1. **Push and back up** (owner).
2. **Finish ADR-018:** swap the remaining engines to pinned craft crates behind the adapters, in this order: cadcraft geometry and measurement math → vectorcraft pathops (true partial eraser) → cadcraft snapping (WASM benchmark) → designcraft compose (text reflow). Add `cargo xtask upstream-check`.
3. **Work through `UI_BACKLOG.md`:** sweep the remaining screens onto the shared components; add E2E smoke tests for every screen.
4. **Turn the PARTIAL features into real ones, or cut them by ADR:** an offline LaTeX vector compiler (consider a vetted MIT/Apache math typesetter) and real handwriting recognition (an offline model, if licence and size fit).
5. **Android (ADR-019 → approval → build).** See §13.
6. **Windows track refresh:** the winprint patch sync, the installer pipeline, `cargo deny` on the Windows target.
7. **Hardening:** fuzzing the PDF parser with hostile files, performance budgets (time to first page, memory per page), accessibility pass.

---

## 4. Architecture

### 4.1 Repository layout (check against the repo)
```
master/
  app-tauri/                 # Tauri v2 app
    src/                     # Svelte 5 + TS frontend
      lib/core/              # centralized controllers (see 4.2)
      lib/components/        # UI components (shared library + screens)
    src-tauri/               # Rust app crate (commands, studio.rs, platform code)
  crates/                    # internal Rust crates (see 4.3)
  xtask/                     # cargo xtask: gate, studio-audit, feature-map-check, protocol-lint, ui-strict.txt, ui-px-allowlist.txt
  tools/                     # sb.sh, run.sh (scheduler), verify.sh, snapshot-state.sh, restore-context.sh, routing-stats.py
  docs/                      # ADRs, specs, matrices, manifests
  logs/                      # CHECKPOINT, STATE_AUTO, BUGS, ERROR_LOG, DOGFOOD*, ROUTING, AGENT_ACTIVITY, FLAKY, runs/
  e2e/                       # WebDriver harness, corpus, out/
  .claude/                   # agents/*.md, settings.json (hooks)
  CLAUDE.md                  # ≤ ~200 lines of rules
  KNOWN_ISSUES.md, CHANGELOG.md, THIRD_PARTY_NOTICES.md, UPGRADING.md
```

### 4.2 Frontend controllers (`app-tauri/src/lib/core/`): one owner per concern
| Controller | Owns | Rules |
|---|---|---|
| `themeContract.svelte.ts` | all design tokens (`var(--…)`) | no raw `#hex`/`rgb()`/px in `.svelte` UI chrome; enforced by the audit |
| `toolEngine.svelte.ts` | the active tool and tool settings | components read and set tools only through it |
| `viewportEngine.svelte.ts` | pan, zoom, affine matrices, page↔screen transforms | every coordinate conversion goes through it |
| `commandEngine.svelte.ts` | **all** shortcuts, undo/redo, command routing | **no** `onkeydown` / `addEventListener('keydown')` inside components |
| `inkEngine.svelte.ts` | stylus input → variable-width Bézier outlines (Catmull-Rom, pressure normals), RDP simplification, smart shapes | runs at pointer speed in the WebView; never a Tauri IPC call per pointer event |
| `snapEngine.svelte.ts` | OSNAP spatial grid (`cellSizePt=32`), priority endpoint > intersection > midpoint > perpendicular > grid | per-event budget < 2 ms |
| `measureEngine.svelte.ts` | calibration (`unitsPerPt`), length/polylength/area (Shoelace)/perimeter/volume/angle, ruler | units: mm, cm, m, in, ft; architectural presets |
| `selectionEngine.svelte.ts` | marquee, lasso (point-in-polygon), 8-handle transform + rotate, group, align, flip, z-order, format painter | undoable through commandEngine |
| `shapeAndHatchEngine.svelte.ts` + `HatchDefs.svelte` | shapes, revision clouds, SVG hatch patterns | geometry comes from the `lpdf-geom` adapter; patterns from our own parametric definitions |
| `insertEngine.svelte.ts` | text boxes, callouts, stamps, signatures, images, the (Labs) LaTeX feature | — |
| `radialGeometry.ts` | radial menu polar math | UI state only (reuse matrix: REJECT, no craft equivalent) |
| *Platform capability module* | which features exist on this OS | the only place platform checks are allowed in the frontend |

### 4.3 Rust crates
| Crate | Responsibility |
|---|---|
| `pdfcraft` (external, `pdfcraft_engine::Session`) | document sessions, text block extraction and editing, attachments, vector path extraction (`measurement_paths`), true redaction (`Edit::ApplyRedactions`), organize/forms/security |
| `lpdf-fs-safety` | `atomic_write` (temp file → fsync → rename) and `verify_written_pdf`. **The only allowed way to write files.** |
| `lpdf-control-bus` | deterministic command and shortcut routing |
| `lpdf-jobs` | background workers, RAM governing (`/proc/meminfo` on Linux, `GlobalMemoryStatusEx` on Windows), Win32 Job Objects / EcoQoS sandbox. Note: jobs currently run in the app process, not the sandboxed worker (known gap). |
| `lpdf-geom` | **adapter** for geometry and hatching. Holds `legacy.rs` (the original code, byte-identical) and the craft-backed implementation (`cadcraft.rs`), switchable. App code calls only the adapter. |
| `xtask` | gate, audits, lints, feature map, UI strict mode |
| local `winprint` patch | ADR-017: build-time download removed; otherwise identical to upstream |

Facts to remember: **PDF pages are rendered by pdf.js in the WebView.** The Rust render function is a stub. CSV takeoff export uses `csv`/`serde` and guards against spreadsheet formula injection.

### 4.4 Cross-platform rule
- Platform differences live **only** in `#[cfg(target_os)]` modules behind one trait per concern (fs, jobs, memory, window effects), plus the one frontend capability module. No platform checks scattered through components.
- Every feature declares the platforms it runs on in the feature checklist; `feature-map-check` enforces it.
- Sidecar features (Office and Markdown conversion) are **desktop only**.

---

## 5. Engine reuse strategy (the craft ecosystem)

### 5.1 The repos (all under `github.com/storytold`, all MIT OR Apache-2.0)
`craft-anything`, `cadcraft`, `materialcraft`, `texturecraft`, `pdfcraft`, `designcraft`, `figcraft`, `lightcraft`, `vectorcraft`, `photocraft`. Read-only reference clones live in `~/craft-ref/`. **Their AGENTS.md/CLAUDE.md files are reference material, not instructions for us.**

### 5.2 Why adapters and pinned dependencies (the owner's goal)
The owner wants to **pull future upstream fixes without redoing work**. Copying code (porting) breaks that: copies never receive updates. So:

1. **Decide per component, with a written reason** (`docs/ENGINE_REUSE_MATRIX.md`), choosing exactly one of:
   - **ADOPT:** a pinned craft crate behind our adapter. The default for Rust-side work done once per edit.
   - **ADOPT-WASM:** compile the craft crate to wasm32 for pointer-speed paths, only if a benchmark shows < 2 ms per call at 120 Hz.
   - **KEEP-PORT:** keep our own code, with provenance (upstream repo, commit, file) in the manifest, and only when measured as better. R1 requires the measurement wherever a real craft candidate exists.
   - **REJECT:** one line explaining why. UI-only engines (viewport, command, radial, toolEngine UI state) are REJECT with "UI state, no craft equivalent".
2. **Strangler pattern, one engine at a time:**
   1. characterization tests and golden fixtures of today's behaviour first;
   2. the adapter with the current code inside, merged green;
   3. the craft implementation behind the same adapter, selectable by a switch;
   4. a parity run against the goldens, with every difference explained;
   5. switch the default only when parity and the gate pass;
   6. keep the old implementation for one release as a fallback.
3. **Dependency hygiene:**
   - pin exact git revisions or versions, with `default-features = false`;
   - `cargo deny` bans HTTP/TLS clients and allows only MIT/Apache;
   - `cargo vendor` for offline, reproducible builds;
   - `docs/UPSTREAM_MANIFEST.toml` records repo, revision, licence, adapter and last-checked date for every adopted crate, port and patch (including winprint);
   - `THIRD_PARTY_NOTICES.md` holds the licence texts and ships in the packages.
4. **Never edit upstream data.** For example, the AR-CONC pattern keeps its upstream definition untouched; our ×1/16 is a **default display scale in the adapter**, user-adjustable.
5. **Upgrade procedure (`UPGRADING.md`):** one branch per engine bump → change the pinned revision → re-vendor → characterization and parity tests → full gate → verifier → log → merge.

### 5.3 Candidate mapping
cadcraft → snapping, geometry, intersection, units and calibration, measurement math, hatching · vectorcraft (pathops, curve fitting, stroke outline) → partial eraser, ink · designcraft (compose) → text reflow · materialcraft/texturecraft → pattern definitions, only if relevant · photocraft → signature background removal and image stamps, only if clean · figcraft engine → REJECT (`ureq`) · lightcraft/craft-anything → REJECT unless a clear need appears · pdfcraft → check it before keeping any of our own Organize/Forms/Security/OCR/Redaction code.

---

## 6. Feature behaviour specification

Repo specs (`PROTOTYPE_SCOPE.md`, `DRAWBOARD_FEATURE_CHECKLIST.md`, `STUDIO_UI_UX_OVERHAUL_SPEC.md`, `DESIGN_LANGUAGE.md`) take precedence over this section. This section sets the professional standard each feature must meet.

### 6.1 Rules for every feature
- **Data safety:** every save creates a new file through `atomic_write` and `verify_written_pdf`. The original is never modified. A failed verification means no output file and a clear error.
- **Undo:** every user-visible change is undoable through commandEngine and is grouped sensibly (one stroke = one undo step).
- **Long jobs** (OCR, convert, large saves): run in a worker with a progress bar, a working Cancel, and no UI freeze.
- **Errors:** say what happened and what to do, with a retry where it makes sense. Never a raw stack trace; never a silent failure.
- **Disabled controls** explain why (a tooltip such as "Open a PDF first").
- **No overclaiming:** labels describe what the feature actually does. PARTIAL features stay in Labs.
- **Offline:** no network calls, no remote fonts or assets, ever.
- **Platforms:** declared in the checklist; hidden through the capability module where unsupported.

### 6.2 The 12 golden flows (acceptance = release criteria)
1. **Install and cold launch:** the package installs, the app launches in < 3 s, no console errors, and no retries masking a failed launch.
2. **Open:** small, 200+ page, scanned, encrypted (password prompt) and malformed files. A malformed file shows a clear error and never crashes; hostile files are refused, or opened in a clearly labelled repair mode.
3. **Navigate:** pan, zoom, trackpad and touch pinch around the focal point, thumbnails, multiple tabs (falls back to one document at a time if tabs fail review). View-state checks are as strict as the spec.
4. **Ink:** Pen A/B with pressure, a multiply-blend highlighter, the eraser (whole and partial), undo/redo.
5. **Shapes and hatching:** line/arrow/polyline/rectangle/ellipse/polygon, revision cloud. Hatching is **clipped to the polygon** (concave shapes and holes included), and every pattern looks distinct.
6. **Measure:** two-click calibration → length, area, angle → takeoff ledger → CSV export (formula-injection safe). Perimeter includes the closing side for closed shapes only.
7. **Insert:** text box, callout, sticky note, stamp, signature (stylus, webcam with thresholding, or image).
8. **Redact:** after saving, the redacted text must be **gone from the file**, confirmed by an independent extractor (`pdftotext`/`qpdf`), not just visually covered.
9. **Edit PDF text in place:** text blocks are highlighted and edited inline; reflow is correct.
10. **Organize:** reorder, rotate, delete, merge; no leftover temp files.
11. **Save and reopen:** in our app **and** in poppler/pdf.js, markups survive, the original is untouched, and a new file exists.
12. **Soak:** 1 hour of scripted random interaction, no crash, memory stays bounded.

### 6.3 Feature families
- **Organize:** page operations are previewed before applying, and the output is a new file. Multi-page selection supports "All, Even, Odd, 1-5" ranges.
- **Annotate and markup:** each markup type has style properties (colour, width, line style, endings, opacity) edited through a single properties panel. Author and timestamp are recorded for the ledger.
- **Measurement and takeoff:** the scale is per page and shown on screen. Uncalibrated measurements are labelled "uncalibrated". The ledger is sortable and filterable (author, type, page) and exports to CSV.
- **Forms:** fields are detected; values survive save and reopen in external viewers.
- **Security:** passwords and permissions are applied on save into a new file. No key material is logged.
- **Redaction:** two steps, marking and then applying with a confirmation that lists what will be removed. The output is verified with a text extractor.
- **Convert:** PDF ↔ images and basic formats natively. Office and Markdown go through sidecars on desktop only; on platforms without them the options are hidden. The job is cancellable.
- **OCR:** an offline ML model, run as a worker job, producing a searchable text layer in a new file.
- **Text editing:** native text blocks (`studio_extract_text_blocks` / `studio_edit_pdf_text_block`), edited in place. After the swap, reflow uses designcraft compose.
- **Labs:** LaTeX ("Unicode math text" in reality), ink-to-text (basic shape guessing), split view. Off by default and labelled honestly.

---

## 7. UI / UX design language

**Source of truth: `master/docs/DESIGN_LANGUAGE.md`.**

### 7.1 Principles
- Apple-style clarity: content first, chrome stays quiet. **Every control has a real purpose**; no decorative UI.
- Progressive disclosure: common actions visible, advanced ones one step away (popovers, the radial menu, the properties panel).
- Everything that can be done with a shortcut can also be done with a pointer or touch. Shortcuts are an accelerator, never the only way.
- Disabled controls explain why. Empty states teach the next step ("Open a PDF to begin" with one primary button).
- Honest wording: sentence case, a verb on every button ("Save copy", not "OK"), no marketing words such as "pixel-perfect".

### 7.2 Tokens (`themeContract.svelte.ts`)
Colour (light and dark, semantic: surface, text, accent, danger, warning, success, focus), type scale, spacing scale, radii, elevation/frosted glass, motion durations. **UI chrome uses tokens only.** Canvas and geometry code is exempt through `ui-px-allowlist.txt`, with a reason for each entry.

### 7.3 Shared component library
Button, IconButton (always with an aria-label and tooltip), Input, Select, Toggle, Dialog/Sheet, Toast, EmptyState, ErrorState, Progress (with cancel), Toolbar, Sidebar section. Behaviour and accessibility come from a vetted **headless Svelte 5 library** (e.g. Bits UI / Melt UI, MIT, offline) where it fits; styling comes from our tokens. A dev-only **component gallery** route shows every state (default, hover, focus, disabled, error, loading) in light and dark; one screenshot of it is the visual review. The gallery is excluded from release builds.

### 7.4 App shell (Studio, as built in U1–U5)
- **Left sidebar:** collapsible (`Ctrl+B`, ◀ chevron, floating ☰).
- **Top command bar:** 4 frosted-glass zones: workspace/mode · viewport/zoom/overlay · history/OSNAP/ruler · document actions/ledger. It must never exceed the canvas width; labels turn into icons below about 1100 px.
- **Right tool rail:** 7 families (nav, ink, erase, shapes, review, measure, insert), 43 tools, popover drawers with icons, labels, shortcut badges and favourite stars.
- **Radial menu:** 8 wedges, polar tracking, a locked outer-ring fan, a centre ✕, click outside to dismiss, `/` toggles it.
- **Inline PDF text editor:** dashed hotspots in "Edit PDF Text" mode, with an in-place floating editor.
- **Canvas layers:** `#pdf-layer`, `#vector-layer`, `#wet-layer`.

### 7.5 Screen standards (every screen, including the older non-Studio ones)
Shared components only · tokens only · sentence-case wording · an empty state and an error state · keyboard reachable with a visible focus · works at 1200×800 and 1600×1000, light and dark · an E2E smoke test (open → screenshot → no console errors) · listed in `ui-strict.txt` once swept, so it can't regress.

### 7.6 Sweep procedure
Main-flow screens first. Replace one-off elements with shared components; no behaviour changes. Timebox each screen; anything needing a real redesign goes to `UI_BACKLOG.md`. Compare screenshots before and after. The reviewer checks them against `DESIGN_LANGUAGE.md`.

---

## 8. Engineering standards

### 8.1 Rust
- No `unwrap`/`expect`/`panic!`/`todo!`/`unsafe` outside tests (`unsafe` only in `engine::mmap`). `protocol-lint` enforces it.
- Typed error enums (`thiserror`-style) at crate boundaries; messages a user can act on.
- All file mutations through `lpdf_fs_safety::atomic_write` + `verify_written_pdf`.
- Heavy work in `lpdf-jobs` workers, cancellable, with progress reporting.
- Platform code only in cfg modules behind traits.

### 8.2 Svelte / TypeScript
- Svelte 5 runes (`$state`, `$derived`, `$effect`). State lives in the core controllers, not scattered through components.
- No local keydown listeners: everything goes through commandEngine.
- No raw colours or pixel values in UI chrome.
- Pointer-speed code stays in the WebView (TS or WASM); never IPC per pointer event.

### 8.3 Testing
- **A failing test first for every bug** (red → fix → green).
- **Characterization goldens** before any engine swap. A golden is updated only with a written note explaining why the new value is correct (example: perimeter 216 → 288).
- E2E with tauri-driver/WebDriver for the 12 flows and screen smoke tests. **The runner must fail when a requested flow doesn't exist** (it used to exit 0). Launch retries must not hide a failed cold launch. Corpus hashes must not depend on the host machine.
- Flaky tests: an E2E test that passes only on rerun goes in `logs/FLAKY.md` and gets reported. **Never loop on retries.**
- Real-file runs use copies only; personal files never enter the repo.

### 8.4 Git and process
- One feature or fix per branch/worktree; merge to `main` only after a green verify and a verifier sign-off.
- **History is never rewritten** (no amend, rebase, force push or reset --hard). A staging mistake stays and gets a note.
- Commit → tag at milestones. **Agents never push and never handle tokens**; the owner pushes.
- A scope or decision change requires an ADR plus the owner's approval. Approved exceptions are logged (for example the one-time `rm -rf target/debug`).

---

## 9. Quality gates and verification

| Gate | What it runs | When |
|---|---|---|
| `tools/verify.sh` | **the single definition of green**: fmt, clippy, nextest, vitest, svelte-check, `cargo deny`, `cargo audit`, protocol-lint, studio-audit (strict on the listed screens), feature-map-check | before every merge and at every milestone |
| E2E flows | the 12 flows, or the current tier's subset | after P0 batches, before DOGFOOD and the RC |
| Network isolation | the app run under `unshare -n` / firejail `--net=none`: all flows work, zero connection attempts | packaging and the RC |
| Redaction check | independent extractor confirms removal | every redaction change, and the RC |
| Package smoke test | install into a clean container or user, launch, flows 1–2 | every package build |
| Verifier sign-off | **a fresh agent** given only the spec/ADR, the diff and the gate commands; it runs them itself | every merge |

The verifier never sees the builder's reasoning, so it can't simply agree with it. **Save-path, atomic-write, redaction and RC changes are always verified at Opus-high level, overriding any throttle tier.**

---

## 10. Agent system

### 10.1 Organisation
- **Lead / orchestrator** (the main session): owns `main`, gates, triage, merges, commits and checkpoints. **It orchestrates only:** no reading logs, searching code, editing files or running builds itself, except one-line commands. It plans, assigns work, reads reports of ≤ 150 words, decides, merges and commits.
- **Builders** (max 3, now 1–2 under the tier), each in its **own git worktree and branch with its own target folder**, split by file ownership so they never touch the same files: `fixes`, `engines`, `release`.
- **Read-only agents** (can run in parallel): explorer, log-reader, status-snapshot, reviewer, adversarial tester, provenance auditor.
- **Verifiers:** a fresh context for each merge or RC.
- **Human (owner):** approves ADRs and decisions, dogfoods, pushes.

### 10.2 Routing table (`.claude/agents/*.md`, 13 files, live since the 08:00 restart)
| Model / effort | Roles |
|---|---|
| Haiku 5.5, effort low | explorer, log-reader, status-snapshot (mechanical work, output checked by a script or a later step) |
| Sonnet 5.5, low | checkpoint-writer, bug-triager |
| Sonnet 5.5, medium | reviewer, provenance-auditor, release-builder |
| Sonnet 5.5, high | fixes-builder |
| Opus 5.5, medium | lead (the main session, set by hand), engines-builder |
| Opus 5.5, high | merge-verifier, rc-verifier, escalation |
**Never use Fable.** The main session's model doesn't switch on its own; the switching happens because the lead delegates work to subagents.

### 10.3 Escalation and self-tuning
- Haiku fails once → redo on Sonnet low. Sonnet fails the same task twice → redo on Opus high, then return to normal routing.
- Every escalation is logged in `logs/ROUTING.md`. At each milestone, compute the escalation rate per task type: above 25%, move that type up a tier; never escalating over 10+ tasks, try moving it down. (Known event: a Haiku agent failed to deliver its final report once; it was logged, and the lead read the summary file directly.)

### 10.4 Context durability
- **`logs/CHECKPOINT.md`** (≤ ~5,000 characters, updated after **every** state change) holds: the sprint goal and deadline, milestone status, each worktree's state, running agents, P0/P1 owners, the next 3 actions, open decisions, approved exceptions, the current tier and the pace rule. `logs/checkpoints/<worktree>.md` exists per builder; older history goes to `CHECKPOINT_HISTORY.md`.
- **PreCompact hook** → `tools/snapshot-state.sh` writes `STATE_AUTO.md` **from facts, not memory**: git worktrees, branches and commits, open P0/P1, running processes, free disk and RAM, sccache stats, recent activity. It's read-only, takes under 5 s, and always exits 0.
- **SessionStart hook** (compact, resume, startup) → `tools/restore-context.sh` prints the header, CHECKPOINT and STATE_AUTO (capped around 9 KB so they aren't cut to a preview), then: "If they disagree, trust STATE_AUTO and fix CHECKPOINT; continue without waiting."
- A "Compact instructions" section in CLAUDE.md says what to keep (plan, guardrails, worktree state, P0 list, next actions, decisions) and what to drop (log contents).
- **New `.claude/agents/` files load only after a session restart.** Restart only when the lead reports "SAFE TO RESTART — no agents running."

### 10.5 Hooks (deterministic guardrails; CLAUDE.md is advice, hooks are enforcement)
- PreToolUse on Bash: block `cargo clean`, `git push`, `git reset --hard`, and `rm -rf` outside regenerable output; reject heavy builds not run through `tools/run.sh`. (Old E2E output is deleted folder by folder by name, not with `rm -rf`.)
- PostToolUse on Edit/Write: fmt/prettier on the touched file only.
- Stop and SubagentStop: append one line to `logs/AGENT_ACTIVITY.md`.

---

## 11. Efficiency, usage and resource management

### 11.1 Token and usage tricks
1. **Quiet output:** `tools/run.sh <name> <cmd>` writes the full log to `logs/runs/<name>.log` and prints only the exit status, failing test names and the first 30 error lines. Full logs never go into the context.
2. **Delegate, and route models** (§10.2). The Opus lead mostly reads short reports.
3. **No polling:** builds and the soak run as background tasks, and agents wait for the completion notification (at most one check every 10 min).
4. **Combined review and verify** for diffs under 200 lines; separate passes only for engine swaps, save/redaction paths and the RC.
5. **Batch** small P0 fixes into one verification gate.
6. **Reports ≤ 150 words** in a fixed format (PASS/FAIL, findings, files). Inventories, screenshots and tables go in files.
7. **Exact paths in task briefs** instead of "explore the repo". CLAUDE.md ≤ 200 lines (currently about 81), with details in docs.
8. **Fresh contexts:** each task restarts from CHECKPOINT; `/clear` between unrelated tasks.
9. **Scripts before reading:** audits, inventories and stats come from scripts (xtask, `routing-stats.py`), not from agents reading files.
10. **Stop stale work:** cancel a review when its subject changes, rather than letting it finish on outdated code.

### 11.2 Usage budget (Claude Max)
- Weekly usage: 62% (07:26) → 64% → 65% → 66% (09:09) → **77% (14:49)**. It resets **Sunday 04:00 Asia/Dhaka**.
- Budget pace: what's left ÷ hours until Sat 23:59. At 14:49 that was about **0.7%/h**, against an actual burn of about 2%/h → non-essential work was cut.
- **Throttle tiers** (applied from the weekly % the owner gives at each milestone):
  - < 75%: full plan.
  - 75–85%: 2 builders; engines only on critical items; merge verifier on Opus medium.
  - 85–92%: lead + one builder; P0s only; E2E subset (flows 1, 2, 4, 6, 11).
  - > 92%: stop new work, tag whatever is green, write KNOWN_ISSUES.
- **Pace rule:** burn above about 1–1.5%/h since the last check → drop a tier or stop non-P0 work. **Packaging and RC verification are never paused**; they *are* the release.
- **Exception that always holds:** save-path, redaction and RC changes always get Opus-high verification.

### 11.3 Machine resources (Ubuntu PC, 31 GiB RAM, **ML training runs in parallel and has absolute priority**)
- **RAM scheduler (`tools/run.sh`):** budget = MemTotal − training peak (~3.2 GiB) − 6 GiB reserve ≈ 22 GiB. LIGHT jobs (< 1.5 GiB) run freely. HEAVY jobs (full build, link, tauri build, E2E, full nextest) need their measured peak (~4 GiB) plus the reserve free. **2 HEAVY slots** (measured max 3,972 MiB). E2E takes both slots until its peak is measured. Queue order: verifier > P0 gate > E2E > the rest. Wait when MemAvailable < 8 GiB. Numbers live in `docs/RAM_BUDGET.md`.
- **Protecting training:** jobs run under `systemd-run --user --scope -p MemoryHigh/-p MemoryMax`, `choom -n 800` (the OOM killer picks build jobs first), and `nice -n 10 ionice -c3`. A job killed for memory is re-queued once; after that it's reported, never retried in a loop.
- **Builds:** `CARGO_BUILD_JOBS=2`; never chain heavy pnpm and cargo in one line; **never `cargo clean`** (one approved exception, logged). sccache with a local disk cache only, 10 GB cap, shared across worktrees. mold linker. Dev profile: `debug="line-tables-only"`, dependencies at debug 0, `split-debuginfo="unpacked"`, no LTO. `cargo check -p` while working, a full build only at the gate. nextest `--test-threads=2`. Node `--max-old-space-size=3072`, vitest `--maxWorkers=2`. pnpm's global store is shared across worktrees.
- **Disk:** a **separate target folder per worktree** (a shared one let branches overwrite each other's binaries, so a verifier could test the wrong branch, ERR-008). Builds refuse to start below 15 GB free; target folders are reviewed below 22 GB free; regenerable artifacts of merged or removed worktrees and old E2E output may be deleted, but never caches in active use.
- **The owner's machine:** close VS Code worktree folders while agents build (each runs its own rust-analyzer, 2–5 GB). An extra swap file (8 GB) is a safety net.

---

## 12. Packaging and release checklist (`v0.2.0-prototype`)
- [ ] Version **0.2.0** everywhere (KI-005); release profile with **LTO**; stripped binaries.
- [ ] **AppImage + .rpm + .deb**, `SHA256SUMS`; install and uninstall tested in a clean container; KI-006 offline AppImage tooling check.
- [ ] Desktop integration: `.desktop` entry, PDF mime association, icons at all sizes, consistent app ID.
- [ ] XDG paths for config, cache and logs; a "Reset settings" option; local-only logs at a documented location.
- [ ] About dialog: version, build hash, licences (`THIRD_PARTY_NOTICES.md`), a "100% offline" statement.
- [ ] Docs: README (install/run), QUICKSTART.md (the 12 flows as plain steps), KNOWN_ISSUES.md, CHANGELOG.md.
- [ ] SBOM generated offline from `Cargo.lock` and the pnpm lockfile (CycloneDX or similar, licence-vetted tool).
- [ ] Network-isolated run of the final package: all flows work, zero connection attempts.
- [ ] RC verifier (Opus high), then tag `v0.2.0-rc1` at 18:00, and `v0.2.0-prototype` by 23:59.
- [ ] The owner pushes: `gh auth login`, then `git push origin main --tags`.

---

## 13. Android plan (paused until after the prototype)
**ADR-019 (PROPOSED)** must be approved before any code. It must cover:
- **A capability matrix** (Linux / Windows / Android). Sidecar features are not available on Android; Tauri sidecars aren't packaged there.
- **Files:** Android's Storage Access Framework hands out content URIs, not paths. Build an Android backend for `lpdf-fs-safety` that keeps the same guarantees: write to an app-private temp file → verify → copy to the URI the user chose.
- **Jobs:** no Job Objects or child-process sandbox on Android; govern RAM under Android's memory pressure (the OS kills heavy apps).
- **UI:** tablet first, touch and stylus, no hover, no keyboard-only paths. The same Svelte engines with a platform-specific layout shell.
- **Bundle:** app-store size limits are well below our 1 GB; decide how OCR models are delivered.
- **Toolchain:** report the SDK/NDK/emulator disk and RAM needs before installing; with training running, prefer a USB device over an emulator.
- **Gate:** add `cargo check --target aarch64-linux-android` on the core crates.
Rough estimate from the conversation: an Android beta in about 4–8 weeks of steady sessions after approval.

---

## 14. Decision log (summary)
| ID | Decision |
|---|---|
| ADR-015 | approved by the owner (setup-era decision; see the ADR file) |
| ADR-016 | the repo restructure into `master/` (seven-folder layout) |
| ADR-017 | the local `winprint` patch removing a build-time download; ACCEPTED; draft an upstream issue asking for that build dependency to be optional |
| ADR-018 | craft reuse through **adapters plus pinned dependencies**, strangler pattern, manifest, upgrade procedure; never edit upstream data |
| ADR-019 | Android target, PROPOSED, paused |
| ERR-005 | Markdown → PDF via pandoc → DOCX → LibreOffice: fixed |
| ERR-006 | `reqwest` on Windows: fixed (ADR-017); D-Bus "unmaintained" advisories ignored with written reasons (no fix exists) |
| ERR-007 | hatch clipping and distinct patterns: fixed, with our own parametric patterns |
| ERR-008 | shared target folder corrupting verification → per-worktree target folders (one-time deletion of the 54 GB `target/debug` approved) |
| ERR-009 | restore-hook output cut to a preview → capped around 9 KB |
| AR-CONC | upstream pattern definition untouched; ×1/16 default display scale in the adapter |
| Labs | LaTeX, ink-to-text, split view: off by default, honest labels |
| Tabs | if the tab fix fails review → one document at a time |
| Routing | Haiku/Sonnet/Opus by role; never Fable; the lead only orchestrates |
| Save rule | save, atomic-write, redaction and RC changes are always verified at Opus high |

---

## 15. Lessons learned (avoid repeating these)
1. **Handoffs overstate.** Check every claim against code, tests and gate runs.
2. **Copying code kills upgrades.** Depend on pinned crates behind adapters; port only with a recorded reason and provenance.
3. **Shared Cargo target folders across worktrees** give wrong test results. One target folder per worktree.
4. **Idle agents polling** burned most of the usage. Wait for notifications instead.
5. **Opus everywhere** used the weekly budget about 5× too fast. Route by role and delegate.
6. **Raw audit counts mislead:** most pixel "violations" were canvas geometry. Use allowlists with reasons.
7. **Harnesses leak processes:** use process groups, kill everything on exit, and check that nothing is left over.
8. **A runner that exits 0 for missing flows** is a false green. Test the test infrastructure.
9. **Restarting kills running subagents.** Restart only when "SAFE TO RESTART."
10. **Disk fills quickly** with per-worktree targets: watch the thresholds; don't trim a target that's about to be rebuilt.

---

## 16. Moving to another AI tool
Claude Code-specific features and their portable equivalents:

| Claude Code | Portable equivalent |
|---|---|
| `CLAUDE.md` | copy it to `AGENTS.md` (read by Codex and most agent tools) and/or the tool's rules file; keep the same ≤ 200-line rules |
| `.claude/agents/*.md` (roles, models) | the tool's agent/profile config; otherwise paste the role brief at the top of each task; keep the same roles and escalation |
| Hooks (PreCompact, SessionStart, PreToolUse) | the tool's hooks if it has them; otherwise **git hooks + wrapper scripts** (`tools/run.sh` already enforces the build lock, RAM and disk guards; add a pre-push hook blocking agent pushes) and a manual rule: "first action of every session: run `tools/restore-context.sh` and read its output" |
| Subagents | separate sessions/terminals per worktree; the human or lead passes ≤ 150-word reports through the `logs/checkpoints/*.md` files |
| Verifier subagent | a **new, separate session** given only the spec, the diff and `tools/verify.sh`; never the builder's own session |
| Auto-compaction | before a long task, update CHECKPOINT; start new sessions from `restore-context.sh` output |

Everything that matters (gates, scripts, checkpoints, ADRs, manifests, audits) lives **in the repo** and is independent of the tool.

---

## 17a. Timeline and deadline status
- The original goal was `v0.2.0-prototype` by **Sat 2026-11-07 23:59**. At handover, the codebase is a **verified working prototype on `main`**: all P0s fixed, core flows passing, DOGFOOD AppImage proven. The **release packaging and RC verification remain**.
- Realistic plan: finish the RC with Claude after the weekly reset (**Sun 2026-11-08 04:00**). The Saturday deadline only holds if another strong model or a human does packaging B1/B2 on a Linux machine before then.

## 17. Open questions for the owner
1. R3 licence: the 3-line explanation is still owed. ERR-007 is fixed without it (our own parametric patterns).
2. The BUG-061 classification after the closed-tab soak result.
3. Your own dogfood test of the 0.2.0 build on real PDFs, logged in `DOGFOOD.md`.
4. ~~Push~~ **Done:** everything is on GitHub, history rewritten to your name.
5. After the RC: approve the order of the ADR-018 swaps, ADR-019 (Android), and the scope of the UI backlog.
6. Which machine will do Linux packaging and E2E (see §19).

---

## 18. Starter prompts for the next AI

### 18.1 For a strong model (Claude after the reset): finish the RC
```
You are taking over LocalPDF Studio. Clone: git clone https://github.com/Hermes018/Ayra_PDF.git ; code is in master/.
Git history was rewritten on 2026-11-06: commit hashes in older docs are stale; use branch and tag names.
1. Read master/docs/LOCALPDF_MASTER_HANDOVER.md (start with the ⚠️ box and §2.5), then CLAUDE.md/AGENTS.md, logs/CHECKPOINT.md, logs/BUGS.md, docs/ARGON_TASKS.md §0.
2. Set up the worktrees again from branches wt/fixes, wt/release, wt/engines (each with its own target dir), then run tools/verify.sh on main and confirm it's green at tag packaging-prep-a-green or later.
3. Do ARGON_TASKS.md §0 in order: review and merge wt/fixes (BUG-061/062), review tools/sbom_npm.py, packaging B1+B2, RC verification (verify.sh + 12 flows + network-isolated run + redaction check), tag v0.2.0-rc1.
4. Follow the ANCHOR rules (§1.2), gates (§9), agent roles (§10) and efficiency rules (§11). Report in ≤ 150 words: verified green, open P0/P1, next 3 actions, decisions needed.
Never push, never handle tokens, never rewrite history again, never run cargo clean, never modify original PDFs.
```

### 18.2 For a weaker model (Argon): low-risk work only
```
Work only on tasks in master/docs/ARGON_TASKS.md outside §0 (P2 bugs, UI backlog screens, wording, docs, extra screen smoke tests).
Never touch: engines/adapters, lpdf-fs-safety, save/redaction/security paths, xtask gates, hooks, packaging scripts, ADRs.
One task per branch. Every change must pass tools/verify.sh before commit; the strict UI audit must stay green. If a task needs anything on the never-touch list, stop and write it in ARGON_TASKS.md for Claude.
```

## 19. Setting up on the next machine
- **Clone and set the identity:** `git clone https://github.com/Hermes018/Ayra_PDF.git && cd Ayra_PDF && git config user.name "Ayman Rahman" && git config user.email "aymanrahman30@gmail.com"`.
- **Linux is required for the Linux release work.** AppImage/.deb/.rpm packaging, the WebKitGTK app, `tauri-driver` E2E (WebDriver), `unshare`/bwrap network isolation and mold all need Linux. On a **Mac (Apple Silicon)** you can edit code and run the frontend and Rust unit tests, but for packaging and E2E use a Linux machine or a Linux VM (e.g. UTM/OrbStack/Lima with an arm64 or x86_64 Ubuntu guest; x86_64 packages need an x86_64 Linux build host or CI).
- **A CI option:** a GitHub Actions Linux runner can build the packages and run headless E2E (xvfb). Workflows must still keep the app's offline guarantees; CI downloading build dependencies is fine, runtime network is not.
- **Toolchain to reinstall:** Rust (stable), Node + pnpm, the Tauri v2 Linux deps (WebKitGTK etc.), `cargo-deny`, `cargo-audit`, `cargo-nextest`, `tauri-driver`, sccache, mold, poppler-utils/qpdf (redaction checks), LibreOffice + pandoc (sidecar features). Then run `tools/verify.sh`.
- **Agent setup:** `.claude/agents/*.md`, `.claude/settings.json` (hooks) and `CLAUDE.md` are in the repo. A first `.claude/agents/` directory only loads after a session restart. Re-measure `docs/RAM_BUDGET.md` on the new machine (the old numbers assumed 31 GiB and ML training running in parallel).

## 20. Next actions (in order)
1. **Owner:** clone on the new machine, set the git identity, read this file.
2. **Strong model (after Sun 04:00 reset):** ARGON_TASKS §0, i.e. merge the BUG-061/062 work → review `sbom_npm.py` → packaging B1 (LTO measurement, AppImage/.deb/.rpm, SHA256SUMS, SBOM, KI-006) → B2 (clean install, network isolation, docs) → RC verifier (Opus-high) → tag **`v0.2.0-rc1`** → owner pushes.
3. **Owner:** dogfood the RC build on real PDFs (copies only), log in `DOGFOOD.md`.
4. **Strong model:** fix dogfood P0s → tag `v0.2.0-prototype`.
5. **Argon (in parallel, any time):** low-risk items from ARGON_TASKS (outside §0).
6. **After the prototype:** ADR-018 engine swaps → UI backlog → PARTIAL features real or cut → ADR-019 Android → Windows refresh → hardening (§3.2).

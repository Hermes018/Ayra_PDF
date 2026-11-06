# ARGON_TASKS: low-risk tasks for the Argon agent

Read this whole file first. Pick ONE task, do it, finish its Done checklist, then pick the next. Paths are relative to `/home/rakib/Desktop/Ayra_PDF/master/` unless they start with `/`. Frontend source root: `app-tauri/src/` (called `SRC` below).

## 1. Rules for Argon

1. Work in your own worktree and branch (never on `main`). Commit only there; the lead merges. Never push.
2. Every change must pass `master/tools/verify.sh` (run inside your worktree) before you commit. Heavy commands (cargo, pnpm, vitest full runs, E2E) go only through `master/tools/run.sh <name> <cmd...>`; one at a time, never chained. Log lands in `master/logs/runs/<name>.log`.
3. One task at a time. If the task fixes a bug, write the failing test FIRST, run it, see it fail, then fix it, then show it passing.
4. Never touch the DO-NOT-TOUCH list (section 2). If a task seems to need it, STOP and write the question in your checkpoint file.
5. No new features, no new dependencies, no behaviour changes in IPC calls, outputs or save paths.
6. Never add a `ui-px-allowlist.txt` entry without a written reason on the same line (format: `path:line - reason`).
7. Your worktree is `~/Desktop/Ayra_PDF-wt/argon`, branch `wt/argon`, created by the lead. Do not edit `CHANGELOG.md` or `KNOWN_ISSUES.md`. The lead owns `master/logs/` (except your own `master/logs/checkpoints/<worktree>.md`), tree maps and `descriptions.tsv`.
8. Svelte rules: no hex/rgb literals in `.svelte`, no `{@html}`, no local keydown listeners, no frontend `fetch(`. Rust rules: no `unwrap()/expect()/panic!/todo!` outside tests.
9. Never invent function or file names: open the file and check before you use a name.
10. A task that fails twice: stop, record it in your checkpoint, move on.

## 2. DO NOT TOUCH

- Engines: `master/crates/lpdf-engine-*`, `lpdf-geom`, `lpdf-engine-switcher`, `vendor/`.
- `master/crates/lpdf-fs-safety`.
- Redaction code: the redact operation in `lpdf-engine-pdfcraft`, Studio "Commit redactions" in `app-tauri/src-tauri/src/commands/studio.rs`, SecurityView redaction.
- Every save and output-path code path: `lpdf-jobs/src/output_guard.rs`, `outputNames.ts`, `studioSave.ts`, `documentSessions.ts`, `organizeApply.ts`.
- Password handling: `lpdf_core_types::Secret` and anything that holds a password.
- xtask gates: `master/xtask/src` (including `ui_audit.rs`) and `verify.sh`.
- Hooks and tooling: `.claude/`, `master/tools/`.
- Packaging: `ubuntu/`, `fedora/`, `windows/`, `macos/`, `tauri.*.conf.json`.
- CSP and `tauri.conf.json`.
- `SRC/lib/core/commandEngine.svelte.ts`.
- E2E harness core: `master/e2e/common.py`, `wd.py`, `run.py`, plus `ink_match.py` and the flow files 1-12.
- `SRC/lib/views/studio/SpatialCanvas.svelte` (it holds the Save, Flatten and Commit redactions code), `studio/DocumentTabs.svelte`, `SettingsView.svelte`, `SecurityView.svelte`, `FormsView.svelte` and anything under `src-tauri/`.
- `README.md`, `QUICKSTART.md`, `CHANGELOG.md`, `KNOWN_ISSUES.md` (the lead writes them during packaging).
- The gallery marker: `lib/gallery/marker.ts` and its use in `Gallery.svelte` must stay exactly as they are; `tools/check-no-gallery.sh` depends on it.

## 3. Strict-audit workflow (used by AT-10 to AT-14)

1. Read `docs/DESIGN_LANGUAGE.md` section 7 (Sweep rules) and section 3 (control list).
2. In the target file replace raw `<button>`, `<input>`, `<select>` with the matching `SRC/lib/components/ui/` component (Button, IconButton, Input, Select, Toggle, ...). Replace literals/off-scale px with `var(--token)` values that already exist in `SRC/app.css` or `core/themeContract.svelte.ts`. Keep behaviour identical.
3. Check by grep: `grep -n "onkeydown\|addEventListener('keydown'" <file>` must be empty.
4. Add the path (relative to `SRC`, e.g. `lib/gallery/Gallery.svelte`) to `xtask/ui-strict.txt` under a new `# Argon sweep` comment line. Never remove a line there.
5. Run `master/tools/run.sh audit cargo run -q -p xtask -- studio-audit --strict xtask/ui-strict.txt`. Expected: exit 0, no violations. If it fails, fix the file (not the gate).
6. Run `cd master/app-tauri && ` the vitest for that file only via run.sh (`master/tools/run.sh vt pnpm vitest run <test file>`), then `master/tools/verify.sh`.

## 4. Tasks (all risk LOW)

### Category A: P2 bugs that are wording, layout or tests only

**AT-01 Math-text note wording** (BUG-001). Files: `SRC/lib/components/studio/labs.svelte.ts` (note near line 28), its test file if one exists. Do: make the Labs note for Math text say the inserted text always starts with "∑ " (see `insertEngine.svelte.ts:612`, read only). Wording only. Accept: `master/tools/run.sh vt pnpm vitest run labs` passes (add a one-line assertion that the note contains "∑"). Do not touch `insertEngine.svelte.ts`.

**AT-02 Stale `./labs.ts` comments** (BUG-006). Files: any `.ts`/`.svelte` under `SRC` where `grep -rn "labs\.ts" SRC` hits. Do: change comments to `labs.svelte.ts`. Comments only. Accept: `grep -rn "labs\.ts" SRC` prints nothing; verify.sh green. Do not touch behaviour.

**AT-03 Labs note reachable by screen readers** (BUG-005). Files: `SRC/lib/components/studio/DockableToolbar.svelte` (~line 377) plus its test. Do: add an `aria-label` or visually-hidden text carrying the same Labs note currently only in `title`. Accept: a vitest assertion finds the note via `getByLabelText`/`aria-label`; strict audit still passes (file is already in ui-strict.txt). Do not touch other toolbar logic.

**AT-04 Tooltip contrast test** (BUG-044, test half only). Files: `SRC/lib/components/ui/contrast.test.ts`. Do: its header mentions Tooltip but only the dialog is covered. Add a Tooltip case that mirrors the dialog case. Accept: `master/tools/run.sh vt pnpm vitest run contrast` passes with the new case. Do not change `tooltipPlacement.ts` (the placement/overlap fix is a design question; note it in your checkpoint).

**AT-05 FAQ modal key tests** (BUG-042 test half). Files: `SRC/lib/components/FaqModalKeys.test.ts`. Do: add the two missing cases named in BUG-042: Escape with another dialog open, and Escape after unmount. Test only. Accept: `vitest run FaqModalKeys` passes. Do not touch `commandEngine.svelte.ts` or `FaqModal.svelte`.

**AT-06 EngineReset test coverage** (BUG-034 bullet 2). Files: `SRC/lib/views/__tests__/EngineReset.test.ts`. Do: extend it so plain (non getter/setter) fields such as `eraseGestureBefore` are also checked after reset; read the engine file first to list the plain fields. If a plain field is NOT reset, do not fix the engine: record it in your checkpoint. Accept: vitest on that file passes.

**AT-07 Organize no-deletion test** (BUG-034 bullet 3). Files: a new test beside the existing Organize tests under `SRC/lib/views/__tests__/`. Do: test that a single-step Organize apply, and a failed apply, delete nothing (assert no delete IPC mock is called). Test only; do not edit `organizeApply.ts`. Accept: vitest on the new file passes. If the apply code cannot be exercised without editing it, STOP and report.

**AT-08 (removed):** the flow 7 ink matcher (BUG-047) backs the release evidence, so the lead keeps it. Do not edit `e2e/ink_match.py` or `e2e/test_flow07_matcher.py`.

**AT-09 Screenshot layout races** (BUG-049) and **BUG-056 note**: NOT low risk (both need `common.py`/`run.py` or annotation behaviour). Skipped on purpose. BUG-056 may only be done as wording: if `SRC` shows a user-visible message about page clamping, leave it alone and ask the lead.

### Category B: UI_BACKLOG.md and strict-audit sweeps

Only these backlog rows are low-risk. The rest (StatusBar job count, toast wording in `state.svelte.ts`, FAQ load failure signal, EditView "Add sample highlight" product decision) need new behaviour or a product decision: NOT included. The PINS bar clipping (BUG-045 / backlog "Favourites bar clipped") is a Studio-shell layout fix; it is included below as AT-15 because it is CSS only.

Files not yet in `xtask/ui-strict.txt` (checked): `lib/components/For.svelte`, `lib/components/studio/HatchDefs.svelte`, `lib/components/studio/StudioIcon.svelte`, `lib/components/ui/ColorSwatch.svelte`, `lib/gallery/Gallery.svelte`, `routes/+layout.svelte`, `routes/+page.svelte`, `routes/gallery/+page.svelte`.

**AT-10 Strict: ColorSwatch.** Files: `SRC/lib/components/ui/ColorSwatch.svelte`, `xtask/ui-strict.txt`. Do: follow section 3. It is a ui/ component and should be strict from day one. Accept: strict audit command in section 3 step 5 exits 0 with the file listed.

**AT-11 Strict: Gallery.** Files: `SRC/lib/gallery/Gallery.svelte`, `xtask/ui-strict.txt`. Do: section 3. Gallery is the component showcase (keep the `GALLERY_MARKER` import and its use exactly as they are); raw controls inside demo markup that intentionally show a raw element must be removed, not allowlisted. Accept: strict audit exits 0; E2E flow 13 not changed (the lead runs it).

**AT-12 Strict: StudioIcon and HatchDefs.** Files: those two `.svelte` files, `xtask/ui-strict.txt`. Do: section 3 (likely already clean: just add and run the audit; fix any hit). Accept: audit exits 0.

**AT-13 Strict: route pages.** Files: `SRC/routes/+page.svelte`, `SRC/routes/gallery/+page.svelte`, `SRC/routes/+layout.svelte`, `xtask/ui-strict.txt`. Do: section 3; add them one at a time and run the audit after each. `+page.svelte` calls `attachKeyboard` (BUG-042): leave that call as is. Accept: audit exits 0.

**AT-14 Strict: For.svelte.** Files: `SRC/lib/components/For.svelte`, `xtask/ui-strict.txt`. Do: section 3. Accept: audit exits 0.

**AT-15 PINS bar clipped by status bar** (BUG-045, backlog "Favourites bar clipped"). Files: ONLY `SRC/lib/components/studio/FavoritesBar.svelte` (CSS only: height, overflow, padding, using tokens). `SpatialCanvas.svelte` is OFF LIMITS: it holds the Save, Flatten and Commit redactions code. If the fix needs `SpatialCanvas.svelte` or any TS change, STOP and write the question in your checkpoint. Accept: run `master/tools/run.sh e2e python3 master/e2e/run.py --flows 21` (canvas chrome smoke), open the new screenshot (path printed in the log, under `e2e/out/runs/<run-id>/screens`) and confirm the bar is fully visible at 1200x800; strict audit still exits 0.

### Category C: wording and docs

**AT-16 (removed):** already done in BUG-055; the CLI redact help says "Pages start at 1". Do not edit `master/crates/lpdf-cli`.

**AT-17 (removed):** there is no README.md yet; the lead writes README.md and QUICKSTART.md during packaging. Do not create them.

No other wording or docs tasks were found that are safe: CHANGELOG.md and KNOWN_ISSUES.md are off limits, and the toast wording item needs job-layer changes.

### Category D: extra E2E smoke tests

I checked `e2e/flows/`: flows 13 to 23 already cover the gallery, organize, shell, overlays, studio docs and panels, toolbar, ledger, canvas chrome, pill, and all ten tool views (flow23). No screen without a smoke test was found, so no new-flow task is listed. Do not invent one. If you find a screen that is genuinely missing, write the name in your checkpoint and ask the lead.

## 5. Done checklist (per task)

- [ ] Tests added or updated (failing first for bug fixes; show the failing and the passing run).
- [ ] `master/tools/verify.sh` is green, run inside your worktree (give the `master/logs/runs/<name>.log` path).
- [ ] For any UI/layout task: screenshot paths (light and dark, 1200x800 and 1600x1000) listed in your checkpoint.
- [ ] You edited nothing on the DO-NOT-TOUCH list (`git diff --stat` shows only the files the task names).
- [ ] 3-line summary in `master/logs/checkpoints/<worktree>.md`: what changed, which tests, what is next.
- [ ] Committed on your own branch. Not pushed.

# BUGS (sprint v0.2.0-prototype) — triage owned by the lead

Sources: the E2E baseline and runs, builder reports, the adversarial tester and reviews. Triage rules are in `docs/PROTOTYPE_SCOPE.md`: **P0** breaks a main flow or loses/corrupts data; **P1** visible, with a workaround; **P2** after the release.

Entry format:
```
## BUG-NNN [P0|P1|P2] short title — status: open|fixing|fixed|deferred (KNOWN_ISSUES)
- Flow: # (PROTOTYPE_SCOPE)   Found by: e2e | builder | tester | owner   Date:
- Repro:
- Expected / actual:
- Owner (worktree): engines | fixes | release
- Fix commit / test:
```

## BUG-001 [P2] Math-text note omits the "∑ " prefix — status: open
- Flow: — (Labs only)   Found by: reviewer   Date: 2026-11-05
- Repro: Labs on → insert Math text: the text always starts with "∑ " (insertEngine.svelte.ts:612); the note in labs.svelte.ts:28 doesn't say so.
- Owner: fixes

## BUG-002 [P2] Undo label "Handwriting Ink-to-Text" overclaims — status: open
- Flow: — (Labs only)   Found by: reviewer   Date: 2026-11-05
- Repro: Labs on → Shape-to-character guess → undo tooltip (SpatialCanvas.svelte:1528) shows "Handwriting Ink-to-Text" (insertEngine.svelte.ts:1106).
- Owner: engines (core/)

## BUG-003 [P2] "Reset to defaults" leaves parked Labs shortcut bindings — status: open
- Found by: reviewer   Date: 2026-11-05   Repro: Labs off → Shortcut settings → Reset; turn Labs on → the old Labs chord is back (ShortcutSettings.svelte:59, labs.svelte.ts:135).
- Owner: fixes

## BUG-004 [P2] Ink-to-text fallback can replace a stamp/signature, copy says "ink strokes" — status: open
- Found by: reviewer   Date: 2026-11-05   Where: insertEngine.svelte.ts:1068 (last stroke of any kind).
- Owner: engines (core/) for behaviour, fixes for copy

## BUG-005 [P2] Labs honest note only in a title attribute (a11y) — status: open
- Found by: reviewer   Date: 2026-11-05   Where: DockableToolbar.svelte:377 — screen readers may not announce title.
- Owner: fixes

## BUG-006 [P2] Code comments reference `./labs.ts`, file is `labs.svelte.ts`; empty-stroke selection yields "Ink Note" without colon — status: open
- Found by: verifier   Date: 2026-11-05
- Owner: fixes

## BUG-007 [P2] Labs tools not refused at the core level (toolEngine.selectTool / commandEngine) — status: open (reviewer: sufficient for release)
- Found by: fixes builder + reviewer   Date: 2026-11-05   All UI call sites are guarded and a root effect resets to Pan; a core-level refusal belongs in core/ (engines).
- Owner: engines

## BUG-008 [P2] Studio prefs never keep the toolbar dock position — status: open
- Found by: fixes builder   Where: SpatialCanvas.svelte persistStudioPrefs always saves dockEdge 'left'; loadStudioPrefs ignores it.
- Owner: fixes

## BUG-009 [P2] PropertyPill insert picker hidden for note/stamp/signature/image tools — status: open
- Found by: fixes builder   Where: PropertyPill.svelte:450 inside the stroke-properties branch (line 169).
- Owner: fixes

## BUG-010 [P1] toolEngine label for insert.latex still "LaTeX Math Block" — status: open
- Found by: fixes builder   Where: core/toolEngine.svelte.ts:131 (overridden at display sites; fix at source).
- Owner: engines

---
# E2E baseline triage (2026-11-06 00:30, release builder baseline on 562e4f9-era code: 4 PASS / 7 FAIL / 1 BLOCKED)

| Flow | Baseline |
|---|---|
| 1 | BLOCKED (no packages); cold launch 1.01 s, 0 console errors |
| 2 | FAIL |
| 3 | FAIL |
| 4 | FAIL |
| 5 | FAIL |
| 6 | PASS |
| 7 | FAIL |
| 8 | PASS |
| 9 | FAIL |
| 10 | FAIL |
| 11 | PASS (warning) |
| 12 | PASS (5-minute soak: RSS 664→750 MB peak, file descriptors flat, no temp files) |

Ownership for fixes: **fixes** owns UI components/views plus core/inkEngine, insertEngine, viewportEngine and measureEngine for these bug fixes. **engines** owns Rust (crates/, src-tauri), hatching (HatchDefs) and new adapter files. Order = work order.

## BUG-011 [P0] Encrypted PDF never prompts for a password — status: fixed (263bb7e, tag fixes-p0-green)
- Flow 2. Found by: e2e. Repro: open corpus/encrypted.pdf (user pw lpdf-test). pdf.js throws PasswordException; 0 pages, no prompt. Owner: fixes

## BUG-012 [P0] CSP blocks data:/blob: images, so Organize thumbnails are blank — status: fixed (263bb7e, tag fixes-p0-green)
- Flow 3. Found by: e2e. CSP `default-src 'self'` has no img-src. Fix: `img-src 'self' data: blob:` in tauri.conf.json. connect-src stays IPC-only (DON'T-1 unaffected); the lead approves this as a low-risk CSP change (images only). Check that snapshot and webcam images are fixed too. Owner: fixes

## BUG-013 [P0] Edit Text hotspot unreachable by pointer — status: fixed (263bb7e, tag fixes-p0-green)
- Flow 9. Found by: e2e. `#wet-layer` covers `.pdf-text-block-hotspot`, and the default Pan tool takes the click. The edit itself writes correctly. Owner: fixes

## BUG-014 [P0] Organize Grid "Apply" drops rotation but reports success — status: fixed (263bb7e, tag fixes-p0-green)
- Flow 10 (silent data loss). Found by: e2e. Reorder + delete + rotate: the rotation is lost and success is still alerted. The sidebar Rotate tool works. Owner: fixes

## BUG-015 [P0] ERR-007: saved hatch outside the polygon; only 3/8 patterns distinct — status: fixed (52b7a17, tag fixes-err007-green)
- Flow 5. Found by: e2e (37/56 segments outside the concave polygon) and survey. Owner: engines (checkpoint 2: cadcraft via lpdf-geom)
- ERR-007 fix moves to the fixes builder after the restart (throttle tier 85–92 %): licence-clean clipping port + own parametric patterns (owner 07:40).

## BUG-016 [P0] A second document is added but never shown (no tabs or split panes) — status: fixed (263bb7e, tag fixes-p0-green)
- Flow 3. Found by: e2e. A second file opened via CLI/"open with" goes into activeFiles but isn't displayed. The checklist claims multi-tab/split (SplitLayout.svelte). Make the existing split/tab path work; this is a fix, not a new feature. Owner: fixes

## BUG-017 [P0] Draw-mode signature loses the drawn strokes and stamps text instead — status: fixed (263bb7e, tag fixes-p0-green)
- Flow 7 (data loss). Found by: e2e. Stamps "Signed: A. Architect, PE" instead of the drawing. Owner: fixes

## BUG-018 [P0] Malformed/hostile PDFs open silently with no error or degraded notice — status: fixed (263bb7e, tag fixes-p0-green)
- Flow 2(c). Found by: e2e. broken-xref gives an empty canvas; bombs gives 1 page; no message. Owner: fixes (UI notice) plus engines if the backend must report degraded mode

## BUG-019 [P0] Erasers aren't in undo history; Undo removes a different stroke — status: fixed (263bb7e, tag fixes-p0-green)
- Flow 4 (wrong undo = data loss). Found by: e2e. Owner: fixes

## BUG-020 [P0] Fit Width / Fit Page assume A4 and ignore the page's 32 px offset (page overflows the 990 px canvas) — status: fixed (263bb7e, tag fixes-p0-green)
- Flow 3. Found by: e2e. Owner: fixes

## BUG-021 [P1] Repeated saves overwrite `<orig>.annotated.pdf`; RenderingCancelledException logged on every save — status: fixed (263bb7e, tag fixes-p0-green)
- Flow 11. Found by: e2e. Protocol output rule: new file by default, collision-safe naming. Owner: fixes

## BUG-022 [P1] Perimeter of a 72 pt square measures 216, not 288 (shape not closed) — status: fixed (263bb7e, tag fixes-p0-green)
- Found by: engines goldens. Owner: fixes (measureEngine)

## BUG-023 [P1] studio_detect_tables_and_fields fabricates an "auto" field when nothing is found (DON'T-5) — status: open
- Found by: engines. Owner: engines

## BUG-024 [P1] Hatch annotation apply error swallowed (`let _ = session.apply(...)`) — status: open
- Found by: Linux builder + engines (studio.rs ~545). Owner: engines

## BUG-025 [P2] Snap geometry capped at 4,096 segments, silently truncated (DON'T-7) — status: open
- Found by: engines. Owner: engines

## BUG-026 [P2] One ctrl+wheel notch zooms ×2.72 — status: open
- Found by: e2e. Owner: fixes

## BUG-027 [P2] Reopening a saved PDF without its .studio.json sidecar loses editable markups — status: open
- Flow 11 warning. Found by: e2e. Owner: engines (save format), after the release

## BUG-028 [P1] Voice note with an empty recording saves a made-up 440 Hz tone instead of failing (DON'T-5) — status: open
- Found by: engines builder (matrix row T42). `buildOfflineWavBytes` (insertEngine.svelte.ts:~412) substitutes a synthetic tone when the recording yields no bytes. It must refuse with a clear "Nothing was recorded" error instead. Owner: fixes (after the P0 merge)

## BUG-029 [P1] F4 guard is trivially bypassed and covers only GetInfo/RenderPage — status: open
- Found by: engines CP2b reviewer. guard.rs:53-93 checks only a direct, literal `/Length`.
  - Bypasses: an indirect `/Length 9 0 R`; a hex-escaped name `/L#65ngth`; a real flate bomb with an honest small /Length (the F4 decompression-ratio cap is missing).
  - Coverage gap: the guard runs only in `load()` (engine-switcher lib.rs:83). Merge, extract, compress and the studio.rs pdfcraft Session paths skip it.
  - The corpus `hostile-bombs.pdf` is caught. Owner: engines
## BUG-030 [P1] Every load reads the whole file into RAM (twice per RenderPage) — status: open
- Found by: engines CP2b reviewer. engine-switcher lib.rs:83 calls `std::fs::read` before the guard, which contradicts the "cheap, bounded" doc. Fine for the 1k-page corpus (253 KB); bad for 100 MB+ scans. Fix: a fixed-buffer streaming scan. Owner: engines

## BUG-031 [P1] E2E harness leaves its private dbus-daemon running after the run — status: fixed (0291a99, tag release-bug031-green)
- Found by: lead (scope run-racc58…, dbus-daemon with /tmp/lpdf-bus-*/bus.conf alive 1h23m after pkg_smoke started at 06:24). reap() covers app/driver/WebKitWebDriver but not the private bus. Owner: release (paused by the throttle tier)
- Raised to P1 (owner). Fix: run each harness run in its own process group/session and kill the whole group on exit, dbus included; add a post-run check that no harness processes remain. Assigned: release builder, after the session restart.

## BUG-032 [P1] Backend `studio_save_annotations` does not refuse output==input or an existing output path — status: open
- Found by: R5 merge-verifier. The guard is frontend-only, so the "originals are never modified" rule rests on one layer. Owner: fixes

## BUG-033 [P1] Close dialog: "Save" then close discards strokes drawn while the save is in flight, with no prompt — status: open
- Found by: R5 reviewer. DocumentTabs.svelte saveThenClose. Unlikely, because the dialog is modal. Owner: fixes

## BUG-034 [P2] R5 hardening leftovers — status: open
- Save geometry check compares page counts only, so an out-of-range pageIndex falls back to A4.
- EngineReset.test.ts sees only getter/setter fields (plain fields such as eraseGestureBefore are not covered).
- No test that a single-step or failed Organize apply deletes nothing.
- Discard during an in-flight save still opens the saved copy as a tab and leaves a stale baseline entry.
- svelte-check a11y warning: `<form role=dialog>` at SpatialCanvas.svelte:1760.
- Owner: fixes

## BUG-035 [P1] Hatch pitch and dash lengths stretch at 45/135° — status: open
- fill.rs:108-125 measures hatch pitch vertically (the b offset), not perpendicular to the line, and measures dash lengths along the x or y axis. Spacing and dashes stretch at 45/135°. Fix it, or document it and update the golden.
- Found by: ERR-007 reviewer
- Owner: fixes

## BUG-036 [P2] ERR-007 test and edge leftovers — status: open
- studio_hatch.rs ~370: the 1e17 case uses `let _ =`, so it cannot fail.
- hatch_save_e2e.rs: the `want ⊆ drawn` check runs one way only; the trailing `let _ = Path::new` is dead code.
- studio.rs hatch_loops: a polygon with only 2 finite points silently falls back to a bounding-box hatch, and only one loop is passed.
- The preview pattern phase can differ from the saved PDF by up to one tile.
- Add a clean-room statement to fill.rs and UPSTREAM_MANIFEST: generic sweep/crossings code, nothing copied from cadcraft.
- Found by: ERR-007 reviewer
- Owner: fixes

## BUG-037 [P1] Peak RSS 2.9 GB on a 648-page, 2.2 MB file (DF-03), above the 2 GiB budget — status: open
- Peak RSS 2930 MB (app + WebKit children) across open / markup / save / reopen / sampled pan. Other files 0.65-1.2 GB. All 648 canvases appear to be created at open. First step: measure open-only RSS.
- Found by: automated dogfood (DOGFOOD_AUTO.md)
- Owner: fixes

## BUG-038 [P2] Edit Text hotspot labels cover little of the poppler text on page 1 (DF-02..08) — status: open
- Word coverage vs pdftotext page 1 is 0.03-0.66 on 7 files (precision 0.75-0.95); DF-03 yields a single block. Possible causes: aria-label truncation, tokenisation, or missing blocks. No data loss.
- Found by: automated dogfood (DOGFOOD_AUTO.md)
- Owner: fixes

## BUG-039 [P2] Coverage gap: no fillable form in the dogfood set — status: open
- No fillable, encrypted or damaged file in the 8 files, so forms (step 4) was N/A and the error-message path is untested. Need a blank public fillable form.
- Found by: automated dogfood (DOGFOOD_AUTO.md)
- Owner: release

## BUG-040 [P2] 6 element-level Enter/Space handlers on role=button elements (local key listeners) — status: deferred (after release)
- Dropzone ×2, AiSummarizeView ×2, PipelinesView ×1, FaqModal backdrop ×1; allowlisted in xtask/ui-px-allowlist.txt with reasons. They are activation handlers on clickable non-button elements, not app shortcuts: the shared Button component (component library batch) replaces them during the sweep, so the listeners go away instead of moving to commandEngine. Owner: fixes

## BUG-041 [P2] BUG-031 follow-ups — status: open
- run.py: on SIGTERM/SIGINT, SystemExit skips the post-finally LEAK message and report["leak"] (leftovers are still killed, and the exit code is 128+sig). test_leak_check.py:51-62: check 4 uses an inline child script, so run.py's real signal handler and setsid are not exercised. Found by: release review. Owner: release

## BUG-042 [P2] UI audit/triage follow-ups — status: open
- `attachKeyboard(window)` is called in two places (+page.svelte:34 and SpatialCanvas.svelte:907), so Escape handlers can run twice; harmless today because the handler is idempotent. Escape no longer closes the FAQ during chord capture (arguably better; record it as intended). FaqModalKeys.test.ts lacks two cases: Escape with another dialog open, and Escape after unmount. Found by: A2 review. Owner: fixes

## BUG-043 [P1] commandEngine swallows Space on focused buttons: Space can't press any button, anywhere in the app (keyboard accessibility) — status: fixed (8592f67, tag ui-library-green)
- Found by: C0 library builder. A global Space binding in commandEngine.svelte.ts calls preventDefault even when a button (or another native control) has focus. Fix (a bug fix, allowed under the freeze): skip the binding when the event target is an interactive element. Add a characterization test first, then a test that Space activates a focused Button. Owner: fixes (sweep batch)

## BUG-044 [P2] Tooltip covers neighbouring text (e.g. the "Activations" caption in the gallery) even though it stays on-screen — status: open
- tooltipPlacement.ts clamps to the viewport and flips correctly but does not avoid neighbouring content. Also, contrast.test.ts covers only the dialog, although its header mentions Tooltip. Found by: C0 delta recheck. Owner: fixes

## BUG-045 [P2] The "PINS" toolbar at bottom-left overlaps the page (seen in the shell screenshots; it predates the sweep) — status: open
- Found by: sweep g2 visual review. Candidate for the Studio chrome sweep group, or UI_BACKLOG.md. Owner: fixes

## BUG-046 [P1] Flow 7 false FAIL: the harness matches saved annotations by text, so a drawn signature (no text) can never match — status: fixed (tag release-bug046-green)
- Found by: sweep group 6 builder. E2E flow 7 fails the same way on the merged main binary (logs/runs/sweep-e2e7-base.log in wt/fixes), so this predates the sweep. **Triage (lead):** NOT data loss. The saved output's /Ink annotation has an /InkList of 20 points (40 numbers), an /AP appearance stream and /Rect [73.2 423.1 219.3 444.7] (qpdf + parse). flow07_insert.py:113-116 matches by `text`, and `want=''` gives 0 hits. Fix: for content-less annotations (drawn signature), match by subtype {/Ink,/Stamp} plus a non-empty InkList or AP inside the expected bbox, and check stroke count against what was drawn. Blocks flow 7 = 12/12 at the RC, so P1. Owner: release. Also P2: sticky note saved as /FreeText (expected /Text) and stamp as /FreeText (expected /Stamp), per the flow's native-type WARNs.

## BUG-047 [P2] Flow 7 ink matcher accepts an /AP/N stream that draws nothing — status: open
- ink_match.py:30-35 accepts any /AP/N with /Length > 0, so a `q Q` stream (~4 bytes) with a valid /Rect and no /InkList would pass. Fix: require a path operator (m/l/c/re) or a minimum length. Only page 0 is read (wrong-page strokes fail, which is correct today). Found by: BUG-046 review. Owner: release

## BUG-048 [P2] E2E screenshots go to a shared folder (e2e/out/screens) that a later run can wipe — status: fixed (tag release-bug048-green)
- All sweep screenshots in wt/fixes disappeared after a later flow 2 run, so a reviewer found none for group 7. Fix: write screenshots into the per-run folder (out/runs/<run-id>/screens) and never clear the shared folder; print the path. Found by: lead (g7 visual review). Owner: release

## BUG-049 [P2] Per-run screenshot layout: small races — status: open
- common.py:365-372: the one-time move of an old real out/screens leaves the path briefly missing; two overlapping first runs could race. run.py:111: run ids have 1 s resolution, so identical runs in the same second share a folder. Only one E2E run happens at a time today. Found by: BUG-048 review. Owner: release

## BUG-050 [P0] Optimize/Edit (and any view naming outputs by `.replace(/\.pdf$/i, …)`) can write over the ORIGINAL when the input has no .pdf extension — status: fixed (tag fixes-bug050-green)
- Found by: g10 Opus-high read-only check. EditView/OptimizeView build the output path from the input path with a `.pdf` suffix replace; with no extension the output equals the input, so the job writes over the original (hard rule: originals are never modified). It predates the sweep. Fix: collision-safe output naming (outputNames/freeOutputPath) in every view, plus a backend guard that refuses output==input or an existing output in the job/save entry points (also closes BUG-032). Failing tests first. Owner: fixes. Opus-high verification.

## BUG-051 [P1] BUG-050 follow-ups — status: fixed (tag fixes-bug051-green)
- Regression from the guard: Convert/Pipelines/AiSummarize/Markdown names are not collision-safe, so a second run fails with OUTPUT_PATH_UNSAFE (loud and safe, no data loss). MarkdownView.svelte:39 `.replace(/\.md$/)` can yield the input (only the backend refuses it).
- Unguarded writers: studio_export_takeoff_csv, export_diagnostics (user-chosen path), convert_pdf, extract_text (exposed, unused). App-owned files (page-scales sidecar, prefs/settings/library) are exempt by design.
- Tests: OutputPathsBug050.test.ts covers only 2 of 9 views; Rust `an_existing_output_file_is_never_replaced` can't fail (Repair errors on non-PDF input anyway).
- Found by: BUG-050 Opus verifier + reviewer. Owner: fixes. Opus-high verification (backend writers).

## BUG-052 [P1] Output-path robustness after BUG-050/051 — status: fixed (tag fixes-bug052-green)
- Views show the raw error string when the backend refuses an output (OUTPUT_PATH_UNSAFE) instead of an ErrorState that offers the next free name (Pipelines :129, AiSummarize :107). Pipelines submit awaits outside try, before its busy guard (unhandled rejection; a double-click reserves two names). Markdown's isSubmitting may not reset on throw. x-2.pdf → x-2-2.pdf. Settings overwrite confirm doesn't focus Cancel. export_diagnostics can "Replace" a .pdf that isn't open (should refuse .pdf like the CSV export).
- Found by: BUG-051 reviewer + Opus verifier. No data-loss path (backend guards). Owner: fixes.

## BUG-053 [P0] Security → Redact can redact the WRONG page: the UI says "Page (starts at 0)", the engine does page.saturating_sub(1) — status: fixing
- Found by: g11 reviewer. Typing 0 or 1 both redact page 1, so a user following the label redacts the page before the one they meant, and the text they meant to hide stays visible. Flow 8 covers only Studio's Commit Redactions, not SecurityView. Fix: 1-based page number end to end, validated against the page count; the backend refuses 0 or > count with a typed error; a Rust test checks per-page text after redaction. Same round, from the g11 Opus check: blank region fields must be refused (they now silently default to 0/100), and retained passwords must be cleared when the input file changes. Owner: fixes. Opus-high verification.

## BUG-054 [P1] Studio Commit redactions clamps an out-of-range page to the LAST page instead of refusing (studio.rs:401 `.min(page_total-1)`) — status: fixing
- Found by: BUG-053 Opus pass. Pages come from the canvas, so unlikely in practice, but a silent clamp can redact the wrong page. Fix: refuse with REDACTION_PAGE_OUT_OF_RANGE and write nothing. Owner: fixes. Opus-high verification.
## BUG-055 [P2] Redaction/security follow-ups — status: open
- lpdf-cli/src/lib.rs:105: `localpdf redact` help doesn't say pages start at 1. Security/Forms: if the file changes while a job runs, a later Retry can send the current (possibly blank) password; disable Retry after a file change. Owner: fixes

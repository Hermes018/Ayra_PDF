# checkpoints/fixes.md — fixes builder (wt/fixes). Builder keeps this current; < 150 lines.
- **Task:** P0 review r5 — FINAL tabs round (base main 9919398 incl. engines CP2b; lead restaged r4). Uncommitted.
- **r5 status (07:55):** tests red first (logs/runs/r5-red.log: 18 failing), green (r5-views.log, r5-views2.log).
  Fixed: close-through-store (BLOCKER); inserts blocked until restored + busy cursor; Save refuses on
  missing/incomplete geometry; full ENGINE_STATE reset + enumeration test; one save in flight + name
  reservation; accurate refusal wording; render mode keeps snap; follower pane: no handlers/keyboard,
  retries; honest split note. Engines contract wired (F4, repair mode, path_exists, intermediate delete).
  Fallback to single-document mode: addFiles replace-instead-of-add (one small diff). Plan: runs/plan-P0-r4.md (r5 section).
- **Owns:** UI components/views, core/{inkEngine,insertEngine,viewportEngine,measureEngine}.svelte.ts, tauri.conf.json CSP.
- **Accepted by lead:** BUG-011, 012, 013, 014, 017, 019, 020, 022.
- **r2 fixes** (tests extended first, red in logs/runs/r2-red.log → green r2-green2.log; plan runs/plan-P0-r2.md):
  1. F11 BLOCKER fixed — DocumentTabs asks Save / Discard / Cancel before closing a dirty tab (active or
     inactive). Dirty = markups differ from what was loaded from disk / last saved (documentSessions.ts
     baselines). Save switches to the tab if needed, saves a new file, then closes. StudioTabs.test.ts.
  2. Probe fixed — GetInfo probe is a silent internal job (IpcClient.submitSilentJob; global listener in
     state.svelte.ts skips silent ids, waits for in-flight silent submits), runs once per document per run,
     never for password-protected files (no false "damaged"). StudioDamagedPdf/StudioPassword tests.
  3. Organize names — freeOutputPath(…, 'organized' | 'organizing'); at most one intermediate (rotations
     in ONE RunPipeline job, backend cleans its stage dir). OrganizeApply.test.ts.
  4. outputNames.freeOutputPath checks disk (read_file_bytes "not found" = free; stopgap).
  5. ipc.runJob: AbortSignal cancel (Organize "Cancel" → cancel_job) + 120 s no-progress stop, clear errors.
  6. Load catch: RenderingCancelledException → console.debug + ignore; other errors console.error +
     banner (stale loads log only). StudioSaveNames.test.ts mock throws both.
  7. Tab tooltip: "Switching tabs clears Undo history." (lead: add to KNOWN_ISSUES).
- **Engines asks:** (a) GetInfo for hostile files: end `Failed` with error_message naming the guard
  (F4: decompression ratio / object count / depth / watchdog) instead of a silent 1-page success; when the
  lopdf fallback opened it, emit `Completed{degraded:true}` (event must reach the frontend). (b) cheap
  `path_exists(path) -> bool` IPC (replace read_file_bytes probe). (c) either an `ExtractPages`/`DeletePages`
  pipeline step or `remove_app_output(path)` limited to files the app created, to drop the one Organize
  intermediate (`*.organizing-*.pdf`) left when pages are both reordered/deleted and rotated.
- **Tests:** verify.sh (LPDF_PRIO=p0) GREEN: gate, feature-map, studio-audit, svelte-check, vitest 255/255, tree-map. measure.session golden regenerated via vitest -u (BUG-022, lead-approved; note in engineGoldens.test.ts). Open-polyline test added (red with forced closing: logs/runs/polyline-red.log).
- **Blockers:** none.
- **Next (after lead says engines merged):** rebase onto main, then wire: F4_RESOURCE_GUARD → "This file
  looks damaged or hostile and was not opened"; Completed{degraded} → "Opened in repair mode" (non-blocking);
  path_exists replaces read_file_bytes probe in outputNames.pathExistsOnDisk; delete_organize_intermediate
  after successful Organize apply (log on error, keep KI-002, never fail the apply).

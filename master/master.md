# master/ — shared workspace (all platforms)

Everything every platform build needs lives here: the Rust crates, the desktop app (Svelte 5 frontend plus Tauri v2 shell), tests, the project protocol and logs, and the cross-platform docs. **All general development happens in this folder.** Platform folders (`../ubuntu`, `../fedora`, `../windows`, ...) receive only the files specific to their platform, when work on that platform needs them. See `../tree.md` for the full repository map.

## Everyday commands (run from `master/`)

| Task | Command |
|---|---|
| First checkout on a new machine | `git submodule update --init --recursive` (pdfcraft engine) and `bash corpus/fetch.sh` (test PDFs) |
| Frontend dependencies | `corepack enable` once (Tauri builds call `pnpm`), then `pnpm --dir app-tauri install --frozen-lockfile` |
| Run the app (dev) | `corepack pnpm --dir app-tauri tauri dev` |
| Frontend type check / tests | `corepack pnpm --dir app-tauri check` · `corepack pnpm --dir app-tauri exec vitest run` |
| Rust tests | `cargo test --workspace` (the `app-tauri` crate needs the webkit2gtk dev packages; see `../ubuntu/ubuntu.md`) |
| Project checks | `cargo xtask protocol-lint` · `cargo xtask feature-map-check` · `cargo xtask studio-audit --strict` |
| Full gate | `cargo xtask gate` (fmt, clippy, nextest, deny, audit, protocol-lint) |
| Platform package | `../ubuntu/packaging/build-deb.sh` · `../fedora/packaging/build-rpm.sh` · see `../windows/windows.md` |

Start every agent session by reading `../AGENTS.md`.

## Map of `master/`

Every entry has a one-line description. `master/vendor/pdfcraft` is an upstream submodule and is listed as one entry. Build output (`target/`, `node_modules/`, `build/`) and downloaded corpus PDFs are not tracked and not listed.

```text
master/
├── .cargo/                                                   # Cargo configuration for the workspace
│   ├── audit.toml                                            # cargo-audit settings (advisory ignore list)
│   └── config.toml                                           # Defines the `cargo xtask` alias
├── app-tauri/                                                # Desktop app: Svelte 5 frontend (src/) + Tauri v2 Rust shell (src-tauri/)
│   ├── .vscode/                                              # Editor settings for the frontend
│   │   ├── extensions.json                                   # Recommended VS Code extensions (Svelte, Tauri, rust-analyzer)
│   │   └── settings.json                                     # VS Code workspace settings
│   ├── src-tauri/                                            # Tauri v2 Rust shell (the desktop binary)
│   │   ├── capabilities/                                     # Tauri v2 permission allow-lists per window
│   │   │   ├── default.json                                  # Permissions for the main window
│   │   │   └── studio-split.json                             # Permissions for studio split windows
│   │   ├── icons/                                            # App icons used by every bundle (.icns for macOS, .ico for Windows)
│   │   │   ├── 128x128.png                                   # App icon 128 px
│   │   │   ├── 128x128@2x.png                                # App icon 256 px (HiDPI)
│   │   │   ├── 32x32.png                                     # App icon 32 px
│   │   │   ├── icon.icns                                     # macOS icon set
│   │   │   ├── icon.ico                                      # Windows icon
│   │   │   └── icon.png                                      # Base app icon
│   │   ├── src/                                              # Desktop shell source
│   │   │   ├── commands/                                     # Grouped IPC command modules
│   │   │   │   ├── mod.rs                                    # Command module index
│   │   │   │   └── studio.rs                                 # Studio IPC: prefs, annotation save/load/flatten, text blocks, snap geometry, scales, audio, takeoff, libraries
│   │   │   ├── lib.rs                                        # App setup, job manager wiring, core IPC commands, file dialogs (kdialog/zenity), window commands
│   │   │   └── main.rs                                       # Binary entry point (calls lib::run)
│   │   ├── tests/                                            # Integration tests for the desktop shell crate
│   │   │   ├── goldens/                                      # Characterization goldens for Rust studio commands (ADR-018 strangler step 1)
│   │   │   │   ├── takeoff.golden.json                       # Golden: takeoff CSV export bytes
│   │   │   │   └── text-blocks.standard-text.p0.golden.json  # Golden: text-block extract/edit on corpus standard-text.pdf page 1
│   │   │   └── hatch_save_e2e.rs                             # Tests (ERR-007 flow 5): save a hatched concave polygon to a new PDF, re-extract segments from the content stream, all inside
│   │   ├── .gitignore                                        # Ignores Tauri generated schemas/build output
│   │   ├── build.rs                                          # Tauri build script
│   │   ├── Cargo.toml                                        # Desktop binary crate manifest (Tauri plugins, workspace crates, Windows-only deps)
│   │   └── tauri.conf.json                                   # SHARED Tauri config (window, CSP, icons, generic bundle). OS overlays in <os>/packaging/ are merged at build
│   ├── src/                                                  # Svelte 5 single-page frontend
│   │   ├── lib/                                              # Frontend library code
│   │   │   ├── bindings/                                     # TypeScript types generated from Rust by ts-rs (do not edit by hand)
│   │   │   │   ├── AiModelInfo.ts                            # Generated type: imported local AI model info
│   │   │   │   ├── AiModelManifest.ts                        # Generated type: AI model manifest (hash, licence)
│   │   │   │   ├── AiWorkerRequest.ts                        # Generated type: request sent to the AI worker
│   │   │   │   ├── AiWorkerResponse.ts                       # Generated type: AI worker reply
│   │   │   │   ├── AnnotationDef.ts                          # Generated type: annotation definition
│   │   │   │   ├── AnnotationShape.ts                        # Generated type: annotation shape kinds
│   │   │   │   ├── AppSettings.ts                            # Generated type: persisted app settings
│   │   │   │   ├── ControlMessage.ts                         # Generated type: control-bus pointer/key sample
│   │   │   │   ├── ExtractiveSummaryReport.ts                # Generated type: basic (no-model) summary result
│   │   │   │   ├── HardwareTier.ts                           # Generated type: hardware tier L/M/H
│   │   │   │   ├── HatchPatternId.ts                         # Generated type: CAD hatch pattern ids
│   │   │   │   ├── JobEvent.ts                               # Generated type: job progress/done/error event
│   │   │   │   ├── JobPriority.ts                            # Generated type: job priority
│   │   │   │   ├── JobRecord.ts                              # Generated type: job list entry
│   │   │   │   ├── JobRequest.ts                             # Generated type: every job kind the backend accepts
│   │   │   │   ├── JobStatus.ts                              # Generated type: job state
│   │   │   │   ├── LlmSummaryOptions.ts                      # Generated type: local-LLM summary options
│   │   │   │   ├── LlmSummaryReport.ts                       # Generated type: local-LLM summary result
│   │   │   │   ├── MarkdownBatchOptions.ts                   # Generated type: Markdown batch options
│   │   │   │   ├── MarkdownBatchReport.ts                    # Generated type: Markdown batch result
│   │   │   │   ├── MarkdownBatchTarget.ts                    # Generated type: Markdown batch target format
│   │   │   │   ├── MarkdownDocFormat.ts                      # Generated type: Markdown document formats
│   │   │   │   ├── OfficeInputFormat.ts                      # Generated type: Office input formats
│   │   │   │   ├── OfficeOutputFormat.ts                     # Generated type: Office output formats
│   │   │   │   ├── OptimizeProfile.ts                        # Generated type: compression levels
│   │   │   │   ├── OutputPolicy.ts                           # Generated type: output naming/overwrite policy
│   │   │   │   ├── PdfAConformance.ts                        # Generated type: PDF/A levels
│   │   │   │   ├── PdfAPreflightReport.ts                    # Generated type: PDF/A preflight result
│   │   │   │   ├── Permissions.ts                            # Generated type: PDF permission flags
│   │   │   │   ├── PipelinePreset.ts                         # Generated type: workflow preset
│   │   │   │   ├── PipelineStep.ts                           # Generated type: workflow step
│   │   │   │   ├── RedactionRegion.ts                        # Generated type: redaction rectangle
│   │   │   │   ├── SceneNode.ts                              # Generated type: studio scene node
│   │   │   │   ├── SummaryLength.ts                          # Generated type: summary length
│   │   │   │   ├── ThemePreference.ts                        # Generated type: light/dark/system
│   │   │   │   ├── TranslateOptions.ts                       # Generated type: translate options
│   │   │   │   ├── TranslatePdfReport.ts                     # Generated type: translate result
│   │   │   │   └── WatermarkSettings.ts                      # Generated type: watermark settings
│   │   │   ├── components/                                   # Shared UI components
│   │   │   │   ├── studio/                                   # Components of the spatial "Studio" editor
│   │   │   │   │   ├── AnnotationLedger.audio.test.ts        # Tests: closing the audio-note dialog while recording asks "Discard recording?"; discard stops the recorder and mic
│   │   │   │   │   ├── AnnotationLedger.svelte               # Side drawer: annotation list, takeoff CSV, markup library, LaTeX/signature/audio dialogs
│   │   │   │   │   ├── CalibrationModal.svelte               # Set a page's drawing scale for measurements
│   │   │   │   │   ├── CalibrationModal.test.ts              # Tests: calibration dialog asks "Discard calibration?" on Escape/backdrop only when values changed
│   │   │   │   │   ├── DockableToolbar.svelte                # Main tool rail with grouped tool drawers, shortcuts, favourites
│   │   │   │   │   ├── DocumentTabs.svelte                   # Document tabs for multiple open PDFs in Studio (BUG-016)
│   │   │   │   │   ├── FavoritesBar.svelte                   # Pinned favourite tools/presets
│   │   │   │   │   ├── HatchDefs.svelte                      # SVG pattern definitions for CAD hatching fills
│   │   │   │   │   ├── HatchDefs.test.ts                     # Tests: hatch preview tiles are drawn from the same pattern parameters as the saved output
│   │   │   │   │   ├── labs.svelte.ts                        # Labs gate: the one place that decides if Labs tools (math text, shape-to-character) are visible/selectable; honest copy
│   │   │   │   │   ├── MeasureOverlay.svelte                 # Draws measurement lines, labels, ruler/protractor
│   │   │   │   │   ├── PillLayout.test.ts                    # Tests (BUG-045): property pill title on one line with tooltip; pill stacks above PINS and Help
│   │   │   │   │   ├── PropertyPill.svelte                   # Floating style panel for the active tool (colour, width, line style)
│   │   │   │   │   ├── RadialMenu.svelte                     # Circular quick-tool menu
│   │   │   │   │   ├── ShortcutSettings.svelte               # Keyboard shortcut editor
│   │   │   │   │   └── StudioIcon.svelte                     # SVG icon registry for studio tools
│   │   │   │   ├── ui/                                       # Shared UI component library (DESIGN_LANGUAGE.md): tokens only, strict audit; used by the sweep
│   │   │   │   │   ├── Button.svelte                         # Button: primary/secondary/destructive/plain, sizes, loading; disabled requires a reason (tooltip)
│   │   │   │   │   ├── ColorSwatch.svelte                    # Colour swatch button: aria-label names the colour, aria-pressed + ring for the selected one
│   │   │   │   │   ├── contrast.test.ts                      # Tests: dialog/title/body text contrast from tokens meets 4.5:1 (body) and 3:1 (large), light and dark
│   │   │   │   │   ├── Dialog.svelte                         # Dialog/Sheet: focus trap, Escape via commandEngine registerEscape
│   │   │   │   │   ├── EmptyState.svelte                     # Empty state: title, explanation, primary action
│   │   │   │   │   ├── ErrorState.svelte                     # Error state: what happened + what to do + optional Retry
│   │   │   │   │   ├── IconButton.svelte                     # Icon-only button; aria-label required
│   │   │   │   │   ├── Input.svelte                          # Text input with label, hint and error
│   │   │   │   │   ├── Progress.svelte                       # Progress bar for long jobs, optional Cancel
│   │   │   │   │   ├── Select.svelte                         # Select with label and error
│   │   │   │   │   ├── SidebarSection.svelte                 # Collapsible sidebar section
│   │   │   │   │   ├── Toggle.svelte                         # On/off switch with label
│   │   │   │   │   ├── Tooltip.svelte                        # Tooltip used for disabled reasons and icon labels
│   │   │   │   │   ├── tooltip.test.ts                       # Tests: tooltip clamped to the viewport and flips side when there is no room (anchor at x≈0)
│   │   │   │   │   ├── tooltipPlacement.ts                   # Tooltip placement: viewport clamp and side flip
│   │   │   │   │   ├── types.ts                              # Shared prop types for ui/ components
│   │   │   │   │   └── ui.test.ts                            # Tests: render, keyboard (Enter/Space, Tab, Escape) and disabled-reason exposure per ui/ component
│   │   │   │   ├── Dropzone.svelte                           # File drop/pick area (native OS paths)
│   │   │   │   ├── FaqModal.svelte                           # Help/FAQ dialog, lazy-loads translated FAQ
│   │   │   │   ├── FaqModalKeys.test.ts                      # Tests: FaqModal key behaviour (Escape via commandEngine, focus in inputs/dialogs) — characterization kept green across the move
│   │   │   │   ├── For.svelte                                # Tiny generic keyed list helper component
│   │   │   │   ├── GridEditor.svelte                         # Page-thumbnail grid for organizing pages (reorder, rotate, delete)
│   │   │   │   ├── MainLayout.svelte                         # App frame: sidebar + content + status bar, sidebar collapse
│   │   │   │   ├── organizeApply.ts                          # Organize Apply: runs reorder/delete/rotate jobs in sequence and reports the true result (BUG-014)
│   │   │   │   ├── PdfViewer.svelte                          # Read-only page preview using pdf.js
│   │   │   │   ├── Sidebar.svelte                            # Left navigation between tool groups
│   │   │   │   ├── StatusBar.svelte                          # Bottom status line (engine, job state)
│   │   │   │   ├── ToastContainer.svelte                     # Toast notifications
│   │   │   │   └── ViewShell.svelte                          # Shared tool-view frame: title, file bar, Progress (Cancel) and ErrorState (Retry)
│   │   │   ├── core/                                         # Studio engines: state + logic (no DOM), each unit-tested
│   │   │   │   ├── __fixtures__/                             # Characterization goldens of the current TS engines (regenerate: vitest -u)
│   │   │   │   │   ├── ink.eraser.golden.json                # Golden: partial/whole eraser output
│   │   │   │   │   ├── ink.rdp-and-outline.golden.json       # Golden: ink RDP simplification and stroke outline
│   │   │   │   │   ├── ink.recognize.golden.json             # Golden: smart-shape recognition results
│   │   │   │   │   ├── insert.signature-threshold.golden.json  # Golden: signature background threshold output
│   │   │   │   │   ├── measure.math.golden.json              # Golden: length/area/angle/perimeter math
│   │   │   │   │   ├── measure.session.golden.json           # Golden: calibrated measurement session
│   │   │   │   │   ├── selection.hit.golden.json             # Golden: selection hit-testing
│   │   │   │   │   ├── selection.transforms.golden.json      # Golden: move/resize/rotate/align/flip transforms
│   │   │   │   │   ├── shapes.created.golden.json            # Golden: created shapes and revision clouds
│   │   │   │   │   ├── shapes.paths.golden.json              # Golden: shape/cloud SVG path strings
│   │   │   │   │   ├── snap-geometry.standard-form.p0.golden.json  # Golden: snap segments extracted from corpus standard-form.pdf page 1
│   │   │   │   │   ├── snap.standard-form.p0.golden.json     # Golden: snap results on corpus geometry
│   │   │   │   │   └── snap.synthetic.golden.json            # Golden: snap results on synthetic geometry
│   │   │   │   ├── commandEngine.capture.test.ts             # Tests: Escape during chord capture cancels capture only (characterization + fix)
│   │   │   │   ├── commandEngine.space.test.ts               # Tests (BUG-043): Space activates focused interactive elements; canvas Space behaviour characterized and kept
│   │   │   │   ├── commandEngine.svelte.ts                   # Central keyboard shortcut/command dispatcher (only place keydown is handled)
│   │   │   │   ├── core.test.ts                              # Tests: viewport zoom/pan/fit and engine basics
│   │   │   │   ├── defaultPresets.json                       # Default pen/highlighter/shape presets
│   │   │   │   ├── defaultShortcuts.json                     # Default keyboard shortcuts
│   │   │   │   ├── engineGoldens.test.ts                     # Tests: compare current TS engine outputs with the goldens
│   │   │   │   ├── hatchPatterns.json                        # Hatch pattern parameters generated from lpdf-geom patterns.rs (single source for the on-screen preview)
│   │   │   │   ├── hatchPatterns.ts                          # Typed loader for hatchPatterns.json used by HatchDefs.svelte
│   │   │   │   ├── inkEngine.svelte.ts                       # Freehand ink strokes, smoothing, shape recognition, laser pointer
│   │   │   │   ├── insertAndTakeoff.test.ts                  # Tests: insert suite and takeoff CSV export
│   │   │   │   ├── insertEngine.svelte.ts                    # Insert tools: images, stamps, signatures, audio, "LaTeX" (Unicode-only), workspace library
│   │   │   │   ├── jobRunner.svelte.ts                       # Runs one backend job for a tool view: progress, Cancel, Retry, error state (wraps IpcClient.runJob)
│   │   │   │   ├── jobRunner.test.ts                         # Tests: jobRunner progress, cancel, retry and error states
│   │   │   │   ├── measureEngine.svelte.ts                   # Length/area/angle/volume measurement and scale maths
│   │   │   │   ├── parseNumber.ts                            # parseNumberOr: explicit number parsing for inputs (empty -> default, 0 stays 0)
│   │   │   │   ├── radialGeometry.test.ts                    # Tests: radial menu hit testing
│   │   │   │   ├── radialGeometry.ts                         # Radial menu geometry (sector/ring hit testing)
│   │   │   │   ├── radialLayout.json                         # Radial menu sectors and flyout tools
│   │   │   │   ├── selectionEngine.svelte.ts                 # Select, marquee/lasso, move/resize/rotate, group, align, flip
│   │   │   │   ├── shapeAndHatchEngine.svelte.ts             # Lines, arrows, polygons, clouds, hatching, review markups
│   │   │   │   ├── shapeAndSelection.test.ts                 # Tests: shapes and selection
│   │   │   │   ├── snapAndMeasure.test.ts                    # Tests: snapping and measurement
│   │   │   │   ├── snapEngine.svelte.ts                      # CAD-style snapping to PDF vector geometry (endpoint, midpoint, intersection)
│   │   │   │   ├── studioReactivity.test.ts                  # Tests: tool/preset reactivity and ink shape recognition
│   │   │   │   ├── themeContract.svelte.ts                   # Theme tokens (light/dark) used by every studio component
│   │   │   │   ├── themeContract.test.ts                     # Tests: themeContract is the single token source (applyToRoot, NEW tokens, glass-blur parity with app.css)
│   │   │   │   ├── toolEngine.svelte.ts                      # Active tool, tool presets and favourites
│   │   │   │   └── viewportEngine.svelte.ts                  # Canvas zoom/pan/fit maths
│   │   │   ├── gallery/                                      # Dev-only component gallery (excluded from release builds)
│   │   │   │   ├── Gallery.svelte                            # Component gallery: every ui/ component in every state, light + dark (only with VITE_LPDF_GALLERY=1)
│   │   │   │   └── marker.ts                                 # Single gallery marker constant used by Gallery.svelte and check-no-gallery.sh (positive + absent checks)
│   │   │   ├── i18n/                                         # Help/FAQ translations
│   │   │   │   ├── faq/                                      # FAQ content per language
│   │   │   │   │   ├── bn.json                               # FAQ in Bengali
│   │   │   │   │   ├── en.json                               # FAQ in English
│   │   │   │   │   ├── fr.json                               # FAQ in French
│   │   │   │   │   ├── ja.json                               # FAQ in Japanese
│   │   │   │   │   ├── ta.json                               # FAQ in Tamil
│   │   │   │   │   └── zh.json                               # FAQ in Chinese
│   │   │   │   └── faqStore.svelte.ts                        # Loads FAQ text for the chosen language
│   │   │   ├── views/                                        # One view per tool group
│   │   │   │   ├── __tests__/                                # View tests (Vitest + Testing Library, mocked IPC)
│   │   │   │   │   ├── A11yConformance.test.ts               # Tests: accessibility basics across views
│   │   │   │   │   ├── AiSummarizeView.test.ts               # Tests: AI summarize view
│   │   │   │   │   ├── BatchQueueView.test.ts                # Tests: batch queue view
│   │   │   │   │   ├── ConvertFromPdfView.test.ts            # Tests: convert-from view
│   │   │   │   │   ├── ConvertTabsKeys.test.ts               # Tests: Convert tabs roving tabindex; Arrow/Home/End via commandEngine key scope
│   │   │   │   │   ├── ConvertToPdfView.test.ts              # Tests: convert-to view
│   │   │   │   │   ├── EditViewLabs.test.ts                  # Tests: sample highlight (demo) is hidden unless Labs is on
│   │   │   │   │   ├── EditViewNumbers.test.ts               # Tests: Edit view number fields keep 0 and use defaults only when empty
│   │   │   │   │   ├── EngineReset.test.ts                   # Tests: every engine field is classified and reset on document switch (fails when a new field is unclassified)
│   │   │   │   │   ├── IpcAndSessionTimeouts.test.ts         # Tests: whenDocumentLive/probe timeouts, stall timer starts at dequeue, listener cleanup
│   │   │   │   │   ├── LabsGating.test.ts                    # Tests: Labs off by default, all Labs entry points hidden, honest labels derived from the classifier
│   │   │   │   │   ├── MeasurePerimeter.test.ts              # Tests: perimeter closes the shape (BUG-022)
│   │   │   │   │   ├── OrganizeApply.test.ts                 # Tests: Organize Apply keeps rotation, reports failures (BUG-014)
│   │   │   │   │   ├── OrganizeView.test.ts                  # Tests: organize view
│   │   │   │   │   ├── OutputPathsBug050.test.ts             # Tests (BUG-050): every view builds a collision-safe output path that is never the input or an existing file
│   │   │   │   │   ├── PipelinesView.test.ts                 # Tests: pipelines view
│   │   │   │   │   ├── SettingsView.test.ts                  # Tests: settings view
│   │   │   │   │   ├── SignatureDraw.test.ts                 # Tests: drawn signature saved as ink, not text (BUG-017)
│   │   │   │   │   ├── StudioDamagedPdf.test.ts              # Tests: malformed PDFs show an error/degraded notice (BUG-018)
│   │   │   │   │   ├── StudioDocumentStore.test.ts           # Tests: document store activate(), Organize round trip, save refuses on switch, per-document scales, split view Labs-only
│   │   │   │   │   ├── StudioEditText.test.ts                # Tests: Edit Text hotspots reachable by pointer (BUG-013)
│   │   │   │   │   ├── StudioEmptyState.test.ts              # Tests: Studio empty state, no phantom pages, document-only actions disabled (UI-BUG-006)
│   │   │   │   │   ├── StudioEraserUndo.test.ts              # Tests: erasers are undoable, undo restores the right stroke (BUG-019)
│   │   │   │   │   ├── StudioLoadRaces.test.ts               # Tests: tab switch/close mid-render keeps markups and still prompts; draw-during-load merge
│   │   │   │   │   ├── StudioPassword.test.ts                # Tests: encrypted PDFs prompt for a password (BUG-011)
│   │   │   │   │   ├── StudioRound5.test.ts                  # Tests (P0 round 5): close via store leaves no session, inserts blocked until restored, save refuses without page geometry, one save in flight
│   │   │   │   │   ├── StudioSaveNames.test.ts               # Tests: repeated saves use new collision-safe names (BUG-021)
│   │   │   │   │   ├── StudioTabs.test.ts                    # Tests: second document opens in a tab, view state kept per tab (BUG-016)
│   │   │   │   │   ├── StudioUndoTooltip.test.ts             # Tests: Studio Undo tooltip text follows commands.undoLabel; aria-label stays "Undo"
│   │   │   │   │   ├── TauriCsp.test.ts                      # Tests: CSP allows data:/blob: images, connect-src still IPC-only (BUG-012)
│   │   │   │   │   └── ViewportFit.test.ts                   # Tests: Fit Width/Page use real page size and offset (BUG-020)
│   │   │   │   ├── studio/                                   # The spatial Studio editor (default view)
│   │   │   │   │   ├── documentSessions.ts                   # Per-document Studio session state (view, page) for tab switching
│   │   │   │   │   ├── outputNames.ts                        # Collision-safe output file names, never overwrite previous outputs (BUG-021)
│   │   │   │   │   ├── pageGeometry.ts                       # Page geometry shared by the canvas and the saver: world (laid-out CSS px) to PDF user space (pt, y up) conversion
│   │   │   │   │   ├── SpatialCanvas.svelte                  # Studio canvas: page rendering, pointer input, layers, responsive top bar, empty state (largest file)
│   │   │   │   │   ├── SplitLayout.svelte                    # Side-by-side document panes with draggable divider
│   │   │   │   │   └── studioSave.ts                         # The single Studio saver: snapshots path/seq/revision, refuses ("Nothing was written") if the document changed or is still loading
│   │   │   │   ├── AiSummarizeView.svelte                    # PDF Intelligence: summarize / translate (basic mode + optional local model)
│   │   │   │   ├── BatchQueueView.svelte                     # Batch job queue
│   │   │   │   ├── ConvertFromPdfView.svelte                 # PDF → images/Word/HTML/text/Office/PDF-A
│   │   │   │   ├── ConvertToPdfView.svelte                   # Images/Office/HTML → PDF
│   │   │   │   ├── ConvertView.svelte                        # Convert hub (switches to/from views)
│   │   │   │   ├── EditView.svelte                           # Watermark, header/footer, page numbers
│   │   │   │   ├── FormsView.svelte                          # Form filling and flattening
│   │   │   │   ├── MarkdownView.svelte                       # Markdown ↔ PDF/DOCX/HTML tools
│   │   │   │   ├── OptimizeView.svelte                       # Compress, repair, OCR
│   │   │   │   ├── OrganizeView.svelte                       # Merge, split, rotate, delete, extract pages
│   │   │   │   ├── PipelinesView.svelte                      # Chained tool workflows (presets)
│   │   │   │   ├── SecurityView.svelte                       # Protect, unlock, redact, sign
│   │   │   │   └── SettingsView.svelte                       # Settings: theme, tier, output policy, logs, AI models
│   │   │   ├── i18n.ts                                       # Translation lookup helper
│   │   │   ├── ipc.ts                                        # Typed wrappers around Tauri `invoke` (jobs, settings, dialogs, files)
│   │   │   └── state.svelte.ts                               # Global app state (current view, open files, theme, sidebar) with runes
│   │   ├── locales/                                          # UI translation strings
│   │   │   └── en.json                                       # English UI strings
│   │   ├── routes/                                           # SvelteKit routes (single page app)
│   │   │   ├── gallery/                                      # Gallery route (build-flag gated)
│   │   │   │   └── +page.svelte                              # Gallery route page; renders nothing unless VITE_LPDF_GALLERY=1
│   │   │   ├── +layout.svelte                                # Root layout: global CSS + toasts
│   │   │   ├── +layout.ts                                    # Disables SSR/prerender (pure client SPA for Tauri)
│   │   │   └── +page.svelte                                  # The one page: switches between tool views (default Studio) and handles OS "open with" files
│   │   ├── app.css                                           # Global styles and design tokens (CSS variables, light/dark, glass theme)
│   │   ├── app.html                                          # HTML shell that SvelteKit renders into
│   │   └── vitest.d.ts                                       # Vitest type references
│   ├── static/                                               # Files served as-is by the webview
│   │   ├── favicon.png                                       # App favicon
│   │   ├── pdf.min.js                                        # Vendored Mozilla pdf.js, used only to draw page previews (ADR-009 exception)
│   │   ├── pdf.worker.min.js                                 # pdf.js web worker companion
│   │   ├── svelte.svg                                        # Template logo asset
│   │   ├── tauri.svg                                         # Template logo asset
│   │   └── vite.svg                                          # Template logo asset
│   ├── .gitignore                                            # Ignores node_modules, build output and SvelteKit cache
│   ├── package.json                                          # Frontend dependencies and scripts (dev, build, check, tauri)
│   ├── pnpm-lock.yaml                                        # Pinned npm dependency versions (pnpm)
│   ├── README.md                                             # Stock Tauri + SvelteKit template readme
│   ├── svelte.config.js                                      # SvelteKit config: static adapter, SPA fallback (ADR-008 exception)
│   ├── tsconfig.json                                         # TypeScript compiler settings
│   ├── vite.config.js                                        # Vite dev server/build config (port 1420, Tauri-friendly settings)
│   ├── vitest-setup.ts                                       # Vitest setup: jest-dom matchers and Tauri IPC mocks
│   └── vitest.config.ts                                      # Vitest config (jsdom environment, test file globs)
├── corpus/                                                   # Test PDFs: manifest + fetch script (the PDFs themselves are downloaded, not committed)
│   ├── fetch.sh                                              # Downloads/generates the corpus incl. scanned.pdf and encrypted.pdf (POSIX sh; needs wget/curl, qpdf, pdftoppm)
│   ├── MANIFEST                                              # List of corpus files and what each tests
│   └── SHA256SUMS                                            # Expected hashes of corpus files
├── crates/                                                   # Rust library/binary crates (the engine side)
│   ├── lpdf-ai-worker/                                       # Sandboxed local-AI worker process (JSON over stdin/stdout)
│   │   ├── src/                                              # Crate source
│   │   │   ├── lib.rs                                        # Request/response types and session logic
│   │   │   └── main.rs                                       # Worker binary entry point
│   │   ├── tests/                                            # Integration tests
│   │   │   └── ai_worker_tests.rs                            # Tests: AI worker protocol
│   │   └── Cargo.toml                                        # Crate manifest
│   ├── lpdf-cli/                                             # `localpdf` command-line tool on the same engine
│   │   ├── src/                                              # Crate source
│   │   │   ├── lib.rs                                        # CLI commands and engine construction
│   │   │   └── main.rs                                       # CLI binary entry point
│   │   ├── tests/                                            # Integration tests
│   │   │   └── cli_tests.rs                                  # Tests: end-to-end CLI runs on the corpus
│   │   └── Cargo.toml                                        # Crate manifest
│   ├── lpdf-control-bus/                                     # Recordable pointer/key input stream for the studio (replay testing)
│   │   ├── src/                                              # Crate source
│   │   │   └── lib.rs                                        # Control message types, executor trait, replay
│   │   ├── tests/                                            # Integration tests
│   │   │   └── replay.rs                                     # Tests: deterministic replay
│   │   └── Cargo.toml                                        # Crate manifest
│   ├── lpdf-core-types/                                      # Shared types: error codes, settings, config, hardware tier
│   │   ├── src/                                              # Crate source
│   │   │   ├── config.rs                                     # App settings/config and AI model manifest types
│   │   │   ├── error.rs                                      # `LpdfError` typed errors
│   │   │   └── lib.rs                                        # Crate root, re-exports
│   │   └── Cargo.toml                                        # Crate manifest
│   ├── lpdf-engine-fallback/                                 # Backup engine (lopdf) for repair/rewrite when pdfcraft fails; many ops "unsupported"
│   │   ├── src/                                              # Crate source
│   │   │   └── lib.rs                                        # FallbackAdapter implementation
│   │   └── Cargo.toml                                        # Crate manifest
│   ├── lpdf-engine-pdfcraft/                                 # Primary engine adapter over pdfcraft + Office/Pandoc sidecars + summarizer/translator
│   │   ├── src/                                              # Crate source
│   │   │   ├── lib.rs                                        # PdfcraftAdapter: every PDF operation (note: `render` is still a stub)
│   │   │   ├── office_sidecar.rs                             # LibreOffice headless bridge (offline, sandboxed, timeouts)
│   │   │   ├── pandoc_sidecar.rs                             # Pandoc bridge for Markdown/DOCX/HTML
│   │   │   ├── summarizer.rs                                 # Extractive (no-model) and local-LLM summarizer
│   │   │   └── translator.rs                                 # Experimental block-level PDF translation
│   │   ├── tests/                                            # Integration tests
│   │   │   ├── extraction_tests.rs                           # Tests: text/image extraction
│   │   │   ├── pandoc_tests.rs                               # Tests: Pandoc sidecar (mock binaries)
│   │   │   ├── pdfa_tests.rs                                 # Tests: PDF/A preflight
│   │   │   ├── sidecar_tests.rs                              # Tests: Office sidecar (mock binaries, isolation)
│   │   │   └── summarizer_tests.rs                           # Tests: summarizer
│   │   └── Cargo.toml                                        # Crate manifest (pdfcraft crates pinned by git rev)
│   ├── lpdf-engine-port/                                     # `PdfEngine` trait every engine implements + shared contract tests
│   │   ├── src/                                              # Crate source
│   │   │   └── lib.rs                                        # Trait, option/report types, contract test macro
│   │   └── Cargo.toml                                        # Crate manifest
│   ├── lpdf-engine-switcher/                                 # Tries pdfcraft then fallback; also hosts fuzz/hostile/perf/soak/SBOM harnesses
│   │   ├── src/                                              # Crate source
│   │   │   ├── fuzz.rs                                       # Deterministic fuzzing of engine inputs
│   │   │   ├── guard.rs                                      # F4 resource guard: refuses hostile files (e.g. declared stream length larger than the file) before any engine parses them
│   │   │   ├── hostile.rs                                    # Hostile-PDF generators (bombs, broken xref, deep trees)
│   │   │   ├── lib.rs                                        # EngineSwitcher (primary → fallback, degraded mode)
│   │   │   ├── perf.rs                                       # Per-tier performance/memory budget checks
│   │   │   ├── sbom.rs                                       # CycloneDX SBOM + supply-chain checks (licences, forbid(unsafe) allow-list)
│   │   │   └── soak.rs                                       # Soak / leak testing
│   │   ├── tests/                                            # Integration tests
│   │   │   ├── fuzz_smoke_tests.rs                           # Tests: fuzz/hostile/perf/sbom/a11y/soak smoke runs
│   │   │   └── guard_tests.rs                                # Tests: F4 guard trips on hostile corpus files and passes normal ones
│   │   └── Cargo.toml                                        # Crate manifest
│   ├── lpdf-fs-safety/                                       # Atomic writes, filename sanitizing, child-process sandbox (Linux unshare, Windows Job Object)
│   │   ├── src/                                              # Crate source
│   │   │   └── lib.rs                                        # Safety primitives; Windows FFI isolated in cfg(windows) modules (ADR-015)
│   │   ├── tests/                                            # Integration tests
│   │   │   └── fs_safety_tests.rs                            # Tests: atomic write, sanitizing
│   │   └── Cargo.toml                                        # Crate manifest (Windows-only windows-sys)
│   ├── lpdf-geom/                                            # Geometry adapter crate (ADR-018): hatching behind our own API; legacy impl now, cadcraft next
│   │   ├── src/                                              # Crate source
│   │   │   ├── cadcraft.rs                                   # Cadcraft-backed hatch implementation (HatchEngine::Cadcraft): real polygon clipping (ERR-007)
│   │   │   ├── fill.rs                                       # Studio hatch clipping (original work): every hatch line clipped to the polygon by even-odd rule over multiple rings (holes); half-open vertex rule; work cap
│   │   │   ├── legacy.rs                                     # Legacy hatch implementation moved from studio.rs (byte-identical)
│   │   │   ├── lib.rs                                        # Adapter API: hatch_segments, HatchEngine switch, GeomError codes
│   │   │   └── patterns.rs                                   # Studio hatch pattern definitions (original work): 9 patterns as line families (angle, pitch, offset, phase, dash)
│   │   ├── tests/                                            # Integration tests
│   │   │   ├── goldens/                                      # Hatch goldens
│   │   │   │   ├── hatch_cadcraft.golden.json                # Golden: cadcraft hatch segments per pattern (rect + concave L), all inside the loop
│   │   │   │   ├── hatch_legacy.golden.json                  # Golden: legacy hatch segments per pattern (rect + concave L)
│   │   │   │   └── hatch_studio.golden.json                  # Golden: Studio hatch segments per pattern (ERR-007 intentional change, approved by owner 2026-11-06)
│   │   │   ├── cadcraft_parity.rs                            # Tests: cadcraft hatch inside concave polygons, all patterns distinct, parity notes vs legacy
│   │   │   ├── hatch_golden.rs                               # Tests: hatch output vs goldens (ERR-007 cases marked known-wrong)
│   │   │   ├── studio_golden.rs                              # Tests: Studio hatch output vs hatch_studio golden
│   │   │   └── studio_hatch.rs                               # Tests (ERR-007): segments inside rect/concave L/star/holed polygon, no edge crossings, all patterns distinct
│   │   └── Cargo.toml                                        # Crate manifest
│   ├── lpdf-hw/                                              # Hardware detection → tier L/M/H and budgets
│   │   ├── src/                                              # Crate source
│   │   │   └── lib.rs                                        # HardwareProfile (RAM/cores via sysinfo)
│   │   └── Cargo.toml                                        # Crate manifest
│   ├── lpdf-jobs/                                            # Background job queue: priorities, cancel, progress events, pipelines
│   │   ├── src/                                              # Crate source
│   │   │   ├── events.rs                                     # Job progress/done/error events
│   │   │   ├── lib.rs                                        # Crate root + RAM budget / worker concurrency (sysinfo)
│   │   │   ├── output_guard.rs                               # Output path guard (BUG-050/032): refuses outputs that canonicalize to the input or an existing file (typed OutputPathError)
│   │   │   ├── pipeline.rs                                   # Workflow steps and built-in presets
│   │   │   └── queue.rs                                      # JobManager and every job kind (runs in-process today; sandboxed worker not wired yet)
│   │   ├── tests/                                            # Integration tests
│   │   │   ├── bug018_tests.rs                               # Tests (BUG-018): GetInfo on hostile files ends Failed with F4 code; fallback opens report degraded
│   │   │   ├── bug050_tests.rs                               # Tests (BUG-050): a job whose output equals the input fails with OUTPUT_PATH_UNSAFE and leaves the original intact
│   │   │   ├── output_guard_tests.rs                         # Tests: output_guard unit cases (same path, .., symlink, existing file, distinct new file)
│   │   │   └── queue_tests.rs                                # Tests: queue behaviour
│   │   └── Cargo.toml                                        # Crate manifest
│   ├── lpdf-tools-convert/                                   # Thin convert wrappers over PdfEngine (currently unused by other crates)
│   │   ├── src/                                              # Crate source
│   │   │   └── lib.rs                                        # Convert helpers
│   │   └── Cargo.toml                                        # Crate manifest
│   ├── lpdf-tools-edit/                                      # Thin edit wrappers over PdfEngine (currently unused)
│   │   ├── src/                                              # Crate source
│   │   │   └── lib.rs                                        # Edit helpers
│   │   └── Cargo.toml                                        # Crate manifest
│   ├── lpdf-tools-markdown/                                  # Thin Markdown wrappers over PdfEngine (currently unused)
│   │   ├── src/                                              # Crate source
│   │   │   └── lib.rs                                        # Markdown helpers
│   │   └── Cargo.toml                                        # Crate manifest
│   ├── lpdf-tools-optimize/                                  # Thin optimize wrappers over PdfEngine (currently unused)
│   │   ├── src/                                              # Crate source
│   │   │   └── lib.rs                                        # Optimize helpers
│   │   └── Cargo.toml                                        # Crate manifest
│   ├── lpdf-tools-organize/                                  # Thin organize wrappers over PdfEngine (currently unused)
│   │   ├── src/                                              # Crate source
│   │   │   └── lib.rs                                        # Organize helpers
│   │   └── Cargo.toml                                        # Crate manifest
│   ├── lpdf-tools-security/                                  # Thin security wrappers over PdfEngine (currently unused)
│   │   ├── src/                                              # Crate source
│   │   │   └── lib.rs                                        # Security helpers
│   │   └── Cargo.toml                                        # Crate manifest
│   └── lpdf-worker/                                          # `lpdf-worker` sandboxed job process (Landlock); built and shipped, not yet called by the app
│       ├── src/                                              # Crate source
│       │   └── main.rs                                       # Worker entry: applies sandbox, runs one job
│       └── Cargo.toml                                        # Crate manifest
├── docs/                                                     # Cross-platform design specs, plans and audits
│   ├── adr/                                                  # Long-form architecture decision records
│   │   ├── ADR-012-drawboard-artcraft-spatial-studio.md      # Studio editor decision (registered as ADR-014 in logs/DECISIONS_ADR.md)
│   │   └── ADR-018-upstream-engines.md                       # ADR-018 rev 2 (PROPOSED): consume craft engines as pinned deps behind adapter crates, WASM spike, upstream manifest
│   ├── audit/                                                # Earlier audit reports
│   │   ├── debug.md                                          # Debug log for phases 11.7–12.1b
│   │   └── INVENTORY.md                                      # V1 architecture inventory (Oct 2026 snapshot; partly outdated)
│   ├── upstream/                                             # Drafts of issues/PRs for upstream projects (not posted without owner approval)
│   │   └── winprint-optional-pdfium-build-deps.md            # Draft issue: make winprint's PDFium download build-deps optional (ADR-017)
│   ├── APPLE_HIG_GLASS_SPEC.md                               # "Frosted glass" visual design system spec (all platforms)
│   ├── ARCHITECTURE_AND_COMPLIANCE_AUDIT.md                  # Audit of studio phases D1/D2 against the plan
│   ├── CRAFT_ECOSYSTEM_SURVEY.md                             # Survey of the 10 storytold *craft repos: licences, deps, unsafe, what to port/adopt (ADR-018 PROPOSED)
│   ├── DESIGN_LANGUAGE.md                                    # App-wide design language: principles, tokens (themeContract + NEW), shared ui/ components (create vs reuse), patterns, wording guide, sweep "done" rules
│   ├── DRAWBOARD_FEATURE_CHECKLIST.md                        # 119-item studio feature checklist with status (checked by `cargo xtask feature-map-check`)
│   ├── DRAWBOARD_STUDIO_MASTER_PLAN.md                       # Plan for the spatial Studio editor (phases D1–D6)
│   ├── ENGINE_REUSE_MATRIX.md                                # ADR-018 R1 matrix: each hand-written engine function → craft candidate → ADOPT/ADOPT-WASM/KEEP-PORT/REJECT
│   ├── PROTOTYPE_SCOPE.md                                    # v0.2.0-prototype scope: 12 golden-path flows with pass criteria, corpus, triage rules, milestones
│   ├── RAM_BUDGET.md                                         # Measured RAM per job class, training budget, scheduler + cgroup protection rules
│   ├── STUDIO_UI_UX_OVERHAUL_SPEC.md                         # Studio UI/UX overhaul spec (phases U1–U5)
│   ├── UI_BACKLOG.md                                         # UI items deferred to after the release (need behaviour or redesign beyond the consistency sweep)
│   ├── UI_CONSISTENCY_INVENTORY.md                           # Generated by `cargo xtask studio-audit --write-inventory`: violations per screen/file and rule, top issues
│   ├── UPGRADING.md                                          # DRAFT procedure for bumping a pinned upstream engine (branch → gate → verifier → log)
│   └── UPSTREAM_MANIFEST.toml                                # Every adopted craft crate, patch and KEEP-PORT source: repo, rev, licence, adapter, last-checked
├── e2e/                                                      # Golden-path E2E tests (12 flows, tauri-driver/WebDriver, Python stdlib; dev only)
│   ├── flows/                                                # One module per PROTOTYPE_SCOPE flow
│   │   ├── __init__.py                                       # Flow registry
│   │   ├── flow01_install_launch.py                          # Flow 1: install + cold launch < 3 s, no console errors
│   │   ├── flow02_open.py                                    # Flow 2: open small/large/scanned/encrypted/malformed PDFs
│   │   ├── flow03_navigate.py                                # Flow 3: pan/zoom/pinch/fit, thumbnails, multiple documents
│   │   ├── flow04_ink.py                                     # Flow 4: pen, highlighter, erasers, undo/redo
│   │   ├── flow05_shapes_hatch.py                            # Flow 5: shapes, polygon-clipped distinct hatching, revision cloud
│   │   ├── flow06_measure.py                                 # Flow 6: calibrate, length/area/angle, ledger, CSV export
│   │   ├── flow07_insert.py                                  # Flow 7: text box, callout, note, stamp, signature
│   │   ├── flow08_redact.py                                  # Flow 8: redact and confirm removal with pdftotext/qpdf
│   │   ├── flow09_edit_text.py                               # Flow 9: edit PDF text in place
│   │   ├── flow10_organize.py                                # Flow 10: reorder, rotate, delete, merge pages
│   │   ├── flow11_save_reopen.py                             # Flow 11: save new file, original untouched, reopen in app/poppler/pdf.js
│   │   ├── flow12_soak.py                                    # Flow 12: scripted soak, crash/memory/fd/temp checks
│   │   ├── flow13_gallery.py                                 # E2E flow 13: gallery screenshots light/dark at 1200x800 and 1600x1000 (default, focus, hover/disabled, dialog), no console errors
│   │   ├── flow14_sweep_organize.py                          # E2E smoke (sweep): Organize screen opens, light/dark screenshots at 2 sizes, no console errors
│   │   ├── flow15_sweep_shell.py                             # E2E smoke (sweep): app shell (Sidebar, StatusBar, MainLayout) light/dark screenshots at 2 sizes, no console errors
│   │   ├── flow16_sweep_overlays.py                          # E2E smoke (sweep): toasts and the FAQ dialog, light/dark screenshots at 2 sizes, no console errors
│   │   ├── flow17_sweep_studio_docs.py                       # E2E smoke (sweep): document tabs, favourites bar and split layout, light/dark screenshots at 2 sizes, no console errors
│   │   ├── flow18_sweep_studio_panels.py                     # E2E smoke (sweep): calibration dialog and shortcut settings, light/dark screenshots at 2 sizes, no console errors
│   │   ├── flow19_sweep_toolbar.py                           # E2E smoke (sweep): dockable toolbar, radial menu and measure overlay, light/dark screenshots at 2 sizes, no console errors
│   │   ├── flow20_sweep_ledger.py                            # E2E smoke (sweep): annotation ledger and its dialogs, light/dark screenshots at 2 sizes, no console errors
│   │   ├── flow21_sweep_canvas_chrome.py                     # E2E smoke (sweep): Studio canvas chrome (toolbars, banners, overlays), light/dark screenshots at 2 sizes, no console errors
│   │   ├── flow22_sweep_pill.py                              # E2E smoke (sweep): property pill (compact selects, swatches, presets), light/dark screenshots at 2 sizes, no console errors
│   │   └── flow23_sweep_views.py                             # E2E smoke (sweep): Convert tabs, Optimize and Edit views, light/dark screenshots at 2 sizes, no console errors
│   ├── .gitignore                                            # Ignores e2e/out/ (reports, screenshots, copied binary)
│   ├── common.py                                             # Ground-truth helpers (poppler/qpdf/hashes), process sampling, isolation (private D-Bus/XDG)
│   ├── ink_match.py                                          # Matcher for content-less annotations (drawn signature): one /Ink or /Stamp with non-empty InkList or AP, /Rect on page and covering the drawn stroke, enough points (BUG-046)
│   ├── pkg_smoke.py                                          # Clean-user install smoke test for rpm/deb/AppImage (flows 1-2 against the packaged binary, fresh HOME)
│   ├── run.py                                                # Runner: python3 master/e2e/run.py [--flows] [--binary] [--soak-minutes]
│   ├── test_flow07_matcher.py                                # Self-test for ink_match.py: 9 synthetic cases (empty, duplicate, off-page, too few points fail; real strokes pass)
│   ├── test_leak_check.py                                    # Tests (BUG-031): post-run leak check catches leaked harness/dbus processes; nothing left after bus stop or SIGTERM
│   └── wd.py                                                 # Minimal W3C WebDriver client (urllib)
├── fuzz/                                                     # cargo-fuzz workspace (libFuzzer targets)
│   ├── fuzz_targets/                                         # Fuzz entry points
│   │   ├── fuzz_engine_switcher.rs                           # Fuzz: engine switcher
│   │   ├── fuzz_fallback_load.rs                             # Fuzz: fallback loader
│   │   ├── fuzz_fs_and_ai_rpc.rs                             # Fuzz: filename sanitizer + AI worker RPC
│   │   └── fuzz_pdfcraft_load.rs                             # Fuzz: pdfcraft loader
│   ├── .gitignore                                            # Ignores fuzz corpus/artifacts
│   └── Cargo.toml                                            # Fuzz crate manifest
├── logs/                                                     # Project logs (append-only, except CHECKPOINT which is overwritten)
│   ├── checkpoints/                                          # Per-worktree builder checkpoints (each builder keeps its own current)
│   │   ├── engines.md                                        # Engines builder state: task, files, tests, next step, blockers
│   │   ├── fixes.md                                          # Fixes builder state: task, files, tests, next step, blockers
│   │   └── release.md                                        # Release builder state: task, files, tests, next step, blockers
│   ├── ADR-008-sveltekit-spa.md                              # Detail for the SvelteKit SPA exception (registered as ADR-011 in DECISIONS_ADR)
│   ├── ADR-009-vendored-pdfjs.md                             # Detail for the vendored pdf.js exception (registered as ADR-012 in DECISIONS_ADR)
│   ├── AGENT_ACTIVITY.md                                     # One line per agent stop (written by hooks)
│   ├── BUGS.md                                               # Sprint bug list with P0/P1/P2 triage (lead-owned)
│   ├── CHECKPOINT.md                                         # Single "where are we now" snapshot: phase, status, next actions
│   ├── CHECKPOINT_HISTORY.md                                 # Older CHECKPOINT snapshots (append-only), moved out to keep CHECKPOINT short
│   ├── DECISIONS_ADR.md                                      # Decision register (all ADRs, including the ID erratum)
│   ├── DOGFOOD.md                                            # Owner's hands-on testing notes; lead triages into BUGS.md
│   ├── DOGFOOD_AUTO.md                                       # Automated real-world PDF dogfood results (files by anonymised DF-xx ID only; mapping kept outside the repo)
│   ├── ERROR_LOG.md                                          # Errors, root causes and fixes
│   ├── FLAKY.md                                              # Tests that passed only on rerun, with cause and action
│   ├── PLAN_LOG.md                                           # Plans written before each phase
│   ├── PROGRESS_LOG.md                                       # What was done per phase, with verification evidence
│   ├── PROMPT_LOG.md                                         # User prompts and how they were interpreted
│   ├── ROUTING.md                                            # Model routing table, escalation log and per-milestone routing tuning notes
│   ├── sbom.cdx.json                                         # Software bill of materials (regenerated by `cargo xtask sbom`)
│   └── UI_AUDIT_LOG.md                                       # UI bug register and verification notes
├── protocol/                                                 # Project rules (read-only except where noted)
│   ├── 01_MASTER_CONTEXT.md                                  # Product goals and key technology decisions
│   ├── 02_RULES_DOS_DONTS.md                                 # Hard rules: DOs and DON'Ts
│   ├── 03_FAILSAFES.md                                       # Failsafes F1–F27 (each needs an automated test)
│   ├── 04_INSTRUCTION_FADE_PREVENTION.md                     # Anchor block, pre-commit checklist, drift detectors
│   ├── 05_PHASES.md                                          # Phase plan 0–14
│   ├── 06_ARCHITECTURE.md                                    # Process model, IPC, security, file management, hardware tiers
│   ├── 07_FEATURE_MAP.md                                     # The 7-column tool list and how each tool is built
│   ├── 08_DEPENDENCY_VETTING.md                              # Rules for adding any dependency
│   ├── 09_DIRECTORY_TREE.md                                  # Directory tree rule (now points to /tree.md)
│   ├── A11Y_AUDIT.md                                         # Accessibility audit report
│   ├── ENGINE_NOTES.md                                       # Verified pdfcraft/Tauri API notes
│   ├── ENV.md                                                # Recorded toolchain versions per dev machine
│   ├── perf_budgets.json                                     # Performance budget numbers checked by perf tests
│   └── THREAT_MODEL.md                                       # Threat model and mitigations
├── tools/                                                    # Developer scripts and agent-process docs
│   ├── hooks/                                                # Git hooks (install: `git config core.hooksPath master/tools/hooks`)
│   │   ├── claude/                                           # Claude Code hook scripts used by .claude/settings.json
│   │   │   ├── activity.py                                   # Stop/SubagentStop: appends one status line to logs/AGENT_ACTIVITY.md
│   │   │   ├── format.py                                     # PostToolUse formatter: rustfmt / prettier (prettier-clean files only) on the touched file
│   │   │   └── guard.py                                      # PreToolUse guard: blocks cargo clean, git push/reset --hard, rm -rf outside target, builds outside run.sh
│   │   └── pre-commit                                        # Blocks deletions not listed in master/.allowed-deletions
│   ├── tree/                                                 # Generator + data for /tree.md and the per-folder maps
│   │   ├── headers/                                          # Hand-written intro text placed above each generated map
│   │   │   ├── android.md                                    # Intro for android/android.md
│   │   │   ├── fedora.md                                     # Intro + build steps for fedora/fedora.md
│   │   │   ├── ios.md                                        # Intro for ios/ios.md
│   │   │   ├── macos.md                                      # Intro for macos/macos.md
│   │   │   ├── master.md                                     # Intro + everyday commands for master/master.md
│   │   │   ├── tree.md                                       # Intro + layout rules for /tree.md
│   │   │   ├── ubuntu.md                                     # Intro + setup/build/known issues for ubuntu/ubuntu.md
│   │   │   └── windows.md                                    # Intro + build steps for windows/windows.md
│   │   ├── descriptions.tsv                                  # One line per file/folder: path TAB short description (source of every map entry)
│   │   └── render_tree.py                                    # Regenerates all maps; `--check` fails when a file is unmapped or a map is stale
│   ├── ANTIGRAVITY_PROMPTS.md                                # Prompts used with the Antigravity agent for phases 11.7–12.1
│   ├── ANTIGRAVITY_RUNBOOK.md                                # Step runbook for those phases
│   ├── build-lock.sh                                         # Runs a heavy build under the shared flock lock with CARGO_BUILD_JOBS=2 and the shared target dir (ADR-020)
│   ├── check-no-gallery.sh                                   # verify.sh step: builds the release frontend without the flag and fails if gallery code is present
│   ├── CLAUDE_AUDIT_CONTEXT.md                               # Brief for an independent read-only audit
│   ├── make_audit_bundle.sh                                  # Collects evidence into audit_bundle.tar.gz for external review
│   ├── measure-ram.sh                                        # Measures a job's whole-cgroup peak and largest-process RSS into logs/runs/ram-measure.tsv
│   ├── protect.sh                                            # Wraps each agent job: cgroup MemoryMax/High from live MemAvailable, OOM preference, nice/ionice
│   ├── ram-budget.env                                        # Measured peak inputs for the scheduler (HEAVY/LIGHT MiB)
│   ├── restore-context.sh                                    # SessionStart hook (compact/resume/startup): prints CHECKPOINT + STATE_AUTO back into context
│   ├── routing-stats.py                                      # Escalation rate per task type from logs/ROUTING.md, with tier-move suggestions (> 25 % up; Sonnet-high 0/10+ down)
│   ├── run.sh                                                # Runs one heavy command under the build lock, full log to logs/runs/, prints a short summary
│   ├── sample-training.sh                                    # Read-only sampler of the training job's total RSS every 30 s
│   ├── sb.sh                                                 # Runs a command under CPU/RAM limits (systemd-run) for low-memory hosts
│   ├── sched.sh                                              # RAM-aware scheduler: LIGHT/HEAVY admission from live MemAvailable, priority FIFO, slots, logging
│   ├── snapshot-state.sh                                     # PreCompact hook: writes logs/STATE_AUTO.md (git/process/disk facts) in <1 s; never fails
│   └── verify.sh                                             # THE definition of green: Rust gate + frontend + project checks via run.sh
├── ux-spec/                                                  # UX source of truth per tool column
│   ├── 01_organize.md                                        # UX: Organize PDF tools
│   ├── 02_optimize.md                                        # UX: Optimize PDF tools
│   ├── 03_convert_to.md                                      # UX: Convert to PDF tools
│   ├── 04_convert_from.md                                    # UX: Convert from PDF tools
│   ├── 05_edit.md                                            # UX: Edit PDF tools
│   ├── 06_security.md                                        # UX: PDF Security tools
│   ├── 07_intelligence.md                                    # UX: PDF Intelligence tools
│   └── 08_extras.md                                          # UX: Extras strip (Markdown, workflows)
├── vendor/                                                   # Third-party code: pdfcraft submodule + local crate patches
│   ├── craft/                                                # Vendored storytold craft crates (offline builds, ADR-018 R3), pinned revs in docs/UPSTREAM_MANIFEST.toml
│   │   └── cadcraft/                                         # cadcraft@df65974 crates (doc, geom, color, fonts, render) vendored unmodified via cargo vendor (not listed individually)
│   ├── patches/                                              # Local patches of third-party crates applied via [patch] in master/Cargo.toml
│   │   ├── cadcraft-render/                                  # cadcraft-render with a one-line re-export of pattern_lines + de-workspaced Cargo.toml (ADR-018)
│   │   │   ├── src/                                          # Upstream src/ except lib.rs re-export line (not listed individually)
│   │   │   ├── Cargo.toml                                    # Manifest with inherited workspace fields made explicit; siblings pinned to the vendored rev
│   │   │   ├── LICENSE-APACHE                                # Upstream cadcraft-render Apache-2.0 licence (kept with the patched copy)
│   │   │   ├── LICENSE-MIT                                   # Upstream cadcraft-render MIT licence (kept with the patched copy)
│   │   │   ├── NOTICE                                        # Upstream NOTICE file, reproduced in THIRD_PARTY_NOTICES.md (Apache-2.0 §4(d))
│   │   │   └── PATCH_NOTES.md                                # Exact diff vs upstream and how to re-sync
│   │   └── winprint/                                         # winprint (Windows printing, via pdfcraft → craft-print) without its network build-dependencies (ADR-017)
│   │       ├── src/                                          # Byte-identical copy of upstream winprint.rs @ 5f80b9b src/ (44 files, not listed individually)
│   │       ├── Cargo.toml                                    # Upstream manifest minus build/dev-dependencies; `pdfium` default off
│   │       ├── LICENSE.md                                    # Upstream BSD-3-Clause licence
│   │       ├── PATCH_NOTES.md                                # Exactly what differs from upstream and how to re-sync
│   │       └── README.md                                     # Upstream readme
│   └── pdfcraft                                              # pdfcraft PDF engine (git submodule, pinned 75360d1 / v0.15.0) — run `git submodule update --init --recursive`
├── xtask/                                                    # `cargo xtask` project tooling
│   ├── src/                                                  # Tool source
│   │   ├── main.rs                                           # gate, protocol-lint, feature-map-check, studio-audit, fuzz/perf/sbom/soak runners
│   │   ├── notices.rs                                        # xtask notices: generates THIRD_PARTY_NOTICES.md from cargo metadata (Linux graph) + npm licences
│   │   └── ui_audit.rs                                       # UI consistency audit over all .svelte: raw colour/px, local key listeners, one-off controls, icon-only labels, title case, overclaiming; --list, --strict <file>, --write-inventory
│   ├── Cargo.toml                                            # Crate manifest
│   ├── ui-px-allowlist.txt                                   # UI audit raw-px allowlist: canvas/geometry exemptions, one reason per entry (entries without a reason are rejected)
│   └── ui-strict.txt                                         # List of swept .svelte files the UI audit enforces strictly (fails the gate on new violations)
├── .allowed-deletions                                        # Paths the pre-commit hook allows to be deleted (all other deletions are blocked)
├── Cargo.lock                                                # Pinned versions of every Rust dependency (committed for reproducible builds)
├── Cargo.toml                                                # Rust workspace manifest: members, release profile, patches pointing at pdfcraft's vendored crates
├── CHANGELOG.md                                              # Release notes for v0.2.0-prototype (draft; finalised by the lead at tag time)
├── deny.toml                                                 # cargo-deny policy: licence allow-list (no GPL/AGPL), banned crates, advisories
├── KNOWN_ISSUES.md                                           # Cuts and known limitations shipped in v0.2.0-prototype, with workarounds (KI-xxx)
├── master.md                                                 # Map of master/ with build/dev commands
├── MASTER_PROMPT.md                                          # Original project boot prompt given to the first coding agent
├── rust-toolchain.toml                                       # Rust toolchain channel (stable) + rustfmt/clippy components
└── THIRD_PARTY_NOTICES.md                                    # Licences and notices of every shipped dependency (generated by `cargo xtask notices`; bundled as a Tauri resource)
```

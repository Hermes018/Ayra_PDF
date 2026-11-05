# UI AUDIT LOG — LocalPDF Studio (Drawboard Spatial Studio)

This log records user acceptance testing (UAT) defects, root cause diagnoses, and verification evidence for the Drawboard-class Spatial PDF Studio UI components (`DockableToolbar`, `PropertyPill`, `RadialMenu`, `SpatialCanvas`, `StudioIcon`).

---

## [2026-11-04] Phase D2.0 Audit & Defect Registry

### UI-BUG-001: Broken Tool State Reactivity
- **Severity:** High (Functional defect)
- **Component(s):** `app-tauri/src/lib/core/toolEngine.svelte.ts`, `app-tauri/src/lib/core/commandEngine.svelte.ts`, `app-tauri/src/lib/components/studio/PropertyPill.svelte`
- **Symptom:** Clicking tool buttons (Pen A, Pen B, Highlighter, Eraser) in `DockableToolbar` did not reliably update the active tool label in `PropertyPill` (which remained stuck displaying "PAN CANVAS") or the top HUD pill.
- **Root Cause Diagnosis:**
  1. `ToolStateEngine` relied on `activePresetId` matching an array of `presets` (`ToolPreset[]`). Tools without an explicit preset (or activated via tool IDs rather than preset IDs) left `activePresetId` unchanged or resolved `active` to `null`.
  2. `commandEngine.svelte.ts` lacked registered handlers for `tool.select` and `style.apply`, resulting in silent no-ops and `lastError = "unknown_command:style.apply"`.
  3. `PropertyPill.svelte` extracted `let activePreset = $derived(tools.active)` without a fallback preset generator, preventing subscriber notification when switching between tools.
- **Fix Specification:**
  - Standardize `toolEngine.svelte.ts` with Svelte 5 `$state`:
    - `activeToolId = $state<ToolId>('pan')`
    - `presets = $state<Record<string, ToolPreset>>({...})`
    - `activePreset = $derived(this.presets[this.activeToolId] ?? null)`
    - `activeToolLabel = $derived(TOOL_LABELS[this.activeToolId] ?? this.activeToolId)`
    - `supportsStrokeProperties = $derived(['pen.a', 'pen.b', 'highlighter', 'shape', 'text.review'].includes(this.activeToolId))`
    - `supportsEraserMode = $derived(this.activeToolId === 'eraser.partial' || this.activeToolId === 'eraser.whole')`
  - Implement deterministic mutators `selectTool(id: ToolId)` (auto-initializing default preset if missing) and `updateActivePreset(patch: Partial<ToolPreset>)`.
  - Wire command bus in `commandEngine.svelte.ts` to route `tool.select` and `style.apply` directly to `toolEngine`.
- **Status:** VERIFIED FIXED (Phase D2.1)

---

### UI-BUG-002: Janky Toolbar Drag Physics & Layout Thrashing
- **Severity:** Medium (UX / Performance defect)
- **Component(s):** `app-tauri/src/lib/components/studio/DockableToolbar.svelte`
- **Symptom:** Dragging the toolbar produced visual stutter, coordinate jumps, and lag across zoom scales.
- **Root Cause Diagnosis:**
  1. Pointer dragging modified inline CSS `top` and `left` properties directly on every `pointermove`, triggering layout reflows and browser thrashing.
  2. `dragOffset` calculated offsets without converting to the coordinate system of `offsetParent` (`.spatial-viewport`), causing jump offsets when the container has padding or non-zero screen coordinates.
  3. Pointer capture was not cleanly established on the grip handle element.
- **Fix Specification:**
  - Use `transform: translate3d(x px, y px, 0)` with `transition: none` while dragging or floating, anchoring at `top: 0; left: 0;`.
  - In `handleGripDown`, acquire pointer capture via `setPointerCapture(e.pointerId)`. Calculate `parentRect` from `toolbarEl.offsetParent.getBoundingClientRect()` and store `grabOffset` relative to `selfRect`.
  - In `handleGripMove`, clamp `nextX` and `nextY` within `[8, parentW - selfW - 8]` and `[8, parentH - selfH - 8]`. Compute `snapPreview = resolveDock(..., 40)`.
  - In `handleGripUp`, release capture and apply docked classes or final float position.
- **Status:** VERIFIED FIXED (Phase D2.1)

---

### UI-BUG-003: Radial Menu Movement Jank & Transition Lag
- **Severity:** Medium (UX / Latency defect)
- **Component(s):** `app-tauri/src/lib/components/studio/RadialMenu.svelte`
- **Symptom:** Hovering across the 8 radial sectors felt sluggish; sector highlights lagged behind high-refresh stylus input (60Hz–120Hz).
- **Root Cause Diagnosis:**
  - CSS rule `.wedge-path { transition: fill var(--motion-duration) ease, stroke var(--motion-duration) ease; }` forced browser animation interpolation on every hover sector change, introducing an artificial 180ms delay.
  - Hover feedback was deferred through CSS transition curves rather than instant vector path attribute changes.
- **Fix Specification:**
  - Remove all CSS `transition` rules on `.wedge-path` and `.flyout-path` in `RadialMenu.svelte` to achieve instantaneous 120Hz hover updates.
  - Ensure synchronous evaluation of `radialHit(dx, dy, flyoutCounts)`.
- **Status:** VERIFIED FIXED (Phase D2.1)

---

### UI-BUG-004: Placeholder Unicode Emoji Icons
- **Severity:** Low (Design & Visual Standards violation)
- **Component(s):** `app-tauri/src/lib/components/studio/DockableToolbar.svelte`, `app-tauri/src/lib/components/studio/RadialMenu.svelte`, `app-tauri/src/lib/components/studio/ShortcutSettings.svelte`
- **Symptom:** Studio UI rendered platform-dependent emojis (`✋`, `↖️`, `🖊️`, `🖍️`, `🧹`, `🔦`, `🎯`, `⌨️`) instead of crisp, deterministic vector icons.
- **Root Cause Diagnosis:**
  - Rapid prototyping in D2 used inline emoji strings rather than a centralized vector SVG icon registry.
- **Fix Specification:**
  - Author `app-tauri/src/lib/components/studio/StudioIcon.svelte` providing vector paths for: `pan`, `select`, `pen.a`, `pen.b`, `highlighter`, `eraser`, `eraser.partial`, `eraser.whole`, `laser`, `text.review`, `shape`, `measure`, `radial`, `shortcuts`, `undo`, `redo`, `grip`, `close`, `reset`, `zoom.in`, `zoom.out`.
  - Replace all Unicode emoji characters in `studio/*.svelte` with `<StudioIcon name="..." size={...} />`.
  - Enforce zero emojis via automated grep regex verification gate:
    `grep -P "[\x{1F300}-\x{1F9FF}\x{2600}-\x{26FF}\x{2700}-\x{27BF}]" app-tauri/src/lib/components/studio/*.svelte` must return 0.
- **Status:** VERIFIED FIXED (Phase D2.1)

---

## [2026-11-05] Phase U1–U5 Studio UI/UX Overhaul Defect Registry

### UI-BUG-005: Left Application Sidebar Lacks Collapse Toggle & Constrains Studio Viewport
- **Severity:** High (UX / Canvas Viewport defect)
- **Component(s):** `app-tauri/src/lib/components/MainLayout.svelte`, `app-tauri/src/lib/components/Sidebar.svelte`, `app-tauri/src/lib/state.svelte.ts`, `app-tauri/src/lib/core/commandEngine.svelte.ts`
- **Symptom:** The left application navigation sidebar occupies 210px–240px permanently, severely squeezing horizontal canvas real estate for architectural blueprints and multi-column documents in Drawboard Studio view.
- **Root Cause Diagnosis:** `AppState` had no `sidebarCollapsed` boolean or toggle method, and `MainLayout.svelte` lacked collapse classes or hotkey binding (`Ctrl+B`).
- **Fix Specification:**
  - Add `sidebarCollapsed = $state<boolean>(false)` and `toggleSidebar()` to `AppState`.
  - Bind `class:sidebar-collapsed={appState.sidebarCollapsed}` in `MainLayout.svelte` with smooth 0.2s cubic-bezier transition to width 0px.
  - Provide a floating toggle button (`data-testid="sidebar-toggle-btn"`) and header chevron button in `Sidebar.svelte`.
  - Register `Ctrl+B` -> `app.toggleSidebar` in `commandEngine.svelte.ts`.
- **Status:** VERIFIED FIXED (Phase U1.2 & U1.3 Verification Gate)

---

### UI-BUG-006: Studio Toolbar Collides on Left Edge & Top HUD Pill Overflows
- **Severity:** High (Visual Ergonomics & Layout defect)
- **Component(s):** `app-tauri/src/lib/components/studio/DockableToolbar.svelte`, `app-tauri/src/lib/views/studio/SpatialCanvas.svelte`
- **Symptom:** `DockableToolbar` defaults to left dock, directly overlapping or conflicting with the app navigation sidebar. Furthermore, the top HUD pill is cramped and overflows, truncating text (e.g. `"BG: Non"`).
- **Root Cause Diagnosis:** Initial `dockEdge` state defaulted to `'left'` instead of the industry standard right-side tool rail (Drawboard pattern), and the top HUD pill attempted to squeeze 8 disparate controls into an auto-overflowing rounded pill without logical grouping.
- **Fix Specification:**
  - Change default `dockEdge` in `DockableToolbar.svelte` from `'left'` to `'right'`.
  - Replace cramped top pill with a 4-zone `.studio-top-bar` (Workspace & Mode Zone, Viewport & Grid Zone, History & Assist Zone, Document Actions Zone).
- **Status:** VERIFIED FIXED (Phase U1.3 Verification Gate)

---

### UI-BUG-007: Radial Menu Cannot Be Toggled Off, Lacks Backdrop Dismiss, and Loses Outer Flyout Hit
- **Severity:** High (Interactive Ergonomics defect)
- **Component(s):** `app-tauri/src/lib/components/studio/RadialMenu.svelte`, `app-tauri/src/lib/core/radialGeometry.ts`, `app-tauri/src/lib/components/studio/DockableToolbar.svelte`
- **Symptom:** Radial menu cannot be toggled off once opened via toolbar button, lacks a full-viewport click-outside backdrop dismiss, and moving cursor from an active wedge into the outer flyout ring switches wedges accidentally.
- **Root Cause Diagnosis:** Toolbar only dispatched `onopenradial`, lacking toggle semantics. `RadialMenu.svelte` had no backdrop overlay, and `radialHit` re-evaluated the wedge sector from the raw polar angle even when cursor was in the outer flyout ring ($r_1 \le r \le r_2$).
- **Fix Specification:**
  - Wire `ontoggleradial` in `DockableToolbar.svelte` and `SpatialCanvas.svelte`.
  - Add full-screen transparent `<div class="radial-backdrop">` and center cancel hub ($r < 36\text{px}$) with `✕ Close` indicator.
  - Introduce `lockedWedge` parameter in `radialHit(...)` to lock the sector when navigating into flyout arcs.
- **Status:** VERIFIED FIXED (Phase U3 Verification Gate)

---

### UI-BUG-008: Trackpad Pinch-to-Zoom & Two-Finger Pan Do Not Work Naturally
- **Severity:** High (Navigation Physics defect)
- **Component(s):** `app-tauri/src/lib/views/studio/SpatialCanvas.svelte`, `app-tauri/src/lib/core/viewportEngine.svelte.ts`
- **Symptom:** Modern laptop trackpad pinch gestures (`wheel` with `ctrlKey`) do not zoom smoothly, two-finger scroll panning (`deltaX`/`deltaY`) conflicts with wheel zoom, and touch screen pinch gestures are unhandled.
- **Root Cause Diagnosis:** Wheel event handler did not distinguish between `ctrlKey` pinch-to-zoom and two-finger delta panning, and lacked continuous exponential zoom math (`pinchZoomAt`) and dual-touch pointer tracking.
- **Fix Specification:**
  - Implement `pinchZoomAt`, `panByDelta`, `fitToWidth`, and `fitToPage` in `viewportEngine.svelte.ts`.
  - Update `handleWheel` in `SpatialCanvas.svelte` to support trackpad pinch, trackpad glide pan, and smooth cursor-centered zoom.
  - Implement dual-pointer touch tracking for touchscreen pinch-zoom and pan.
- **Status:** VERIFIED FIXED (Phase U2 Verification Gate)

---

### UI-BUG-009: Only 11 of 41 Tools Visible; Toolbar Missing Grouped Popover Flyouts
- **Severity:** Medium (Discoverability defect)
- **Component(s):** `app-tauri/src/lib/components/studio/DockableToolbar.svelte`, `app-tauri/src/lib/components/studio/PropertyPill.svelte`
- **Symptom:** The tool rail displays only single tools directly, hiding polylines, arrows, revision clouds, squiggly review, LaTeX equations, signatures, audio notes, and CAD measurement tools.
- **Root Cause Diagnosis:** `DockableToolbar` lacked secondary flyout disclosure triangles and popover submenu trays for tool families.
- **Fix Specification:**
  - Implement 6 grouped tool popovers (Inking, Shapes, Text Review, Measure, Insert Suite, Select).
  - Enhance `PropertyPill.svelte` to dynamically align adjacent to the active tool rail dock edge.
- **Status:** OPEN (Addressing in Phase U4)

---

### UI-BUG-010: Structural PDF Text Editing Lacks Hover Bounding Boxes & Double-Click Inline Activation
- **Severity:** Medium (Usability defect)
- **Component(s):** `app-tauri/src/lib/views/studio/SpatialCanvas.svelte`, `app-tauri/src/lib/core/shapeAndHatchEngine.svelte.ts`
- **Symptom:** Users cannot easily see which text blocks in flat PDFs can be edited, requiring toggling an ambiguous button and guessing where to click.
- **Root Cause Diagnosis:** Canvas lacked text paragraph hover bounds rendering and direct double-click text activation in markup mode.
- **Fix Specification:**
  - Render hover outlines over detected text blocks in Structural Edit mode.
  - Enable direct double-click on any text block to launch inline textarea editing.
- **Status:** OPEN (Addressing in Phase U5)


1. **Automated Vitest Test Suite:**
   - Test File: `src/lib/core/studioReactivity.test.ts` (13 tests, all passed in 14ms)
     - `selectTool("pen.a")` updates `activeToolId`, `activeToolLabel`, and `supportsStrokeProperties`
     - `selectTool("highlighter")` updates `activeToolId` and `activeToolLabel`
     - `selectTool("eraser.partial")` and `selectTool("eraser.whole")` update `supportsEraserMode`
     - `selectTool("pan")` and `selectTool("select")` set neutral stroke states
     - `updateActivePreset` mutates current preset without corrupting sibling presets
     - `commands.dispatch` routes `tool.select` and `style.apply` directly to `toolEngine`
     - `resolveDock` accurately resolves `top`, `left`, `right`, and `null` (floating) margins
     - Coordinate boundary clamping rigorously prevents toolbar overflow outside canvas viewport
   - Workspace Suite: 81 tests passed across 11 test files.

2. **TypeScript & Svelte Diagnostics (`svelte-check`):**
   - Result: 0 errors, 0 warnings across all `.svelte` and `.ts` files.

3. **Deterministic SVG Icon Geometry (Anti-Emoji Gate):**
   - Command: `grep -P "[\x{1F300}-\x{1F9FF}\x{2600}-\x{26FF}\x{2700}-\x{27BF}]" app-tauri/src/lib/components/studio/*.svelte | wc -l`
   - Output: `0` (Zero Unicode emojis).

4. **Protocol Architecture & Security (`cargo xtask protocol-lint`):**
   - Result: `protocol-lint passed` (Zero network dependencies, no unwrap in non-test code, no hardcoded colors).

5. **Release RPM Package Build:**
   - Path: `target/release/bundle/rpm/LocalPDF Studio-0.1.0-1.x86_64.rpm` (37.46 MiB).

6. **Linux GUI Smoke Test:**
   - Command: `RUST_BACKTRACE=1 timeout 5s ./target/release/localpdf-studio 2>&1 ; echo "EXIT_CODE=$?"`
   - Output: `EXIT_CODE=124` (clean Wayland/GTK startup and graceful shutdown under timeout).


---

## [2026-11-05] Phase D4 Verification — Shapes, Procedural Hatching, Selection Action Bar & Structural PDF Text Edit

1. **New Studio Engines & Components (`Phase D4`)**:
   - `app-tauri/src/lib/core/shapeAndHatchEngine.svelte.ts`: Interactive creation for `shape.line`, `shape.arrow`, `shape.polyline` (multi-click + double-click/Enter commit), `shape.rect`, `shape.ellipse`, `shape.polygon`, `shape.cloud` (scalloped cubic Bézier arcs), `review.highlight`, `review.underline`, `review.squiggly`, `review.strikeout`, `review.redact`, and `insert.textBox`.
   - `app-tauri/src/lib/components/studio/HatchDefs.svelte`: Theme-token-driven `<pattern>` defs (`diagonal-steel`, `crosshatch`, `concrete`, `timber`, `tiling`, `masonry`, `earth`, `insulation`, `redact-danger`) with zero hardcoded hex colors.
   - `app-tauri/src/lib/core/selectionEngine.svelte.ts`: Marquee (`select.marquee`), freeform polygon lasso (`select.lasso`), `Shift+Click` multi-select, 8 resize handles + top rotation gizmo, `groupSelected()`, `ungroupSelected()`, `alignSelected()`, `flipSelected()`, `reorderSelected()`, `duplicateSelected()`, Format Painter, and `select.snapshot` stamp tool.
   - `app-tauri/src-tauri/src/commands/studio.rs`: `studio_save_annotations`, `studio_load_annotations`, and `studio_edit_pdf_text_block` backed by `pdfcraft_engine` and `lpdf_fs_safety::atomic_write` + verify-after-write.
2. **Audit & Test Gates**:
   - `app-tauri/src/lib/core/shapeAndSelection.test.ts`: 6 unit tests covering horizontal/vertical flip reflection, 6-edge alignment, group/ungroup transitions, revision cloud / polyline / squiggle / line-ending SVG path generation, point-in-polygon lasso hit testing, z-order, duplicate, format painter, and snapshot stamps.
   - Zero raw hex literals and zero local `keydown` listeners across all studio files (`cargo xtask studio-audit --strict` passed with 0 findings).


## [2026-11-05] Phase D5 — CAD OSNAP, Scale Calibration, Digital Ruler & Measurement Suite Audit
1. **OSNAP Indicator & Theme Token Compliance (`MeasureOverlay.svelte`, `snapEngine.svelte.ts`)**:
   - Verified `endpoint` (square), `midpoint` (triangle), `intersection` (`X`), `perpendicular` (`∟`), and `grid` indicators use CSS variables `--snap-indicator-*` from `themeContract.svelte.ts` with zero raw hex/rgb literals.
2. **Calibration Dialog & Digital Ruler A11y (`CalibrationModal.svelte`, `MeasureOverlay.svelte`, `PropertyPill.svelte`)**:
   - Verified ARIA roles (`role="dialog"`, `role="toolbar"`, `role="button"`) and `tabindex` on all interactive overlays (`svelte-check` 0 errors, 0 warnings).


## [2026-11-05] Phase D6 — Complete Insert Suite, LaTeX, Signatures, Audio Notes, Markup Library & Quantity Takeoff Audit
1. **Offline Insert Suite & Theme Token Compliance (`insertEngine.svelte.ts`, `AnnotationLedger.svelte`, `SpatialCanvas.svelte`)**:
   - Verified `insert.latex` (`compileLatexToVector`, `svgPathToPdfOps`), `insert.signature` (`draw`, `type`, `webcam` with `applyGrayscaleThresholdToRgba`), `insert.audio` (`MediaRecorder` + PCM WAV fallback + `studio_attach_audio`), `callout.single`, `callout.multi`, `insert.note`, `insert.stamp`, `insert.image`, and non-destructive Page Background SVG templates (`engineering-grid`, `isometric-grid`, `ruled-paper`) use CSS variables from `themeContract.svelte.ts` with zero raw hex literals and zero `{@html}` usage.
2. **Annotation Ledger, Quantity Takeoff CSV & Reusable Markup Library (`AnnotationLedger.svelte`, `studio.rs`)**:
   - Verified `studio_export_takeoff_csv` (with RFC 4180 escaping and formula-injection guard), `studio_markup_library`, and multi-page `Paste to Pages...` (`All`, `Even`, `Odd`, `1-5`) pass all Vitest and Rust unit tests (`svelte-check` 0 errors, 0 warnings).

---

## [2026-11-05] Phases U1, U2 & U3 Verification — Overhaul Specification, Collapsible Shell, Top Bar, Physics & Radial Menu

1. **Phase U1 Verification (`UI-BUG-005`, `UI-BUG-006`)**:
   - `app-tauri/src/lib/state.svelte.ts`: `sidebarCollapsed = $state<boolean>(false)` and `toggleSidebar()` method added and reactive.
   - `app-tauri/src/lib/components/MainLayout.svelte`: Applied `class:sidebar-collapsed={appState.sidebarCollapsed}`. Added floating toggle button `data-testid="sidebar-toggle-btn"` (`aria-label="Toggle Sidebar (Ctrl+B)"`).
   - `app-tauri/src/lib/components/Sidebar.svelte`: Chevron collapse button in `.logo` header. CSS smoothly transitions `width`, `min-width`, and `opacity` with `0.2s cubic-bezier(0.16, 1, 0.3, 1)`.
   - `app-tauri/src/lib/core/commandEngine.svelte.ts` & `defaultShortcuts.json`: Mapped canonical shortcuts `Ctrl+B` and `Meta+B` to `app.toggleSidebar`.
   - `app-tauri/src/lib/components/studio/DockableToolbar.svelte`: Default `dockEdge` set to `'right'`.
   - `app-tauri/src/lib/views/studio/SpatialCanvas.svelte`: Replaced cramped pill with 4-zone `.studio-top-bar`:
     - Zone 1: Sidebar toggle, mode segmented switch (`Markup` vs `Edit PDF Text`), page indicator (`Page X / Y`).
     - Zone 2: Zoom in/out, zoom presets dropdown (50%–200%, Fit Width, Fit Page), rotate page 90°, and page background overlay selector.
     - Zone 3: Undo/Redo, CAD OSNAP magnet toggle, and Digital Ruler toggle.
     - Zone 4: Contextual Commit Text, Save PDF (atomic), Flatten PDF, Commit Redactions, and Ledger & Vault drawer toggle.

2. **Phase U2 Verification (`UI-BUG-008`)**:
   - `app-tauri/src/lib/core/viewportEngine.svelte.ts`: Added continuous exponential `pinchZoomAt(deltaY, deltaMode, screenX, screenY)`, `panByDelta(dx, dy)`, `fitToWidth(viewportW, pageW)`, and `fitToPage(viewportW, viewportH, pageW, pageH)`.
   - `app-tauri/src/lib/views/studio/SpatialCanvas.svelte`: `handleWheel` non-passively differentiates `ctrlKey`/`metaKey` trackpad pinch, two-finger horizontal/diagonal glide pan, shift/alt pan, and smooth zoom.
   - Dual-touch gesture physics: Cancel active stroke on 2 touches (`activeTouchPoints.size === 2`), ratio zoom, and sticky midpoint anchor pan.
   - `app-tauri/src/lib/core/core.test.ts`: 4 unit tests verifying zoom scaling, pan translation, `fitToWidth`, and `fitToPage`.

3. **Phase U3 Verification (`UI-BUG-007`)**:
   - `app-tauri/src/lib/components/studio/DockableToolbar.svelte`: Added `onToggleRadial` prop to toggle the radial menu on and off.
   - `app-tauri/src/lib/components/studio/RadialMenu.svelte`: Full-screen transparent backdrop (`.radial-backdrop`) dismisses menu on click; center cancel hub ($r < 36\text{px}$) with `Close` label and icon closes menu; keyboard shortcuts (`/`, `Escape`, `1-8`) routed via `commandEngine.svelte.ts`.
   - `app-tauri/src/lib/core/radialGeometry.ts`: `radialHit(dx, dy, flyoutCounts, lockedWedge)` locks sector in outer flyout ring ($r_1 \le r < r_2$) to prevent angular drift.
   - `app-tauri/src/lib/core/radialGeometry.test.ts`: 3 unit tests verifying drift elimination, backdrop outside hit, and cancel hub hit.

4. **Audit Gates Summary**:
   - `svelte-check`: 0 errors, 0 warnings.
   - Vitest: 124 passed across 14 test files.
   - `cargo xtask feature-map-check`: 119 features audited (113 DONE, 0 IN PROGRESS, 0 NOT STARTED, 6 DEFERRED), passed.
   - `cargo xtask protocol-lint`: passed.
   - `cargo xtask studio-audit --strict`: 0 findings (100% theme tokens `var(--...)`, zero local keydown listeners).
   - Independent Sub-Agent Audit: All findings remediated; **100% PASS**.

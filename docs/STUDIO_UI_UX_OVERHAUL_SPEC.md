# Drawboard Studio UI/UX Overhaul Specification (Phases U1 – U5)

> **Document Status**: Active Master Specification & Source of Truth  
> **Target Release**: LocalPDF Studio v1.0.0  
> **Compliance Guardrails**: Zero-OOM sequential verification, 100% theme contract CSS variables (`--glass-*`, `--canvas-*`, `--accent`, etc.), zero local keydown listeners (all routing via `commandEngine.svelte.ts`), zero-network air-gap protocol (`ADR-005`), and independent fresh-eye audit vetting at each phase gate.

---

## Executive Summary & Architecture Principles

LocalPDF Studio's core PDF manipulation, CAD measurement, vector drawing, and annotation persistence engines are fully complete (113/113 offline features verified across Phases D1–D6.1 and Windows W1–W4). However, real-world usability testing uncovered six primary interaction bottlenecks (`UI-BUG-005` through `UI-BUG-010`):
1. **Collapsible Workspace Shell**: The global left application sidebar occupied 210px–240px permanently, severely constraining horizontal blueprint canvas area in Studio mode.
2. **Right-Side Studio Rail**: The Drawboard Studio tool rail docked to the left by default, colliding with the application sidebar rather than anchoring cleanly on the right viewport edge.
3. **Studio Top Command Bar**: The top canvas HUD was cramped into a narrow floating pill, causing text truncation (e.g. `"BG: Non"`), overflowing controls, and missing key canvas telemetry.
4. **Natural Physics Navigation**: Trackpad pinch-to-zoom (`ctrlKey` + wheel), two-finger trackpad panning (`deltaX`/`deltaY`), and two-finger multi-touch gestures were not unified with mouse wheel actions.
5. **Radial Marking Menu Ergonomics**: The 8-wedge radial menu lacked a backdrop click-outside dismiss, could not be toggled off via its toolbar button, and suffered from angular drift when moving the cursor from a wedge to the outer flyout ring.
6. **Tool Rail Discoverability & Popover Flyouts**: Only 11 of 41 tools were exposed directly on the toolbar, hiding shapes, review markups, CAD measurements, and insert tools.
7. **Discoverable Structural PDF Text Editing**: True inline text manipulation ("Structural Edit") was locked behind an ambiguous pill toggle without hover bounding boxes or double-click inline text activation.

This specification details the 5-phase architectural overhaul to transform LocalPDF Studio into a peerless, desktop-grade technical drawing and document annotation studio.

---

## Detailed Hierarchical Phase Roadmap

```
PHASE U1: SPECIFICATION LOCK, COLLAPSIBLE LEFT SIDEBAR & RIGHT-SIDE STUDIO LAYOUT
├── Sub-Phase U1.1: Master UI/UX Spec & Defect Registration
│   ├── U1.1.a: Single-shot workspace inspection
│   ├── U1.1.b: Author docs/STUDIO_UI_UX_OVERHAUL_SPEC.md
│   └── U1.1.c: Register UI-BUG-005 through UI-BUG-010 in logs/UI_AUDIT_LOG.md
├── Sub-Phase U1.2: Collapsible Left Application Sidebar
│   ├── U1.2.a: Reactive sidebarCollapsed state in state.svelte.ts
│   ├── U1.2.b: Responsive app-shell collapse & floating toggle in MainLayout & Sidebar
│   └── U1.2.c: Command binding (Ctrl+B / app.toggleSidebar) in commandEngine.svelte.ts
└── Sub-Phase U1.3: Right-Side Drawboard Tool Rail & Top Studio Command Bar
    ├── U1.3.a: DockableToolbar default dockEdge = 'right'
    ├── U1.3.b: 4-Zone Studio Top Command Bar (.studio-top-bar) in SpatialCanvas.svelte
    └── U1.3.c: Sub-agent verification gate & UI-BUG-005 / UI-BUG-006 closure

PHASE U2: TRACKPAD PINCH-TO-ZOOM, TWO-FINGER PAN & MULTI-TOUCH GESTURE PHYSICS
├── Sub-Phase U2.1: Trackpad Pinch & Smooth Two-Finger Pan
│   ├── U2.1.a: Continuous exponential pinchZoomAt & panByDelta in viewportEngine.svelte.ts
│   ├── U2.1.b: Intelligent non-passive wheel handler in SpatialCanvas.svelte
│   └── U2.1.c: Fit to Width & Fit to Page presets
└── Sub-Phase U2.2: Multi-Touch Touchscreen Physics
    ├── U2.2.a: Touch pointer tracking map in SpatialCanvas.svelte
    ├── U2.2.b: Dual-touch distance ratio zoom & midpoint pan
    └── U2.2.c: Unit tests in core.test.ts & UI-BUG-008 closure

PHASE U3: RADIAL MENU TOGGLE-OFF, CLICK-OUTSIDE DISMISS & 2-STAGE FLYOUT NAVIGATION
├── Sub-Phase U3.1: Toggle Control & Backdrop Dismissal
│   ├── U3.1.a: Bidirectional ontoggleradial wiring in toolbar and canvas
│   ├── U3.1.b: Full-screen transparent backdrop & center cancel hub
│   └── U3.1.c: Keyboard shortcuts (/, Escape, 1-8)
└── Sub-Phase U3.2: Deterministic 2-Stage Radial Hit Testing
    ├── U3.2.a: Elimination of angular drift in radialGeometry.ts
    ├── U3.2.b: lockedWedge parameter & bounded flyout arc mapping
    └── U3.2.c: Explicit pointer events on SVG wedge/flyout groups & high-contrast labels

PHASE U4: EXPANDED POPUP TOOLBAR FLYOUTS & REFINED FLOATING PROPERTY PILL
├── Sub-Phase U4.1: Grouped Popover Flyouts on Dockable Toolbar
│   ├── U4.1.a: Tool group hierarchy (Pens, Shapes, Review, Measure, Insert, Select)
│   ├── U4.1.b: Persistent flyout anchors with secondary arrow indicators
│   └── U4.1.c: Click-outside dismiss and keyboard tab navigation
├── Sub-Phase U4.2: High-Density Floating Property Pill Overhaul
│   ├── U4.2.a: Auto-docking adjacent to active tool rail
│   ├── U4.2.b: Contextual controls (width, opacity, line endings, CAD scale, hatches)
│   └── U4.2.c: Color swatches mapped strictly to theme tokens
└── Sub-Phase U4.3: Verification Gate & UI-BUG-009 Closure

PHASE U5: DISCOVERABLE STRUCTURAL PDF TEXT EDITING & DOUBLE-CLICK INLINE ACTIVATION
├── Sub-Phase U5.1: Discoverable Text Block Highlighting
│   ├── U5.1.a: Hover outlines on native PDF text paragraph boundaries
│   ├── U5.1.b: Visual affordance indicating structural vs annotation mode
│   └── U5.1.c: Font family, size, and alignment inspector
├── Sub-Phase U5.2: Double-Click Inline Text Activation
│   ├── U5.2.a: Direct double-click into text editing without toolbar mode switch
│   ├── U5.2.b: Resizable textarea overlay with live font sizing
│   └── U5.2.c: Atomic write commit and PDF reload via studio_edit_pdf_text_block
└── Sub-Phase U5.3: End-to-End Release RPM Build, Smoke Test & Verification Gate
```

---

## Detailed Specifications by Phase

### Phase U1: Specification Lock, Collapsible Left Sidebar & Right-Side Studio Layout

#### Sub-Phase U1.1: Spec & Defect Registration
- Register `UI-BUG-005` through `UI-BUG-010` in `logs/UI_AUDIT_LOG.md`.
- Establish strict zero-OOM execution laws: sequential single-command runs, `CARGO_BUILD_JOBS=2`, zero `cargo clean`.

#### Sub-Phase U1.2: Collapsible Left Application Sidebar
- In `app-tauri/src/lib/state.svelte.ts`:
  - Add `sidebarCollapsed = $state<boolean>(false)`.
  - Add `toggleSidebar(): void { this.sidebarCollapsed = !this.sidebarCollapsed; }`.
- In `app-tauri/src/lib/components/MainLayout.svelte`:
  - Bind `class:sidebar-collapsed={appState.sidebarCollapsed}` on `.app-container`.
  - Transition sidebar width smoothly (`transition: width 0.2s cubic-bezier(0.16, 1, 0.3, 1)`). When collapsed, width is `0px`, with `overflow: hidden`.
  - Provide a fixed top-left floating button (`data-testid="sidebar-toggle-btn"`, `aria-label="Toggle Sidebar (Ctrl+B)"`) visible whenever collapsed or hovering near the left edge.
- In `app-tauri/src/lib/components/Sidebar.svelte`:
  - Add a collapse icon button (`ChevronLeft` / `SidebarToggle`) in `.logo` header.
- In `app-tauri/src/lib/core/commandEngine.svelte.ts` & `defaultShortcuts.json`:
  - Map `Ctrl+B` / `Cmd+B` to `app.toggleSidebar`.

#### Sub-Phase U1.3: Right-Side Drawboard Tool Rail & Top Studio Command Bar
- In `app-tauri/src/lib/components/studio/DockableToolbar.svelte`:
  - Change default `dockEdge` from `'left'` to `'right'`.
  - Position toolbar flush with right edge when `dockEdge === 'right'`, avoiding overlap with left menus.
- In `app-tauri/src/lib/views/studio/SpatialCanvas.svelte`:
  - Replace the overflowing pill with `.studio-top-bar` positioned across the top center:
    - **Zone 1 (Workspace & Mode)**: Sidebar toggle button (`☰`), Mode segmented button (`Markup` vs `Edit PDF Text`), Document page indicator (`Page X / Y`).
    - **Zone 2 (Viewport & Zoom)**: Zoom Out (`-`), Zoom readout dropdown (`50%`, `75%`, `100%`, `125%`, `150%`, `200%`, `Fit Width`, `Fit Page`), Zoom In (`+`), Page rotation (`90°`), Non-destructive Page Background Overlay picker (`None`, `Engineering Grid`, `Isometric Grid`, `Ruled Paper` — with full text labels).
    - **Zone 3 (History & Assist)**: Undo (`Ctrl+Z`), Redo (`Ctrl+Shift+Z`), CAD OSNAP magnet toggle (`Snap: ON/OFF`), Digital Ruler toggle.
    - **Zone 4 (Actions & Ledger)**: Save PDF (atomic write), Flatten PDF, Commit Redactions (when active), Ledger & Library drawer toggle.

---

### Phase U2: Trackpad Pinch-to-Zoom, Two-Finger Pan & Multi-Touch Gesture Physics

#### Sub-Phase U2.1: Trackpad Pinch & Smooth Two-Finger Pan
- In `app-tauri/src/lib/core/viewportEngine.svelte.ts`:
  - Add `pinchZoomAt(deltaY: number, deltaMode: number, screenX: number, screenY: number): void`:
    - Normalizes delta: lines $\times 16$, pages $\times 100$.
    - Factor: $\exp(-\text{normDeltaY} \times 0.01)$.
    - Clamped zoom within $[0.1, 16.0]$.
  - Add `panByDelta(dxScreen: number, dyScreen: number): void`:
    - Applies $(-dxScreen, -dyScreen)$ directly to viewport pan offset.
  - Add `fitToWidth(viewportWidthPx: number, pageWidthPt?: number): void`.
  - Add `fitToPage(viewportWidthPx: number, viewportHeightPx: number, pageWidthPt?: number, pageHeightPt?: number): void`.
- In `app-tauri/src/lib/views/studio/SpatialCanvas.svelte`:
  - In `handleWheel(e: WheelEvent)`:
    - If `e.ctrlKey || e.metaKey` (Trackpad pinch or Ctrl+wheel) -> `viewport.pinchZoomAt(e.deltaY, e.deltaMode, localX, localY)`.
    - If `!e.ctrlKey`:
      - If `Math.abs(e.deltaX) > 0` (Trackpad two-finger horizontal/diagonal glide) -> `viewport.panByDelta(e.deltaX, e.deltaY)`.
      - If pure vertical wheel:
        - If `e.shiftKey || e.altKey` -> horizontal/vertical pan.
        - Otherwise, smooth zoom centered on cursor `(localX, localY)` using `viewport.pinchZoomAt(e.deltaY * 0.65, e.deltaMode, localX, localY)`.

#### Sub-Phase U2.2: Multi-Touch Touchscreen Physics
- Track `activeTouchPoints = new Map<number, { x: number; y: number }>()`.
- When `activeTouchPoints.size === 2`:
  - Cancel any active drawing stroke (`ink.cancelStroke()`).
  - Calculate distance $D = \hypot(x_2 - x_1, y_2 - y_1)$ and midpoint $M = ((x_1 + x_2)/2, (y_1 + y_2)/2)$.
  - Pan by $(M_{\text{prev}} - M_{\text{new}})$ and zoom by ratio $D_{\text{new}} / D_{\text{prev}}$ around $M_{\text{new}}$.

---

### Phase U3: Radial Menu Toggle-Off, Click-Outside Dismiss & 2-Stage Flyout Navigation

#### Sub-Phase U3.1: Toggle Control & Backdrop Dismissal
- In `DockableToolbar.svelte`:
  - Replace `onOpenRadial` with `onToggleRadial`.
  - When radial is already open, clicking toolbar radial button toggles it OFF.
- In `RadialMenu.svelte`:
  - Full-screen transparent backdrop `<div class="radial-backdrop" onpointerdown={handleBackdrop}>` (`z-index: 199`).
  - Clicking backdrop ($r > 176\text{px}$) or center cancel hub ($r < 36\text{px}$) dismisses radial menu immediately.
  - Center hub renders a clean `✕` icon and `Close` text label.
  - Keyboard listeners: `/` toggles menu, `Escape` closes, `1`–`8` selects corresponding wedge.

#### Sub-Phase U3.2: Deterministic 2-Stage Radial Hit Testing
- In `app-tauri/src/lib/core/radialGeometry.ts`:
  - Update `radialHit(dx, dy, flyoutCounts, lockedWedge)`:
    - Inner ring ($r \in [36, 108]\text{px}$): calculates sector $0..7$ and sets `activeWedge = sector`.
    - Outer ring ($r \in [108, 176]\text{px}$): if `lockedWedge !== null`, preserves `wedge = lockedWedge` and resolves `flyoutIndex` based on angular offset from wedge centerline within $\pm\pi/4$, eliminating angular jumping.
- In `RadialMenu.svelte`:
  - Explicit `onpointerenter` and `onpointerdown` handlers on each wedge `<path>` and flyout `<path>`.
  - Render icon and readable text label pill on every wedge and flyout arc.

---

### Phase U4: Expanded Popover Flyouts & Refined Property Pill

#### Sub-Phase U4.1: Grouped Popover Flyouts on Dockable Toolbar
- Group all 41 tools into 6 primary tool families:
  1. **Inking & Laser**: Pen A, Pen B, Pen C, Highlighter, Laser Pointer, Whole Eraser, Partial Eraser.
  2. **Shapes & Clouds**: Line, Arrow, Polyline, Rectangle, Ellipse, Polygon, Revision Cloud.
  3. **Text & Review**: Text Box, Note, Highlight, Underline, Squiggly, Strikeout, Redaction.
  4. **CAD Measure**: Calibrate, Length, Polylength, Rect Area, Poly Area, Perimeter, Protractor, Digital Ruler.
  5. **Insert Suite**: Single Callout, Multi Callout, LaTeX, Signature, Audio, Stamp, Image.
  6. **Select & Transform**: Marquee, Lasso, Snapshot Stamp.
- Each family on `DockableToolbar` has a secondary flyout arrow opening a vertical glass popover tray.

#### Sub-Phase U4.2: High-Density Floating Property Pill Overhaul
- Position property pill dynamically alongside the active dock edge (`dock-right` -> sits to left of rail; `dock-left` -> sits to right of rail).
- Contextually displays: stroke color swatches, stroke width, opacity, line style, line endings, hatch pattern, and CAD unit calibration.

---

### Phase U5: Discoverable Structural PDF Text Editing & Double-Click Inline Activation

#### Sub-Phase U5.1: Discoverable Text Block Highlighting
- When Structural Edit mode is active, extracted PDF text blocks render with subtle dashed hover bounding boxes.
- Hovering shows font name, size, and character count pill.

#### Sub-Phase U5.2: Double-Click Inline Text Activation
- Double-clicking any text block opens a transparent inline `<textarea>` matched to font size, color, and line height.
- Pressing `Enter` (or clicking outside) commits the edit via `studio_edit_pdf_text_block` with atomic write and canvas refresh.

#### Sub-Phase U5.3: Verification, Packaging & Release
- Verify all test suites (`check`, `vitest`, `cargo test`, `xtask`).
- Build release RPM (`LocalPDF Studio-0.1.0-1.x86_64.rpm`) and execute 5s Wayland smoke test (`EXIT_CODE=124`).

# Architecture, Global State & Opus 5.5 Plan Compliance Audit

> **Audit Date**: 2026-04-12  
> **Scope**: Full codebase structural audit against `/home/ayra/Downloads/pdf_drawing_annotation_app_spec.md`, [`docs/DRAWBOARD_STUDIO_MASTER_PLAN.md`](./DRAWBOARD_STUDIO_MASTER_PLAN.md) (Opus 5.5 Plan), and [`docs/adr/ADR-012-drawboard-artcraft-spatial-studio.md`](./adr/ADR-012-drawboard-artcraft-spatial-studio.md).

---

## 1. Executive Summary

| Audit Dimension | Verdict | Summary |
|---|---|---|
| **Legacy Backend & Tool Suites (Phases 0–12)** | **PASS (95%)** | 16 Rust crates compile cleanly with `#![forbid(unsafe_code)]`, zero network dependencies, and 26 end-to-end PDF/AI/Sidecar features verified. |
| **Last Update (`Phase D1`, `D2`, `D2.1`) Frontend** | **PARTIAL (68%)** | Svelte 5 Runes stores (`themeContract`, `toolEngine`, `viewportEngine`, `commandEngine`), `DockableToolbar`, `PropertyPill`, `RadialMenu`, `ShortcutSettings`, and 81 Vitest tests work well in-memory, **but** key backend/integration deliverables from Opus 5.5's D1/D2 spec were skipped. |
| **Global Features vs. Placeholders** | **FAIL (Needs D2.2 Fix)** | 1) `SpatialCanvas.svelte` ignores `appState.currentFile` and renders **2 hardcoded blank A4 placeholder divs**. 2) Dual unsynchronized theme engines (`appState.themePreference` vs `themeContract.mode`). 3) Three legacy views (`EditView`, `OptimizeView`, `SecurityView`) still have static placeholder preview boxes. |
| **Opus 5.5 (`ADR-012`) Structural Compliance** | **DRIFT DETECTED** | `crates/lpdf-control-bus` and `commands/studio.rs` (`prefs_save`/`prefs_load`) were never created; `xtask` lacked frontend anti-hardcode lints and `feature-map-check`; `toolEngine` collapsed 41 granular `ToolId`s down to 11. |

---

## 2. Audit of the Last Update (`Phases D1`, `D2`, `D2.1`)

### 2.1 What Is Up to the Mark (Verified Working)
1. **Svelte 5 Runes Migration (`D2.1`)**:
   - [`toolEngine.svelte.ts`](../app-tauri/src/lib/core/toolEngine.svelte.ts), [`viewportEngine.svelte.ts`](../app-tauri/src/lib/core/viewportEngine.svelte.ts), [`commandEngine.svelte.ts`](../app-tauri/src/lib/core/commandEngine.svelte.ts), and [`themeContract.svelte.ts`](../app-tauri/src/lib/core/themeContract.svelte.ts) use native Svelte 5 `$state` reactivity and pass all **81 Vitest unit tests** (`pnpm test`).
2. **GPU-Composited Floating Toolbar (`D2.1`)**:
   - [`DockableToolbar.svelte`](../app-tauri/src/lib/components/studio/DockableToolbar.svelte) uses `setPointerCapture` and `transform: translate3d(...)` during free-float dragging with snap-to-edge threshold (`48px`) for `top`, `bottom`, `left`, and `right` docks.
3. **Zero-Latency Radial Menu & Pure SVG Icon Registry (`D2.1`)**:
   - [`RadialMenu.svelte`](../app-tauri/src/lib/components/studio/RadialMenu.svelte) computes 8-sector polar geometry (`innerRadius = 32`, `outerRadius = 108`) with zero CSS transitions on `<path>` sectors.
   - [`StudioIcon.svelte`](../app-tauri/src/lib/components/studio/StudioIcon.svelte) provides 21 pure inline SVG stroke icons with zero Unicode emojis.

### 2.2 Gaps & Skipped Deliverables in `D1` / `D2` / `D2.1`
1. **Missing `crates/lpdf-control-bus` Crate (Required by Opus 5.5 §1.5 & §9 Phase D1)**:
   - Opus 5.5 specified a dedicated Rust crate `crates/lpdf-control-bus` defining `ControlMessage` (`#[serde(tag = "kind", content = "payload")]`), `PointerSample`, `BusError`, `Executor` trait, deterministic `replay()`, and `crates/lpdf-control-bus/tests/replay.rs` (`1,000` synthetic messages).
   - **Actual State**: Crate was never created. TypeScript `.d.ts` files in `app-tauri/src/lib/bindings/` were handwritten instead of generated from Rust via `ts-rs`.
2. **Missing Tauri IPC Studio Commands (`prefs_save`, `prefs_load`, `control_validate`)**:
   - Opus 5.5 Phase D1 required `app-tauri/src-tauri/src/commands/studio.rs` with atomic JSON preference persistence (`prefs.json`).
   - **Actual State**: Not implemented. Consequently, custom hotkeys in [`ShortcutSettings.svelte`](../app-tauri/src/lib/components/studio/ShortcutSettings.svelte), dock placement in [`DockableToolbar.svelte`](../app-tauri/src/lib/components/studio/DockableToolbar.svelte), and tool presets in [`toolEngine.svelte.ts`](../app-tauri/src/lib/core/toolEngine.svelte.ts) reset on every app restart.
3. **Granular `ToolId` Narrowing (`41` IDs Collapsed to `11`)**:
   - Opus 5.5 §1.2 specified 41 distinct `ToolId` variants (`select.marquee`, `select.lasso`, `shape.line`, `shape.arrow`, `shape.rect`, `shape.ellipse`, `shape.cloud`, `shape.polygon`, `shape.polyline`, `measure.calibrate`, `measure.length`, `measure.polylength`, `measure.area`, `measure.perimeter`, `measure.protractor`, `measure.ruler`, `measure.count`, etc.).
   - In [`toolEngine.svelte.ts` (lines 7–18, 70–88)](../app-tauri/src/lib/core/toolEngine.svelte.ts#L7-L88), `ToolId` was narrowed to 11 generic strings, and `normalizeToolId()` collapses every sub-tool in a flyout (`shape.rect`, `shape.cloud`, `measure.protractor`, etc.) into generic `'shape'` or `'measure'`. This prevents Phases D3–D5 from knowing *which* shape or measurement tool the user selected in [`DockableToolbar.svelte`](../app-tauri/src/lib/components/studio/DockableToolbar.svelte).
4. **Unmounted & Incomplete `SplitLayout.svelte` + Missing `FavoritesBar.svelte`**:
   - [`SplitLayout.svelte`](../app-tauri/src/lib/views/studio/SplitLayout.svelte) is not mounted anywhere in the app (`+page.svelte` renders `<SpatialCanvas />` directly), its `.split-handle` divider has no pointer drag listener to adjust `splitRatio`, and `app-tauri/src-tauri/capabilities/studio-split.json` does not exist.
   - `FavoritesBar.svelte` (Opus 5.5 §2.2 / Phase D2) was not created (`tools.favorites` is unused in the UI).

---

## 3. Global Features vs. Placeholders Audit

### 3.1 `SpatialCanvas.svelte` Ignores `appState.currentFile` (Hardcoded Placeholders)
- **Finding**: While all legacy views read `appState.currentFile` from [`state.svelte.ts`](../app-tauri/src/lib/state.svelte.ts), [`SpatialCanvas.svelte` (lines 63–68, 326–339)](../app-tauri/src/lib/views/studio/SpatialCanvas.svelte#L63-L68) hardcodes a static 2-page array (`DEFAULT_PAGES`) and renders two blank white boxes containing placeholder watermark text:
  - `Spatial Studio PDF Canvas • Page 1`
  - `Spatial Studio PDF Canvas • Page 2`
- **Impact**: Opening a real PDF in the app and viewing the default `Studio` workspace does not display the user's PDF.
- **Remediation**: Wire `appState.currentFile` + vendored `pdf.min.js` (already proven in [`PdfViewer.svelte`](../app-tauri/src/lib/components/PdfViewer.svelte)) into `SpatialCanvas.svelte` so real PDF pages and dimensions populate `viewport.setPageMetrics()` and render onto the base layer.

### 3.2 Split-Brain Theme Engines (`appState` vs. `themeContract`)
- **Finding**: There are two disconnected theme managers:
  1. [`appState` (`state.svelte.ts` lines 16, 46–61)](../app-tauri/src/lib/state.svelte.ts#L46-L61) stores `themePreference` (`'Light' | 'Dark' | 'System'`) in `localStorage` and sets `data-theme="light|dark"` on `document.documentElement`, driving CSS variables in [`app.css`](../app-tauri/src/app.css) (`--glass-bg-surface`, `--accent-primary`, `--text-primary`).
  2. [`theme` (`themeContract.svelte.ts` lines 69–113)](../app-tauri/src/lib/core/themeContract.svelte.ts#L69-L113) independently manages `mode` (`'light' | 'dark' | 'system'`) and sets `--glass-bg`, `--accent`, `--glass-text` on `document.documentElement`.
- **Impact**: Toggling the theme in [`SettingsView.svelte`](../app-tauri/src/lib/views/SettingsView.svelte) calls `appState.setTheme()`, which never updates `theme.mode`. Studio components remain out of sync with the application theme.

### 3.3 Layout Container Constraint in `MainLayout.svelte`
- **Finding**: [`MainLayout.svelte` (lines 17–21, 68–85)](../app-tauri/src/lib/components/MainLayout.svelte#L68-L85) wraps every view inside `<main class="main-content"><div class="view-inner">` with `padding: clamp(1rem, 2.5vw, 2rem)` and `max-width: 1200px; margin: 0 auto;`.
- **Impact**: When `appState.currentView === 'Studio'`, the spatial canvas is boxed into a padded `1200px` column instead of filling the full viewport behind floating glass chrome.

### 3.4 Legacy Views Still Using Static Placeholder Main Panes
| View File | Lines | Placeholder Content | Actual Status |
|---|---|---|---|
| [`EditView.svelte`](../app-tauri/src/lib/views/EditView.svelte#L197-L215) | `197–215` | `<div class="canvas-placeholder"><p>PDF Viewer & Interactive Canvas</p>...` + hardcoded demo quad `[[100, 200, 300, 200, ...]]` for "Highlight Text (Demo)" | Sidebar actions (`add_watermark`, `add_sticky_note`, `flatten_comments`, `add_text_overlay`) call real Rust IPC, but main pane is a static placeholder instead of embedding `<PdfViewer />`. |
| [`OptimizeView.svelte`](../app-tauri/src/lib/views/OptimizeView.svelte#L149-L151) | `149–151` | `<div class="placeholder-preview"><p>Compression preview & sizes will be displayed here in a future update.</p></div>` | Sidebar actions (`compress_pdf`, `repair_pdf`, `ocr_pdf`) are real and working; main preview pane is a placeholder instead of showing `<PdfViewer />`. |
| [`SecurityView.svelte`](../app-tauri/src/lib/views/SecurityView.svelte#L210-L212) | `210–212` | `<div class="placeholder-preview"><p>Security preview & properties will be displayed here in a future update.</p></div>` | Sidebar actions (`protect_pdf`, `unlock_pdf`, `redact_regions`) are real and working; main preview pane is a placeholder instead of showing `<PdfViewer />`. |

---

## 4. Anti-Hardcoding & Protocol Rule Violations (`ADR-012 §1.6`)

| Rule ID | Requirement (`ADR-012` / `DRAWBOARD_STUDIO_MASTER_PLAN.md §1.6`) | Current Violation Found |
|---|---|---|
| **`no-local-keydown`** | Zero `addEventListener('keydown')` or `<svelte:window onkeydown>` outside `commandEngine.svelte.ts`. | 1. [`SpatialCanvas.svelte` line 90](../app-tauri/src/lib/views/studio/SpatialCanvas.svelte#L90): `window.addEventListener('keydown', handleKeyDown)` for Spacebar pan.<br>2. [`RadialMenu.svelte` line 125](../app-tauri/src/lib/components/studio/RadialMenu.svelte#L125): `<svelte:window onkeydown={handleKeyDown} />` for Escape/1–8 keys. |
| **`no-hardcoded-colour`** | Zero hex literals (`#[0-9a-fA-F]{3,8}`) outside `themeContract.svelte.ts`. | [`toolEngine.svelte.ts` lines 117, 127, 140](../app-tauri/src/lib/core/toolEngine.svelte.ts#L117-L140): `createFallbackPreset()` hardcodes `#ffd60a`, `#ff3b30`, and `#1d1d1f` instead of reading `theme.palette.semantic`. |
| **Documentation Sync** | `docs/DIRECTORY_TREE.md` and `protocol/ENGINE_NOTES.md` must match actual workspace state. | 1. [`docs/DIRECTORY_TREE.md` line 66](./DIRECTORY_TREE.md#L66) lists `app-tauri/src-tauri/src/commands.rs`, which does not exist (commands are in `lib.rs`).<br>2. [`protocol/ENGINE_NOTES.md` line 5](../protocol/ENGINE_NOTES.md#L5) lists old `pdfcraft` SHA `dc4b3eee` instead of `75360d1d`. |

---

## 5. Remediation Status (`Phase D2.2` — COMPLETED)

All compliance findings identified in Sections 2–4 have been resolved and verified via `cargo xtask studio-audit --strict` (`0 findings`):
1. **Unified Theme & Full-Bleed Layout (`DONE`)**: Synced `appState.applyTheme()` (`state.svelte.ts`) with `theme.mode` (`themeContract.svelte.ts`), replaced hardcoded hex values in `toolEngine.svelte.ts` with `theme.token(...)`, and made `MainLayout.svelte` full-bleed when `currentView === 'Studio'`.
2. **Wired Real PDF Rendering into `SpatialCanvas.svelte` & Legacy Views (`DONE`)**: Connected `appState.currentFile` and `pdf.min.js` to `SpatialCanvas.svelte` so actual PDF pages render at calibrated DPI into `.page-pdf-canvas`, and replaced the static placeholders in `EditView.svelte`, `OptimizeView.svelte`, and `SecurityView.svelte` with `<PdfViewer />`.
3. **Restored All Granular `ToolId`s & Centralized Keyboard Routing (`DONE`)**: Preserved all granular tool IDs (`shape.rect`, `measure.calibrate`, `select.lasso`, etc.) in `toolEngine.svelte.ts` and moved Spacebar spring-load pan and RadialMenu key handling into `commandEngine.svelte.ts` (`attachKeyboard`).
4. **Added `crates/lpdf-control-bus`, `FavoritesBar.svelte`, Mounted `SplitLayout.svelte`, & `commands/studio.rs` IPC (`DONE`)**: Implemented `ControlMessage`, `Executor`, `replay()` (verified with 1,000-message replay test), and atomic `prefs_save`/`prefs_load` IPC.

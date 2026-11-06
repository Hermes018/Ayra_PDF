# Changelog — LocalPDF Studio

All notable changes to LocalPDF Studio are documented here.

## [v0.2.0-rc1] — 2026-11-06

### Highlights
- **High-Fidelity Offline Engine**: Pure native Rust core powered by `pdfcraft` (hayro/lopdf) with zero network calls and complete air-gapped security.
- **Overhauled UI & Apple HIG Glass Theme**: Modern Svelte 5 spatial canvas with dark/light themes, keyboard shortcuts, dockable toolbars, and high-contrast accessibility compliance.
- **CAD Snapping (OSNAP) & Hatching**: Object snapping for precise technical drawings, measurement calibrations, and standard vector hatch patterns.
- **Enterprise Supply Chain**: CycloneDX 1.5 SBOM generated with all transitive dependencies and licenses stamped; zero GPL/AGPL in linked binaries.
- **Packaging B1 & B2**: Native Linux `.rpm`, `.deb`, and portable `.AppImage` packages compiled with thin LTO and deterministic builds.

### Added
- **Drawboard Studio (Spatial Canvas)**:
  - Document viewing, smooth panning, and responsive zooming presets.
  - Multi-tab document workspace with independent annotation state.
  - Full vector markup toolset: Pen, Highlighter, Eraser, Shapes, Text Box, Stamps, Signatures, and Callouts.
  - CAD Measurement suite: Two-point calibration, length, area, perimeter, and CSV export.
  - Object Snapping (OSNAP): Snap to endpoints, midpoints, intersections, and perpendiculars.
- **Document Operations**:
  - Atomic Save: Writes verified output files (`<name>.annotated.pdf`); originals remain untouched.
  - True Redaction: Irreversibly purges sensitive text/vector/raster streams.
  - Organize View: Visual page reordering, rotation, deletion, page extraction, and document merging.
  - In-Place Text Editing: Direct editing of text blocks with clean reflow.
- **Robustness & Diagnostics**:
  - Soak test retry logic (BUG-062): counted WebDriver reconnects for uninterrupted testing.
  - Low-memory optimization: Thin LTO profile and 1-slot heavy job scheduler for 8GB RAM laptops.

### Changed
- Standardized command engine and theme contracts across all Svelte 5 views.
- Upgraded CycloneDX SBOM generator to include all 173 transitive npm packages from `pnpm-lock.yaml`.

### Fixed
- Fixed Labs gating and tooltip contrast across all dialogs.
- Fixed single-step Organize deletion invariants.

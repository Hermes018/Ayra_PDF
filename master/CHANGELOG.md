# Changelog — LocalPDF Studio

DRAFT for v0.2.0-prototype (release builder, 2026-11-06). The lead finalises it at tag time.

Markers:
- **(pending BUG-NNN)**: the item depends on an open P0 in `logs/BUGS.md`. Drop it, or drop the
  marker, once that bug is closed and its E2E flow passes.
- **(not E2E-tested)**: no golden-path flow covers the item yet.
- **(flow N)**: proven by that E2E flow in the 2026-11-06 baseline (`e2e/out/`, `docs/PROTOTYPE_SCOPE.md`).

## [0.2.0-prototype] — 2026-11-07 (Linux x86_64)

### Added
- **Drawboard Studio** (spatial PDF markup canvas):
  - Open PDFs from the command line or the desktop "Open with" entry. Small, 1,000-page and scanned
    files open with the right page count (flow 2). Encrypted files: **(pending BUG-011)**. Damaged
    files show no notice yet: **(pending BUG-018)**.
  - Pan, zoom buttons, zoom presets, ctrl+wheel, pinch and Fit Page (flow 3). Fit Width:
    **(pending BUG-020)**. Several open documents (tabs or split view): **(pending BUG-016)**.
  - Pen and highlighter ink (flow 4). The partial eraser splits strokes (flow 4). Undo of an erase:
    **(pending BUG-019)**. Laser pointer and smart shapes: **(not E2E-tested)**.
  - Rectangle, ellipse, polygon and revision cloud (flow 5). Hatch patterns in saved files:
    **(pending BUG-015)**.
  - Text box, callout, sticky note and stamp inserts, kept through save and reopen (flow 7). Drawn
    signature: **(pending BUG-017)**.
  - Redaction removes the marked text from a new output file, which passes `qpdf --check`. Other
    text is kept, and the original is unchanged (flow 8).
  - CAD measure: two-point calibration; length, area and angle within 0.5 %; Ledger entries; and a
    quantity takeoff CSV (flow 6). Perimeter: P1 BUG-022 is open. Polylength, digital ruler and
    snap to geometry (OSNAP): **(not E2E-tested)**.
  - Selection tools (marquee, lasso, group, align, flip, z-order), radial menu, favourites bar,
    custom shortcuts and the markup library: **(not E2E-tested)**.
- Edit Text mode, to edit native PDF text in place: **(pending BUG-013)**. The pointer can't reach
  the text blocks. The writer itself produces a correct `.edited.pdf` (flow 9, via a test-only
  click).
- Save writes editable annotations to a NEW file (`<name>.annotated.pdf`). The original keeps its
  hash, the output passes `qpdf --check`, and poppler and pdf.js read it (flow 11). Flatten:
  **(not E2E-tested)** apart from its redaction use (flow 8).
- Organize: reorder, delete and merge, checked page by page in the output (flow 10). Rotation via
  the page grid's Apply: **(pending BUG-014)**. The separate Rotate Pages tool works (flow 10).
  Page thumbnails: **(pending BUG-012)**.
- Labs setting (off by default) for the experimental LaTeX insert and ink-to-text.
- Linux packages: `.rpm` (Fedora), `.deb` (Ubuntu) and a portable AppImage. Each package also ships
  the `localpdf` CLI and the `lpdf-worker` sandbox worker, and registers a `.desktop` entry that
  opens PDFs. An extracted-package smoke test passed flow 1, and flow 2 except BUG-011/018
  (`e2e/pkg_smoke.py`). A system install with `dnf`/`apt` has not been tested (it needs root).

### Changed
- Repository layout: the shared workspace is now `master/`, and the OS folders hold packaging only
  (ADR-016).
- Engine reuse rules (ADR-018 rev 3).
- The Studio top bar fits narrow canvases. An empty Studio shows an "Open a PDF" state, and a cold
  launch takes under 3 s (flow 1).

### Fixed
- `forbid(unsafe_code)` regression (ERR-002).
- Dash-portable sidecar mock tests (ERR-003).
- Dependency advisories cleared; no HTTP client in the Windows tree (ERR-006).

### Security / privacy
- Offline: all 12 flows ran in a network namespace with loopback only (`e2e/run.py --offline`).
  No flow depended on the network. Their failures match the online run.
- No telemetry and no remote assets.

### Known issues
- See `KNOWN_ISSUES.md` (maintained by the lead).

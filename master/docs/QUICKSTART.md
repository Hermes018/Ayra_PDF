# LocalPDF Studio Quickstart Guide

This guide walks through the **12 Golden End-to-End User Workflows** of LocalPDF Studio.

---

### Flow 01: Install & Launch
1. Download the release package for your distribution (`.rpm`, `.deb`, or `.AppImage`).
2. Launch the application from your desktop launcher or terminal.
3. LocalPDF Studio starts in under 2 seconds with clean initial state and zero network calls.

### Flow 02: Open Document
1. Click **Open PDF** on the home screen or press `Ctrl+O`.
2. Select any standard PDF file.
3. The document renders in the high-fidelity native canvas with interactive page navigation.

### Flow 03: Pan, Zoom & Navigate
1. Use mouse wheel or pinch gesture to zoom in/out smoothly.
2. Select zoom presets (`Fit Width`, `Fit Page`, `100%`, `200%`) from the bottom toolbar.
3. Jump between pages using the left thumbnail drawer or keyboard arrows (`Page Up` / `Page Down`).

### Flow 04: Freehand Ink & Annotation
1. Click the **Markup / Pen** tool on the toolbar (`P`).
2. Choose stroke color, width, and smoothing level.
3. Draw freely across document pages; strokes are rendered with sub-pixel Bézier curve smoothing.

### Flow 05: CAD Shapes, Hatch Patterns & Snapping (OSNAP)
1. Select the **Shape** tool (Rectangle, Ellipse, Polygon, Cloud).
2. Enable **OSNAP** (Object Snapping) on the bottom toolbar to snap to vertices, midpoints, and intersections.
3. Apply hatch patterns (ANSI31, cross-hatch, solid fill) with customizable opacity and angle.

### Flow 06: Calibrated Measurement & Scale
1. Select the **Measure** tool (`M`).
2. Calibrate scale by drawing a line across a known dimension and entering its physical length (e.g., `10 ft` or `5 m`).
3. Take linear, perimeter, or area measurements with auto-calculated engineering units.

### Flow 07: Insert Elements (Images, Stamps, Signatures)
1. Select **Insert** from the top view switcher.
2. Place raster images (PNG, JPEG), vector stamps (`Approved`, `Confidential`, `Draft`), or signature blocks.
3. Scale, rotate, and position elements interactively on the spatial canvas.

### Flow 08: Cryptographic & True Visual Redaction
1. Select the **Redact** tool (`R`).
2. Mark sensitive text or image areas with redaction boxes.
3. Click **Commit Redactions**. The underlying text, vector shapes, and raster pixel data are irreversibly purged from the PDF stream (verified by pdftotext/qpdf).

### Flow 09: In-Place Text Editing
1. Select **Edit Text** tool (`T`).
2. Click on existing PDF text blocks to edit contents, adjust font size, alignment, or color.
3. Changes reflow cleanly within document bounding boxes.

### Flow 10: Page Organization & Layout
1. Switch to the **Organize** tab (`Ctrl+Shift+O`).
2. Drag and drop thumbnails to reorder pages.
3. Rotate, delete, extract selected pages, or merge additional external PDFs into the active document.

### Flow 11: Atomic Save & Reopen Verification
1. Click **Save Copy** or press `Ctrl+S`.
2. Choose destination filename.
3. Edits are atomically flushed to a new file, and automatically verified against corruption before release. Your original source document remains untouched.

### Flow 12: Long-Session Stability (Soak)
1. Maintain open documents across extended editing sessions.
2. Perform repetitive markup, navigation, and save cycles.
3. System memory and file descriptors remain bounded without leak escalation.

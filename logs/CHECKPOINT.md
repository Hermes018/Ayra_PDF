# CHECKPOINT (OVERWRITE this file; single source of "where am I")
- Date: 2026-10-29
- Phase / sub-phase: Phase 12.5 COMPLETE — v0.1.0-rc5 Refined (Theme Collision & Inverted Colors on Convert View Fixed)
- Last green tag: v0.1.0-rc5
- In progress: Ready for host reinstall (`sudo dnf reinstall -y "./target/release/bundle/rpm/LocalPDF Studio-0.1.0-1.x86_64.rpm"`)
- Open risks: Memory spikes on large rasterized render passes across low tiers (ADR-002). Rendering performance on >1000 pages for the grid editor might require IntersectionObserver virtualization.
- Next 3 actions: 1) Run `sudo dnf reinstall` on Fedora host 2) Verify light/dark theme contrast on Convert tabs and cards 3) Run UAT on windowed-mode layouts (800px-1080px)
- Active ADRs: 001-010, 004 (Local LLM Architecture & Candle Selection), 005 (Native Feature Expansion), 006 (Document Reconstruction Reality Check), 008 (SvelteKit SPA), 009 (Vendored PDF.js)
- Pinned pdfcraft commit: dc4b3eee84f73bad8564906c141bdda9caad0fd6

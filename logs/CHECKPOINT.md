# CHECKPOINT (OVERWRITE this file; single source of "where am I")
- Date: 2026-10-15
- Phase / sub-phase: Phase 12.3 (Human UAT & Final Release)
- Last green tag: phase-12.2-rc2
- In progress: Human UAT of RC2 RPM package on Fedora 44 x86_64
- Open risks: Memory spikes on large rasterized render passes across low tiers (ADR-002). Rendering performance on >1000 pages for the grid editor might require IntersectionObserver virtualization. GitHub push blocked pending user credential config (F-RC1-005).
- Next 3 actions: 1) Install RC2 RPM on target Fedora 44 desktop and test Dolphin "Open With..." 2) Verify CLI and worker commands via /usr/bin/localpdf 3) Perform final release signoff
- Active ADRs: 001-010, 004 (Local LLM Architecture & Candle Selection), 005 (Native Feature Expansion), 006 (Document Reconstruction Reality Check), 008 (SvelteKit SPA), 009 (Vendored PDF.js)
- Pinned pdfcraft commit: dc4b3eee84f73bad8564906c141bdda9caad0fd6


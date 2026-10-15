# CHECKPOINT (OVERWRITE this file; single source of "where am I")
- Date: 2026-10-15
- Phase / sub-phase: Phase 12.3 COMPLETE — v0.1.0-rc2 Ready for Host Installation
- Last green tag: v0.1.0-rc2
- In progress: Ready for end-user host installation (`sudo dnf install target/release/bundle/rpm/LocalPDF Studio-0.1.0-1.x86_64.rpm`)
- Open risks: Memory spikes on large rasterized render passes across low tiers (ADR-002). Rendering performance on >1000 pages for the grid editor might require IntersectionObserver virtualization. GitHub push blocked pending user credential config (F-RC1-005).
- Next 3 actions: 1) Run `sudo dnf install` on Fedora 44 host 2) Verify desktop menu launcher and KDE Dolphin "Open With..." 3) Test CLI `/usr/bin/localpdf` workflows on host system
- Active ADRs: 001-010, 004 (Local LLM Architecture & Candle Selection), 005 (Native Feature Expansion), 006 (Document Reconstruction Reality Check), 008 (SvelteKit SPA), 009 (Vendored PDF.js)
- Pinned pdfcraft commit: dc4b3eee84f73bad8564906c141bdda9caad0fd6

